use super::*;
use std::collections::HashMap;

#[derive(Default)]
pub struct Tasker {
    system_configs: Vec<SystemConfig>,
    set_configs: Vec<SystemSetConfig>,
}

impl Tasker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn systems<const N: usize>(&mut self, systems: [SystemConfig; N]) -> &mut Self {
        systems
            .into_iter()
            .for_each(|system| self.system_configs.push(system));
        self
    }

    pub fn configure_sets<const N: usize>(
        &mut self,
        set_configs: [SystemSetConfig; N],
    ) -> &mut Self {
        set_configs
            .into_iter()
            .for_each(|set| self.set_configs.push(set));
        self
    }

    pub fn runner(self) -> TaskerRunner {
        let mut all_states = self
            .set_configs
            .iter()
            .flat_map(|config| match &config.on_state {
                OnSystemState::On(states) => states.clone(),
                _ => vec![],
            })
            .collect::<Vec<_>>();
        all_states.push(StateId::init_id());
        all_states.dedup();

        let state_system_schedules = all_states
            .into_iter()
            .map(|state| Ok((state, compile_schedule_for_state(&self, state)?)))
            .collect::<Result<_, ()>>()
            .unwrap();

        TaskerRunner {
            tasker: self,
            state_system_schedules,
        }
    }
}

pub struct TaskerRunner {
    tasker: Tasker,
    state_system_schedules: HashMap<StateId, Vec<SetId>>,
}

impl TaskerRunner {
    pub fn tick_systems(&mut self, galaxy: &mut Galaxy) {
        //  TODO FIX: Handle errors.
        let state = galaxy.get_state();
        if let Some(schedule) = self.state_system_schedules.get(&state) {
            for &set in schedule.iter() {
                execute_set(&self.tasker, set, galaxy).unwrap();
            }
        }
    }

    pub fn tick_systems_transistions(&mut self, galaxy: &mut Galaxy) {
        //  TODO FIX: Handle errors.
        //  TODO FIX: Validate set dependencies.
        if let Some(last_state) = galaxy.handle_state_transition() {
            self.tasker
                .set_configs
                .iter()
                .filter(|set| match &set.on_state {
                    OnSystemState::OnExit(enter) => enter.contains(&last_state),
                    _ => false,
                })
                .for_each(|set| {
                    execute_set(&self.tasker, set.set, galaxy).unwrap();
                });

            let current_state = galaxy.get_state();
            self.tasker
                .set_configs
                .iter()
                .filter(|set| match &set.on_state {
                    OnSystemState::OnEnter(enter) => enter.contains(&current_state),
                    _ => false,
                })
                .for_each(|set| {
                    execute_set(&self.tasker, set.set, galaxy).unwrap();
                });
        }
    }

    pub fn run(&mut self, galaxy: &mut Galaxy) {
        loop {
            galaxy.update();
            self.tick_systems_transistions(galaxy);
            self.tick_systems(galaxy);
        }
    }
}

fn execute_set(tasker: &Tasker, set: SetId, galaxy: &Galaxy) -> Result<(), String> {
    tasker
        .system_configs
        .iter()
        .filter(|sys| sys.of_set == set.get_id())
        .try_for_each(|sys| sys.sys.run(galaxy))?;
    Ok(())
}

fn compile_schedule_for_state(tasker: &Tasker, state: StateId) -> Result<Vec<SetId>, ()> {
    let mut result = vec![];

    let mut set_config = tasker
        .set_configs
        .iter()
        .cloned()
        .map(|set| (set, false))
        .collect::<Vec<_>>();

    fn recurse(
        idx: usize,
        changing_config: &mut [(SystemSetConfig, bool)],
        result: &mut Vec<SetId>,
        circular_dep_check_list: &mut Vec<SetId>,
    ) -> Result<Option<usize>, ()> {
        let (set, added) = &mut changing_config[idx];
        let set_id = set.set;

        if circular_dep_check_list.contains(&set_id) {
            Err(())?
        }

        if *added {
            return Ok(None);
        }

        circular_dep_check_list.push(set_id);
        *added = true;

        if let Some(dep) = &set.dependency {
            match dep {
                (set_dep, SetDependency::Before) => {
                    let set_dep = *set_dep;
                    let dep_idx = changing_config
                        .iter()
                        .position(|(s, _)| s.set == set_dep)
                        .unwrap();
                    let dep_position =
                        recurse(dep_idx, changing_config, result, circular_dep_check_list)?
                            .unwrap_or_else(|| result.iter().position(|s| *s == set_dep).unwrap());
                    result.insert(dep_position, set_id);
                    Ok(Some(dep_position))
                }
                (set_dep, SetDependency::After) => {
                    let set_dep = *set_dep;
                    let dep_idx = changing_config
                        .iter()
                        .position(|(s, _)| s.set == set_dep)
                        .unwrap();
                    let dep_position =
                        recurse(dep_idx, changing_config, result, circular_dep_check_list)?
                            .unwrap_or_else(|| result.iter().position(|s| *s == set_dep).unwrap());
                    result.insert(dep_position + 1, set_id);
                    Ok(Some(dep_position + 1))
                }
            }
        } else {
            result.push(set.set);
            Ok(Some(result.len() - 1))
        }
    }

    tasker
        .set_configs
        .iter()
        .enumerate()
        .try_for_each(|(idx, set)| {
            let matches_state = match &set.on_state {
                OnSystemState::Always => true,
                OnSystemState::On(states) => states.contains(&state),
                _ => false,
            };
            if !set_config[idx].1 && matches_state {
                let mut circular_dep_check_list = vec![];
                recurse(
                    idx,
                    &mut set_config,
                    &mut result,
                    &mut circular_dep_check_list,
                )?;
            }
            Ok(())
        })?;

    Ok(result)
}
