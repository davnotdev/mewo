use super::component::{Component, ComponentManager, ComponentTypeId};
use super::component_stamp::ComponentStamp;
use super::entity::{BoxedEntityModifyCallback, Entity};
use super::entity_manager::EntityManager;
use super::resource::{BoxedResourceModifyCallback, ResourceManager};
use crate::error::{ECSError, Result};
use crate::SparseSet;
use std::any::TypeId;

pub struct World {
    entity_mgr: EntityManager,
    component_mgr: ComponentManager,
    resource_mgr: ResourceManager,
    //  indexed by entity.id
    entity_dep_info: SparseSet<ComponentStamp>,
}

const ENTITY_DEP_INFO_RESERVE_CONST: usize = 64;

impl World {
    pub fn create() -> World {
        World {
            entity_mgr: EntityManager::create(),
            component_mgr: ComponentManager::create(),
            resource_mgr: ResourceManager::create(),
            entity_dep_info: SparseSet::create_with_capacity(ENTITY_DEP_INFO_RESERVE_CONST),
        }
    }

    pub fn modify_resources(&mut self, callback: &BoxedResourceModifyCallback) {
        callback.call(&mut self.resource_mgr);
    }

    pub fn modify_entity(&mut self, entity: Entity, callback: &BoxedEntityModifyCallback) {
        callback.call(entity, self);
    }

    pub fn insert_entity(&mut self) -> Entity {
        let entity = self.entity_mgr.register_entity();
        self.entity_dep_info
            .insert(entity.as_index(), ComponentStamp::create(&self));
        entity
    }

    pub fn remove_entity(&mut self, entity: Entity) -> Result<()> {
        let dep_info = if let Some(dep_info) = self.entity_dep_info.remove(entity.as_index()) {
            dep_info
        } else {
            return Err(ECSError::EntityDoesNotExist(entity));
        };
        for cid in 0..self.component_mgr.get_component_type_count() {
            if dep_info.get(cid)? {
                self.component_mgr
                    .get_mut_boxed_storage(cid)?
                    .get_mut_untyped_storage()
                    .remove_component_with_entity(entity)?
            }
        }
        self.entity_mgr.deregister_entity(entity).unwrap();
        Ok(())
    }

    pub fn insert_component_with_entity<C>(&mut self, entity: Entity, obj: C) -> Result<()>
    where
        C: 'static + Component,
    {
        let cid = self.get_component_manager().get_component_id_of::<C>()?;
        if let Some(stamp) = self.entity_dep_info.get_mut(entity.as_index()) {
            stamp.stamp(cid)?;
        } else {
            return Err(ECSError::EntityDoesNotExist(entity));
        };
        self.get_mut_component_manager()
            .get_mut_boxed_storage(cid)?
            .get_mut_storage::<C>()?
            .insert_component_with_entity(entity, obj)?;
        Ok(())
    }

    pub fn remove_component_with_entity<C>(&mut self, entity: Entity) -> Result<()>
    where
        C: 'static + Component,
    {
        let cid = self.get_component_manager().get_component_id_of::<C>()?;
        if let Some(stamp) = self.entity_dep_info.get_mut(entity.as_index()) {
            stamp.unstamp(cid)?;
        } else {
            return Err(ECSError::EntityDoesNotExist(entity));
        };
        self.get_mut_component_manager()
            .get_mut_boxed_storage(cid)?
            .get_mut_untyped_storage()
            .remove_component_with_entity(entity)?;
        Ok(())
    }

    pub fn component_id<C>(&self) -> Result<ComponentTypeId>
    where
        C: 'static + Component,
    {
        self.get_component_manager().get_component_id_of::<C>()
    }
}

impl World {
    pub fn get_entity_manager(&self) -> &EntityManager {
        &self.entity_mgr
    }

    pub fn get_resource_manager(&self) -> &ResourceManager {
        &self.resource_mgr
    }

    pub fn get_component_manager(&self) -> &ComponentManager {
        &self.component_mgr
    }

    pub fn get_mut_component_manager(&mut self) -> &mut ComponentManager {
        &mut self.component_mgr
    }

    pub fn get_entity_dep_info(&self, entity: Entity) -> Result<&ComponentStamp> {
        if let Some(s) = self.entity_dep_info.get(entity.id as usize) {
            Ok(s)
        } else {
            Err(ECSError::EntityDoesNotExist(entity))
        }
    }

    pub fn get_raw_component_with_type_id_and_entity(
        &self,
        ty: TypeId,
        entity: Entity,
    ) -> Result<*const ()> {
        self.component_mgr
            .get_boxed_storage(self.component_mgr.get_component_id(ty)?)?
            .get_untyped_storage()
            .get_raw_component_with_entity(entity)
    }
}
