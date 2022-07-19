mod component;
mod data;
mod event;
mod resource;
mod runtime;
mod unbug;

pub type Id = usize;
pub type HashType = u64;

pub use component::{
    ArchetypeAccess, ComponentGroupQuery, ComponentHash, ComponentQueryAccessType,
    ComponentQueryFilterType, ComponentTypeEntry, ComponentTypeManager, Entity,
    EntityModifyBuilder, EntityTransformBuilder, EntityTransformer,
};
pub use data::{CloneFunction, DropFunction, TVal, ValueClone, ValueDrop};
pub use event::{EventHash, EventInsert, EventOption, EventTypeEntry};
pub use resource::{ResourceHash, ResourceManager, ResourceQueryAccessType, ResourceTypeEntry};
pub use runtime::{
    Executor, Galaxy, RawPlugin, RawPluginBundle, StraightExecutor, System, SystemBuilder,
    SystemFunction,
};
pub use unbug::{
    debug_request_dump, DebugDumpHook, DebugDumpTargets, DebugLogHook, DebugMessage,
    DebugMessageLevel, InternalError, InternalErrorType,
};
