use super::{
    data::{TVal, ValueDrop},
    error::*,
    HashType,
};
use std::collections::HashMap;

pub type ResourceHash = HashType;

//  Similar to Events, resources have no need to be cloned.
pub struct ResourceTypeEntry {
    pub size: usize,
    pub name: String,
    pub hash: ResourceHash,
    pub drop: ValueDrop,
}

pub struct ResourceManager {
    hash_map: HashMap<ResourceHash, (ResourceTypeEntry, Option<TVal>)>,
}

impl ResourceManager {
    pub fn create() -> Self {
        ResourceManager {
            hash_map: HashMap::new(),
        }
    }

    pub fn register(&mut self, entry: ResourceTypeEntry) -> Result<()> {
        if self.hash_map.contains_key(&entry.hash) {
            Err(RuntimeError::DuplicateResourceTypeHash { hash: entry.hash })?
        }
        self.hash_map.insert(entry.hash, (entry, None));
        Ok(())
    }

    pub fn get_type(&self, hash: ResourceHash) -> Result<&ResourceTypeEntry> {
        Ok(&self
            .hash_map
            .get(&hash)
            .ok_or(RuntimeError::BadResourceTypeHash { hash })?
            .0)
    }

    pub fn get_resource(&self, hash: ResourceHash) -> Result<Option<&TVal>> {
        Ok(self
            .hash_map
            .get(&hash)
            .ok_or(RuntimeError::BadResourceTypeHash { hash })?
            .1
            .as_ref())
    }

    pub fn get_mut_resource(&mut self, hash: ResourceHash) -> Result<Option<&mut TVal>> {
        Ok(self
            .hash_map
            .get_mut(&hash)
            .ok_or(RuntimeError::BadResourceTypeHash { hash })?
            .1
            .as_mut())
    }

    pub fn insert(&mut self, hash: ResourceHash, val: TVal) -> Result<()> {
        let (_, current) = self
            .hash_map
            .get_mut(&hash)
            .ok_or(RuntimeError::BadResourceTypeHash { hash })?;
        *current = Some(val);
        Ok(())
    }

    pub fn remove(&mut self, hash: ResourceHash) -> Result<()> {
        let (_, current) = self
            .hash_map
            .get_mut(&hash)
            .ok_or(RuntimeError::BadResourceTypeHash { hash })?;
        *current = None;
        Ok(())
    }

    pub fn flush(&mut self, modifies: &mut ResourceModify) {
        for modify in modifies.get() {
            modify.call(self);
        }
        modifies.flush();
    }
}

pub struct ResourceModifyFunction<F>(pub F);
pub trait GenericResourceModifyFunction {
    fn call(&self, rcmgr: &mut ResourceManager);
}

impl<F> GenericResourceModifyFunction for ResourceModifyFunction<F>
where
    F: Fn(&mut ResourceManager) -> (),
{
    fn call(&self, rcmgr: &mut ResourceManager) {
        (self.0)(rcmgr)
    }
}

pub struct ResourceModify {
    modifies: Vec<Box<dyn GenericResourceModifyFunction>>,
}

impl ResourceModify {
    pub fn create() -> Self {
        ResourceModify {
            modifies: Vec::new(),
        }
    }

    pub fn insert(&mut self, f: Box<dyn GenericResourceModifyFunction>) {
        self.modifies.push(f);
    }

    pub(self) fn get(&self) -> &Vec<Box<dyn GenericResourceModifyFunction>> {
        &self.modifies
    }

    fn flush(&mut self) {
        self.modifies.clear();
    }
}
