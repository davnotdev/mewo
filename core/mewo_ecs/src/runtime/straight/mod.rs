use super::exec::{
    EntityTransformer, EventHash, EventInsert, EventManager, EventOption, Executor, GalaxyRuntime,
    ResourceManager, ResourceModify, System,
};
use std::collections::HashMap;

pub struct StraightExecutor {
    evmgr: EventManager,
    rcmgr: ResourceManager,
    ev_insert: EventInsert,
    rc_modify: ResourceModify,
    entity_transformer: EntityTransformer,
    systems: HashMap<EventOption<EventHash>, Vec<System>>,
}

impl Executor for StraightExecutor {
    fn create(
        mut evmgr: EventManager,
        mut rcmgr: ResourceManager,
        systems: Vec<System>,
        galaxy: &mut GalaxyRuntime,
    ) -> Self {
        let mut ev_insert = EventInsert::create();
        let mut rc_modify = ResourceModify::create();
        let mut entity_transformer = EntityTransformer::create();
        let mut exec_systems = HashMap::new();
        for system in systems.into_iter() {
            match system.event {
                EventOption::Startup => (system.function)(
                    None,
                    &mut ev_insert,
                    &rcmgr,
                    &mut rc_modify,
                    &mut entity_transformer,
                    &galaxy,
                    system.archetype_access_key,
                ),
                _ => {
                    if let None = exec_systems.get_mut(&system.event) {
                        exec_systems.insert(system.event, Vec::new());
                    }
                    exec_systems.get_mut(&system.event).unwrap().push(system);
                }
            }
        }

        while let Some(transform) = entity_transformer.get() {
            galaxy.apply_transform(transform);
        }

        rcmgr.flush(&mut rc_modify);
        evmgr.flush(&mut ev_insert).unwrap();

        StraightExecutor {
            evmgr,
            rcmgr,
            ev_insert,
            rc_modify,
            entity_transformer,
            systems: exec_systems,
        }
    }

    fn update(&mut self, galaxy: &mut GalaxyRuntime) {
        for (&hash, systems) in self.systems.iter() {
            match hash {
                EventOption::Event(hash) => {
                    for idx in 0..self.evmgr.get_event_count(hash).unwrap() {
                        let ev = self.evmgr.get_event(hash, idx).unwrap();
                        for system in systems.iter() {
                            (system.function)(
                                Some(ev),
                                &mut self.ev_insert,
                                &self.rcmgr,
                                &mut self.rc_modify,
                                &mut self.entity_transformer,
                                galaxy,
                                system.archetype_access_key,
                            );
                        }
                    }
                }
                EventOption::Update => {
                    for system in systems.iter() {
                        (system.function)(
                            None,
                            &mut self.ev_insert,
                            &self.rcmgr,
                            &mut self.rc_modify,
                            &mut self.entity_transformer,
                            galaxy,
                            system.archetype_access_key,
                        );
                    }
                }
                EventOption::Startup => unreachable!(),
            }
        }
        self.rcmgr.flush(&mut self.rc_modify);
        self.evmgr.flush(&mut self.ev_insert).unwrap();
        while let Some(transform) = self.entity_transformer.get() {
            galaxy.apply_transform(transform);
        }
    }
}
