use super::{
    locker::{LockState, Locker},
    *,
};
use crate::{
    data::{DVec, SparseSet},
    error::*,
};

#[allow(dead_code)]
#[derive(Debug)]
struct ArchetypeStorageComponentEntry {
    row: usize,
    locker: Locker,
}

//  TODO row? coli? idx? Which is which? Even I don't down.

#[derive(Debug)]
pub struct ArchetypeStorage {
    pub(self) datas: Vec<DVec>,
    pub(self) component_tys: SparseSet<ComponentTypeId, ArchetypeStorageComponentEntry>,
    pub(self) entities: Vec<Entity>,
    pub(self) inserted_check: Vec<ComponentTypeId>,
}

impl ArchetypeStorage {
    pub fn create(
        ctymgr: &ComponentTypeManager,
        group: &ComponentGroup,
    ) -> Result<ArchetypeStorage> {
        let tylen = group.get().len();
        let mut storage = ArchetypeStorage {
            entities: Vec::new(),
            datas: Vec::with_capacity(tylen),
            component_tys: SparseSet::create_with_capacity(tylen),
            inserted_check: Vec::with_capacity(tylen),
        };

        for (idx, &ty) in group.get().iter().enumerate() {
            let entry = ctymgr.get(ty)?;
            storage.component_tys.insert(
                ty,
                ArchetypeStorageComponentEntry {
                    row: idx,
                    locker: Locker::create(),
                },
            );
            storage.datas.push(DVec::create(entry.size, entry.drop));
        }

        Ok(storage)
    }

    pub fn try_lock_component(&self, cty: ComponentTypeId, state: LockState) -> Result<bool> {
        if let Some(ArchetypeStorageComponentEntry { locker, .. }) = self.component_tys.get(cty) {
            Ok(locker.try_obtain_lock(state))
        } else {
            Err(RuntimeError::BadComponentType { ctyid: cty })
        }
    }

    pub fn unlock_component(&self, cty: ComponentTypeId, state: LockState) -> Result<()> {
        if let Some(ArchetypeStorageComponentEntry { locker, .. }) = self.component_tys.get(cty) {
            locker.release_lock(state);
            Ok(())
        } else {
            Err(RuntimeError::BadComponentType { ctyid: cty })
        }
    }

    //  For tests.
    pub fn get(&self, entity: Entity, cty: ComponentTypeId) -> Result<*const u8> {
        let coli = self.get_entity_column(entity)?;
        if let Some(data) = self.datas.get(cty) {
            Ok(data.get(coli).unwrap())
        } else {
            Err(RuntimeError::BadComponentType { ctyid: cty })
        }
    }

    pub fn get_iter(&self, cty: ComponentTypeId) -> Result<*const u8> {
        if let Some(data) = self.datas.get(cty) {
            Ok(data.ptr())
        } else {
            Err(RuntimeError::BadComponentType { ctyid: cty })
        }
    }

    pub fn get_iter_count(&self) -> usize {
        self.entities.len()
    }

    pub fn get_iter_entity(&self, idx: usize) -> Entity {
        *self.entities.get(idx).unwrap()
    }

    pub fn insert(&mut self, entity: Entity) -> Result<ArchetypeStorageInsert<'_>> {
        if self.get_entity_column(entity).is_ok() {
            return Err(RuntimeError::BadEntity { e: entity });
        }
        self.entities.push(entity);
        Ok(ArchetypeStorageInsert::create(self))
    }

    pub fn remove(&mut self, entity: Entity) -> Result<()> {
        let coli = self.get_entity_column(entity)?;
        self.entities.swap_remove(coli);
        //  Well, component types are sorted right?
        for (data, _) in self.datas.iter_mut().zip(self.component_tys.get_dense()) {
            assert_eq!(data.swap_remove(coli), Some(()));
        }
        Ok(())
    }

    pub fn copy_entity<'astore>(
        src: &'astore mut Self,
        dst: &'astore mut Self,
        entity: Entity,
    ) -> Result<ArchetypeStorageInsert<'astore>> {
        let coli = src.get_entity_column(entity)?;
        let mut insert = dst.insert(entity)?;
        for (data, (cty, _locker)) in src
            .datas
            .iter_mut()
            .zip(src.component_tys.get_dense().iter())
        {
            let &cty = cty;
            insert.insert(cty, data.get(coli).unwrap())?;
            assert_eq!(data.swap_remove(coli), Some(()));
        }
        Ok(insert)
    }
}

impl ArchetypeStorage {
    fn get_entity_column(&self, entity: Entity) -> Result<usize> {
        if let Some(coli) = self.entities.iter().position(|&e| e == entity) {
            Ok(coli)
        } else {
            Err(RuntimeError::BadEntity { e: entity })
        }
    }
}

pub struct ArchetypeStorageInsert<'astore> {
    storage: &'astore mut ArchetypeStorage,
}

impl<'astore> ArchetypeStorageInsert<'astore> {
    pub fn create(storage: &'astore mut ArchetypeStorage) -> ArchetypeStorageInsert<'astore> {
        for (cty, _locker) in storage.component_tys.get_dense().iter() {
            let &cty = cty;
            storage.inserted_check.push(cty);
        }
        ArchetypeStorageInsert { storage }
    }

    pub fn insert(&mut self, component_ty: ComponentTypeId, c: *const u8) -> Result<()> {
        if let Some(idx) = self
            .storage
            .inserted_check
            .iter()
            .position(|&ty| ty == component_ty)
        {
            let ty = self.storage.inserted_check.swap_remove(idx);
            let row = self.storage.component_tys.get(ty).unwrap().row;
            self.storage.datas.get_mut(row).unwrap().resize(1, c);
            Ok(())
        } else {
            Err(RuntimeError::BadComponentType {
                ctyid: component_ty,
            })
        }
    }

    pub fn done(&self) -> Result<()> {
        if self.storage.inserted_check.len() != 0 {
            Err(RuntimeError::ArchetypeStorageInsertIncomplete {
                missing: self.storage.inserted_check.clone(),
            })
        } else {
            Ok(())
        }
    }
}

#[test]
fn test_storage() -> Result<()> {
    use crate::{
        component::ComponentTypeEntry,
        data::{ValueClone, ValueDrop},
    };

    let usize_size = std::mem::size_of::<usize>();
    let u8_size = std::mem::size_of::<u8>();

    let mut ctymgr = ComponentTypeManager::create();
    let usize_id = ctymgr.register(ComponentTypeEntry {
        name: "usize".to_string(),
        size: usize_size,
        hash: 0,
        drop: ValueDrop::empty(),
        clone: ValueClone::empty(),
    })?;
    let u8_id = ctymgr.register(ComponentTypeEntry {
        name: "u8_size".to_string(),
        size: u8_size,
        hash: 1,
        drop: ValueDrop::empty(),
        clone: ValueClone::empty(),
    })?;

    let entity_a = Entity::from_id(1);
    let entity_b = Entity::from_id(69);

    fn ptr<T>(t: &T) -> *const u8 {
        (t as *const T) as *const u8
    }

    let mut group = ComponentGroup::builder();
    group.insert(usize_id);
    group.insert(u8_id);
    let group = group.build()?;
    let mut storage = ArchetypeStorage::create(&ctymgr, &group)?;

    let mut insert = storage.insert(entity_b)?;
    let data_b_u8 = 8u8;
    let data_b_usize = 10usize;
    assert!(insert.insert(u8_id, ptr(&data_b_u8)).is_ok());
    assert!(insert.insert(u8_id, ptr(&1u8)).is_err());
    assert!(insert.done().is_err());
    assert!(insert.insert(usize_id, ptr(&data_b_usize)).is_ok());
    assert!(insert.done().is_ok());
    assert!(storage.insert(entity_b).is_err());

    let mut insert = storage.insert(entity_a)?;
    let data_a_u8 = 9u8;
    let data_a_usize = 12usize; //  12 is a lucky number :)
    assert!(insert.insert(usize_id, ptr(&data_a_usize)).is_ok());
    assert!(insert.insert(u8_id, ptr(&data_a_u8)).is_ok());
    assert!(insert.done().is_ok());

    unsafe {
        assert_eq!(*(storage.get(entity_a, u8_id)? as *const u8), data_a_u8);
        assert_eq!(
            *(storage.get(entity_a, usize_id)? as *const usize),
            data_a_usize,
        );
        assert_eq!(*(storage.get(entity_b, u8_id)? as *const u8), data_b_u8);
        assert_eq!(
            *(storage.get(entity_b, usize_id)? as *const usize),
            data_b_usize
        );
    }

    assert!(storage.remove(entity_a).is_ok());

    unsafe {
        assert!(storage.get(entity_a, u8_id).is_err());
        assert!(storage.get(entity_a, usize_id).is_err());
        assert_eq!(*(storage.get(entity_b, u8_id)? as *const u8), data_b_u8);
        assert_eq!(
            *(storage.get(entity_b, usize_id)? as *const usize),
            data_b_usize
        );
    }

    Ok(())
}
