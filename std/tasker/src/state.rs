use mewo_galaxy::data::hash_type_and_val;

use super::*;
use std::hash::Hash;

pub trait SystemState: 'static + Hash + Sized {
    fn hash_with_val(self) -> u64 {
        hash_type_and_val(self)
    }
}

pub fn state<S: SystemState>(s: S) -> StateId {
    StateId::from_hash(s.hash_with_val())
}

#[derive(Debug, Default, Clone)]
pub enum OnSystemState {
    #[default]
    Always,
    On(Vec<StateId>),
    OnEnter(Vec<StateId>),
    OnExit(Vec<StateId>),
}
