use super::*;
use log::{Level, Metadata, Record};
use parking_lot::RwLock;
use std::{collections::HashSet, sync::Once, time::Instant};

struct Rust2MewoLogger {
    deny: RwLock<HashSet<Level>>,
}

impl log::Log for Rust2MewoLogger {
    fn enabled(&self, _: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        let logger = Logger::get_global_logger().read();

        if self.deny.read().contains(&record.level()) {
            return;
        }

        let target = match record.level() {
            Level::Info | Level::Debug | Level::Trace => LogTarget::Info,
            Level::Warn => LogTarget::Warn,
            //  TODO FIX: Perhaps this should be Fatal?
            Level::Error => LogTarget::Error,
        };

        logger.insert_record(LogRecord {
            target,
            time: Instant::now(),
            line: record.line().unwrap(),
            file: record.file().unwrap().to_string(),
            msg: record.args().to_string(),
        })
    }

    fn flush(&self) {}
}

fn get_global_rs_logger() -> &'static Rust2MewoLogger {
    static mut RS_LOGGER: Option<Rust2MewoLogger> = None;
    static INIT: Once = Once::new();
    INIT.call_once(|| unsafe {
        RS_LOGGER = Some(Rust2MewoLogger {
            deny: RwLock::new(HashSet::from_iter([Level::Debug, Level::Trace].into_iter())),
        });
        log::set_logger(RS_LOGGER.as_ref().unwrap()).unwrap();
    });
    unsafe { RS_LOGGER.as_ref().unwrap() }
}

pub fn rs_log_init(_: &Galaxy) {}

pub fn rs_log_allow(allow: Level) {
    get_global_rs_logger().deny.write().insert(allow);
}

pub fn rs_log_deny(deny: Level) {
    get_global_rs_logger().deny.write().remove(&deny);
}
