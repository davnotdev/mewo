use sparseset::SparseSet;
use super::error::{
    Result, 
    ECSError,
};
use super::entity::{
    Entity,
    EntityWrapper
};
use super::resource::{
    ResourceManager,
    ResourceModifyCallback,
};
use super::component_manager::ComponentManager;
use super::component_stamp::ComponentStamp;
use super::entity_manager::EntityManager;

pub type EntityModifyCallback = fn(EntityWrapper);

pub struct World {
    entity_mgr: EntityManager,
    component_mgr: ComponentManager,
    resource_mgr: ResourceManager,
//  indexed by entity.id
    entity_dep_info: SparseSet<ComponentStamp>,
    world_changed: bool,
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

    pub fn modify_resources(&mut self, callback: ResourceModifyCallback) {
        (callback)(&mut self.resource_mgr)
    }

    pub fn modify_entity(&mut self, entity: Entity, callback: EntityModifyCallback) {
        let wrapper = EntityWrapper::create(entity, self);
        (callback)(wrapper);
    }

    pub fn insert_entity(&mut self, callback: Option<EntityModifyCallback>) -> Entity {
        let entity = self.entity_mgr.register_entity();
        self.entity_dep_info.insert(entity.as_index(), ComponentStamp::create(&self));
        if let Some(callback) = callback {
            self.modify_entity(entity, callback);
        }
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
        self.world_changed = true;
    }

    pub fn insert_component_with_entity<C>(&mut self, entity: Entity, obj: C) 
        where C: 'static
    {
        self.get_mut_component_manager()
            .get_mut_boxed_storage_of::<C>()
            .get_mut_storage::<C>()
            .insert_component_with_entity(obj, entity)
            .unwrap();
        let cid = self.get_component_manager()
            .get_component_id::<C>()
            .unwrap();
        self.entity_dep_info.get_mut(entity.as_index())
            .unwrap()
            .stamp(cid);
        self.world_changed = true;
    }

    pub fn remove_component_with_entity<C>(&mut self, entity: Entity) 
        where C: 'static
    {
        self.get_mut_component_manager()
            .get_mut_boxed_storage_of::<C>()
            .get_mut_untyped_storage()
            .remove_component_with_entity(entity)
            .unwrap();
        let cid = self.get_component_manager()
            .get_component_id::<C>()
            .unwrap();
        self.entity_dep_info.get_mut(entity.as_index())
            .unwrap()
            .unstamp(cid);
        self.world_changed = true;
    }

    pub fn is_world_changed(&self) -> bool {
        self.world_changed
    }
 
    pub fn reset_world_changed(&mut self) {
        self.world_changed = false;
    }
}

impl World {
    pub fn get_component_with_entity<C>(&self, entity: Entity) -> &C 
        where C: 'static
    {
        self.get_component_manager()
            .get_boxed_storage_of::<C>()
            .get_storage::<C>()
            .get_component_with_entity(entity)
            .unwrap()
    }

    pub fn get_mut_component_with_entity<C>(&mut self, entity: Entity) -> &mut C 
        where C: 'static
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
