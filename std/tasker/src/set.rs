use mewo_galaxy::data::hash_type_and_val;

use super::*;
use std::hash::Hash;

//  TODO CHK: Ok, I may be wrong, but what are the practical uses for multiple set dependencies?
//  I feel like until there is a use, only one dependency should be allowed just to keep me sane.
#[derive(Debug, Clone)]
pub enum SetDependency {
    Before,
    After,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct SetId(u64);

impl SetId {
    pub fn from_hash(val: u64) -> Self {
        SetId(val)
    }

    pub fn get_id(&self) -> u64 {
        self.0
    }
}

pub trait SystemSet: 'static + Hash + Sized {
    fn hash_with_val(self) -> u64 {
        hash_type_and_val(self)
    }

    fn config(self) -> SystemSetConfig {
        SystemSetConfig::new(SetId::from_hash(self.hash_with_val()))
    }
}

#[derive(Debug, Clone)]
pub struct SystemSetConfig {
    pub set: SetId,
    pub dependency: Option<(SetId, SetDependency)>,
    pub on_state: OnSystemState,
}

impl SystemSetConfig {
    fn new(set: SetId) -> Self {
        Self {
            set,
            dependency: None,
            on_state: OnSystemState::default(),
        }
    }

    pub fn set_dependency<S: 'static + SystemSet>(mut self, set: S, dep: SetDependency) -> Self {
        let set = SetId::from_hash(set.hash_with_val());
        self.dependency = Some((set, dep));
        self
    }

    pub fn on_state(mut self, on_state: OnSystemState) -> Self {
        self.on_state = on_state;
        self
    }
}
