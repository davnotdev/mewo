mod component;
mod data;
mod error;
mod event;
mod resource;
mod runtime;

pub type Id = usize;
pub type HashType = u64;
pub type ComponentHash = HashType;

pub use component::{
    ArchetypeAccess, ComponentGroupQuery, ComponentQueryAccessType, ComponentQueryFilterType,
    ComponentTypeEntry, ComponentTypeManager, Entity, EntityModifyBuilder, EntityTransformBuilder,
    EntityTransformer,
};
pub use data::TVal;
pub use event::{EventHash, EventInsert, EventOption, EventTypeEntry};
pub use runtime::{
    Executor, Galaxy, RawPlugin, RawPluginBundle, StraightExecutor, System, SystemBuilder,
};
