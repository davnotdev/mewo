use super::{
    exec::Executor,
    plugin::{RawPlugin, RawPluginBundle},
};
use crate::{
    component::{
        ArchetypeManager, ComponentTypeManager, EntityManager, EntityModifyBuilder,
        EntityTransformBuilder,
    },
    event::EventManager,
    resource::ResourceManager,
    unbug::{
        debug_insert_dump_hook, debug_insert_log_hook, prelude::*, DebugDumpHook, DebugLogHook,
    },
};

pub struct Galaxy {
    plugins: RawPluginBundle,
}

impl Galaxy {
    pub fn create() -> Self {
        Galaxy {
            plugins: RawPluginBundle::create(),
        }
    }

    pub fn plugins(mut self, plugins: Vec<RawPlugin>) -> Self {
        plugins
            .into_iter()
            .map(|plugin| self.plugins.plugin(plugin))
            .for_each(|res| res.iex_unwrap());
        self
    }

    pub fn debug_log_hook(self, hook: DebugLogHook) -> Self {
        debug_insert_log_hook(hook);
        self
    }

    pub fn debug_dump_hook(self, hook: DebugDumpHook) -> Self {
        debug_insert_dump_hook(hook);
        self
    }

    pub fn runtime<E: Executor>(self) -> (GalaxyRuntime, E) {
        GalaxyRuntime::prepare(self)
    }

    pub fn run<E: Executor>(self) -> ! {
        let (runtime, exec) = GalaxyRuntime::prepare::<E>(self);
        runtime.run(exec)
    }
}

pub struct GalaxyRuntime {
    emgr: EntityManager,
    amgr: ArchetypeManager,
    ctymgr: ComponentTypeManager,
}

impl GalaxyRuntime {
    pub fn prepare<E: Executor>(galaxy: Galaxy) -> (Self, E) {
        let mut runtime = GalaxyRuntime {
            emgr: EntityManager::create(),
            amgr: ArchetypeManager::create(),
            ctymgr: ComponentTypeManager::create(),
        };
        let mut rcmgr = ResourceManager::create();
        let mut evmgr = EventManager::create();
        let plugins = galaxy.plugins.consume();

        let mut plugin_systems = Vec::new();
        plugins.into_iter().for_each(
            |RawPlugin {
                 mut systems,
                 components,
                 events,
                 resources,
                 ..
             }| {
                //  Systems
                plugin_systems.append(&mut systems);
                //  Component Types
                components
                    .into_iter()
                    .map(|cty_entry| runtime.ctymgr.register(cty_entry))
                    .for_each(|res| {
                        res.iex_unwrap();
                    });
                //  Events
                events
                    .into_iter()
                    .map(|ev_entry| evmgr.register(ev_entry))
                    .for_each(|res| res.iex_unwrap());
                //  Resources
                resources
                    .into_iter()
                    .map(|rc_entry| rcmgr.register(rc_entry))
                    .for_each(|res| res.iex_unwrap());
            },
        );

        let systems = plugin_systems
            .into_iter()
            .map(|sys| sys.build(&runtime.ctymgr, &mut runtime.amgr).iex_unwrap())
            .collect::<Vec<super::System>>();
        let exec = E::create(evmgr, rcmgr, systems, &mut runtime);
        (runtime, exec)
    }

    pub fn apply_transform(&mut self, mut transform: EntityTransformBuilder) {
        match transform.get_mut_modify() {
            EntityModifyBuilder::Create(none) => {
                let entity = self.emgr.register_entity();
                *none = Some(entity)
            }
            &mut EntityModifyBuilder::Destroy(rm) => self.emgr.deregister_entity(rm).unwrap(),
            _ => {}
        };
        let transform = transform.build(&self.ctymgr).unwrap();
        self.amgr.transform_entity(transform, &self.ctymgr).unwrap();
    }

    pub fn get_component_type_manager(&self) -> &ComponentTypeManager {
        &self.ctymgr
    }

    pub fn get_archetype_manager(&self) -> &ArchetypeManager {
        &self.amgr
    }

    pub fn tick<E: Executor>(&mut self, exec: &mut E) {
        exec.update(self);
    }

    pub fn run<E: Executor>(mut self, mut exec: E) -> ! {
        loop {
            self.tick(&mut exec);
        }
    }
}
