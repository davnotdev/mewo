use std::any::Any;
use super::component::{
    Component,
    ComponentTypeId,
};
use super::world::World; 

pub type Id = u32;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct Entity {
    pub id: Id, 
}

impl Entity {
    pub fn from_id(id: Id) -> Entity {
        Entity { id }
    }

    pub fn as_index(&self) -> usize {
        self.id as usize
    } 
}

pub enum EntityComponentModifyType {
    Insert(Option<Box<dyn Any>>),
    Remove,
}

pub struct EntityComponentModify {
    pub cid: ComponentTypeId,
    pub modify: EntityComponentModifyType,
}

#[derive(PartialEq)]
pub enum EntityModifierHandle {
    Spawn,
    Modify(Entity),
}

pub struct EntityModifierStore {
    pub entity: EntityModifierHandle,
    pub components: Vec<EntityComponentModify>,
}

impl EntityModifierStore {
    pub fn create(entity: EntityModifierHandle, world: &World) -> EntityModifierStore {
        let store = EntityModifierStore {
            entity,
            components: Vec::with_capacity(world.get_component_manager().get_component_type_count())
        };
        store
    }

    pub fn modify<'world, 'store>(&'store mut self, world: &'world World) -> EntityModifier<'world, 'store> {
        EntityModifier {
            world,
            components: &mut self.components,
        }
    }
}

pub struct EntityModifier<'world, 'store> {
    world: &'world World,
    components: &'store mut Vec<EntityComponentModify>,
}

impl<'world, 'store> EntityModifier<'world, 'store> {
    pub fn insert_component<C>(&mut self, obj: C) -> &mut Self
        where C: 'static + Component
    {
        let id = self.world
            .get_component_manager()
            .get_component_id_of::<C>()
            .unwrap();
        self.components.push(EntityComponentModify {
            cid: id,
            modify: EntityComponentModifyType::Insert(Some(Box::new(obj))),
        });
        self
    }

    pub fn remove_component<C>(&mut self) -> &mut Self
        where C: 'static + Component
    {
        let id = self.world
            .get_component_manager()
            .get_component_id_of::<C>()
            .unwrap();
        self.components.push(EntityComponentModify {
            cid: id,
            modify: EntityComponentModifyType::Remove,
        });
        self
    }
}

