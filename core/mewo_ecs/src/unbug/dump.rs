const DEBUG_DUMP_TARGET_COUNT: usize = DebugDumpTargets::LEN as usize;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DebugDumpTargets {
    EntityManager = 0,
    ComponentTypeManager = 1,
    ArchetypeManager = 2,
    Plugins = 3,
    Events = 4,
    Resources = 5,

    LEN = 6,
}

//  Returns vec of lines without '\n'.

pub trait TargetedDump: std::fmt::Debug {
    fn target() -> DebugDumpTargets;
}

pub type DebugDumpHook = fn(DebugDumpTargets);

struct DebugDumpTruck {
    dumps: [Option<String>; DEBUG_DUMP_TARGET_COUNT],
    hooks: Vec<DebugDumpHook>,
}

//  Manual locking required
fn global_dump_truck() -> &'static mut DebugDumpTruck {
    static mut GLOBAL_DUMP_TRUCK: DebugDumpTruck = DebugDumpTruck {
        dumps: [None, None, None, None, None, None],
        hooks: Vec::new(),
    };
    unsafe { &mut GLOBAL_DUMP_TRUCK }
}

pub fn debug_insert_dump_hook(hook: DebugDumpHook) {
    let truck = global_dump_truck();
    truck.hooks.push(hook);
}

pub fn debug_dump_changed<D: TargetedDump>(d: &D) {
    let truck = global_dump_truck();
    let dump = format!("{:?}", d);
    let target = D::target();
    truck.dumps[target as usize] = Some(dump);
    for hook in truck.hooks.iter() {
        (hook)(target);
    }
}

pub fn debug_request_dump(dump_target: DebugDumpTargets) -> Option<String> {
    let truck = global_dump_truck();
    let ret = truck.dumps[dump_target as usize].clone();
    ret
}
