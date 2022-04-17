use std::collections::HashMap;
use std::any::{
    Any, 
    TypeId,
};

pub trait Resource {
}

pub struct ResourceModifyCallback<F: Fn(&mut ResourceManager) -> ()>(pub F);
pub trait GenericResourceModifyCallback {
    fn call(&self, rmgr: &mut ResourceManager);
}
pub type BoxedResourceModifyCallback = Box<dyn GenericResourceModifyCallback>;

impl<F> GenericResourceModifyCallback for ResourceModifyCallback<F> 
    where F: Fn(&mut ResourceManager) -> ()
{
    fn call(&self, rmgr: &mut ResourceManager) {
        (self.0)(rmgr)
    }
}

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
        where R: 'static + Resource
    {
        self.data.insert(TypeId::of::<R>(), Box::new(resource));
    }

    pub fn get<R>(&self) -> &R 
        where R: 'static + Resource
    {
        if let Some(any) = self.data.get(&TypeId::of::<R>()) {
            any
                .downcast_ref::<R>()
                .unwrap()
        } else {
            panic!("Resource `{}` Not Found", std::any::type_name::<R>())
        }
    }

    pub fn get_mut<R>(&mut self) -> &mut R 
        where R: 'static + Resource
    {
        if let Some(any) = self.data.get_mut(&TypeId::of::<R>()) {
            any
                .downcast_mut::<R>()
                .unwrap()
        } else {
            panic!("Resource `{}` Not Found", std::any::type_name::<R>())
        }
    }
}

