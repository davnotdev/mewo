use mewo_ecs::*;
use std::time::{Duration, Instant};

pub struct Time {
    last_instant: Instant,
}
impl Resource for Time {}

impl Time {
    pub fn delta_time(&self) -> Duration {
        let now = Instant::now();
        now - self.last_instant
    }
}

fn update_time(args: &mut SystemArgs, _w: Wish<(), ()>) {
    args.cmds.modify_resources(|rmgr| {
        rmgr.get_mut::<Time>().unwrap().last_instant = Instant::now();
    });
}

pub struct TimePlugin;

impl TimePlugin {
    pub fn name() -> &'static str {
        "mewo_tk_common_time"
    }

    pub fn plugin(pb: &mut PluginBuilder) {
        let mut cmds = pb.commands();
        cmds.modify_resources(|rmgr| {
            rmgr.insert::<Time>(Time {
                last_instant: Instant::now(),
            });
        });
        pb.sys(update_time);
    }
}
