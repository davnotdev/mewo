use sparseset::SparseSet;
use std::collections::HashMap;
use std::iter::Iterator;
use std::any::TypeId;
use super::component_manager::{
    ComponentManager,
    ComponentTypeId,
};
use super::system::SantaClaus;
use super::entity::Entity;
use super::mask::BoolMask;
use super::world::World;

#[derive(Copy, Clone, PartialEq)]
pub enum ComponentAccessMode {
    Read,
    Write,
}

//  Gifts are built around the way components are stored in Storage<C>

//  *const () points to `data: Vec<C>`
type UnsafeComponentSlice = *const ();
type UnsafeEntitySlice = *const Vec<Entity>;

pub enum ComponentIndexBuffer {
    All,
    Include(BoolMask),
}

pub struct GlobalGift {
    data: SparseSet<(UnsafeComponentSlice, UnsafeEntitySlice)>,
    component_types: HashMap<TypeId, ComponentTypeId>,
}

impl GlobalGift {
    pub fn create(component_mgr: &ComponentManager) -> GlobalGift {
        let data = SparseSet::with_capacity(component_mgr.get_component_type_count());
        let component_types = (*component_mgr.get_component_types()).clone();
        GlobalGift {
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

pub struct Gift {
    index_buf: SparseSet<(ComponentAccessMode, ComponentIndexBuffer)>, 
}

impl Gift {
    pub fn create(world: &World, global_gift: &GlobalGift, santa: &SantaClaus) -> Self {
        let mut ret = Gift {
            index_buf: SparseSet::with_capacity(global_gift.data.capacity()),
        };
        ret.update_index_buf(world, santa);
        ret
    }

    pub fn update_index_buf(&mut self, world: &World, santa: &SantaClaus) {
        self.index_buf.clear();
        for (access, data) in [
            (ComponentAccessMode::Write, santa.writes.iter()), 
            (ComponentAccessMode::Read, santa.reads.iter()),
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

pub struct GiftInstance<'gift, 'global> {
    gift_store: &'gift Gift,
    global_gift: &'global GlobalGift,
}

impl<'gift, 'global> GiftInstance<'gift, 'global> {
    pub fn create(gift_store: &'gift Gift, global_gift: &'global GlobalGift) -> Self {
        GiftInstance {
            gift_store, global_gift,
        } 
    }

    pub fn write<C>(&self) -> GiftInstanceWriteIter<'gift, 'global, C>
        where C: 'static
    {
        let id = *self.global_gift.component_types.get(&TypeId::of::<C>())
            .unwrap();
        let (mode, indices) = self.gift_store.index_buf.get(id)
            .unwrap();
        if *mode != ComponentAccessMode::Write {
            panic!("replace this later")
        }
        let (cdata, edata) = *self.global_gift.data.get(id)
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

    pub fn read<C>(&self) -> GiftInstanceReadIter<'gift, 'global, C>
        where C: 'static
    {
        let id = *self.global_gift.component_types.get(&TypeId::of::<C>())
            .unwrap();
        let (mode, indices) = self.gift_store.index_buf.get(id)
            .unwrap();
        if *mode != ComponentAccessMode::Read {
            panic!("replace this later")
        }
        let (cdata, edata) = *self.global_gift.data.get(id)
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

pub struct GiftInstanceWriteIter<'gift, 'global, C> {
    indices: &'gift ComponentIndexBuffer,
    storage: &'global mut Vec<C>,
    entities: &'global Vec<Entity>,
    index: usize,
}

pub struct GiftInstanceReadIter<'gift, 'global, C> {
    indices: &'gift ComponentIndexBuffer,
    storage: &'global Vec<C>,
    entities: &'global Vec<Entity>,
    index: usize,
}

impl<'gift, 'global, C> Iterator for GiftInstanceWriteIter<'gift, 'global, C> {
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

impl<'gift, 'global, C> Iterator for GiftInstanceReadIter<'gift, 'global, C> {
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
fn test_gift() {
    #[derive(Debug, Clone, PartialEq, Eq)]
    struct SomeWrite {
        val: u32,
    }

    let mut world = World::create();
    world
        .get_mut_component_manager()
        .register_component_type::<SomeWrite>()
        .unwrap();
    let our_entity = world.insert_entity(Some(|mut e| {
        e.insert_component(SomeWrite {
            val: 2,
        });
    }));
    let mut global_gift = GlobalGift::create(world.get_component_manager());
    global_gift.recreate_slices(world.get_component_manager());
    let sys = |gift: &mut GiftInstance| {
        for (data, _e) in gift.write::<SomeWrite>() {
            data.val += 10; 
        }
    };
    let santa = SantaClaus::wishlist(&world)
        .writes(vec![0], None, None)
        .finish();
    let gift = Gift::create(&world, &global_gift, &santa);
    let mut gift_inst = GiftInstance::create(&gift, &global_gift);
    (sys)(&mut gift_inst);
    assert_eq!(
        world
            .get_component_manager()
            .get_boxed_storage_of::<SomeWrite>()
            .get_storage::<SomeWrite>()
            .get_component_with_entity(our_entity)
            .unwrap(),
        &SomeWrite { val: 2+10 }
    );
}

