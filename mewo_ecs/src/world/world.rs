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

    pub fn modify_entity(&mut self, entity_mod: &mut EntityModifierStore) {
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
                    self.insert_component_with_entity(entity, data.unwrap(), cid);
                }
                EntityComponentModifyType::Remove => {
                    self.remove_component_with_entity(entity, cid);
                },
            };
        }
        self.world_changed = true;
    }

    pub fn insert_entity(&mut self) -> Entity {
        let entity = self.entity_mgr.register_entity();
        self.entity_dep_info.insert(entity.as_index(), ComponentStamp::create(&self));
        self.world_changed = true;
        entity
    }

    pub fn remove_entity(&mut self, entity: Entity) {
        let dep_info = self.entity_dep_info.remove(entity.as_index())
            .unwrap();
        for cid in 0..dep_info.get_mask().get_len() {
            if dep_info.get(cid) {
                self.component_mgr
                    .get_mut_boxed_storage(cid)
                    .get_mut_untyped_storage()
                    .remove_component_with_entity(entity)
                    .unwrap();
            }
        }
        self.entity_mgr.deregister_entity(entity).unwrap();
        self.world_changed = true;
    }

    pub fn insert_component_with_entity(&mut self, entity: Entity, obj: Box<dyn Any>, cid: ComponentTypeId) {
        self.get_mut_component_manager()
            .get_mut_boxed_storage(cid)
            .get_mut_untyped_storage()
            .insert_component_with_entity(entity, obj)
            .unwrap();
        self.entity_dep_info.get_mut(entity.as_index())
            .unwrap()
            .stamp(cid);
        self.world_changed = true;
    }

    pub fn remove_component_with_entity(&mut self, entity: Entity, cid: ComponentTypeId) {
        self.get_mut_component_manager()
            .get_mut_boxed_storage(cid)
            .get_mut_untyped_storage()
            .remove_component_with_entity(entity)
            .unwrap();
        self.entity_dep_info.get_mut(entity.as_index())
            .unwrap()
            .unstamp(cid);
        self.world_changed = true;
    }

    pub fn component<C>(&self) -> ComponentTypeId
        where C: 'static + Component
    {
        self.get_component_manager()
            .get_component_id_of::<C>()
            .unwrap()
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

    pub fn get_component_with_entity<C>(&self, entity: Entity) -> &C 
        where C: 'static + Component
    {
        self.get_component_manager()
            .get_boxed_storage_of::<C>()
            .get_storage::<C>()
            .get_component_with_entity_of(entity)
            .unwrap()
    }

    pub fn get_mut_component_with_entity<C>(&mut self, entity: Entity) -> &mut C 
        where C: 'static + Component
    {
        self.get_mut_component_manager()
            .get_mut_boxed_storage_of::<C>()
            .get_mut_storage::<C>()
            .get_mut_component_with_entity(entity)
            .unwrap()

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
