use super::{access::ArchetypeAccessKeyManager, storage::ArchetypeStorage, *};

impl ArchetypeManager {
    pub fn create() -> Self {
        let mut amgr = ArchetypeManager {
            lock_count: AtomicU32::new(0),
            storages: SparseSet::create(),
            entity_set: SparseSet::create(),
            cgmgr: ComponentGroupManager::create(),
            akmgr: ArchetypeAccessKeyManager::create(),
        };
        //  Creating null storage. Therefore, component types do not concern us.
        amgr.storages.insert(
            amgr.cgmgr.get_null_group_id(),
            ArchetypeStorage::create(&ComponentTypeManager::create(), &ComponentGroup::create())
                .unwrap(),
        );
        amgr
    }

    pub(super) fn insert_entity(&mut self, entity: Entity) -> Result<()> {
        self.yeet_locked()?;

        if let Some(_) = self.entity_set.get(entity.id()) {
            Err(RuntimeError::BadEntity { e: entity })
        } else {
            self.entity_set
                .insert(entity.id(), self.cgmgr.get_null_group_id());
            let storage = self
                .storages
                .get_mut(self.cgmgr.get_null_group_id())
                .unwrap();
            let insert = storage.insert(entity)?;
            insert.done()?;
            Ok(())
        }
    }

    pub(super) fn remove_entity(&mut self, entity: Entity) -> Result<()> {
        self.yeet_locked()?;

        if let Some(&locator) = self.entity_set.get(entity.id()) {
            let storage = self.storages.get_mut(locator).unwrap();
            assert_eq!(storage.remove(entity), Ok(()));
            self.entity_set.remove(entity.id());
            Ok(())
        } else {
            Err(RuntimeError::BadEntity { e: entity })
        }
    }

    //  TODO Potential crash if removed entity has later EntityModify::Modify.
    pub fn transform_entity(
        &mut self,
        et: EntityTransform,
        ctmgr: &ComponentTypeManager,
    ) -> Result<()> {
        self.yeet_locked()?;

        let e = match et.get_modify() {
            EntityModify::Modify(e) => e,
            EntityModify::Create(e) => {
                self.insert_entity(e)?;
                e
            }
            EntityModify::Destroy(e) => return self.remove_entity(e),
        };
        let &entity_gid = if let Some(l) = self.entity_set.get(e.id()) {
            l
        } else {
            return Err(RuntimeError::BadEntity { e });
        };
        let old_group = self.cgmgr.get(entity_gid)?;
        let mut group_modify = old_group.clone().modify();

        for &cid in et.get_removes().iter() {
            group_modify.remove(cid)?;
        }

        for (cid, _tval) in et.get_inserts().iter() {
            let cid = *cid;
            group_modify.insert(cid);
        }

        let group = group_modify.build()?;
        let gid = if let Ok(gid) = self.create_gid(&group, ctmgr) {
            gid
        } else {
            self.cgmgr.get_id_from_group(&group).unwrap()
        };

        let storage = self.storages.get_mut(entity_gid).unwrap();
        let mut insert = unsafe {
            ArchetypeStorage::copy_entity(
                (storage as *mut ArchetypeStorage).as_mut().unwrap(),
                self.storages.get_mut(gid).unwrap(),
                e,
            )?
        };
        for (cid, tval) in et.get_inserts().iter() {
            let cid = *cid;
            insert.insert(cid, tval.get())?
        }
        insert.done()?;
        self.storages.get_mut(entity_gid).unwrap().remove(e)?;
        *self.entity_set.get_mut(e.id()).unwrap() = gid;
        Ok(())
    }
}

impl ArchetypeManager {
    fn create_gid(
        &mut self,
        group: &ComponentGroup,
        ctymgr: &ComponentTypeManager,
    ) -> Result<ComponentGroupId> {
        let gid = self.cgmgr.register(group.clone())?;
        self.storages
            .insert(gid, ArchetypeStorage::create(ctymgr, &group)?);
        self.akmgr.update(gid, group);
        Ok(gid)
    }
}
