use super::system::SystemBuilder;
use crate::debug::prelude::*;

#[derive(Debug)]
pub struct RawPlugin {
    pub name: String,
    pub systems: Vec<SystemBuilder>,
}

#[derive(Debug)]
pub struct RawPluginBundle {
    plugins: Vec<RawPlugin>,
}

impl RawPluginBundle {
    pub fn create() -> Self {
        RawPluginBundle {
            plugins: Vec::new(),
        }
    }

    pub fn plugin(&mut self, plugin: RawPlugin) {
        self.plugins.push(plugin);
        debug_dump_changed(self);
    }

    pub fn consume(self) -> Vec<RawPlugin> {
        self.plugins
    }
}

impl TargetedDump for RawPluginBundle {
    fn target() -> DebugDumpTargets {
        DebugDumpTargets::Plugins
    }
}
