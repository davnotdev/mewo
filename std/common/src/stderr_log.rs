use super::*;
use std::time::Instant;

#[derive(Resource)]
pub struct StderrLog {
    load_time: Instant,
    sub: LogSubscription,
    tabc: usize,
}

fn eprint_tabs(tabc: usize) {
    (0..tabc).for_each(|_| eprint!("\t"))
}

impl StderrLog {
    fn log(&mut self) {
        let mut logger = Logger::get_global_logger().write();
        while let Some(ev) = logger.take(self.sub) {
            match ev {
                LogEvent::FoldStart(name) => {
                    eprint_tabs(self.tabc);
                    eprintln!("{} {{", name);
                    self.tabc += 1;
                }
                LogEvent::FoldEnd => {
                    self.tabc -= 1;
                    eprint_tabs(self.tabc);
                    eprintln!("}}");
                }
                LogEvent::Record(rec) => {
                    eprint_tabs(self.tabc);
                    eprintln!(
                        "[{:09}:{}:{}] {}",
                        rec.time.duration_since(self.load_time).as_millis(),
                        rec.file,
                        rec.line,
                        rec.msg
                    );
                }
            }
        }
    }
}

impl Drop for StderrLog {
    fn drop(&mut self) {
        self.log();
    }
}

pub fn stderr_log_init(g: &Galaxy) {
    let sub = Logger::get_global_logger().write().subscribe();
    g.insert_resource(StderrLog {
        sub,
        tabc: 0,
        load_time: Instant::now(),
    });
}

pub fn stderr_log_update(g: &Galaxy) {
    if let Some(mut log) = g.get_mut_resource::<StderrLog>() {
        log.log();
    } else {
        panic!()
    }
}
