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

#[derive(Debug)]
pub struct ResourcePlanet {
    //  Used when locking rwlocks to prevent abba problem.
    global_lock: Mutex<()>,
    resources: HashMap<ResourceId, (TypeEntry, RwLock<Option<TVal>>)>,
}

impl ResourcePlanet {
    pub fn new() -> Self {
        ResourcePlanet {
            global_lock: Mutex::new(()),
            resources: HashMap::new(),
        }
    }

    pub fn insert_type(&mut self, id: ResourceId, ty: TypeEntry) -> Result<()> {
        if self.resources.contains_key(&id) {
            Err(ecs_err!(
                ErrorType::ResourcePlanetInsertType { id, ty: ty.clone() },
                self
            ))?
        }
        self.resources.insert(id, (ty, RwLock::new(None)));
        Ok(())
    }

    pub fn get_type(&self, id: ResourceId) -> Option<&TypeEntry> {
        let _lock = self.global_lock.lock();
        self.resources.get(&id).as_ref().map(|(ty, _)| ty)
    }

    pub fn get_read_lock(&self, id: ResourceId) -> Result<&Option<TVal>> {
        let _lock = self.global_lock.lock();
        let lock = &self
            .resources
            .get(&id)
            .ok_or(ecs_err!(ErrorType::ResourcePlanetAccess { id }, self))?
            .1;
        std::mem::forget(lock.read());
        Ok(unsafe { &*lock.data_ptr() })
    }

    pub fn get_read_unlock(&self, id: ResourceId) -> Result<()> {
        unsafe {
            self.resources
                .get(&id)
                .ok_or(ecs_err!(ErrorType::ResourcePlanetAccess { id }, self))?
                .1
                .force_unlock_read()
        }
        Ok(())
    }

    pub fn get_write_lock(&self, id: ResourceId) -> Result<&mut Option<TVal>> {
        let _lock = self.global_lock.lock();
        let lock = &self
            .resources
            .get(&id)
            .ok_or(ecs_err!(ErrorType::ResourcePlanetAccess { id }, self))?
            .1;
        std::mem::forget(lock.write());
        Ok(unsafe { &mut *lock.data_ptr() })
    }

    pub fn get_write_unlock(&self, id: ResourceId) -> Result<()> {
        unsafe {
            self.resources
                .get(&id)
                .ok_or(ecs_err!(ErrorType::ResourcePlanetAccess { id }, self))?
                .1
                .force_unlock_write()
        }
        Ok(())
    }
}
