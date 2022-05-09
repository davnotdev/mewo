use crate::*;

pub struct AppInstance<E>
where
    E: Executor,
{
    exec: E,
    world: World,
}

impl<E> AppInstance<E>
where
    E: Executor,
{
    pub fn run(&mut self) {
        loop {
            self.exec.run_systems(&mut self.world);
            self.exec.run_commands(&mut self.world);
        }
    }

    pub fn get_world(&self) -> &World {
        &self.world
    }

    pub fn get_mut_world(&mut self) -> &mut World {
        &mut self.world
    }
}

pub struct App {
    deps: Vec<String>,
    world: World,
    commands: WorldCommands,
    system_builder: SystemBuilder,
}

impl App {
    pub fn builder() -> Self {
        App {
            deps: Vec::new(),
            world: World::create(),
            commands: WorldCommands::create(),
            system_builder: SystemBuilder::create(),
        }
    }

    pub fn plugin<P: Plugin>(mut self, p: P) -> Self {
        self.dep(p);
        self
    }

    pub fn dep<P: Plugin>(&mut self, _: P) -> &mut Self {
        let dep_name = String::from(P::name());
        if !self.deps.contains(&dep_name) {
            P::plugin(self);
            self.deps.push(dep_name);
        }
        self
    }

    pub fn sys<WS>(&mut self, system: SystemFunction<WS>) -> &mut Self
    where
        WS: 'static + Wishes,
    {
        self.system_builder.sys(&self.world, system);
        self
    }

    pub fn component<C>(&mut self) -> &mut Self
    where
        C: 'static + Component,
    {
        self.world
            .get_mut_component_manager()
            .register_component_type::<C>()
            .unwrap();
        self
    }

    pub fn commands(&mut self) -> &mut WorldCommands {
        &mut self.commands
    }

    pub fn build<E>(mut self) -> AppInstance<E>
    where
        E: Executor,
    {
        let mut systems = self.system_builder.consume();

    //  fixes an issue where systems are added
    //  but before all components are registered
        for (_sysf, sys_datas) in systems.iter_mut() {
            for sys_data in sys_datas.iter_mut() {
                sys_data.realign_len(&self.world);
            }
        }

        let exec = E::create(&mut self.world, systems, self.commands);
        AppInstance {
            exec,
            world: self.world,
        }
    }
}
