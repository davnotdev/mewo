use super::{entity::EntityBus, event::EventBus, resource::ResourceBus, wish::Wish};

pub type SystemFunction<WE, WA, WF> = fn(SystemBus, Wish<WE, WA, WF>);

pub struct SystemBus<'exec> {
    pub entities: EntityBus<'exec>,
    pub events: EventBus<'exec>,
    pub resources: ResourceBus<'exec, 'exec>,
}

impl<'exec> SystemBus<'exec> {
    pub fn create(
        entities: EntityBus<'exec>,
        events: EventBus<'exec>,
        resources: ResourceBus<'exec, 'exec>,
    ) -> Self {
        SystemBus {
            entities,
            events,
            resources,
        }
    }
}
