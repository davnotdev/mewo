use super::entity::Entity;
use crate::error::{ComponentErrorIdentifier, ECSError, Result};
use crate::SparseSet;
use std::any::Any;

pub struct BoxedStorage {
    storage: Box<dyn UntypedStorage>,
}

impl BoxedStorage {
    pub fn create<C>() -> BoxedStorage
    where
        C: 'static,
    {
        BoxedStorage {
            storage: Box::new(Storage::<C>::create()),
        }
    }

    pub fn get_storage<C>(&self) -> Result<&Storage<C>>
    where
        C: 'static,
    {
        if let Some(storage) = self.storage.as_any().downcast_ref::<Storage<C>>() {
            Ok(storage)
        } else {
            Err(ECSError::ComponentTypeDoesNotExist(
                ComponentErrorIdentifier::Name(std::any::type_name::<C>()),
            ))
        }
    }

    pub fn get_mut_storage<C>(&mut self) -> Result<&mut Storage<C>>
    where
        C: 'static,
    {
        if let Some(storage) = self.storage.as_any_mut().downcast_mut::<Storage<C>>() {
            Ok(storage)
        } else {
            Err(ECSError::ComponentTypeDoesNotExist(
                ComponentErrorIdentifier::Name(std::any::type_name::<C>()),
            ))
        }
    }

    pub fn get_untyped_storage(&self) -> &dyn UntypedStorage {
        self.storage.as_ref()
    }

    pub fn get_mut_untyped_storage(&mut self) -> &mut dyn UntypedStorage {
        self.storage.as_mut()
    }
}

pub trait UntypedStorage {
    fn get_raw_component_with_entity(&self, entity: Entity) -> Result<*const ()>;
    fn remove_component_with_entity(&mut self, entity: Entity) -> Result<()>;
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

pub struct Storage<C> {
    data: SparseSet<C>,
}

impl<C> Storage<C>
where
    C: 'static,
{
    pub fn create() -> Self {
        Storage {
            data: SparseSet::create(),
        }
    }

    pub fn insert_component_with_entity(&mut self, entity: Entity, data: C) -> Result<()> {
        if self.data.get(entity.as_index()).is_some() {
            return Err(ECSError::EntityAlreadyHasComponent(
                entity,
                ComponentErrorIdentifier::Name(std::any::type_name::<C>()),
            ));
        } else {
            self.data.insert(entity.as_index(), data);
            Ok(())
        }
    }

    pub fn get_component_with_entity(&self, entity: Entity) -> Result<&C> {
        if let Some(c) = self.data.get(entity.as_index()) {
            Ok(c)
        } else {
            Err(ECSError::EntityDoesNotHaveComponent(
                entity,
                ComponentErrorIdentifier::Name(std::any::type_name::<C>()),
            ))
        }
    }

    pub fn get_mut_component_with_entity(&mut self, entity: Entity) -> Result<&mut C> {
        if let Some(c) = self.data.get_mut(entity.as_index()) {
            Ok(c)
        } else {
            Err(ECSError::EntityDoesNotHaveComponent(
                entity,
                ComponentErrorIdentifier::Name(std::any::type_name::<C>()),
            ))
        }
    }
}

impl<C> UntypedStorage for Storage<C>
where
    C: 'static,
{
    fn get_raw_component_with_entity(&self, entity: Entity) -> Result<*const ()> {
        if let Some(data) = self.data.get(entity.as_index()) {
            Ok(data as *const C as *const ())
        } else {
            Err(ECSError::EntityDoesNotHaveComponent(
                entity,
                ComponentErrorIdentifier::Name(std::any::type_name::<C>()),
            ))
        }
    }

    fn remove_component_with_entity(&mut self, entity: Entity) -> Result<()> {
        if self.data.remove(entity.as_index()).is_some() {
            Ok(())
        } else {
            Err(ECSError::EntityDoesNotHaveComponent(
                entity,
                ComponentErrorIdentifier::Name(std::any::type_name::<C>()),
            ))
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
        .insert_component_with_entity(entity_a, SomeComponent(69))
        .unwrap();
    assert_eq!(
        boxed_storage
            .get_mut_storage::<SomeComponent>()
            .unwrap()
            .insert_component_with_entity(entity_a, SomeComponent(69)),
        Err(ECSError::EntityAlreadyHasComponent(
            entity_a,
            ComponentErrorIdentifier::Name(std::any::type_name::<SomeComponent>())
        ))
    );
    boxed_storage
        .get_mut_storage::<SomeComponent>()
        .unwrap()
        .insert_component_with_entity(entity_b, SomeComponent(420))
        .unwrap();
    assert_eq!(
        boxed_storage
            .get_mut_storage::<SomeComponent>()
            .unwrap()
            .get_component_with_entity(entity_a),
        Ok(&SomeComponent(69))
    );
    assert_eq!(
        boxed_storage
            .get_mut_storage::<SomeComponent>()
            .unwrap()
            .get_component_with_entity(entity_b),
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
            .get_component_with_entity(entity_a),
        Err(ECSError::EntityDoesNotHaveComponent(
            entity_a,
            ComponentErrorIdentifier::Name(std::any::type_name::<SomeComponent>())
        ))
    );
    assert_eq!(
        boxed_storage
            .get_mut_storage::<SomeComponent>()
            .unwrap()
            .get_component_with_entity(entity_b),
        Ok(&SomeComponent(420))
    );
}
