use super::{ComponentHash, ComponentTypeId};
use crate::{
    data::{ValueClone, ValueDrop},
    debug::prelude::*,
};
use std::collections::HashMap;

#[derive(Debug)]
pub struct ComponentTypeEntry {
    pub size: usize,
    pub name: String,
    pub hash: ComponentHash,
    pub drop: ValueDrop,
    pub clone: ValueClone,
}

#[derive(Debug)]
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
            Err(InternalError {
                line: line!(),
                file: file!(),
                dumps: vec![DebugDumpTargets::ComponentTypeManager],
                ty: InternalErrorType::DuplicateComponentTypeHash { hash },
                explain: Some("Hashes should be handled by the burrito."),
            })?
        }
        self.component_tys.push(entry);
        let id = self.component_tys.len() - 1;
        self.hash_map.insert(hash, id);
        debug_dump_changed(self);
        Ok(id)
    }

    pub fn get(&self, id: ComponentTypeId) -> Result<&ComponentTypeEntry> {
        if let Some(e) = self.component_tys.get(id) {
            Ok(e)
        } else {
            Err(InternalError {
                line: line!(),
                file: file!(),
                dumps: vec![DebugDumpTargets::ComponentTypeManager],
                ty: InternalErrorType::BadComponentType { ctyid: id },
                explain: Some("This component is not registered."),
            })?
        }
    }

    pub fn get_id_with_hash(&self, hash: ComponentHash) -> Result<ComponentTypeId> {
        self.hash_map
            .get(&hash)
            .map(|cty| *cty)
            .ok_or(InternalError {
                line: line!(),
                file: file!(),
                dumps: vec![DebugDumpTargets::ComponentTypeManager],
                ty: InternalErrorType::BadComponentTypeHash { hash },
                explain: Some("This component is not registered."),
            })
    }
}

impl TargetedDump for ComponentTypeManager {
    fn target() -> DebugDumpTargets {
        DebugDumpTargets::ComponentTypeManager
    }
}
