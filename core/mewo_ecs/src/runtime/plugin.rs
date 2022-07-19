use super::system::SystemBuilder;
use crate::{
    component::ComponentTypeEntry, event::EventTypeEntry, resource::ResourceTypeEntry,
    unbug::prelude::*,
};

#[derive(Debug)]
pub struct RawPlugin {
    pub name: String,
    pub deps: Vec<String>,
    pub events: Vec<EventTypeEntry>,
    pub systems: Vec<SystemBuilder>,
    pub components: Vec<ComponentTypeEntry>,
    pub resources: Vec<ResourceTypeEntry>,
}

#[derive(Debug)]
pub struct RawPluginBundle {
    plugins: Vec<RawPlugin>,
    included_deps: Vec<String>,
}

impl RawPluginBundle {
    pub fn create() -> Self {
        RawPluginBundle {
            plugins: Vec::new(),
            included_deps: Vec::new(),
        }
    }

    pub fn plugin(&mut self, plugin: RawPlugin) -> Result<()> {
        let mut unmet = Vec::new();
        for dep in plugin.deps.iter() {
            if *dep == plugin.name {
                Err(InternalError {
                    line: line!(),
                    file: file!(),
                    dumps: vec![],
                    explain: None,
                    ty: InternalErrorType::PluginDependsOnSelf {
                        plugin: plugin.name.clone(),
                    },
                })?
            }
            if let None = self.included_deps.iter().position(|d| d == dep) {
                unmet.push(dep.clone());
            }
        }
        if !unmet.is_empty() {
            return Err(InternalError {
                line: line!(),
                file: file!(),
                dumps: vec![DebugDumpTargets::Plugins],
                explain: None,
                ty: InternalErrorType::PluginDependenciesNoMet {
                    plugin: plugin.name,
                    unmet,
                },
            });
        }
        let name = plugin.name.clone();
        self.plugins.push(plugin);
        self.included_deps.push(name);
        debug_dump_changed(self);
        Ok(())
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

#[test]
fn test_plugin_bundle() {
    let a = RawPlugin {
        name: String::from("a"),
        deps: vec![],
        events: vec![],
        systems: vec![],
        components: Vec::new(),
        resources: Vec::new(),
    };

    let b = RawPlugin {
        name: String::from("b"),
        deps: vec![String::from("a")],
        events: vec![],
        systems: vec![],
        components: Vec::new(),
        resources: Vec::new(),
    };

    let c = RawPlugin {
        name: String::from("c"),
        deps: vec![String::from("c")],
        events: vec![],
        systems: vec![],
        components: Vec::new(),
        resources: Vec::new(),
    };

    fn clone(plugin: &RawPlugin) -> RawPlugin {
        RawPlugin {
            name: plugin.name.clone(),
            deps: plugin.deps.clone(),
            events: vec![],
            systems: Vec::new(),
            components: Vec::new(),
            resources: Vec::new(),
        }
    }

    let mut pb = RawPluginBundle::create();
    assert_eq!(pb.plugin(clone(&a)), Ok(()));

    let mut pb = RawPluginBundle::create();
    assert!(match pb.plugin(clone(&b)) {
        Err(InternalError {
            ty:
                InternalErrorType::PluginDependenciesNoMet {
                    plugin: bstr,
                    unmet: astr,
                },
            ..
        }) if (bstr == "b" && astr == vec!["a"]) => true,
        _ => false,
    });

    let mut pb = RawPluginBundle::create();
    assert_eq!(pb.plugin(clone(&a)), Ok(()));
    assert_eq!(pb.plugin(clone(&b)), Ok(()));

    let mut pb = RawPluginBundle::create();
    assert!(match pb.plugin(clone(&c)) {
        Err(InternalError {
            ty: InternalErrorType::PluginDependsOnSelf { plugin: cstr },
            ..
        }) if cstr == "c" => true,
        _ => false,
    });
}
