use super::{
    data::{ThreadLocal, ThreadLocalGuard},
    ecs::{
        ComponentGroupId, ComponentGroupPlanet, ComponentTypeId, ComponentTypePlanet, Entity,
        EntityPlanet, EventId, EventModify, EventPlanet, QueryAccess, QueryAccessType,
        QueryFilterType, QueryId, QueryLockType, QueryPlanet, ResourceId, ResourcePlanet,
        StorageModifyTransform, StoragePlanet, StorageTransform,
    },
};
use parking_lot::RwLock;

mod access;
mod component;
mod entity;
mod event;
mod query;
mod resource;

#[cfg(test)]
mod test;

pub use access::{
    ComponentAccessNonOptional, ComponentAccessOptional, ComponentAccessesNonOptional,
    ComponentAccessesNormal, ComponentAccessesOptional,
};
pub use component::{CheapComponent, Component, GenericComponent, UniqueComponent};
pub use event::Event;
pub use resource::{Resource, ResourceReadGuard, ResourceWriteGuard};

pub struct Galaxy {
    //  These RwLocks allow the galaxy to dynamically insert queries, components, etc during
    //  runtime via `maybe_insert` functions. Based on their current usage, ABBA deadlocks
    //  are not possible.
    ep: RwLock<EntityPlanet>,
    ctyp: RwLock<ComponentTypePlanet>,
    cgp: RwLock<ComponentGroupPlanet>,
    rcp: RwLock<ResourcePlanet>,
    evp: RwLock<EventPlanet>,
    qp: RwLock<QueryPlanet>,
    sp: RwLock<StoragePlanet>,

    ev_modify: ThreadLocal<EventModify>,
    st_transforms: ThreadLocal<Vec<StorageTransform>>,
}

impl Galaxy {
    pub fn new() -> Self {
        let mut cgp = ComponentGroupPlanet::new();
        Galaxy {
            sp: RwLock::new(StoragePlanet::new(&mut cgp).unwrap()),
            cgp: RwLock::new(cgp),

            rcp: RwLock::new(ResourcePlanet::new()),
            evp: RwLock::new(EventPlanet::new()),
            ctyp: RwLock::new(ComponentTypePlanet::new()),
            qp: RwLock::new(QueryPlanet::new()),

            ep: RwLock::new(EntityPlanet::new()),

            ev_modify: ThreadLocal::new(),
            st_transforms: ThreadLocal::new(),
        }
    }

    pub fn update(&mut self) {
        let mut evp = self.evp.write();

        let mut ep = self.ep.write();
        let ctyp = self.ctyp.read();
        let mut cgp = self.cgp.write();
        let mut sp = self.sp.write();
        let mut qp = self.qp.write();

        let ev_modifies = unsafe { self.ev_modify.get_inner() };
        for mut ev_modify in ev_modifies.iter_mut() {
            evp.modify(&mut ev_modify).unwrap();
        }

        let st_transforms = unsafe { self.st_transforms.get_inner() };
        for st_trans in st_transforms.iter_mut() {
            for trans in st_trans.iter_mut() {
                //  Eh, it'll get cleared anyway.
                let trans =
                    std::mem::replace(trans, StorageTransform::Remove(Entity::from(0, 888)));
                sp.transform(&mut ep, &ctyp, &mut cgp, &mut qp, trans)
                    .unwrap();
            }
            st_trans.clear()
        }

        sp.update();
    }

    fn get_event_modify(&self) -> ThreadLocalGuard<EventModify> {
        self.ev_modify.get_or(|| EventModify::new())
    }

    fn get_storage_transforms(&self) -> ThreadLocalGuard<Vec<StorageTransform>> {
        self.st_transforms.get_or(|| Vec::new())
    }
}
