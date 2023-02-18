use super::*;
use std::collections::{HashMap, HashSet};

//  TODO FIX: Prevent circular dependencies.

#[derive(Debug)]
pub struct Tasker {
    system_configs: Vec<SystemConfig>,
    set_configs: Vec<SystemSetConfig>,
}

impl Tasker {
    pub fn new() -> Self {
        Tasker {
            system_configs: vec![],
            set_configs: vec![],
        }
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
            .map(|state| (state, compile_schedule_for_state(&self, state)))
            .collect();

        TaskerRunner {
            tasker: self,
            state_system_schedules,
        }
    }
}

impl Default for Tasker {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct TaskerRunner {
    tasker: Tasker,
    state_system_schedules: HashMap<StateId, Vec<u64>>,
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

fn execute_set(tasker: &Tasker, set: u64, galaxy: &Galaxy) -> Result<(), String> {
    tasker
        .system_configs
        .iter()
        .filter(|sys| sys.of_set == set)
        .try_for_each(|sys| sys.sys.run(galaxy))?;
    Ok(())
}

fn compile_schedule_for_state(tasker: &Tasker, state: StateId) -> Vec<u64> {
    //  Create adjacency list.
    //  TODO OPT: Memoize this.
    let mut dependencies = HashMap::new();
    tasker.set_configs.iter().for_each(|set| {
        if !match &set.on_state {
            OnSystemState::Always => true,
            OnSystemState::On(states) => states.contains(&state),
            _ => false,
        } {
            return;
        }

        dependencies.entry(set.set).or_insert_with(Vec::new);
        for &after_dependency in set.afters.iter() {
            let deps = dependencies.get_mut(&set.set).unwrap();
            deps.push(after_dependency);
        }
        for &before_dependency in set.befores.iter() {
            dependencies
                .entry(before_dependency)
                .or_insert_with(Vec::new);
            let deps = dependencies.get_mut(&before_dependency).unwrap();
            deps.push(set.set);
        }
    });

    //  Create execution schedule based on dependencies.
    let mut schedule = vec![];
    let mut visited = HashSet::new();

    fn recur(
        this: u64,
        dependencies: &HashMap<u64, Vec<u64>>,
        visited: &mut HashSet<u64>,
        schedule: &mut Vec<u64>,
    ) {
        visited.insert(this);

        dependencies.get(&this).unwrap().iter().for_each(|&dep| {
            if !visited.contains(&dep) {
                recur(dep, dependencies, visited, schedule);
            }
        });

        schedule.push(this);
    }

    dependencies.keys().for_each(|&dep| {
        if !visited.contains(&dep) {
            recur(dep, &dependencies, &mut visited, &mut schedule);
        }
    });

    schedule
}
