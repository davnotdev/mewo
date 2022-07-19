use super::dump::*;
use crate::{
    component::{ArchetypeAccessKey, ComponentGroupId, ComponentHash, ComponentTypeId, Entity},
    event::EventHash,
    resource::ResourceHash,
};

pub type Result<T> = std::result::Result<T, InternalError>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InternalError {
    pub line: u32,
    pub file: &'static str,
    pub dumps: Vec<DebugDumpTargets>,
    pub explain: Option<&'static str>,
    pub ty: InternalErrorType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InternalErrorType {
    BadEntity { e: Entity },
    BadComponentType { ctyid: ComponentTypeId },
    BadComponentTypeName { name: String },
    BadComponentTypeHash { hash: u64 },
    DuplicateComponentTypeHash { hash: ComponentHash },
    BadComponentGroup { gid: ComponentGroupId },
    ArchetypeStorageInsertIncomplete { missing: Vec<ComponentTypeId> },
    BadArchetypeManagerAccessIndex { idx: usize, max: usize },
    BadArchetypeAccessKey { akid: ArchetypeAccessKey },
    ArchetypeStorageLocked,
    PluginNoName,
    PluginNoSystems,
    PluginDependsOnSelf { plugin: String },
    PluginDependenciesNoMet { plugin: String, unmet: Vec<String> },
    SystemNoPluginName { system: String },
    DuplicateEventTypeHash { hash: EventHash },
    BadEventTypeHash { hash: EventHash },
    BadEventStorageGetIndex { idx: usize },
    DuplicateResourceTypeHash { hash: ResourceHash },
    BadResourceTypeHash { hash: EventHash },
}

pub enum DebugMessageLevel {
    Internal,
}

impl ToString for DebugMessageLevel {
    fn to_string(&self) -> String {
        match self {
            Self::Internal => "Internal",
        }
        .to_string()
    }
}

pub struct DebugMessage {
    pub line: u32,
    pub file: String,
    pub msg: String,
    pub dumps: Vec<DebugDumpTargets>,
    pub level: DebugMessageLevel,
}

pub type DebugLogHook = fn(&DebugMessage);

pub fn debug_insert_log_hook(hook: DebugLogHook) {
    let hooks = global_log_hooks();
    hooks.push(hook);
}

fn global_log_hooks() -> &'static mut Vec<DebugLogHook> {
    static mut GLOBAL_LOG_HOOKS: Vec<DebugLogHook> = Vec::new();
    unsafe { &mut GLOBAL_LOG_HOOKS }
}

fn run_log_hooks(msg: DebugMessage) {
    let hooks = global_log_hooks();
    for hook in hooks.iter() {
        (hook)(&msg);
    }
}

//  Explain and unwrap, throwing an interal error.
pub trait ErrorExplain {
    type T;
    fn iex_unwrap(self) -> Self::T;
}

impl<T> ErrorExplain for Result<T> {
    type T = T;
    fn iex_unwrap(self) -> Self::T {
        if let Ok(ret) = self {
            return ret;
        } else if let Err(e) = self {
            let mut err = format!("{:?}", e.ty);
            if let Some(explain) = e.explain {
                err += &format!(" ~ {}", explain);
            } else {
                err += " ~ No additional information.";
            }
            let dbg_msg = DebugMessage {
                line: e.line,
                file: e.file.to_string(),
                msg: err,
                dumps: e.dumps,
                level: DebugMessageLevel::Internal,
            };
            run_log_hooks(dbg_msg);
            panic!()
        }
        unreachable!()
    }
}
