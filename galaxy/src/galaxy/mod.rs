use super::ecs::{
    ComponentGroupId, ComponentGroupPlanet, ComponentInfo, ComponentStorageType, ComponentTypeId,
    ComponentTypePlanet, Entity, EntityPlanet, EventId, EventModify, EventPlanet, QueryAccess,
    QueryAccessType, QueryFilterType, QueryId, QueryLockType, QueryPlanet, ResourceId,
    ResourcePlanet, StorageModifyTransform, StoragePlanet, StorageTransform,
};
use parking_lot::RwLock;

mod access;
mod component;
mod entity;
mod event;
mod exec;
mod query;
mod resource;

#[cfg(test)]
mod test;

pub use access::{
    ComponentAccessNonOptional, ComponentAccessOptional, ComponentAccessesNonOptional,
    ComponentAccessesNormal, ComponentAccessesOptional,
};
pub use component::Component;
pub use event::Event;
pub use exec::Executor;
pub use resource::{Resource, ResourceReadGuard, ResourceWriteGuard};

pub struct Galaxy<EX> {
    //  These RwLocks allow the galaxy to dynamically insert queries, components, etc during
    //  runtime via `maybe_insert` functions. Based on their current usage, ABBA deadlocks
    //  are not possible.
    exec: EX,

    ep: RwLock<EntityPlanet>,
    ctyp: RwLock<ComponentTypePlanet>,
    cgp: RwLock<ComponentGroupPlanet>,
    rcp: RwLock<ResourcePlanet>,
    evp: RwLock<EventPlanet>,
    qp: RwLock<QueryPlanet>,
    sp: RwLock<StoragePlanet>,
}

impl<EX> Galaxy<EX>
where
    EX: Executor,
{
    pub fn new() -> Self {
        let mut cgp = ComponentGroupPlanet::new();
        Galaxy {
            exec: EX::new(),

            sp: RwLock::new(StoragePlanet::new(&mut cgp).unwrap()),
            cgp: RwLock::new(cgp),

            rcp: RwLock::new(ResourcePlanet::new()),
            evp: RwLock::new(EventPlanet::new()),
            ctyp: RwLock::new(ComponentTypePlanet::new()),
            qp: RwLock::new(QueryPlanet::new()),

            ep: RwLock::new(EntityPlanet::new()),
        }
    }

    pub fn update(&mut self) {
        let mut evp = self.evp.write();

        let mut ep = self.ep.write();
        let ctyp = self.ctyp.read();
        let mut cgp = self.cgp.write();
        let mut sp = self.sp.write();
        let mut qp = self.qp.write();

        for mut ev_modify in self.exec.get_all_event_modify() {
            evp.modify(&mut ev_modify).unwrap();
        }

        for st_trans in self.exec.get_all_storage_transforms() {
            for trans in st_trans {
                //  Eh, it'll get cleared anyway.
                let trans = std::mem::replace(trans, StorageTransform::Remove(Entity::from(0, 0)));
                sp.transform(&mut ep, &ctyp, &mut cgp, &mut qp, trans)
                    .unwrap();
            }
        }
        self.exec.clear_all_storage_transforms();
    }
}
