mod dump;
mod log;
pub mod prelude;

pub use dump::{debug_insert_dump_hook, debug_request_dump, DebugDumpHook, DebugDumpTargets};
pub use log::{
    debug_insert_log_hook, DebugLogHook, DebugMessage, DebugMessageLevel, InternalError,
    InternalErrorType,
};

//  Since we only insert hooks on the same thread and never borrow mutably ever again, we don't
//  need a lock.
//
//  fn global_debug_lock() -> &'static AtomicBool {
//      static mut GLOBAL_DEBUG_LOCK: AtomicBool = AtomicBool::new(false);
//      unsafe { &GLOBAL_DEBUG_LOCK }
//  }
