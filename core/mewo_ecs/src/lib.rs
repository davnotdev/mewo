mod component;
mod data;
mod error;
mod event;
mod resource;
mod runtime;

//  TODO DVecs should be able to drop their contents.

pub type Id = usize;
pub type HashType = u64;

pub use component::{
    ArchetypeAccess, ComponentGroupQuery, ComponentHash, ComponentQueryAccessType,
    ComponentQueryFilterType, ComponentTypeEntry, ComponentTypeManager, Entity,
    EntityModifyBuilder, EntityTransformBuilder, EntityTransformer,
};
pub use data::TVal;
pub use event::{EventHash, EventInsert, EventOption, EventTypeEntry};
pub use resource::{
    ResourceHash, ResourceManager, ResourceModify, ResourceModifyFunction, ResourceTypeEntry,
};
pub use runtime::{
    Executor, Galaxy, RawPlugin, RawPluginBundle, StraightExecutor, System, SystemBuilder,
};
