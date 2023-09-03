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

pub fn run_spawn(galaxy: Arc<RwLock<Galaxy>>, systems: &[fn(&Galaxy)]) -> thread::JoinHandle<()> {
    let systems = systems.to_owned();
    thread::spawn(move || loop {
        {
            let galaxy = galaxy.read().unwrap();
            systems.iter().for_each(|sys| sys(&galaxy));
        }
        {
            let mut galaxy = galaxy.write().unwrap();
            galaxy.update();
        }
    })
}
