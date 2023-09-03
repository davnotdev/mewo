use super::*;
use std::thread;

pub fn run_single(mut galaxy: Galaxy, systems: &[fn(&Galaxy)]) {
    loop {
        systems.iter().for_each(|sys| sys(&galaxy));
        if galaxy.update().is_none() {
            return;
        }
    }
}

pub fn run_spawn(
    galaxy: Arc<RwLock<Galaxy>>,
    systems: &[fn(&Galaxy)],
    pre_update: fn(&Arc<RwLock<Galaxy>>),
    post_update: fn(&Arc<RwLock<Galaxy>>),
) -> thread::JoinHandle<()> {
    let systems = systems.to_owned();
    thread::spawn(move || loop {
        pre_update(&galaxy);

        {
            let galaxy = galaxy.read();
            systems.iter().for_each(|sys| sys(&galaxy));
        }
        {
            let mut galaxy = galaxy.write();
            if galaxy.update().is_none() {
                return;
            }
        }

        post_update(&galaxy);
    })
}

pub fn run_spawn_overlapped(
    galaxy: Arc<RwLock<Galaxy>>,
    systems: &[fn(&Galaxy)],
    pre_update: fn(&Arc<RwLock<Galaxy>>),
    post_update: fn(&Arc<RwLock<Galaxy>>),
) -> thread::JoinHandle<()> {
    let systems = systems.to_owned();
    thread::spawn(move || loop {
        pre_update(&galaxy);

        {
            systems.iter().for_each(|sys| {
                let galaxy = galaxy.read();
                sys(&galaxy)
            });
        }
        {
            let mut galaxy = galaxy.write();
            if galaxy.update().is_none() {
                return;
            }
        }

        post_update(&galaxy);
    })
}
