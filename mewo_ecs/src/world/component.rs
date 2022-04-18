use crate::error::{ 
    Result, 
    ECSError, 
    ComponentErrorIdentifier 
};
use std::any::TypeId;
use std::collections::HashMap;
use sparseset::SparseSet;
use super::storage::BoxedStorage;

pub trait Component {
}

pub type ComponentTypeId = usize;
const COMPONENT_MANAGER_STORAGE_RESERVE_COUNT: usize = 32;

pub struct ComponentManager {
    storages: SparseSet<BoxedStorage>,
    component_types: HashMap<TypeId, ComponentTypeId>,
}

impl ComponentManager {
    pub fn create() -> ComponentManager {
        ComponentManager {
            storages: SparseSet::with_capacity(COMPONENT_MANAGER_STORAGE_RESERVE_COUNT),
            component_types: HashMap::new(),
        }
    } 

    pub fn register_component_type<C>(&mut self) -> Result<ComponentTypeId> 
        where C: 'static + Component
    {
        let t = TypeId::of::<C>();
        if self.component_types.contains_key(&t) {
            return Err(ECSError::ComponentTypeExists(ComponentErrorIdentifier::Name(std::any::type_name::<C>())))
        }
        let id = self.get_component_type_count();
        self.component_types.insert(t, id);
        self.storages.insert(id, BoxedStorage::create::<C>());
        Ok(id)
    }

    pub fn get_component_id(&self, ty: TypeId) -> Result<ComponentTypeId> {
        match self.component_types.get(&ty) {
            Some(id) => Ok(*id),
            None => Err(ECSError::ComponentTypeDoesNotExist(ComponentErrorIdentifier::Unknown)),
        }
    }

    pub fn get_component_id_of<C>(&self) -> Result<ComponentTypeId> 
        where C: 'static + Component
    {
        match self.component_types.get(&TypeId::of::<C>()) {
            Some(id) => Ok(*id),
            None => Err(ECSError::ComponentTypeDoesNotExist(ComponentErrorIdentifier::Name(std::any::type_name::<C>()))),
        }
    }

    pub fn get_component_types(&self) -> &HashMap<TypeId, ComponentTypeId> {
        &self.component_types
    }

    pub fn get_component_type_count(&self) -> ComponentTypeId {
        self.component_types.len()
    }

    pub fn get_boxed_storage_of<C>(&self) -> Result<&BoxedStorage>
        where C: 'static + Component
    {
        if let Some(storage) = self.storages.get(self.component_types[&TypeId::of::<C>()]) {
            Ok(storage)
        } else {
            Err(ECSError::ComponentTypeDoesNotExist(ComponentErrorIdentifier::Name(std::any::type_name::<C>())))
        }
    }

    pub fn get_mut_boxed_storage_of<C>(&mut self) -> Result<&mut BoxedStorage>
        where C: 'static + Component
    {
        if let Some(storage) = self.storages.get_mut(self.component_types[&TypeId::of::<C>()]) {
            Ok(storage)
        } else {
            Err(ECSError::ComponentTypeDoesNotExist(ComponentErrorIdentifier::Name(std::any::type_name::<C>())))
        }
    }

    pub fn get_boxed_storage(&self, id: ComponentTypeId) -> Result<&BoxedStorage> {
        if let Some(storage) = self.storages.get(id) {
            Ok(storage)
        } else {
            Err(ECSError::ComponentTypeDoesNotExist(ComponentErrorIdentifier::Id(id)))
        }
    }

    pub fn get_mut_boxed_storage(&mut self, id: ComponentTypeId) -> Result<&mut BoxedStorage> {
        if let Some(storage) = self.storages.get_mut(id) {
            Ok(storage)
        } else {
            Err(ECSError::ComponentTypeDoesNotExist(ComponentErrorIdentifier::Id(id)))
        }
    }
}

