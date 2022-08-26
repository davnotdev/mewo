mod access;
mod archetype;
mod storage;

#[cfg(test)]
mod test;

use super::{
    component::ComponentTypeManager,
    component_group::{ComponentGroup, ComponentGroupManager},
    query::{ComponentGroupQuery, ComponentQueryAccessType, ComponentQueryFilterType},
    transform::{EntityModify, EntityTransform},
    ComponentGroupId, ComponentTypeId, Entity,
};
use crate::{data::SparseSet, debug::prelude::*, Id};
use std::sync::atomic::{AtomicU32, Ordering};

pub use access::ArchetypeAccess;
pub type ArchetypeAccessKey = Id;

#[derive(Debug)]
pub struct ArchetypeManager {
    lock_count: AtomicU32,
    storages: SparseSet<ComponentGroupId, storage::ArchetypeStorage>,
    entity_set: SparseSet<Entity, ComponentGroupId>,
    cgmgr: ComponentGroupManager,
    akmgr: access::ArchetypeAccessKeyManager,
}

impl ArchetypeManager {
    //  https://areweyeetyet.rs/    ~ Proof Rust is the best language.
    fn yeet_locked(&self) -> Result<()> {
        if self.lock_count.load(Ordering::Acquire) == 0 {
            Ok(())
        } else {
            Err(InternalError {
                line: line!(),
                file: file!(),
                dumps: vec![DebugDumpTargets::ArchetypeManager],
                ty: InternalErrorType::ArchetypeStorageLocked,
                explain: None,
            })
        }
    }

    //  For tests.
    pub fn find_component(&self, cty: ComponentTypeId, entity: Entity) -> *const u8 {
        for (_gid, storage) in self.storages.get_dense() {
            if let Ok(ptr) = storage.get(entity, cty) {
                return ptr;
            }
        }
        panic!()
    }
}

impl TargetedDump for ArchetypeManager {
    fn target() -> DebugDumpTargets {
        DebugDumpTargets::ArchetypeManager
    }
}
