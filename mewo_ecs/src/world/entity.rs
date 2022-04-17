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

    pub fn as_index(&self) -> usize {
        self.id as usize
    } }

pub struct EntityWrapper<'world> {
    entity: Entity,
    world: &'world mut World,
}

impl<'world> EntityWrapper<'world> {
    pub fn create(entity: Entity, scene: &'world mut World) -> EntityWrapper<'world> {
        EntityWrapper {
            entity, world: scene,
        }
    }

    pub fn insert_component<C>(&mut self, obj: C) 
        where C: 'static + Component
    {
        self.world.insert_component_with_entity(self.entity, obj);
    }

    pub fn remove_component<C>(&mut self) 
        where C: 'static + Component
    {
        self.world.remove_component_with_entity::<C>(self.entity);
    }

    pub fn get_component<C>(&self) -> &C
        where C: 'static + Component
    {
        self.world.get_component_with_entity::<C>(self.entity)
    }

    pub fn get_mut_component<C>(&mut self) -> &mut C
        where C: 'static + Component
    {
        self.world.get_mut_component_with_entity::<C>(self.entity)
    }
}

