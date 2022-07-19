use super::exec::{
    EntityTransformer, EventHash, EventInsert, EventManager, EventOption, Executor, GalaxyRuntime,
    ResourceManager, System,
};
use crate::unbug::prelude::*;
use std::collections::HashMap;

pub struct StraightExecutor {
    evmgr: EventManager,
    rcmgr: ResourceManager,
    ev_insert: EventInsert,
    entity_transformer: EntityTransformer,
    systems: HashMap<EventOption<EventHash>, Vec<System>>,
}

impl Executor for StraightExecutor {
    fn create(
        mut evmgr: EventManager,
        rcmgr: ResourceManager,
        systems: Vec<System>,
        galaxy: &mut GalaxyRuntime,
    ) -> Self {
        let mut ev_insert = EventInsert::create();
        let mut entity_transformer = EntityTransformer::create();
        let mut exec_systems = HashMap::new();
        for system in systems.into_iter() {
            match system.event {
                EventOption::Startup => system.run(
                    galaxy,
                    None,
                    &rcmgr,
                    &mut ev_insert,
                    &mut entity_transformer,
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

        evmgr.flush(&mut ev_insert).iex_unwrap();

        StraightExecutor {
            evmgr,
            rcmgr,
            ev_insert,
            entity_transformer,
            systems: exec_systems,
        }
    }

    fn update(&mut self, galaxy: &mut GalaxyRuntime) {
        for (&hash, systems) in self.systems.iter() {
            match hash {
                EventOption::Event(hash) => {
                    for idx in 0..self.evmgr.get_event_count(hash).iex_unwrap() {
                        let ev = self.evmgr.get_event(hash, idx).iex_unwrap();
                        for system in systems.iter() {
                            system.run(
                                galaxy,
                                Some(ev),
                                &self.rcmgr,
                                &mut self.ev_insert,
                                &mut self.entity_transformer,
                            );
                        }
                    }
                }
                EventOption::Update => {
                    for system in systems.iter() {
                        system.run(
                            galaxy,
                            None,
                            &self.rcmgr,
                            &mut self.ev_insert,
                            &mut self.entity_transformer,
                        );
                    }
                }
                EventOption::Startup => unreachable!(),
            }
        }
        self.evmgr.flush(&mut self.ev_insert).iex_unwrap();
        while let Some(transform) = self.entity_transformer.get() {
            galaxy.apply_transform(transform);
        }
    }
}
