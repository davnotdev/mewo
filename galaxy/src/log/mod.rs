use crate::data::ThreadLocal;
use parking_lot::RwLock;
use std::{sync::Once, time::Instant};

#[cfg(test)]
mod test;

//  Logging is done via Folds and Records:
//
//      FoldStart(System("some_sys"))
//          Record(Error("Everything blew up!"))
//      FoldEnd
//      FoldStart(GalaxyUpdate)
//          Record(Error("Bad Entity"))
//      FoldEnd

static mut GLOBAL_LOGGER: Option<RwLock<Logger>> = None;

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct LogSubscription(usize);

impl LogSubscription {
    pub fn id(&self) -> usize {
        self.0
    }
}

//  TODO EXT: implement a Fatal LogTarget.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LogTarget {
    Info,
    Warn,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogRecord {
    pub time: Instant,
    pub line: u32,
    pub file: String,
    pub target: LogTarget,
    pub msg: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LogEvent {
    FoldStart(&'static str),
    Record(LogRecord),
    FoldEnd,
}

struct LogChannel {
    evs: Vec<LogEvent>,
}

impl LogChannel {
    pub fn new() -> Self {
        LogChannel { evs: Vec::new() }
    }
}

pub struct LogFold;

impl LogFold {
    pub fn new(name: &'static str) -> Self {
        let logger = Logger::get_global_logger().read();
        logger
            .channel
            .get_or(LogChannel::new)
            .evs
            .push(LogEvent::FoldStart(name));
        LogFold
    }
}

impl Drop for LogFold {
    fn drop(&mut self) {
        let logger = Logger::get_global_logger().read();
        logger
            .channel
            .get_or(LogChannel::new)
            .evs
            .push(LogEvent::FoldEnd);
    }
}

pub struct Logger {
    channel: ThreadLocal<LogChannel>,
    //  Subscription -> (current idx) per thread
    subs: Vec<Vec<usize>>,
}

impl Logger {
    pub fn new() -> Self {
        Logger {
            channel: ThreadLocal::new(),
            subs: Vec::new(),
        }
    }

    pub fn insert_record(&self, record: LogRecord) {
        self.channel
            .get_or(LogChannel::new)
            .evs
            .push(LogEvent::Record(record));
    }

    pub fn subscribe(&mut self) -> LogSubscription {
        let id = self.subs.len();
        self.subs.push(Vec::new());
        LogSubscription(id)
    }

    pub fn take(&mut self, id: LogSubscription) -> Option<LogEvent> {
        let news = &mut self.subs[id.id()];
        let channels = unsafe { self.channel.get_inner() };

        //  Make sure that all threads including new ones are accounted for.
        for _ in news.len()..channels.len() {
            news.push(0);
        }

        for (channel_idx, news_current) in news.iter_mut().enumerate() {
            if *news_current < channels[channel_idx].evs.len() {
                *news_current += 1;
                return Some(channels[channel_idx].evs[*news_current - 1].clone());
            }
        }

        None
    }

    pub fn get_global_logger() -> &'static RwLock<Logger> {
        static INIT: Once = Once::new();
        INIT.call_once(|| unsafe {
            GLOBAL_LOGGER = Some(RwLock::new(Logger::new()));
        });
        unsafe { GLOBAL_LOGGER.as_ref() }.unwrap()
    }
}

impl Default for Logger {
    fn default() -> Self {
        Self::new()
    }
}

//  Macros inside macros are experimental (I believe).

#[macro_export]
macro_rules! mfold {
    ($name:expr) => {
        LogFold::new($name)
    };
}

#[macro_export]
macro_rules! minfo {
    ($($arg:tt)*) => {{
        let record = LogRecord {
            time: std::time::Instant::now(),
            line: line!(),
            file: file!().to_string(),
            target: LogTarget::Info,
            msg: format!($($arg)*),
        };
        Logger::get_global_logger().read().insert_record(record);
    }};
}

#[macro_export]
macro_rules! mwarn {
    ($($arg:tt)*) => {{
        let record = LogRecord {
            time: std::time::Instant::now(),
            line: line!(),
            file: file!().to_string(),
            target: LogTarget::Warn,
            msg: format!($($arg)*),
        };
        Logger::get_global_logger().read().insert_record(record);
    }};
}

#[macro_export]
macro_rules! merr {
    ($($arg:tt)*) => {{
        let record = LogRecord {
            time: std::time::Instant::now(),
            line: line!(),
            file: file!().to_string(),
            target: LogTarget::Error,
            msg: format!($($arg)*),
        };
        Logger::get_global_logger().read().insert_record(record);
    }};
}
