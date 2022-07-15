use super::params::Resources;
use mewo_ecs::{DropFunction, ResourceHash, ResourceManager, TVal, ValueDrop};
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    marker::PhantomData,
};

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

//  Q: Why is getting a resource considered &mut.
//  A: Well, this somewhat prevents you from taking a &mut R0 twice, and it also encourages you to
//  use multiple gets meaning more unlocks for other threads. However, the downside is that it may
//  be annoying as hell. Question for you. Should we keep this?

pub struct ResourceBus<'exec> {
    rcmgr: &'exec ResourceManager,
}

impl<'exec> ResourceBus<'exec> {
    pub fn create(rcmgr: &'exec ResourceManager) -> Self {
        ResourceBus { rcmgr }
    }

    pub fn get<RS>(&mut self) -> ResourceBurrito<'exec, '_, RS>
    where
        RS: Resources,
    {
        RS::lock(self.rcmgr);
        ResourceBurrito::<RS>::create(self)
    }

    pub fn insert<R>(&self, rc: R) -> &Self
    where
        R: Resource,
    {
        self.rcmgr.lock(mewo_ecs::ResourceQueryAccessType::Write);
        *self.rcmgr.locked_get_mut(R::resource_hash()).unwrap() = Some(TVal::create(
            R::resource_size(),
            &rc as *const R as *const u8,
            ValueDrop::create(R::resource_drop_callback()),
        ));
        self.rcmgr.unlock(mewo_ecs::ResourceQueryAccessType::Write);
        std::mem::forget(rc);
        self
    }

    pub fn remove<R>(&self) -> &Self
    where
        R: Resource,
    {
        self.rcmgr.lock(mewo_ecs::ResourceQueryAccessType::Write);
        *self.rcmgr.locked_get_mut(R::resource_hash()).unwrap() = None;
        self.rcmgr.unlock(mewo_ecs::ResourceQueryAccessType::Write);
        self
    }
}

pub struct ResourceBurrito<'exec, 'rbus, RS: Resources> {
    rbus: &'rbus mut ResourceBus<'exec>,
    phantom: PhantomData<RS>,
}

impl<'exec, 'rbus, RS> ResourceBurrito<'exec, 'rbus, RS>
where
    RS: Resources,
{
    pub fn create(rbus: &'rbus mut ResourceBus<'exec>) -> Self {
        ResourceBurrito {
            rbus,
            phantom: PhantomData,
        }
    }

    pub fn get(&mut self) -> Option<RS> {
        RS::get(self.rbus.rcmgr)
    }
}

impl<'exec, 'rbus, RS> Drop for ResourceBurrito<'exec, 'rbus, RS>
where
    RS: Resources,
{
    fn drop(&mut self) {
        RS::unlock(self.rbus.rcmgr)
    }
}
