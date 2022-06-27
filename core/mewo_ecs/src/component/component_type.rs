use super::{ComponentHash, ComponentTypeId};
use crate::{
    data::{TValCloneFunction, TValDropFunction},
    error::*,
};
use std::collections::HashMap;

pub struct ComponentTypeEntry {
    pub size: usize,
    pub name: String,
    pub hash: ComponentHash,
    pub drop: TValDropFunction,
    pub clone: TValCloneFunction,
}

pub struct ComponentTypeManager {
    component_tys: Vec<ComponentTypeEntry>,
    hash_map: HashMap<ComponentHash, ComponentTypeId>,
}

impl ComponentTypeManager {
    pub fn create() -> ComponentTypeManager {
        ComponentTypeManager {
            component_tys: Vec::new(),
            hash_map: HashMap::new(),
        }
    }

    pub fn register(&mut self, entry: ComponentTypeEntry) -> Result<ComponentTypeId> {
        let hash = entry.hash;
        if self.hash_map.contains_key(&hash) {
            Err(RuntimeError::DuplicateComponentTypeHash { hash })?
        }
        self.component_tys.push(entry);
        let id = self.component_tys.len() - 1;
        self.hash_map.insert(hash, id);
        Ok(id)
    }

    pub fn get(&self, id: ComponentTypeId) -> Result<&ComponentTypeEntry> {
        if let Some(e) = self.component_tys.get(id) {
            Ok(e)
        } else {
            Err(RuntimeError::BadComponentType { ctyid: id })
        }
    }

    pub fn get_id_with_hash(&self, hash: ComponentHash) -> Result<ComponentTypeId> {
        self.hash_map
            .get(&hash)
            .map(|cty| *cty)
            .ok_or(RuntimeError::BadComponentTypeHash { hash })
    }
}
