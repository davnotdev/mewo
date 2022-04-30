use crate::*;
use error::Result;

pub struct App<E>
where
    E: Executor,
{
    exec: E,
    world: World,
}

impl<E> App<E>
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

pub struct AppBuilder {
    world: World,
    commands: WorldCommands,
    plugins: Vec<(String, Vec<(BoxedSystem, Vec<SystemDataSet>)>)>,
    plugins_start: Vec<(String, Vec<(BoxedSystem, Vec<SystemDataSet>)>)>,
    plugins_end: Vec<(String, Vec<(BoxedSystem, Vec<SystemDataSet>)>)>,
}

impl AppBuilder {
    pub fn create() -> Self {
        AppBuilder {
            commands: WorldCommands::create(),
            world: World::create(),
            plugins: Vec::new(),
            plugins_start: Vec::new(),
            plugins_end: Vec::new(),
        }
    }

    fn plugin_build(
        world: &mut World,
        callback: PluginBuildCallback,
    ) -> (
        Vec<String>,
        Vec<(BoxedSystem, Vec<SystemDataSet>)>,
        WorldCommands,
    ) {
        let mut builder = PluginBuilder::create(world);
        (callback)(&mut builder);
        (
            builder.deps,
            builder.system_builder.consume(),
            builder.commands,
        )
    }

    fn check_deps(&self, deps: Vec<String>) -> Result<()> {
        for dep in deps {
            for (name, _plugin) in &self.plugins {
                if dep == *name {
                    break;
                }
            }
            return Err(ECSError::PluginDependencyNotFound(dep));
        }
        Ok(())
    }

    pub fn plugin(mut self, name: &str, callback: PluginBuildCallback) -> Self {
        let (deps, sys, cmds) = Self::plugin_build(&mut self.world, callback);
        self.check_deps(deps).unwrap();
        self.commands.merge(cmds);
        self.plugins.push((String::from(name), sys));
        self
    }

    pub fn plugin_start(mut self, name: &str, callback: PluginBuildCallback) -> Self {
        let (deps, sys, cmds) = Self::plugin_build(&mut self.world, callback);
        self.check_deps(deps).unwrap();
        self.commands.merge(cmds);
        self.plugins_start.push((String::from(name), sys));
        self
    }

    pub fn plugin_end(mut self, name: &str, callback: PluginBuildCallback) -> Self {
        let (deps, sys, cmds) = Self::plugin_build(&mut self.world, callback);
        self.check_deps(deps).unwrap();
        self.commands.merge(cmds);
        self.plugins_end.push((String::from(name), sys));
        self
    }

    pub fn build<E>(self) -> App<E>
    where
        E: Executor,
    {
        let mut finals = Vec::new();
        for (_name, plugin) in self
            .plugins_start
            .into_iter()
            .chain(self.plugins.into_iter())
            .chain(self.plugins_end.into_iter())
        {
            for sys in plugin.into_iter() {
                finals.push(sys)
            }
        }
        App {
            exec: E::create(&self.world, finals),
            world: self.world,
        }
    }
}
