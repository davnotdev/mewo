use mewo_galaxy::prelude::{Galaxy, StateId};

pub mod prelude;
mod set;
mod state;
mod sys;
mod tasker;

#[cfg(test)]
mod test;

use set::SystemSetConfig;
use sys::SystemConfig;

pub use set::SystemSet;
pub use state::OnSystemState;
pub use state::SystemState;

#[derive(Hash)]
pub struct Init;

impl SystemState for Init {
    fn hash_with_val(self) -> u64 {
        StateId::init_id().id()
    }
}

//  TODO EXT: Nested states.
//  TODO FIX: Proper documentation of expected behavior
