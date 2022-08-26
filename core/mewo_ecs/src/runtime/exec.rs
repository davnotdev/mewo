pub use super::{
    galaxy::GalaxyRuntime,
    system::{EarlySystemPhase, System},
};
pub use crate::{
    component::EntityTransformer,
    data::{hash_type, ValueDrop},
    event::{EventHash, EventInsert, EventManager, EventTypeEntry},
    resource::ResourceManager,
};

pub trait Executor {
    fn create(systems: Vec<System>) -> Self;
    fn early(&mut self, galaxy: &mut GalaxyRuntime);
    fn update(&mut self, galaxy: &mut GalaxyRuntime);
}
