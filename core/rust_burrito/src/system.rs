use super::{entity::EntityBus, event::EventBus, resource::ResourceBus};

pub struct SystemBus<'exec> {
    pub entities: EntityBus<'exec>,
    pub events: EventBus<'exec>,
    pub resources: ResourceBus<'exec>,
    idx: usize,
    len: usize,
}

impl<'exec> SystemBus<'exec> {
    pub fn create(
        entities: EntityBus<'exec>,
        events: EventBus<'exec>,
        resources: ResourceBus<'exec>,
        idx: usize,
        len: usize,
    ) -> Self {
        SystemBus {
            entities,
            events,
            resources,
            idx,
            len,
        }
    }

    pub fn get_execution_count(&self) -> usize {
        self.len
    }

    pub fn get_current_execution(&self) -> usize {
        self.idx
    }

    pub fn is_first(&self) -> bool {
        self.idx == 0
    }

    pub fn is_last(&self) -> bool {
        assert!(self.len != 0);
        self.idx == self.len - 1
    }
}
