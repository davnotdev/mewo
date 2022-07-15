use rust_burrito::*;
use std::time::{Duration, Instant};

pub struct TimePlugin;
impl Plugin for TimePlugin {
    fn name() -> &'static str {
        "mewo_common_time"
    }

    fn plugin(pb: PluginBuilder) -> PluginBuilder {
        pb.resource::<Time>()
            .sys(|sb, _: Events<Startup>, _: Components<(), ()>| {
                sb.resources.insert(Time::create());
            })
            .sys(|mut sb, _: Events<()>, _: Components<(), ()>| {
                if let Some(time) = sb.resources.get::<&mut Time>().get() {
                    time.last_instant = Instant::now();
                }
            })
    }
}

pub struct Time {
    last_instant: Instant,
}
impl Resource for Time {}

impl Time {
    pub fn create() -> Self {
        Time {
            last_instant: Instant::now(),
        }
    }

    pub fn delta_time(&mut self) -> Duration {
        let now = Instant::now();
        let ret = now - self.last_instant;
        ret
    }
}

pub struct Timer {
    elapsed: Duration,
    interval: Duration,
}

impl Timer {
    pub fn create(interval: Duration) -> Self {
        Timer {
            elapsed: Duration::from_secs(0),
            interval,
        }
    }

    pub fn tick(&mut self, dt: Duration) -> &mut Self {
        self.elapsed += dt;
        self
    }

    pub fn passed(&mut self) -> bool {
        if self.elapsed >= self.interval {
            self.elapsed = Duration::from_secs(0);
            true
        } else {
            false
        }
    }
}
