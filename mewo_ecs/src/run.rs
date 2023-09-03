use super::*;
use std::thread;

/// Block the current thread, driving the `galaxy` with `systems`.
pub fn run_single(mut galaxy: Galaxy, systems: &[fn(&Galaxy)]) {
    loop {
        systems.iter().for_each(|sys| sys(&galaxy));
        if galaxy.update().is_none() {
            return;
        }
    }
}

/// Spawn a new thread and drive `galaxy` with `systems`.
/// `pre_update` and `post_update` are called before updating and after updating respectively.
/// `pre_update` and `post_update` don't lock `galaxy`, so they don't impose side effects on other
/// threads.
/// In other words, you can safely block or wait in these functions.
///
/// Compared to [`self::run_spawn`], this function read locks `galaxy` once meaning that other
/// threads cannot update until all systems have completed.
pub fn run_spawn_locked(
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

/// Spawn a new thread and drive `galaxy` with `systems`.
/// `pre_update` and `post_update` are called before updating and after updating respectively.
/// `pre_update` and `post_update` don't lock `galaxy`, so they don't impose side effects on other
/// threads.
/// In other words, you can safely block or wait in these functions.
///
/// Compared to [`self::run_spawn_locked`], this function read locks `galaxy` on each system call.
/// This means that other threads can update between system calls.
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
