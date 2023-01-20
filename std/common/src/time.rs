use super::*;
use std::time::{Duration, Instant};

#[derive(SingleResource)]
pub struct GlobalTime {
    last_instant: Instant,
}

impl GlobalTime {
    pub fn new() -> Self {
        GlobalTime {
            last_instant: Instant::now(),
        }
    }

    pub fn delta_time(&mut self) -> Duration {
        let now = Instant::now();
        let dt = now - self.last_instant;
        self.last_instant = now;
        dt
    }
}

impl Default for GlobalTime {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy)]
pub struct Timer {
    elapsed: Duration,
    interval: Duration,
}

impl Timer {
    pub fn new(interval: Duration) -> Self {
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

pub fn time_init(g: &Galaxy) {
    g.insert_resource(GlobalTime::single_resource(), GlobalTime::new());
}

pub fn time_global_update(g: &Galaxy) {
    if let Some(mut time) = g.get_mut_resource::<GlobalTime, _>(GlobalTime::single_resource()) {
        time.last_instant = Instant::now();
    } else {
        //  TODO FIX: Switch to some logging system.
        panic!("time_init not called");
    }
}
