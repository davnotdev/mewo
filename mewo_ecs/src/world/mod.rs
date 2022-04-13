mod mask;
mod storage;
mod entity_manager;
mod component_manager;

pub mod gift;
pub mod error;
pub mod world;
pub mod entity;
pub mod system;
pub mod command;
pub mod resource;
pub mod component_stamp;

pub use gift::{
    Gift,
    GlobalGift,
    GiftInstance,
};
pub use error::ECSError;
pub use world::{
    World,
    EntityModifyCallback,
};
pub use entity::{
    Entity, 
    EntityWrapper,
};
pub use system::{
    MainSystem,
    SantaClaus,
};
pub use command::WorldCommands;
pub use resource::{
    ResourceManager,
    ResourceModifyCallback,
};

