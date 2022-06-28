use crate::{
    component::{ArchetypeAccessKey, ComponentGroupId, ComponentHash, ComponentTypeId, Entity},
    event::EventHash,
};

pub type Result<T> = std::result::Result<T, RuntimeError>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeError {
    BadEntity {
        e: Entity,
    },
    BadComponentType {
        ctyid: ComponentTypeId,
    },
    BadComponentTypeName {
        name: String,
    },
    BadComponentTypeHash {
        hash: u64,
    },
    DuplicateComponentTypeHash {
        hash: ComponentHash,
    },
    BadComponentGroup {
        gid: ComponentGroupId,
    },

    ArchetypeStorageInsertIncomplete {
        missing: Vec<ComponentTypeId>,
    },

    BadArchetypeManagerTransformEntityInsertComponent {
        target: Entity,
        insert: ComponentTypeId,
    },
    BadArchetypeManagerTransformEntityRemoveComponent {
        target: Entity,
        remove: ComponentTypeId,
    },

    BadArchetypeManagerAccessIndex {
        idx: usize,
        max: usize,
    },

    BadArchetypeAccessKey {
        akid: ArchetypeAccessKey,
    },

    ArchetypeStorageLocked,

    PluginNoName,
    PluginNoSystems,
    PluginDependsOnSelf {
        plugin: String,
    },
    PluginDependenciesNoMet {
        plugin: String,
        unmet: Vec<String>,
    },

    SystemNoPluginName {
        system: String,
    },

    DuplicateEventTypeHash {
        hash: EventHash,
    },

    BadEventTypeHash {
        hash: EventHash,
    },

    BadEventStorageGetIndex {
        idx: usize,
    },
}
