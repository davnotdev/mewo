use super::command::WorldCommands;
use super::component_stamp::ComponentStamp;
use super::entity::Entity;
use super::resource::ResourceManager;
use super::wish::WishData;
use super::wish::Wishes;
use super::world::World;
use crate::error::Result;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ComponentAccessMode {
    Read,
    Write,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FilterMode {
    With,
    Without,
}

pub struct SystemArgs<'rmgr, 'cmds> {
    pub rmgr: &'rmgr ResourceManager,
    pub cmds: &'cmds mut WorldCommands,
}

pub type SystemFunction<WS> = fn(&mut SystemArgs, WS);

pub type BoxedSystem = Box<dyn UntypedSystemCallback>;
pub trait UntypedSystemCallback {
    fn get_wish_datas(&self) -> Vec<WishData>;
    fn call(&self, world: &World, cmds: &mut WorldCommands, sets: &Vec<SystemDataSetInstance>);
}

impl<WS> UntypedSystemCallback for SystemFunction<WS>
where
    WS: Wishes,
{
    fn get_wish_datas(&self) -> Vec<WishData> {
        WS::get_wish_datas()
    }

    fn call(&self, world: &World, cmds: &mut WorldCommands, set: &Vec<SystemDataSetInstance>) {
        let mut args = SystemArgs {
            cmds,
            rmgr: world.get_resource_manager(),
        };
        (self)(
            &mut args,
            WS::create(
                world as *const World,
                set as *const Vec<SystemDataSetInstance>,
            ),
        )
    }
}

pub struct SystemFilter {
    pub with: Option<ComponentStamp>,
    pub without: Option<ComponentStamp>,
}

pub struct SystemDataSet {
    pub total_withs: ComponentStamp,
    pub reads: ComponentStamp,
    pub writes: ComponentStamp,
    pub filter: SystemFilter,
}

impl SystemDataSet {
    pub fn from_wish_data(world: &World, data: &WishData) -> Result<Self> {
        let component_mgr = world.get_component_manager();
        let mut reads = ComponentStamp::create(world);
        let mut writes = ComponentStamp::create(world);
        for (ty, access) in data.tyids.iter() {
            let id = component_mgr.get_component_id(*ty)?;
            match access {
                ComponentAccessMode::Read => reads.stamp(id)?,
                ComponentAccessMode::Write => writes.stamp(id)?,
            };
        }
        let mut with = ComponentStamp::create(world);
        let mut without = ComponentStamp::create(world);
        for (ty, filter) in data.filters.iter() {
            let id = component_mgr.get_component_id(*ty)?;
            match filter {
                FilterMode::With => with.stamp(id)?,
                FilterMode::Without => without.stamp(id)?,
            };
        }
        let mut total_withs = reads.clone();
        total_withs.merge(&writes)?;
        total_withs.merge(&with)?;
        Ok(SystemDataSet {
            total_withs,
            reads,
            writes,
            filter: SystemFilter {
                with: if with.is_empty() { None } else { Some(with) },
                without: if without.is_empty() {
                    None
                } else {
                    Some(without)
                },
            },
        })
    }

    pub fn match_entity(&self, world: &World, entity: Entity) -> Result<bool> {
        let dep_info = world.get_entity_dep_info(entity)?;
        Ok(ComponentStamp::system_match(
            dep_info,
            &self.total_withs,
            &self.filter.without,
        ))
    }

    pub fn realign_len(&mut self, world: &World) {
        self.total_withs.realign_len(world);
        self.reads.realign_len(world);
        self.writes.realign_len(world);
        if let Some(with) = &mut self.filter.with {
            with.realign_len(world);
        }
        if let Some(without) = &mut self.filter.without {
            without.realign_len(world);
        }
    }
}

pub struct SystemDataSetInstance {
    pub entities: Vec<Entity>,
}

impl SystemDataSetInstance {
    pub fn create(world: &World, data: &SystemDataSet) -> Result<Self> {
        let mut inst = SystemDataSetInstance {
            entities: Vec::new(),
        };
        let entity_mgr = world.get_entity_manager();
        for ei in 0..entity_mgr.get_entity_count() {
            let entity = Entity::from_index(ei);
            if entity_mgr.entity_exists(entity) {
                if data.match_entity(world, entity)? {
                    inst.entities.push(entity);
                }
            }
        }
        Ok(inst)
    }

    pub fn any_entity_modify(&mut self, world: &World, data: &SystemDataSet, entity: Entity) {
        if data.match_entity(world, entity).unwrap() {
            if !self.entities.contains(&entity) {
                self.entities.push(entity);
            }
        }
    }

    pub fn any_entity_remove(&mut self, entity: Entity) {
        for (i, self_entity) in self.entities.iter().enumerate() {
            if *self_entity == entity {
                self.entities.remove(i);
                break;
            }
        }
    }
}

pub struct SystemBuilder(Vec<(BoxedSystem, Vec<SystemDataSet>)>);

impl SystemBuilder {
    pub fn create() -> Self {
        SystemBuilder(Vec::new())
    }

    pub fn sys<WS>(&mut self, world: &World, callback: SystemFunction<WS>) -> &mut Self
    where
        WS: 'static + Wishes,
    {
        let wishes = callback.get_wish_datas();
        let mut sets = Vec::new();
        for wish in wishes.into_iter() {
            let data = SystemDataSet::from_wish_data(world, &wish).unwrap();
            sets.push(data);
        }
        self.0.push((Box::new(callback), sets));
        self
    }

    pub fn consume(self) -> Vec<(BoxedSystem, Vec<SystemDataSet>)> {
        self.0
    }
}
