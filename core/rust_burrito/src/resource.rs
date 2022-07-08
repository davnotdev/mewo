use mewo_ecs::{
    DropFunction, ResourceHash, ResourceManager, ResourceModify, ResourceModifyFunction, TVal,
    ValueDrop,
};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub trait Resource: Sized + 'static {
    fn resource_name() -> String {
        format!(
            "{}_{}",
            env!("CARGO_PKG_NAME"),
            std::any::type_name::<Self>()
        )
    }

    fn resource_hash() -> ResourceHash {
        let mut hasher = DefaultHasher::new();
        std::any::TypeId::of::<Self>().hash(&mut hasher);
        hasher.finish()
    }

    fn resource_drop_callback() -> DropFunction {
        |ptr| unsafe { drop(std::ptr::read(ptr as *const Self as *mut Self)) }
    }

    fn resource_size() -> usize {
        std::mem::size_of::<Self>()
    }
}

pub struct ResourceBus<'rcmodify> {
    modify: &'rcmodify mut ResourceModify,
}

impl<'rcmodify> ResourceBus<'rcmodify> {
    pub fn create(modify: &'rcmodify mut ResourceModify) -> Self {
        ResourceBus { modify }
    }

    pub fn modify<F>(&mut self, f: F)
    where
        F: Fn(ResourceModifyInstance) + 'static,
    {
        self.modify.insert(Box::new(ResourceModifyFunction(
            move |rcmgr: &mut ResourceManager| (f)(ResourceModifyInstance::create(rcmgr)),
        )))
    }
}

pub struct ResourceModifyInstance<'rcmgr> {
    rcmgr: &'rcmgr mut ResourceManager,
}

impl<'rcmgr> ResourceModifyInstance<'rcmgr> {
    pub fn create(rcmgr: &'rcmgr mut ResourceManager) -> Self {
        ResourceModifyInstance { rcmgr }
    }

    pub fn get<R: Resource>(&self) -> Option<&R> {
        self.rcmgr
            .get_resource(R::resource_hash())
            .unwrap()
            .map(|val| unsafe { &*(val.get() as *const R) })
    }

    pub fn get_mut<R: Resource>(&mut self) -> Option<&mut R> {
        self.rcmgr
            .get_mut_resource(R::resource_hash())
            .unwrap()
            .map(|val| unsafe { &mut *(val.get() as *mut R) })
    }

    pub fn insert<R: Resource>(&mut self, r: R) {
        self.rcmgr
            .insert(
                R::resource_hash(),
                TVal::create(
                    R::resource_size(),
                    &r as *const R as *const u8,
                    ValueDrop::create(R::resource_drop_callback()),
                ),
            )
            .unwrap();
        std::mem::forget(r);
    }

    pub fn remove<R: Resource>(&mut self) {
        self.rcmgr.remove(R::resource_hash()).unwrap();
    }
}
