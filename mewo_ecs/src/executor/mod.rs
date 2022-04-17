pub use crate::world::{
    WishInstance,
    World, 
    Entity, 
    System, 
    SystemArgs,
    SystemWish,
    SystemData, 
    GlobalWish,
    BoxedSystem,
    WorldCommands,
    WorldCommandsStore,
    EntityModifierHandle,
};

pub use straight::StraightExecutor;
pub type DefaultExecutor = StraightExecutor;

pub mod straight;

pub trait Executor {
    fn create(world: &World, sys: Vec<(BoxedSystem, SystemData)>) -> Self;
    fn execute(&mut self, world: &mut World);
}
