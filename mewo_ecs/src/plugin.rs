use crate::*;

pub type PluginBuildCallback = fn(&mut PluginBuilder);

pub struct PluginBuilder<'world> {
    world: &'world mut World,
    pub deps: Vec<String>,
    pub commands: WorldCommands,
    pub system_builder: SystemBuilder,
}

impl<'world> PluginBuilder<'world> {
    pub fn create(world: &'world mut World) -> PluginBuilder {
        PluginBuilder {
            commands: WorldCommands::create(),
            deps: Vec::new(),
            world,
            system_builder: SystemBuilder::create(),
        }
    }

    pub fn sys<WT, WF>(&mut self, system: SystemCallback<WT, WF>) -> &mut Self
    where
        WT: 'static + WishTypes,
        WF: 'static + WishFilters,
    {
        self.system_builder.sys::<WT, WF>(self.world, system);
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

    pub fn dep(&mut self, plugin_name: &str) -> &mut Self {
        self.deps.push(String::from(plugin_name));
        self
    }

    pub fn commands(&mut self) -> &mut WorldCommands {
        &mut self.commands
    }
}
