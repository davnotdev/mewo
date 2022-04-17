mod mask;
mod storage;
mod entity_manager;

pub mod wish;
pub mod world;
pub mod entity;
pub mod system;
pub mod command;
pub mod resource;
pub mod component;
pub mod component_stamp;

pub use wish::{
    Read,
    Write,
    WishArg,
};
pub use world::{
    World,
};
pub use entity::{
    Entity, 
    EntityModifier,
    EntityModifierStore,
    EntityModifierHandle,
    EntityComponentModify,
    EntityComponentModifyType,
};
pub use system::{
    Wish,
    System,
    GlobalWish,
    SystemWish,
    SystemArgs,
    SystemData,
    BoxedSystem,
    SystemFilter,
    WishInstance,
    SystemCallback,
    GiftInstanceReadIter,
    GiftInstanceWriteIter,
};
pub use command::{
    WorldCommands,
    WorldCommandsStore,
};
pub use resource::{
    Resource,
    ResourceManager,
    ResourceModifyCallback,
    BoxedResourceModifyCallback,
    GenericResourceModifyCallback,
};
pub use component::{
    Component,
    ComponentTypeId,
};
pub use component_stamp::ComponentStamp;

