use crate::util::{hash_type, name_type};
use mewo_ecs::{
    DropFunction, ResourceHash, ResourceManager, ResourceQueryAccessType, ResourceTypeEntry,
    SharedResourceManager, TVal, ValueDrop,
};
use std::marker::PhantomData;

mod impls;

pub trait Resource: Sized + 'static {
    fn resource_trait_entry() -> ResourceTypeEntry {
        ResourceTypeEntry {
            name: name_type::<Self>(),
            hash: hash_type::<Self>(),
        }
    }
    fn resource_trait_drop_callback() -> DropFunction {
        |ptr| unsafe { drop(std::ptr::read(ptr as *const Self as *mut Self)) }
    }

    fn resource_trait_size() -> usize {
        std::mem::size_of::<Self>()
    }

    fn resource_trait_hash() -> ResourceHash {
        hash_type::<Self>()
    }
}

pub trait Resources {
    fn accesses() -> Vec<(ResourceHash, ResourceQueryAccessType)>;
    fn lock(rcmgr: &ResourceManager);
    fn unlock(rcmgr: &ResourceManager);
    fn get(rcmgr: &ResourceManager) -> Option<Self>
    where
        Self: Sized;
    fn maybe_register(rcmgr: &SharedResourceManager);
}

trait ResourceAccess {
    fn data(data: *const u8) -> Self;
    fn hash() -> ResourceHash;
    fn access() -> ResourceQueryAccessType;
    fn maybe_register(rcmgr: &SharedResourceManager);
}

//  Q: Why is getting a resource considered &mut.
//  A: Well, this somewhat prevents you from taking a &mut R0 twice since shared references may, and it also encourages you to
//  use multiple gets meaning more unlocks for other threads. However, the downside is that it may
//  be annoying as hell. Question for you. Should we keep this?

pub struct ResourceBus<'galaxy> {
    rcmgr: &'galaxy SharedResourceManager,
}

impl<'galaxy> ResourceBus<'galaxy> {
    pub fn create(rcmgr: &'galaxy SharedResourceManager) -> Self {
        ResourceBus { rcmgr }
    }

    pub fn get<RS>(&mut self) -> ResourceBurrito<'galaxy, '_, RS>
    where
        RS: Resources,
    {
        RS::maybe_register(self.rcmgr);
        RS::lock(&self.rcmgr.read().unwrap());
        ResourceBurrito::<RS>::create(self)
    }

    pub fn insert<R>(&self, rc: R) -> &Self
    where
        R: Resource,
    {
        <&R as ResourceAccess>::maybe_register(self.rcmgr);
        let rcmgr = self.rcmgr.read().unwrap();
        rcmgr.lock(mewo_ecs::ResourceQueryAccessType::Write);
        *rcmgr
            .locked_get_mut(R::resource_trait_entry().hash)
            .unwrap() = Some(TVal::create(
            R::resource_trait_size(),
            &rc as *const R as *const u8,
            ValueDrop::create(R::resource_trait_drop_callback()),
        ));
        rcmgr.unlock(mewo_ecs::ResourceQueryAccessType::Write);
        std::mem::forget(rc);
        self
    }

    pub fn remove<R>(&self) -> &Self
    where
        R: Resource,
    {
        <&R as ResourceAccess>::maybe_register(self.rcmgr);
        let rcmgr = self.rcmgr.read().unwrap();
        rcmgr.lock(mewo_ecs::ResourceQueryAccessType::Write);
        *rcmgr
            .locked_get_mut(R::resource_trait_entry().hash)
            .unwrap() = None;
        rcmgr.unlock(mewo_ecs::ResourceQueryAccessType::Write);
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
        RS::get(&self.rbus.rcmgr.read().unwrap())
    }
}

impl<'exec, 'rbus, RS> Drop for ResourceBurrito<'exec, 'rbus, RS>
where
    RS: Resources,
{
    fn drop(&mut self) {
        RS::unlock(&self.rbus.rcmgr.read().unwrap())
    }
}
