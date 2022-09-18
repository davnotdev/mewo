use super::{Galaxy, ResourceId};
use crate::data::{data_clone, data_drop, hash_type, TVal, TypeEntry, ValueClone, ValueDrop};
use std::ops::Deref;

pub trait Resource: Clone {
    fn mewo_resource_id() -> ResourceId
    where
        Self: 'static,
    {
        ResourceId::from_hash(hash_type::<Self>())
    }

    fn mewo_resource_type_entry() -> TypeEntry {
        TypeEntry {
            size: Self::mewo_resource_size(),
            name: String::from(std::any::type_name::<Self>()),
            drop: Self::mewo_resource_drop(),
            clone: Self::mewo_resource_clone(),
        }
    }

    fn mewo_resource_size() -> usize {
        std::mem::size_of::<Self>()
    }

    fn mewo_resource_drop() -> ValueDrop {
        data_drop::<Self>()
    }

    fn mewo_resource_clone() -> ValueClone {
        data_clone::<Self>()
    }
}

pub struct ResourceReadGuard<'gal, EX, R> {
    r: Option<&'gal R>,
    galaxy: &'gal Galaxy<EX>,
    id: ResourceId,
}

impl<'gal, EX, R> Drop for ResourceReadGuard<'gal, EX, R> {
    fn drop(&mut self) {
        self.galaxy.rcp.read().get_read_unlock(self.id).unwrap();
    }
}

impl<'gal, EX, R> Deref for ResourceReadGuard<'gal, EX, R> {
    type Target = Option<&'gal R>;
    fn deref(&self) -> &Self::Target {
        &self.r
    }
}

pub struct ResourceWriteGuard<'gal, EX, R> {
    r: Option<&'gal mut R>,
    galaxy: &'gal Galaxy<EX>,
    id: ResourceId,
}

impl<'gal, EX, R> Drop for ResourceWriteGuard<'gal, EX, R> {
    fn drop(&mut self) {
        self.galaxy.rcp.read().get_write_unlock(self.id).unwrap();
    }
}

impl<'gal, EX, R> Deref for ResourceWriteGuard<'gal, EX, R> {
    type Target = Option<&'gal mut R>;
    fn deref(&self) -> &Self::Target {
        &self.r
    }
}

impl<EX> Galaxy<EX> {
    pub fn insert_resource<R: Resource + 'static>(&self, r: R) -> &Self {
        self.resource_maybe_insert::<R>();
        let id = R::mewo_resource_id();
        let rcp = self.rcp.write();
        {
            let val = rcp.get_write_lock(id).unwrap();
            *val = Some(TVal::new(
                R::mewo_resource_size(),
                &r as *const R as *const u8,
                R::mewo_resource_drop(),
            ));
            std::mem::forget(r);
            rcp.get_write_unlock(id).unwrap();
        }
        self
    }

    pub fn remove_resource<R: Resource + 'static>(&self) -> &Self {
        self.resource_maybe_insert::<R>();
        let id = R::mewo_resource_id();
        let rcp = self.rcp.write();
        {
            let val = rcp.get_write_lock(id).unwrap();
            *val = None;
            rcp.get_write_unlock(id).unwrap();
        }
        self
    }

    pub fn get_resource<R: Resource + 'static>(&self) -> ResourceReadGuard<EX, R> {
        let id = R::mewo_resource_id();
        let rcp = self.rcp.read();
        let rc = rcp.get_read_lock(id).unwrap();
        ResourceReadGuard {
            r: rc.as_ref().map(|val| unsafe { &*(val.get() as *const R) }),
            id,
            galaxy: self,
        }
    }

    pub fn get_mut_resource<R: Resource + 'static>(&self) -> ResourceWriteGuard<EX, R> {
        let id = R::mewo_resource_id();
        let rcp = self.rcp.read();
        let rc = rcp.get_read_lock(id).unwrap();
        ResourceWriteGuard {
            r: rc
                .as_ref()
                .map(|val| unsafe { &mut *(val.get() as *const R as *mut R) }),
            id,
            galaxy: self,
        }
    }

    //  TODO EXT: with_resource, with_mut_resource

    fn resource_maybe_insert<R: Resource + 'static>(&self) {
        let rcp = self.rcp.read();
        let id = R::mewo_resource_id();
        if rcp.get_type(id).is_none() {
            drop(rcp);
            let mut rcp = self.rcp.write();
            rcp.insert_type(id, R::mewo_resource_type_entry()).unwrap();
        }
    }
}
