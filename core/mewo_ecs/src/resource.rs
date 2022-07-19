use super::{
    data::{CentralLock, TVal},
    unbug::prelude::*,
    HashType,
};
use std::collections::HashMap;

pub type ResourceHash = HashType;

#[derive(Clone, Copy)]
pub enum ResourceQueryAccessType {
    Read,
    Write,
}

//  Similar to Events, resources have no need to be cloned.
#[derive(Debug)]
pub struct ResourceTypeEntry {
    pub name: String,
    pub hash: ResourceHash,
}

#[derive(Debug)]
struct Resource {
    entry: ResourceTypeEntry,
    val: Option<TVal>,
}

#[derive(Debug)]
pub struct ResourceManager {
    lock: CentralLock,
    hash_map: HashMap<ResourceHash, Resource>,
}

impl ResourceManager {
    pub fn create() -> Self {
        ResourceManager {
            lock: CentralLock::create(),
            hash_map: HashMap::new(),
        }
    }

    pub fn register(&mut self, entry: ResourceTypeEntry) -> Result<()> {
        if self.hash_map.contains_key(&entry.hash) {
            Err(InternalError {
                line: line!(),
                file: file!(),
                dumps: vec![DebugDumpTargets::Resources],
                explain: Some("Hashes should be handled by the burrito."),
                ty: InternalErrorType::DuplicateResourceTypeHash { hash: entry.hash },
            })?
        }
        self.hash_map
            .insert(entry.hash, Resource { entry, val: None });
        debug_dump_changed(self);
        Ok(())
    }

    pub fn get_type(&self, hash: ResourceHash) -> Result<&ResourceTypeEntry> {
        Ok(&self
            .hash_map
            .get(&hash)
            .ok_or(InternalError {
                line: line!(),
                file: file!(),
                dumps: vec![DebugDumpTargets::Resources],
                ty: InternalErrorType::BadResourceTypeHash { hash },
                explain: Some(
                    "
                    This error should pretty much never occur since 
                    `ResourceManager::get_type` is pretty much never used.",
                ),
            })?
            .entry)
    }

    pub fn locked_get(&self, hash: ResourceHash) -> Result<&Option<TVal>> {
        Ok(&self
            .hash_map
            .get(&hash)
            .ok_or(InternalError {
                line: line!(),
                file: file!(),
                dumps: vec![DebugDumpTargets::Resources],
                ty: InternalErrorType::BadResourceTypeHash { hash },
                explain: Some("Failed on shared resource lock."),
            })?
            .val)
    }

    fn unsafe_get_mut_map(&self) -> &mut HashMap<ResourceHash, Resource> {
        unsafe {
            &mut *(&self.hash_map as *const HashMap<ResourceHash, Resource>
                as *mut HashMap<ResourceHash, Resource>)
        }
    }

    pub fn locked_get_mut(&self, hash: ResourceHash) -> Result<&mut Option<TVal>> {
        Ok(&mut self
            .unsafe_get_mut_map()
            .get_mut(&hash)
            .ok_or(InternalError {
                line: line!(),
                file: file!(),
                dumps: vec![DebugDumpTargets::Resources],
                ty: InternalErrorType::BadResourceTypeHash { hash },
                explain: Some("Failed on mutable resource lock."),
            })?
            .val)
    }

    pub fn lock(&self, rcqat: ResourceQueryAccessType) {
        match rcqat {
            ResourceQueryAccessType::Read => self.lock.lock_read(),
            ResourceQueryAccessType::Write => self.lock.lock_write(),
        }
    }

    pub fn unlock(&self, rcqat: ResourceQueryAccessType) {
        match rcqat {
            ResourceQueryAccessType::Read => self.lock.unlock_read(),
            ResourceQueryAccessType::Write => self.lock.unlock_write(),
        };
    }
}

impl TargetedDump for ResourceManager {
    fn target() -> DebugDumpTargets {
        DebugDumpTargets::Resources
    }
}
