mod entity_manager;
mod storage;

#[cfg(test)]
pub mod test_system;

pub mod command;
pub mod component;
pub mod component_stamp;
pub mod entity;
pub mod resource;
pub mod system;
pub mod wish;
pub mod wish_impl;
pub mod world;
pub use command::WorldCommands;
pub use component::{Component, ComponentTypeId};
pub use component_stamp::ComponentStamp;
pub use entity::{
    BoxedEntityModifyCallback, Entity, EntityModifyCallback, EntityModifyHandle, EntityWrapper,
};
pub use resource::{
    BoxedResourceModifyCallback, GenericResourceModifyCallback, Resource, ResourceManager,
    ResourceModifyCallback,
};
pub use system::{
    BoxedSystem, ComponentAccessMode, FilterMode, SystemArgs, SystemBuilder,
    SystemCallback, SystemDataSet, SystemDataSetInstance, SystemFilter, UntypedSystemCallback,
};
pub use wish::{Wish, WishFilter, WishFilters, WishIter, WishType, WishTypes, With, Without, R, W};
pub use world::World;
