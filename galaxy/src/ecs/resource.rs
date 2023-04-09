use super::error::*;
use crate::data::{TVal, TypeEntry};
use parking_lot::{Mutex, RwLock};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct ResourceId(u64);

impl ResourceId {
    pub fn from_hash(val: u64) -> Self {
        ResourceId(val)
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct ResourceTypeId(u64);

impl ResourceTypeId {
    pub fn from_hash(val: u64) -> Self {
        ResourceTypeId(val)
    }
}

type ResourceTypeItem = (TypeEntry, HashMap<ResourceId, RwLock<Option<TVal>>>);

#[derive(Debug)]
pub struct ResourcePlanet {
    //  Used when locking rwlocks to prevent abba problem.
    global_lock: Mutex<()>,
    resources: HashMap<ResourceTypeId, ResourceTypeItem>,
}

impl ResourcePlanet {
    pub fn new() -> Self {
        ResourcePlanet {
            global_lock: Mutex::new(()),
            resources: HashMap::new(),
        }
    }

    pub fn insert_id(&mut self, tid: ResourceTypeId, id: ResourceId, ty: TypeEntry) -> Result<()> {
        self.resources
            .entry(tid)
            .or_insert_with(|| (ty.clone(), HashMap::new()));
        let resource_ty = self.resources.get_mut(&tid).unwrap();
        if resource_ty.1.contains_key(&id) {
            return Err(ecs_err!(
                ErrorType::ResourcePlanetInsertType { tid, ty },
                self
            ));
        }
        resource_ty.1.insert(id, RwLock::new(None));
        Ok(())
    }

    pub fn get_type(&self, tid: ResourceTypeId) -> Option<&TypeEntry> {
        let _lock = self.global_lock.lock();
        self.resources.get(&tid).as_ref().map(|(ty, _)| ty)
    }

    pub fn get_read_lock(&self, tid: ResourceTypeId, id: ResourceId) -> Result<&Option<TVal>> {
        let _lock = self.global_lock.lock();
        let lock = &self
            .resources
            .get(&tid)
            .ok_or(ecs_err!(ErrorType::ResourcePlanetTypeAccess { tid }, self))?
            .1
            .get(&id)
            .ok_or(ecs_err!(ErrorType::ResourcePlanetAccess { id }, self))?;
        std::mem::forget(lock.read());
        Ok(unsafe { &*lock.data_ptr() })
    }

    pub fn get_read_unlock(&self, tid: ResourceTypeId, id: ResourceId) -> Result<()> {
        unsafe {
            self.resources
                .get(&tid)
                .ok_or(ecs_err!(ErrorType::ResourcePlanetTypeAccess { tid }, self))?
                .1
                .get(&id)
                .ok_or(ecs_err!(ErrorType::ResourcePlanetAccess { id }, self))?
                .force_unlock_read()
        }
        Ok(())
    }

    pub fn get_write_lock(&self, tid: ResourceTypeId, id: ResourceId) -> Result<&mut Option<TVal>> {
        let _lock = self.global_lock.lock();
        let lock = &self
            .resources
            .get(&tid)
            .ok_or(ecs_err!(ErrorType::ResourcePlanetTypeAccess { tid }, self))?
            .1
            .get(&id)
            .ok_or(ecs_err!(ErrorType::ResourcePlanetAccess { id }, self))?;
        std::mem::forget(lock.write());
        Ok(unsafe { &mut *lock.data_ptr() })
    }

    pub fn get_write_unlock(&self, tid: ResourceTypeId, id: ResourceId) -> Result<()> {
        unsafe {
            self.resources
                .get(&tid)
                .ok_or(ecs_err!(ErrorType::ResourcePlanetTypeAccess { tid }, self))?
                .1
                .get(&id)
                .ok_or(ecs_err!(ErrorType::ResourcePlanetAccess { id }, self))?
                .force_unlock_write()
        }
        Ok(())
    }
}
