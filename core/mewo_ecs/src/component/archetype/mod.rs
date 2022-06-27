mod access;
mod archetype;
mod locker;
mod storage;

#[cfg(test)]
mod test;

use super::{
    component_group::{ComponentGroup, ComponentGroupManager},
    component_type::ComponentTypeManager,
    query::{ComponentGroupQuery, ComponentQueryAccessType, ComponentQueryFilterType},
    transform::{EntityModify, EntityTransform},
    ComponentGroupId, ComponentTypeId, Entity,
};
use crate::{data::SparseSet, error::*, Id};
use std::sync::atomic::{AtomicU32, Ordering};

pub use access::ArchetypeAccess;
pub type ArchetypeAccessKey = Id;

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
        if self.lock_count.load(Ordering::Relaxed) == 0 {
            Ok(())
        } else {
            Err(RuntimeError::ArchetypeStorageLocked)
        }
    }

    //  For tests.
    pub fn find_component(&self, cty: ComponentTypeId, entity: Entity) -> *const u8 {
        for (_gid, storage) in self.storages.get_dense() {
            if let Ok(ptr) = storage.get(entity, cty) {
                return ptr
            }
        }
        panic!()
    }
}
