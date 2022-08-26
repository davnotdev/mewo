use super::exec::{
    EarlySystemPhase, EntityTransformer, EventInsert, Executor, GalaxyRuntime, System,
};
use crate::debug::prelude::*;

pub struct StraightExecutor {
    ev_insert: EventInsert,
    entity_transformer: EntityTransformer,
    systems: Vec<System>,
}

impl Executor for StraightExecutor {
    fn create(systems: Vec<System>) -> Self {
        StraightExecutor {
            ev_insert: EventInsert::create(),
            entity_transformer: EntityTransformer::create(),
            systems,
        }
    }

    fn early(&mut self, galaxy: &mut GalaxyRuntime) {
        self.run_early_phase(galaxy, EarlySystemPhase::Bootstrap);
        self.run_early_phase(galaxy, EarlySystemPhase::Startup);
    }

    fn update(&mut self, galaxy: &mut GalaxyRuntime) {
        self.systems.iter().for_each(|sys| {
            sys.run(galaxy, &mut self.ev_insert, &mut self.entity_transformer);
        });
        self.post_update(galaxy);
    }
}

impl StraightExecutor {
    fn run_early_phase(&mut self, galaxy: &mut GalaxyRuntime, phase: EarlySystemPhase) {
        let phase_count = self
            .systems
            .iter()
            .filter(|sys| sys.phase == Some(phase))
            .count();
        let mut done_count = 0;
        let mut tick_count = 0;
        loop {
            self.systems.retain(|sys| {
                if sys.phase == Some(phase) {
                    if sys.run(galaxy, &mut self.ev_insert, &mut self.entity_transformer) {
                        done_count += 1;
                        return false;
                    }
                }
                true
            });

            self.post_update(galaxy);

            if done_count == phase_count {
                break;
            }

            if tick_count > 5 {
                panic!("Phase {:?} is taking too long", phase);
            }

            tick_count += 1;
        }
    }

    fn post_update(&mut self, galaxy: &mut GalaxyRuntime) {
        galaxy
            .get_event_manager()
            .write()
            .unwrap()
            .flush(&mut self.ev_insert)
            .iex_unwrap();
        while let Some(transform) = self.entity_transformer.get() {
            galaxy.apply_transform(transform);
        }
    }
}
