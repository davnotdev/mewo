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
    debug::{
        debug_insert_dump_hook, debug_insert_log_hook, prelude::*, DebugDumpHook, DebugLogHook,
    },
};
use std::sync::RwLock;

//  The RwLocks here are to get borrow checker to SHUT UP.
pub struct Galaxy {
    ctymgr: SharedComponentTypeManager,
    plugins: RwLock<RawPluginBundle>,
}

impl Galaxy {
    pub fn create() -> Self {
        Galaxy {
            ctymgr: SharedComponentTypeManager::new(ComponentTypeManager::create()),
            plugins: RwLock::new(RawPluginBundle::create()),
        }
    }

    pub fn plugin(&self, plugin: RawPlugin) -> &Self {
        let mut plugins = self.plugins.write().unwrap();
        plugins.plugin(plugin);
        self
    }

    pub fn plugins(&self, plugins: Vec<RawPlugin>) -> &Self {
        let mut self_plugins = self.plugins.write().unwrap();
        plugins
            .into_iter()
            .for_each(|plugin| self_plugins.plugin(plugin));
        self
    }

    pub fn debug_log_hook(&self, hook: DebugLogHook) -> &Self {
        debug_insert_log_hook(hook);
        self
    }

    pub fn debug_dump_hook(&self, hook: DebugDumpHook) -> &Self {
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

    pub fn get_component_type_manager(&self) -> &SharedComponentTypeManager {
        &self.ctymgr
    }
}

pub type SharedEventManager = RwLock<EventManager>;
pub type SharedResourceManager = RwLock<ResourceManager>;
pub type SharedComponentTypeManager = RwLock<ComponentTypeManager>;

pub struct GalaxyRuntime {
    emgr: EntityManager,
    amgr: ArchetypeManager,
    ctymgr: SharedComponentTypeManager,
    evmgr: SharedEventManager,
    rcmgr: SharedResourceManager,
}

impl GalaxyRuntime {
    pub fn prepare<E: Executor>(galaxy: Galaxy) -> (Self, E) {
        let mut runtime = GalaxyRuntime {
            ctymgr: galaxy.ctymgr,
            emgr: EntityManager::create(),
            amgr: ArchetypeManager::create(),
            evmgr: SharedEventManager::new(EventManager::create()),
            rcmgr: SharedResourceManager::new(ResourceManager::create()),
        };
        let plugins = galaxy.plugins.into_inner().unwrap().consume();

        let mut plugin_systems = Vec::new();
        plugins
            .into_iter()
            .for_each(|RawPlugin { mut systems, .. }| {
                //  Systems
                plugin_systems.append(&mut systems);
            });

        let systems;
        {
            let ctymgr = runtime.ctymgr.read().unwrap();
            systems = plugin_systems
                .into_iter()
                .map(|sys| sys.build(&ctymgr, &mut runtime.amgr).iex_unwrap())
                .collect::<Vec<super::System>>();
        }
        let mut exec = E::create(systems);
        exec.early(&mut runtime);
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
        let ctymgr = self.ctymgr.read().unwrap();
        let transform = transform.build(&ctymgr).unwrap();
        self.amgr.transform_entity(transform, &ctymgr).unwrap();
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

impl GalaxyRuntime {
    pub fn get_component_type_manager(&self) -> &SharedComponentTypeManager {
        &self.ctymgr
    }

    pub fn get_archetype_manager(&self) -> &ArchetypeManager {
        &self.amgr
    }

    pub fn get_event_manager(&self) -> &SharedEventManager {
        &self.evmgr
    }

    pub fn get_resource_manager(&self) -> &SharedResourceManager {
        &self.rcmgr
    }
}
