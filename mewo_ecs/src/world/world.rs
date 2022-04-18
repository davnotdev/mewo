use sparseset::SparseSet;
use std::any::Any;
use crate::error::{
    Result, 
    ECSError,
};
use super::entity::{
    Entity,
    EntityModifierStore,
    EntityModifierHandle,
    EntityComponentModifyType,
};
use super::resource::{
    ResourceManager,
    BoxedResourceModifyCallback,
};
use super::component::{
    Component,
    ComponentTypeId,
    ComponentManager,
};
use super::component_stamp::ComponentStamp;
use super::entity_manager::EntityManager;

pub struct World {
    entity_mgr: EntityManager,
    component_mgr: ComponentManager,
    resource_mgr: ResourceManager,
//  indexed by entity.id
    entity_dep_info: SparseSet<ComponentStamp>, world_changed: bool,
}

const ENTITY_DEP_INFO_RESERVE_CONST: usize = 64;

impl World {
    pub fn create() -> World {
        World {
            entity_mgr: EntityManager::create(),
            component_mgr: ComponentManager::create(),
            resource_mgr: ResourceManager::create(),
            entity_dep_info: SparseSet::with_capacity(ENTITY_DEP_INFO_RESERVE_CONST),
            world_changed: true,
        }
    }

    pub fn modify_resources(&mut self, callback: &BoxedResourceModifyCallback) {
        callback.call(&mut self.resource_mgr);
    }

    pub fn modify_entity(&mut self, entity_mod: &mut EntityModifierStore) -> Result<()> {
        let entity = if let EntityModifierHandle::Modify(e) = entity_mod.entity {
            e
        } else {
            self.insert_entity()
        };
        for component_mod in entity_mod.components.iter_mut() {
            let cid = component_mod.cid;
            match &mut component_mod.modify {
                EntityComponentModifyType::Insert(insert) => {
                    let data = std::mem::replace(insert, None);
                    self.insert_component_with_entity(entity, data.unwrap(), cid)?;
                }
                EntityComponentModifyType::Remove => {
                    self.remove_component_with_entity(entity, cid)?;
                },
            };
        }
        self.world_changed = true;
        Ok(())
    }

    pub fn insert_entity(&mut self) -> Entity {
        let entity = self.entity_mgr.register_entity();
        self.entity_dep_info.insert(entity.as_index(), ComponentStamp::create(&self));
        self.world_changed = true;
        entity
    }

    pub fn remove_entity(&mut self, entity: Entity) -> Result<()> {
        let dep_info = if let Some(dep_info) = self.entity_dep_info.remove(entity.as_index()) {
            dep_info
        } else {
            return Err(ECSError::EntityDoesNotExist(entity))
        };
        for cid in 0..dep_info.get_mask().get_len() {
            if dep_info.get(cid) {
                self.component_mgr
                    .get_mut_boxed_storage(cid)?
                    .get_mut_untyped_storage()
                    .remove_component_with_entity(entity)?
            }
        }
        self.entity_mgr.deregister_entity(entity).unwrap();
        self.world_changed = true;
        Ok(())
    }

    pub fn insert_component_with_entity(&mut self, entity: Entity, obj: Box<dyn Any>, cid: ComponentTypeId) -> Result<()> {
        if let Some(stamp) = self.entity_dep_info.get_mut(entity.as_index()) {
            stamp.stamp(cid);
        } else {
            return Err(ECSError::EntityDoesNotExist(entity))
        };
        self.get_mut_component_manager()
            .get_mut_boxed_storage(cid)?
            .get_mut_untyped_storage()
            .insert_component_with_entity(entity, obj)?;
        self.world_changed = true;
        Ok(())
    }

    pub fn remove_component_with_entity(&mut self, entity: Entity, cid: ComponentTypeId) -> Result<()> {
        if let Some(stamp) = self.entity_dep_info.get_mut(entity.as_index()) {
            stamp.unstamp(cid);
        } else {
            return Err(ECSError::EntityDoesNotExist(entity))
        };
        self.get_mut_component_manager()
            .get_mut_boxed_storage(cid)?
            .get_mut_untyped_storage()
            .remove_component_with_entity(entity)?;
        self.world_changed = true;
        Ok(())
    }

    pub fn component<C>(&self) -> Result<ComponentTypeId>
        where C: 'static + Component
    {
        self.get_component_manager()
            .get_component_id_of::<C>()
    }

    pub fn is_world_changed(&self) -> bool {
        self.world_changed
    }
 
    pub fn reset_world_changed(&mut self) {
        self.world_changed = false;
    }
}

impl World {
    pub fn get_resource_manager(&self) -> &ResourceManager {
        &self.resource_mgr
    }

    pub fn get_component_with_entity<C>(&self, entity: Entity) -> Result<&C>
        where C: 'static + Component
    {
        self.get_component_manager()
            .get_boxed_storage_of::<C>()?
            .get_storage::<C>()?
            .get_component_with_entity_of(entity)
    }

    pub fn get_mut_component_with_entity<C>(&mut self, entity: Entity) -> Result<&mut C>
        where C: 'static + Component
    {
        self.get_mut_component_manager()
            .get_mut_boxed_storage_of::<C>()?
            .get_mut_storage::<C>()?
            .get_mut_component_with_entity(entity)

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
}
