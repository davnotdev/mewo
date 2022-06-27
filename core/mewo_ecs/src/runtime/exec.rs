pub use super::{galaxy::GalaxyRuntime, system::System};

pub use crate::{
    component::EntityTransformer,
    event::{EventHash, EventInsert, EventManager, EventOption},
};

pub trait Executor {
    fn create(evmgr: EventManager, systems: Vec<System>, galaxy: &mut GalaxyRuntime) -> Self;
    fn update(&mut self, galaxy: &mut GalaxyRuntime);
}
