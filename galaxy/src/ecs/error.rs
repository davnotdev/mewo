use super::{ComponentGroupId, ComponentTypeId, Entity, EventId, QueryAccess, QueryId, ResourceId};
pub use crate::data::TypeEntry;

pub type Result<T> = std::result::Result<T, ECSError>;

#[derive(Debug, Clone)]
pub struct ECSError {
    pub error: ErrorType,
    pub snap: String,
    pub line: u32,
    pub file: &'static str,
}

#[derive(Debug, Clone)]
pub enum ErrorType {
    EntityPlanetRemove {
        entity: Entity,
    },
    ComponentTypePlanetInsertType {
        id: ComponentTypeId,
        ty: TypeEntry,
    },
    ComponentTypePlanetGetType {
        id: ComponentTypeId,
    },
    EventPlanetInsert {
        id: EventId,
        ty: TypeEntry,
    },
    EventPlanetModify,
    EventPlanetGetEvents {
        id: EventId,
    },
    ResourcePlanetInsertType {
        id: ResourceId,
        ty: TypeEntry,
    },
    ResourcePlanetAccess {
        id: ResourceId,
    },
    StorageBlocRemove {
        entity: Entity,
    },
    StorageBlocInsertComponent {
        entity: Entity,
        id: ComponentTypeId,
    },
    StorageBlocCopyEntity {
        entity: Entity,
    },
    StoragePlanetInsertEntity {
        entity: Entity,
    },
    StoragePlanetRemoveEntity {
        entity: Entity,
    },
    StoragePlanetUpdate {
        id: ComponentGroupId,
    },
    StoragePlanetTransformEntity {
        entity: Entity,
    },
    StoragePlanetTransformGroup {
        entity: Entity,
        old_gid: ComponentGroupId,
    },
    StoragePlanetAccess {
        id: ComponentGroupId,
    },
    QueryPlanetInsertAccess {
        access: QueryAccess,
    },
    QueryPlanetGetAccess {
        id: QueryId,
    },
    QueryPlanetUpdate {
        id: ComponentGroupId,
    },
}

#[macro_export]
macro_rules! ecs_err {
    ($ERR:expr, $SNAP: expr) => {
        ECSError {
            error: $ERR,
            snap: format!("{:?}", $SNAP),
            line: line!(),
            file: file!(),
        }
    };
}

pub use crate::ecs_err;
