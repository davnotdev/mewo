use super::*;

impl Galaxy {
    pub fn get_state(&self) -> StateId {
        self.stp.get_state()
    }

    pub fn handle_state_transition(&mut self) -> Option<StateId> {
        self.stp.handle_state_transition()
    }

    pub fn set_state(&self, state: StateId) {
        self.stp.set_state(state);
    }
}
