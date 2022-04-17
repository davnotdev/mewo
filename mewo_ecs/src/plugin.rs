use crate::*;

pub type PluginBuildCallback = fn(&mut PluginBuilder);

pub struct PluginBuilder<'world> {
    world: &'world mut World,
    pub deps: Vec<String>,
    pub systems: Vec<(BoxedSystem, SystemData)>,
    pub commands: WorldCommandsStore,
}

impl<'world> PluginBuilder<'world> {
    pub fn create(world: &'world mut World) -> PluginBuilder {
        PluginBuilder {
            commands: WorldCommandsStore::create(),
            world,
            deps: Vec::new(),
            systems: Vec::new(),
        }
    }

    pub fn sys<Q>(&mut self, system: SystemCallback<Q>) -> &mut Self 
        where Q: 'static + WishArg
    {
        let types = Q::get_types();
        self.systems.push((
            Box::new(System(system)),
            SystemData::from_query_type(&self.world, &types)));
        self
    }

    pub fn component<C>(&mut self) -> &mut Self
        where C: 'static + Component
    {
        self
            .world
            .get_mut_component_manager()
            .register_component_type::<C>()
            .unwrap();
        self
    }

    pub fn dep(&mut self, plugin_name: &str) -> &mut Self {
        self.deps.push(String::from(plugin_name));
        self
    }

    pub fn commands(&mut self) -> WorldCommands {
        self.commands.modify(self.world)
    }
}

