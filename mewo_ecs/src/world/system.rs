use sparseset::SparseSet;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::iter::Iterator;
use std::any::TypeId;
use super::wish::{
    WishArg,
    WishType,
};
use super::component::{
    ComponentManager,
    ComponentTypeId,
};
use super::component_stamp::ComponentStamp;
use super::resource::ResourceManager;
use super::command::WorldCommands;
use super::entity::Entity;
use super::mask::BoolMask;
use super::world::World;

#[derive(Clone, Copy, PartialEq)]
pub enum ComponentAccessMode {
    Read,
    Write,
}

pub struct SystemArgs<'rmgr, 'world, 'cmds> {
    pub rmgr: &'rmgr ResourceManager,
    pub cmds: WorldCommands<'world, 'cmds>,
}

pub type SystemCallback<Q> = fn (Wish<Q>, SystemArgs);
pub struct System<Q: WishArg>(pub SystemCallback<Q>);
pub type BoxedSystem = Box<dyn GenericSystem>;

pub trait GenericSystem {
    fn call(&self, wish: &WishInstance, args: SystemArgs);
}

impl<Q> System<Q> 
    where Q: WishArg
{
    pub fn get_wish_info(&self) -> Vec<WishType> {
        Q::get_types()
    }
}

impl<Q> GenericSystem for System<Q> 
    where Q: WishArg
{
    fn call(&self, wish: &WishInstance, args: SystemArgs) {
        (self.0)(Wish::instance_from(wish), args)
    }
}

pub struct SystemFilter {
    pub with: Option<ComponentStamp>,
    pub without: Option<ComponentStamp>,
}

pub struct SystemData {
    pub reads: Vec<(ComponentTypeId, SystemFilter)>,
    pub writes: Vec<(ComponentTypeId, SystemFilter)>,
}

impl SystemData {
    pub fn from_query_type(world: &World, qts: &Vec<WishType>) -> SystemData {
        let component_manager = world.get_component_manager();
        let mut data = SystemData {
            reads: Vec::new(),
            writes: Vec::new(),
        };
        for qt in qts.iter() {
            let mut filter = SystemFilter {
                with: None,
                without: None,
            };
            if let Some(withs) = &qt.with {
                let mut with_stamp = ComponentStamp::create(world);
                for with in withs.iter() {
                    with_stamp.stamp(component_manager.get_component_id(*with).unwrap());
                }
                filter.with = Some(with_stamp);
            }
            if let Some(withouts) = &qt.without {
                let mut without_stamp = ComponentStamp::create(world);
                for without in withouts.iter() {
                    without_stamp.stamp(component_manager.get_component_id(*without).unwrap());
                }
                filter.without = Some(without_stamp);
            }
            let id = component_manager.get_component_id(qt.tyid).unwrap();
            match qt.access_mode {
                ComponentAccessMode::Read => data.reads.push((id, filter)),
                ComponentAccessMode::Write => data.writes.push((id, filter)),
            }
        };
        data
    }
}

pub struct Wish<'wish, 'global, Q> 
    where Q: WishArg
{
    q: PhantomData<Q>,
    wish: &'wish WishInstance<'wish, 'global>,
}

impl<'wish, 'global, Q> Wish<'wish, 'global, Q> 
    where Q: WishArg
{
    pub fn instance_from(wish: &'wish WishInstance<'wish, 'global>) -> Self {
        Wish {
            wish,
            q: PhantomData,
        }
    }

    pub fn write<C>(&self) -> GiftInstanceWriteIter<'wish, 'global, C>
        where C: 'static
    {
        self.wish.write::<C>() 
    }

    pub fn read<C>(&self) -> GiftInstanceReadIter<'wish, 'global, C>
        where C: 'static
    {
        self.wish.read::<C>() 
    }
}

//  *const () points to `data: Vec<C>`
type UnsafeComponentSlice = *const ();
type UnsafeEntitySlice = *const Vec<Entity>;

pub enum ComponentIndexBuffer {
    All,
    Include(BoolMask),
}

pub struct GlobalWish {
    data: SparseSet<(UnsafeComponentSlice, UnsafeEntitySlice)>,
    component_types: HashMap<TypeId, ComponentTypeId>,
}

impl GlobalWish {
    pub fn create(component_mgr: &ComponentManager) -> GlobalWish {
        let data = SparseSet::with_capacity(component_mgr.get_component_type_count());
        let component_types = (*component_mgr.get_component_types()).clone();
        GlobalWish {
            data, component_types,
        }
    }

    pub fn recreate_slices(&mut self, component_mgr: &ComponentManager) {
        self.component_types = (*component_mgr.get_component_types()).clone();
        self.data.clear();
        for cty in self.component_types.values() {
            let cty = *cty;
            let storage = component_mgr
                .get_boxed_storage(cty)
                .get_untyped_storage();
            self.data.insert(cty, (storage.get_data_ptr(), storage.get_entities() as *const Vec<Entity>));
        }
        
    }
}

pub struct SystemWish {
    index_buf: SparseSet<(ComponentAccessMode, ComponentIndexBuffer)>, 
}

impl SystemWish {
    pub fn create(world: &World, global_wish: &GlobalWish, sys_data: &SystemData) -> Self {
        let mut ret = SystemWish {
            index_buf: SparseSet::with_capacity(global_wish.data.capacity()),
        };
        ret.update_index_buf(world, sys_data);
        ret
    }

    pub fn update_index_buf(&mut self, world: &World, sys_data: &SystemData) {
        self.index_buf.clear();
        for (access, data) in [
            (ComponentAccessMode::Read, sys_data.reads.iter()),
            (ComponentAccessMode::Write, sys_data.writes.iter()), 
        ] {
            for (id, filter) in data {
                if let (None, None) = (&filter.with, &filter.without) {
                    self.index_buf.insert(*id, (access, ComponentIndexBuffer::All));
                } else {
                    let mut index_mask = BoolMask::create();
                    if let Some(with) = &filter.with {
                        for (i, e) in world
                            .get_component_manager()
                            .get_boxed_storage(*id)
                            .get_untyped_storage()
                            .get_entities()
                            .iter()
                            .enumerate()
                        {
                            let dep_info = world.get_entity_dep_info(*e)
                                .unwrap();
                            if dep_info.bitwise_and(&with).compare(&with) {
                                index_mask.set(i, true).unwrap();
                            }
                        }
                    }
                    if let Some(without) = &filter.without {
                        for (i, e) in world
                            .get_component_manager()
                            .get_boxed_storage(*id)
                            .get_untyped_storage()
                            .get_entities()
                            .iter()
                            .enumerate()
                        {
                            let dep_info = world.get_entity_dep_info(*e)
                                .unwrap();
                            if !dep_info.bitwise_and(&without).compare(&without) {
                                index_mask.set(i, true).unwrap();
                            }
                        }
                    }
                    self.index_buf.insert(*id, (access, ComponentIndexBuffer::Include(index_mask)));
                }
            }
        }
    }
}

pub struct WishInstance<'wish, 'global> {
    sys_wish: &'wish SystemWish,
    global_wish: &'global GlobalWish,
}

impl<'wish, 'global> WishInstance<'wish, 'global> {
    pub fn create(sys_wish: &'wish SystemWish, global_wish: &'global GlobalWish) -> Self {
        WishInstance {
            sys_wish, global_wish,
        } 
    }

    pub fn write<C>(&self) -> GiftInstanceWriteIter<'wish, 'global, C>
        where C: 'static
    {
        let id = *self.global_wish.component_types.get(&TypeId::of::<C>())
            .unwrap();
        let (mode, indices) = self.sys_wish.index_buf.get(id)
            .unwrap();
        if *mode != ComponentAccessMode::Write {
            panic!("replace this later")
        }
        let (cdata, edata) = *self.global_wish.data.get(id)
            .unwrap();
        let (storage, entities) = unsafe {
            (
                &mut *std::mem::transmute::<UnsafeComponentSlice, *mut Vec<C>>(cdata),
                &*edata
            )
        };
        GiftInstanceWriteIter {
            indices, 
            storage, 
            entities, 
            index: 0,
        }
    }

    pub fn read<C>(&self) -> GiftInstanceReadIter<'wish, 'global, C>
        where C: 'static
    {
        let id = *self.global_wish.component_types.get(&TypeId::of::<C>())
            .unwrap();
        let (mode, indices) = self.sys_wish.index_buf.get(id)
            .unwrap();
        if *mode != ComponentAccessMode::Read {
            panic!("replace this later")
        }
        let (cdata, edata) = *self.global_wish.data.get(id)
            .unwrap();
        let (storage, entities) = unsafe {
            (
                &*std::mem::transmute::<UnsafeComponentSlice, *const Vec<C>>(cdata),
                &*edata
            )
        };
        GiftInstanceReadIter {
            indices, 
            storage, 
            entities, 
            index: 0,
        }
    }
}

pub struct GiftInstanceWriteIter<'wish, 'global, C> {
    indices: &'wish ComponentIndexBuffer,
    storage: &'global mut Vec<C>,
    entities: &'global Vec<Entity>,
    index: usize,
}

pub struct GiftInstanceReadIter<'wish, 'global, C> {
    indices: &'wish ComponentIndexBuffer,
    storage: &'global Vec<C>,
    entities: &'global Vec<Entity>,
    index: usize,
}

impl<'wish, 'global, C> Iterator for GiftInstanceWriteIter<'wish, 'global, C> {
    type Item = (&'global mut C, Entity);
    fn next(&mut self) -> Option<Self::Item> {
        match self.indices {
            ComponentIndexBuffer::All => {
                if let Some(e) = self.entities.get(self.index) {
                    let retc = unsafe {
                        &mut *self.storage.as_mut_ptr().offset(self.index as isize)
                    };
                    let ret = Some((retc, *e));
                    self.index += 1;
                    ret
                } else {
                    None
                }
            },
            ComponentIndexBuffer::Include(mask) => {
                while let Ok(b) = mask.get(self.index) {
                    if b {
                        if let Some(e) = self.entities.get(self.index) {
                            let retc = unsafe {
                                &mut *self.storage.as_mut_ptr().offset(self.index as isize)
                            };
                            let ret = Some((retc, *e));
                            self.index += 1;
                            return ret
                        } else {
                            unreachable!()
                        }
                    }
                    self.index += 1;
                }
                None
            },
        } 
    }
}

impl<'wish, 'global, C> Iterator for GiftInstanceReadIter<'wish, 'global, C> {
    type Item = (&'global C, Entity);
    fn next(&mut self) -> Option<Self::Item> {
        match self.indices {
            ComponentIndexBuffer::All => {
                if let Some(e) = self.entities.get(self.index) {
                    let ret = Some((&self.storage.as_slice()[self.index], *e));
                    self.index += 1;
                    ret
                } else {
                    None
                }
            },
            ComponentIndexBuffer::Include(mask) => {
                while let Ok(b) = mask.get(self.index) {
                    if b {
                        if let Some(e) = self.entities.get(self.index) {
                            let ret = Some((&self.storage.as_slice()[self.index], *e));
                            self.index += 1;
                            return ret
                        } else {
                            unreachable!()
                        }
                    }
                    self.index += 1;
                }
                None
            },
        } 
    }
}

#[test]
fn test_wish() {
    use crate::{
        Component, 
        EntityModifierStore,
        EntityModifierHandle,
    };
    #[derive(Debug, Clone, PartialEq, Eq)]
    struct SomeWrite {
        val: u32,
    }
    impl Component for SomeWrite {}

    let mut world = World::create();
    world
        .get_mut_component_manager()
        .register_component_type::<SomeWrite>()
        .unwrap();
    let mut entity_mod_store = EntityModifierStore::create(EntityModifierHandle::Spawn, &world);
    let mut entity_mod = entity_mod_store.modify(&world);
    entity_mod.insert_component(SomeWrite { val: 2 });
    world.modify_entity(&mut entity_mod_store);
    let mut global_wish = GlobalWish::create(world.get_component_manager());
    global_wish.recreate_slices(world.get_component_manager());
    let sys = |wish: &mut WishInstance| {
        for (data, _e) in wish.write::<SomeWrite>() {
            data.val += 10; 
        }
    };
    let sys_data = SystemData {
        reads: vec![],
        writes: vec![(0, SystemFilter { with: None, without: None })],
    };
    let wish = SystemWish::create(&world, &global_wish, &sys_data);
    let mut wish_inst = WishInstance::create(&wish, &global_wish);
    (sys)(&mut wish_inst);
    assert_eq!(
        world
            .get_component_manager()
            .get_boxed_storage_of::<SomeWrite>()
            .get_storage::<SomeWrite>()
            .get_component_with_entity_of(Entity::from_id(0))
            .unwrap(),
        &SomeWrite { val: 2 + 10 }
    );
}

