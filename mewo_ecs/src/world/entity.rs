use super::component::Component;
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

    pub fn from_index(id: usize) -> Entity {
        Entity { id: id as Id }
    }

    pub fn as_index(&self) -> usize {
        self.id as usize
    }
}

pub enum EntityModifyHandle {
    Spawn,
    Entity(Entity),
}

pub struct EntityWrapper<'world> {
    world: &'world mut World,
    entity: Entity,
}

impl<'world> EntityWrapper<'world> {
    pub fn from_entity(entity: Entity, world: &'world mut World) -> Self {
        EntityWrapper { world, entity }
    }

    pub fn insert_component<C>(&mut self, data: C) -> &mut Self
    where
        C: 'static + Component,
    {
        self.world
            .insert_component_with_entity::<C>(self.entity, data)
            .unwrap();
        self
    }

    pub fn remove_component<C>(&mut self) -> &mut Self
    where
        C: 'static + Component,
    {
        self.world
            .remove_component_with_entity::<C>(self.entity)
            .unwrap();
        self
    }
}

pub struct EntityModifyCallback<F: Fn(&mut EntityWrapper) -> ()>(pub F);
pub trait GenericEntityModifyCallback {
    fn call(&self, entity: Entity, world: &mut World);
}
pub type BoxedEntityModifyCallback = Box<dyn GenericEntityModifyCallback>;

impl<F> GenericEntityModifyCallback for EntityModifyCallback<F>
where
    F: Fn(&mut EntityWrapper),
{
    fn call(&self, entity: Entity, world: &mut World) {
        let mut wrapper = EntityWrapper::from_entity(entity, world);
        (self.0)(&mut wrapper);
    }
}
