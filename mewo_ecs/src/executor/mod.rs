pub use crate::{
    BoxedSystem, Entity, EntityModifyHandle, SystemArgs, SystemDataSet, SystemDataSetInstance,
    UntypedSystemCallback, World, WorldCommands,
};

pub use straight::StraightExecutor;
pub type DefaultExecutor = StraightExecutor;

pub mod straight;

pub trait Executor {
    fn create(world: &World, systems: Vec<(BoxedSystem, Vec<SystemDataSet>)>) -> Self;
    fn run_systems(&mut self, world: &mut World);
    fn run_commands(&mut self, world: &mut World);
}
