use std::any::Any;
use crate::error::{
    Result,
    ECSError,
    ComponentErrorIdentifier,
};
use super::entity::Entity;

pub struct BoxedStorage {
    storage: Box<dyn GenericStorage>,
}

impl BoxedStorage {
    pub fn create<C>() -> BoxedStorage
        where C: 'static 
    {
        BoxedStorage {
            storage: Box::new(Storage::<C>::create())
        }
    }

    pub fn get_storage<C>(&self) -> Result<&Storage<C>>
        where C: 'static
    {
        if let Some(storage) = self.storage
            .as_any()
            .downcast_ref::<Storage<C>>() {
                Ok(storage)
            } else {
                Err(ECSError::ComponentTypeDoesNotExist(ComponentErrorIdentifier::Name(std::any::type_name::<C>())))
            }
    }

    pub fn get_mut_storage<C>(&mut self) -> Result<&mut Storage<C>>
        where C: 'static
    {
        if let Some(storage) = self.storage
            .as_any_mut()
            .downcast_mut::<Storage<C>>() {
                Ok(storage)
            } else {
                Err(ECSError::ComponentTypeDoesNotExist(ComponentErrorIdentifier::Name(std::any::type_name::<C>())))
            }
    }

    pub fn get_untyped_storage(&self) -> &dyn GenericStorage {
        self.storage
            .as_ref()
    }

    pub fn get_mut_untyped_storage(&mut self) -> &mut dyn GenericStorage {
        self.storage
            .as_mut()
    }
}

pub trait GenericStorage : Any {
    fn get_data_ptr(&self) -> *const ();
    fn get_entities(&self) -> &Vec<Entity>;
    fn insert_component_with_entity(&mut self, entity: Entity, data: Box<dyn Any>) -> Result<()>;
    fn remove_component_with_entity(&mut self, entity: Entity) -> Result<()>;
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

pub struct Storage<C> {
    data: Vec<C>,
    entities: Vec<Entity>,
}

impl<C> Storage<C> 
    where C: 'static 
{
    pub fn create() -> Self {
        Storage {
            data: Vec::new(),
            entities: Vec::new(),
        }
    }

    pub fn insert_component_with_entity(&mut self, data: C, entity: Entity) -> Result<()> {
        if self.entities.contains(&entity) {
            return Err(ECSError::EntityAlreadyHasComponent(entity, ComponentErrorIdentifier::Name(std::any::type_name::<C>())))
        } else {
            self.data.push(data);
            self.entities.push(entity);
            Ok(())
        }
    }

    pub fn get_component_with_entity_of(&self, entity: Entity) -> Result<&C> {
        for (i, e) in self.entities.iter().enumerate() {
            if *e == entity {
                return Ok(self.data.get(i).unwrap())
            }
        }
        Err(ECSError::EntityDoesNotHaveComponent(entity, ComponentErrorIdentifier::Name(std::any::type_name::<C>())))
    }

    pub fn get_mut_component_with_entity(&mut self, entity: Entity) -> Result<&mut C> {
        for (i, e) in self.entities.iter().enumerate() {
            if *e == entity {
                return Ok(self.data.get_mut(i).unwrap())
            }
        }
        Err(ECSError::EntityDoesNotHaveComponent(entity, ComponentErrorIdentifier::Name(std::any::type_name::<C>())))
    }
}

impl<C> GenericStorage for Storage<C> 
    where C: 'static
{
    fn get_data_ptr(&self) -> *const () {
        &self.data as *const Vec<C> as *const ()
    }

    fn get_entities(&self) -> &Vec<Entity> {
        &self.entities
    }

    fn insert_component_with_entity(&mut self, entity: Entity, data: Box<dyn Any>) -> Result<()> {
        if self.entities.contains(&entity) {
            return Err(ECSError::EntityAlreadyHasComponent(entity, ComponentErrorIdentifier::Name(std::any::type_name::<C>())))
        } else {
            let data = *(match data.downcast::<C>() {
                Ok(data) => data,
                Err(_) => unreachable!("Component Type `{}` does not match", std::any::type_name::<C>()),
            });
            self.data.push(data);
            self.entities.push(entity);
            Ok(())
        }
    }

    fn remove_component_with_entity(&mut self, entity: Entity) -> Result<()> {
        let i = self.entities
            .iter()
            .position(|&e| e == entity);
        if let Some(i) = i {
            self.data.swap_remove(i);
            self.entities.swap_remove(i);
            Ok(())
        } else {
            return Err(ECSError::EntityDoesNotHaveComponent(entity, ComponentErrorIdentifier::Name(std::any::type_name::<C>())))
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[test]
fn test_storage() {
    #[derive(Debug, Clone, PartialEq, Eq)]
    struct SomeComponent(usize);
    let entity_a = Entity { id: 1 };
    let entity_b = Entity { id: 4 };
    let mut boxed_storage = BoxedStorage::create::<SomeComponent>();
    boxed_storage
        .get_mut_storage::<SomeComponent>()
        .unwrap()
        .insert_component_with_entity(SomeComponent(69), entity_a)
        .unwrap();
    assert_eq!(
        boxed_storage
            .get_mut_storage::<SomeComponent>()
        .unwrap()
            .insert_component_with_entity(SomeComponent(69), entity_a),
        Err(ECSError::EntityAlreadyHasComponent(entity_a, ComponentErrorIdentifier::Name(std::any::type_name::<SomeComponent>())))
    );
    boxed_storage
        .get_mut_storage::<SomeComponent>()
        .unwrap()
        .insert_component_with_entity(SomeComponent(420), entity_b)
        .unwrap();
    assert_eq!(
        boxed_storage
            .get_mut_storage::<SomeComponent>()
            .unwrap()
            .get_entities(),
        &vec![entity_a, entity_b]
    );
    assert_eq!(
        boxed_storage
            .get_mut_storage::<SomeComponent>()
            .unwrap()
            .get_component_with_entity_of(entity_a),
        Ok(&SomeComponent(69))
    );
    assert_eq!(
        boxed_storage
            .get_mut_storage::<SomeComponent>()
            .unwrap()
            .get_component_with_entity_of(entity_b),
        Ok(&SomeComponent(420))
    );
    boxed_storage
        .get_mut_untyped_storage()
        .remove_component_with_entity(entity_a)
        .unwrap();
    assert_eq!(
        boxed_storage
            .get_mut_storage::<SomeComponent>()
            .unwrap()
            .get_component_with_entity_of(entity_a),
        Err(ECSError::EntityDoesNotHaveComponent(entity_a, ComponentErrorIdentifier::Name(std::any::type_name::<SomeComponent>())))
    );
    assert_eq!(
        boxed_storage
            .get_mut_storage::<SomeComponent>()
            .unwrap()
            .get_component_with_entity_of(entity_b),
        Ok(&SomeComponent(420))
    );
}

