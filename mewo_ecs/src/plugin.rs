use crate::*;

pub type PluginBuildCallback = fn(&mut PluginBuilder);

pub struct PluginBuilder<'world> {
    world: &'world mut World,
    systems: Vec<SystemData>,
}

impl<'world> PluginBuilder<'world> {
    pub fn create(world: &'world mut World) -> PluginBuilder {
        PluginBuilder {
            world,
            systems: Vec::new(),
        }
    }

    pub fn sys(&mut self, system: fn(&World) -> SystemData) -> &mut Self {
        self.systems.push((system)(&self.world));
        self
    }

    pub fn component<C>(&mut self) -> &mut Self
        where C: 'static + Clone
    {
        self
            .world
            .get_mut_component_manager()
            .register_component_type::<C>()
            .unwrap();
        self
    }

    pub fn dep<C>(&mut self) -> &mut Self 
        where C: 'static
    {
        self.world.get_component_manager().get_component_id::<C>().unwrap();
        self
    }

    pub fn consume_systems(self) -> Vec<SystemData> {
        self.systems 
    }
}

