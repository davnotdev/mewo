pub use super::{galaxy::GalaxyRuntime, system::System};

pub use crate::{
    component::EntityTransformer,
    event::{EventHash, EventInsert, EventManager, EventOption},
    resource::{GenericResourceModifyFunction, ResourceManager, ResourceModify},
};

pub trait Executor {
    fn create(
        evmgr: EventManager,
        rcmgr: ResourceManager,
        systems: Vec<System>,
        galaxy: &mut GalaxyRuntime,
    ) -> Self;
    fn update(&mut self, galaxy: &mut GalaxyRuntime);
}
