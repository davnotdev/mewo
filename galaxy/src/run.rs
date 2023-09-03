use super::prelude::*;
use std::{
    sync::{Arc, RwLock},
    thread,
};

pub fn run_single(mut galaxy: Galaxy, systems: &[fn(&Galaxy)]) -> ! {
    loop {
        systems.iter().for_each(|sys| sys(&galaxy));
        galaxy.update();
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
            let galaxy = galaxy.read().unwrap();
            systems.iter().for_each(|sys| sys(&galaxy));
        }
        {
            let mut galaxy = galaxy.write().unwrap();
            galaxy.update();
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
                let galaxy = galaxy.read().unwrap();
                sys(&galaxy)
            });
        }
        {
            let mut galaxy = galaxy.write().unwrap();
            galaxy.update();
        }

        post_update(&galaxy);
    })
}
