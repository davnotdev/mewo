use mewo_galaxy::prelude::*;
use mewo_galaxy_derive::*;

mod backend;
mod convert;
mod fs;
pub mod prelude;
mod server;

#[derive(Event)]
pub struct AssetLoadEvent {
    name: String,
    data: Result<Vec<u8>, ()>,
}

impl AssetLoadEvent {
    pub fn get_name(&self) -> &String {
        &self.name
    }

    pub fn get<C: convert::AssetConverter>(&self) -> C {
        if let Ok(data) = &self.data {
            C::convert(data.clone())
        } else {
            merr!("Load asset {}", self.name);
            panic!("Include a real error exit next time.")
        }
    }
}

#[derive(Event)]
pub struct AssetUnloadEvent {
    name: String,
}

impl AssetUnloadEvent {
    pub fn get_name(&self) -> &String {
        &self.name
    }
}
