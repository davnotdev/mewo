mod component;
mod data;
mod event;
mod resource;
mod runtime;
mod debug;

pub type Id = usize;
pub type HashType = u64;

pub use component::{
    ArchetypeAccess, ArchetypeAccessKey, ArchetypeManager, ComponentGroupQuery, ComponentHash,
    ComponentQueryAccessType, ComponentQueryFilterType, ComponentTypeEntry, ComponentTypeManager,
    Entity, EntityModifyBuilder, EntityTransformBuilder, EntityTransformer,
};
pub use data::{hash_type, CloneFunction, DropFunction, TVal, ValueClone, ValueDrop};
pub use event::{EventHash, EventInsert, EventTypeEntry};
pub use resource::{ResourceHash, ResourceManager, ResourceQueryAccessType, ResourceTypeEntry};
pub use runtime::{
    EarlySystemPhase, Executor, Galaxy, RawPlugin, RawPluginBundle, SharedComponentTypeManager,
    SharedEventManager, SharedResourceManager, StraightExecutor, System, SystemBuilder,
    SystemFunction,
};
pub use debug::{
    debug_error, debug_request_dump, DebugDumpHook, DebugDumpTargets, DebugLogHook, DebugMessage,
    DebugMessageLevel, InternalError, InternalErrorType,
};
