use mewo_galaxy::data::hash_type_and_val;

use super::*;
use std::hash::Hash;

//  TODO OPT: Replace u64 with set type.

pub trait SystemSet: 'static + Hash + Sized {
    fn hash_with_val(self) -> u64 {
        hash_type_and_val(self)
    }

    fn config(self) -> SystemSetConfig {
        SystemSetConfig::new(self.hash_with_val())
    }
}

#[derive(Debug, Clone)]
pub struct SystemSetConfig {
    pub set: u64,
    pub befores: Vec<u64>,
    pub afters: Vec<u64>,
    pub on_state: OnSystemState,
}

impl SystemSetConfig {
    fn new(set: u64) -> Self {
        Self {
            set,
            befores: vec![],
            afters: vec![],
            on_state: OnSystemState::default(),
        }
    }

    pub fn before<S: 'static + SystemSet>(mut self, set: S) -> Self {
        self.befores.push(set.hash_with_val());
        self
    }

    pub fn after<S: 'static + SystemSet>(mut self, set: S) -> Self {
        self.afters.push(set.hash_with_val());
        self
    }

    pub fn on_state(mut self, on_state: OnSystemState) -> Self {
        self.on_state = on_state;
        self
    }
}
