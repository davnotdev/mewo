use mewo_galaxy::data::hash_type_and_val;

use super::*;

pub trait Executable {
    fn run(&self, galaxy: &Galaxy) -> Result<(), String>;
    fn hash_ptr(&self) -> u64 {
        let ptr = self as *const Self as *const ();
        hash_type_and_val(ptr)
    }
}

impl<F: Fn(&Galaxy) -> Result<(), String>> Executable for F {
    fn run(&self, galaxy: &Galaxy) -> Result<(), String> {
        (self)(galaxy)
    }
}

fn any_sys_into_executable<T: 'static>(f: fn(&Galaxy) -> T) -> Box<dyn Executable> {
    Box::new(move |galaxy: &Galaxy| {
        (f)(galaxy);
        Ok(())
    })
}

fn result_sys_into_executable<T: 'static, E: 'static + ToString>(
    f: fn(&Galaxy) -> Result<T, E>,
) -> Box<dyn Executable> {
    Box::new(move |galaxy: &Galaxy| (f)(galaxy).map(|_| ()).map_err(|e| e.to_string()))
}

pub fn system<T: 'static, S: 'static + SystemSet>(f: fn(&Galaxy) -> T, s: S) -> SystemConfig {
    let sys = any_sys_into_executable(f);
    SystemConfig::new(sys, s)
}

pub fn system_result<T: 'static, E: 'static + ToString, S: 'static + SystemSet>(
    f: fn(&Galaxy) -> Result<T, E>,
    s: S,
) -> SystemConfig {
    let sys = result_sys_into_executable(f);
    SystemConfig::new(sys, s)
}

pub struct SystemConfig {
    pub sys: Box<dyn Executable>,
    pub of_set: u64,
}

impl SystemConfig {
    fn new<S: 'static + SystemSet>(sys: Box<dyn Executable>, s: S) -> Self {
        SystemConfig {
            sys,
            of_set: s.hash_with_val(),
        }
    }
}
