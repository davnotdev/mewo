pub use crate::world::{
    Gift,
    GlobalGift,
    GiftInstance,
    World, 
    Entity, 
    System, 
    SantaClaus,
    WorldCommands,
};

pub use straight::StraightExecutor;
pub type DefaultExecutor = StraightExecutor;

pub mod straight;

pub trait Executor {
    fn create(world: &World, sys: Vec<(System, SantaClaus)>) -> Self;
    fn execute(&mut self, world: &mut World);
}
