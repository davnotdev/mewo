use std::collections::HashMap;
use std::any::{
    Any, 
    TypeId,
};

pub type ResourceModifyCallback = fn(&mut ResourceManager);

pub struct ResourceManager {
    data: HashMap<TypeId, Box<dyn Any>>,
}

impl ResourceManager {
    pub fn create() -> ResourceManager {
        ResourceManager {
            data: HashMap::new(),
        }
    }

    pub fn insert<R>(&mut self, resource: R) 
        where R: 'static
    {
        self.data.insert(TypeId::of::<R>(), Box::new(resource));
    }

    pub fn get<R>(&self) -> &R 
        where R: 'static
    {
        self.data
            .get(&TypeId::of::<R>())
            .unwrap()
            .downcast_ref::<R>()
            .unwrap()
    }

    pub fn get_mut<R>(&mut self) -> &mut R 
        where R: 'static
    {
        self.data
            .get_mut(&TypeId::of::<R>())
            .unwrap()
            .downcast_mut::<R>()
            .unwrap()
    }
}

