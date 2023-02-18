use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StateId(u64);

impl StateId {
    pub fn from_hash(hash: u64) -> Self {
        StateId(hash)
    }

    pub fn id(&self) -> u64 {
        self.0
    }

    pub fn init_id() -> Self {
        StateId(1)
    }
}

pub struct StatePlanet {
    unhandled_last_state: Option<StateId>,
    //  0 == None
    next_state: AtomicU64,
    state: StateId,
}

impl StatePlanet {
    pub fn new() -> Self {
        StatePlanet {
            unhandled_last_state: Some(StateId::init_id()),
            next_state: AtomicU64::new(0),
            state: StateId::init_id(),
        }
    }

    pub fn handle_state_transition(&mut self) -> Option<StateId> {
        std::mem::replace(&mut self.unhandled_last_state, None)
    }

    pub fn get_state(&self) -> StateId {
        self.state
    }

    pub fn set_state(&self, state: StateId) {
        self.next_state.store(state.id(), Ordering::SeqCst);
    }

    pub fn update(&mut self) {
        let next_state = self.next_state.get_mut();
        if *next_state != 0 {
            self.unhandled_last_state = Some(self.state);
            self.state = StateId(*next_state);
            *next_state = 0;
        }
    }
}
