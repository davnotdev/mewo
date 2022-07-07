pub mod archetype;
pub mod component_group;
pub mod component_type;
pub mod entity;
pub mod query;
pub mod transform;

pub use archetype::{ArchetypeAccess, ArchetypeAccessKey, ArchetypeManager};
pub use component_group::ComponentGroup;
pub use component_type::{ComponentTypeEntry, ComponentTypeManager};
pub use entity::{Entity, EntityManager};
pub use query::{ComponentGroupQuery, ComponentQueryAccessType, ComponentQueryFilterType};
pub use transform::{EntityModifyBuilder, EntityTransformBuilder, EntityTransformer};

pub use super::Id;
pub type ComponentTypeId = Id;
pub type ComponentGroupId = Id;
pub type ComponentHash = super::HashType;
