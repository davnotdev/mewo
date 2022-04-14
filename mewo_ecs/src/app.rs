use crate::*;

pub struct App<E> 
    where E: Executor
{
    exec: E, 
    world: World,
}

impl<E> App<E> 
    where E: Executor
{
    pub fn run(&mut self) {
        loop {
            self.exec.execute(&mut self.world);
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
    plugins: Vec<(String, Vec<SystemData>)>,
    plugins_start: Vec<(String, Vec<SystemData>)>,
    plugins_end: Vec<(String, Vec<SystemData>)>,
}

impl AppBuilder { pub fn create() -> Self {
        AppBuilder {
            world: World::create(),
            commands: WorldCommands::create(),
            plugins: Vec::new(),
            plugins_start: Vec::new(),
            plugins_end: Vec::new(),
        }
    }

    fn plugin_build(world: &mut World, callback: PluginBuildCallback) -> Vec<SystemData> {
        let mut builder = PluginBuilder::create(world);
        (callback)(&mut builder);
        builder.consume_systems()
    }

    pub fn plugin(mut self, name: &str, callback: PluginBuildCallback) -> Self {
        self.plugins.push((String::from(name), Self::plugin_build(&mut self.world, callback)));
        self
    }

    pub fn plugin_start(mut self, name: &str, callback: PluginBuildCallback) -> Self {
        self.plugins_start.push((String::from(name), Self::plugin_build(&mut self.world, callback)));
        self
    }

    pub fn plugin_end(mut self, name: &str, callback: PluginBuildCallback) -> Self {
        self.plugins_end.push((String::from(name), Self::plugin_build(&mut self.world, callback)));
        self
    }

    pub fn build<E>(self) -> App<E> 
        where E: Executor
    {
        let mut finals = Vec::new();    
        for (_name, plugin) in self
            .plugins_start.into_iter()
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

