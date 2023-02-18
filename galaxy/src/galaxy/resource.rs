use super::{Galaxy, ResourceId};
use crate::data::{data_drop, hash_type_and_val, TVal, TypeEntry, ValueDrop, ValueDuplicate};
use std::{
    hash::Hash,
    ops::{Deref, DerefMut},
};

fn hash_resource_id<RH: Hash + 'static>(rh: RH) -> ResourceId {
    ResourceId::from_hash(hash_type_and_val(rh))
}

pub trait Resource {
    fn mewo_resource_type_entry() -> TypeEntry
    where
        Self: Sized,
    {
        TypeEntry {
            size: Self::mewo_resource_size(),
            name: String::from(std::any::type_name::<Self>()),
            drop: Self::mewo_resource_drop(),
            dup: Self::mewo_resource_dup(),
        }
    }

    fn mewo_resource_size() -> usize
    where
        Self: Sized,
    {
        std::mem::size_of::<Self>()
    }

    fn mewo_resource_drop() -> ValueDrop
    where
        Self: Sized,
    {
        data_drop::<Self>()
    }

    fn mewo_resource_dup() -> ValueDuplicate {
        //  Resource cloning is never used.
        ValueDuplicate::None
    }
}

pub struct ResourceReadGuard<'gal, R> {
    r: &'gal R,
    galaxy: &'gal Galaxy,
    id: ResourceId,
}

impl<'gal, R> Drop for ResourceReadGuard<'gal, R> {
    fn drop(&mut self) {
        self.galaxy.rcp.read().get_read_unlock(self.id).unwrap();
    }
}

impl<'gal, R> Deref for ResourceReadGuard<'gal, R> {
    type Target = R;
    fn deref(&self) -> &Self::Target {
        self.r
    }
}

pub struct ResourceWriteGuard<'gal, R> {
    r: &'gal mut R,
    galaxy: &'gal Galaxy,
    id: ResourceId,
}

impl<'gal, R> Drop for ResourceWriteGuard<'gal, R> {
    fn drop(&mut self) {
        self.galaxy.rcp.read().get_write_unlock(self.id).unwrap();
    }
}

impl<'gal, R> Deref for ResourceWriteGuard<'gal, R> {
    type Target = &'gal mut R;
    fn deref(&self) -> &Self::Target {
        &self.r
    }
}

impl<'gal, R> DerefMut for ResourceWriteGuard<'gal, R> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.r
    }
}

//  TODO FIX: Validate Resource type to prevent unsafe usage.

impl Galaxy {
    pub fn insert_resource<R: Resource, RH: Clone + Hash + 'static>(&self, rh: RH, r: R) -> &Self {
        let id = hash_resource_id(rh.clone());
        self.resource_maybe_insert::<R, RH>(rh);
        let rcp = self.rcp.write();
        {
            let val = rcp.get_write_lock(id).unwrap();
            *val = Some(unsafe {
                TVal::new(
                    R::mewo_resource_size(),
                    &r as *const R as *const u8,
                    R::mewo_resource_drop(),
                )
            });
            std::mem::forget(r);
            rcp.get_write_unlock(id).unwrap();
        }
        self
    }

    pub fn remove_resource<R: Resource, RH: Clone + Hash + 'static>(&self, rh: RH) -> &Self {
        let id = hash_resource_id(rh.clone());
        self.resource_maybe_insert::<R, RH>(rh);
        let rcp = self.rcp.write();
        {
            let val = rcp.get_write_lock(id).unwrap();
            *val = None;
            rcp.get_write_unlock(id).unwrap();
        }
        self
    }

    pub fn get_resource<R: Resource, RH: Hash + 'static>(
        &self,
        rh: RH,
    ) -> Option<ResourceReadGuard<R>> {
        let id = hash_resource_id(rh);
        let rcp = self.rcp.read();
        let rc = rcp.get_read_lock(id).unwrap();
        if rc.is_none() {
            None?
        }
        Some(ResourceReadGuard {
            r: rc
                .as_ref()
                .map(|val| unsafe { &*(val.get() as *const R) })
                .unwrap(),
            id,
            galaxy: self,
        })
    }

    pub fn get_mut_resource<R: Resource, RH: Hash + 'static>(
        &self,
        rh: RH,
    ) -> Option<ResourceWriteGuard<R>> {
        let id = hash_resource_id(rh);
        let rcp = self.rcp.read();
        let rc = rcp.get_read_lock(id).unwrap();
        if rc.is_none() {
            None?
        }
        Some(ResourceWriteGuard {
            r: rc
                .as_ref()
                .map(|val| unsafe { &mut *(val.get() as *const R as *mut R) })
                .unwrap(),
            id,
            galaxy: self,
        })
    }

    fn resource_maybe_insert<R: Resource, RH: Hash + 'static>(&self, rh: RH) {
        let rcp = self.rcp.read();
        let id = hash_resource_id(rh);
        if rcp.get_type(id).is_none() {
            drop(rcp);
            let mut rcp = self.rcp.write();
            rcp.insert_type(id, R::mewo_resource_type_entry()).unwrap();
        }
    }
}
