use super::{
    error::*, ComponentGroup, ComponentGroupId, ComponentGroupPlanet, ComponentTypeId,
    ComponentTypePlanet, Entity, EntityPlanet, QueryPlanet,
};
use crate::data::{DVec, SparseSet, TVal, ValueDuplicate};
use parking_lot::{Mutex, RwLock};
use std::collections::HashMap;

type Column = usize;
type Row = usize;

mod bloc;
mod row;

use bloc::{StorageBloc, StorageBlocInsert};
use row::StorageRow;

#[derive(Debug)]
pub struct StoragePlanet {
    null_group: ComponentGroupId,
    storages: SparseSet<ComponentGroupId, StorageBloc>,
    entities: SparseSet<Entity, ComponentGroupId>,
}

impl StoragePlanet {
    pub fn new(group_planet: &mut ComponentGroupPlanet) -> Result<Self> {
        let mut storages = SparseSet::new();

        //  Null storage.
        let null_group = group_planet.insert(ComponentGroup::new());
        storages.insert(
            null_group.id(),
            StorageBloc::new(&ComponentTypePlanet::new(), &ComponentGroup::new())?,
        );

        Ok(StoragePlanet {
            null_group,
            storages,
            entities: SparseSet::new(),
        })
    }

    fn insert_entity(&mut self, entity: Entity) -> Result<()> {
        if self.entities.get(entity.id()).is_some() {
            Err(ecs_err!(
                ErrorType::StoragePlanetInsertEntity { entity },
                self
            ))
        } else {
            self.entities.insert(entity.id(), self.null_group);
            self.storages
                .get_mut(self.null_group.id())
                .unwrap()
                .insert_entity(
                    &ComponentTypePlanet::new(),
                    entity,
                    StorageBlocInsert::new(),
                )?;
            Ok(())
        }
    }

    fn remove_entity(&mut self, entity: Entity) -> Result<()> {
        if let Some(&gid) = self.entities.get(entity.id()) {
            self.storages
                .get_mut(gid.id())
                .unwrap()
                .remove_entity(entity)?;
            self.entities.remove(entity.id());
            Ok(())
        } else {
            Err(ecs_err!(
                ErrorType::StoragePlanetRemoveEntity { entity },
                self
            ))
        }
    }

    fn modify(
        &mut self,
        cty_planet: &ComponentTypePlanet,
        cg_planet: &mut ComponentGroupPlanet,
        query_planet: &mut QueryPlanet,
        entity: Entity,
        modify: StorageModifyTransform,
    ) -> Result<()> {
        let old_gid = *self.entities.get(entity.id()).ok_or(ecs_err!(
            ErrorType::StoragePlanetTransformEntity { entity },
            (&self)
        ))?;
        let mut group = cg_planet
            .get_group(old_gid)
            .ok_or(ecs_err!(
                ErrorType::StoragePlanetTransformGroup { entity, old_gid },
                (&self, cty_planet)
            ))?
            .clone();
        let mut group_modify = group.modify();

        let mut missings = StorageBlocInsert::new();

        for remove in modify.removes {
            group_modify.remove(remove);
        }
        for (insert, val) in modify.inserts.iter() {
            missings.insert(*insert, val.get());
            group_modify.insert(*insert);
        }
        group_modify.build();

        let new_gid = if let Some(gid) = cg_planet.get_group_id(&group) {
            gid
        } else {
            let new_gid = cg_planet.insert(group.clone());
            query_planet.update_with_group(&cg_planet, new_gid)?;
            self.update_with_group(cty_planet, &cg_planet, new_gid)?;
            self.storages
                .insert(new_gid.id(), StorageBloc::new(cty_planet, &group)?);
            new_gid
        };

        let src_storage = unsafe {
            &mut *(self.storages.get(old_gid.id()).unwrap() as *const StorageBloc
                as *mut StorageBloc)
        };
        let dst_storage = self.storages.get_mut(new_gid.id()).unwrap();
        StorageBloc::copy_entity(src_storage, dst_storage, entity, cty_planet, missings)?;

        *self.entities.get_mut(entity.id()).unwrap() = new_gid;

        for (_, val) in modify.inserts {
            val.take();
        }

        Ok(())
    }

    pub fn transform(
        &mut self,
        ep: &mut EntityPlanet,
        cty_planet: &ComponentTypePlanet,
        cg_planet: &mut ComponentGroupPlanet,
        query_planet: &mut QueryPlanet,
        trans: StorageTransform,
    ) -> Result<()> {
        match trans {
            StorageTransform::Insert(entity, modify) => {
                self.insert_entity(entity)?;
                self.modify(cty_planet, cg_planet, query_planet, entity, modify)?;
            }
            StorageTransform::Modify(entity, modify) => {
                if !modify.is_empty() {
                    self.modify(cty_planet, cg_planet, query_planet, entity, modify)?;
                }
            }
            StorageTransform::Remove(entity) => {
                self.remove_entity(entity)?;
                ep.remove(entity)?;
            }
        }

        Ok(())
    }

    pub fn update_with_group(
        &mut self,
        ty_planet: &ComponentTypePlanet,
        group_planet: &ComponentGroupPlanet,
        group: ComponentGroupId,
    ) -> Result<()> {
        let update_group = group_planet.get_group(group).ok_or(ecs_err!(
            ErrorType::StoragePlanetUpdate { id: group },
            group_planet
        ))?;
        let storage = StorageBloc::new(ty_planet, update_group)?;
        self.storages.insert(group.id(), storage);
        Ok(())
    }

    pub fn get_len(&self, gid: ComponentGroupId) -> usize {
        self.storages.get(gid.id()).unwrap().get_len()
    }

    pub fn get_write_lock(
        &self,
        gid: ComponentGroupId,
        cid: ComponentTypeId,
    ) -> Result<Option<()>> {
        self.storages
            .get(gid.id())
            .ok_or(ecs_err!(ErrorType::StoragePlanetAccess { id: gid }, self))
            .map(|storage| storage.get_write_lock(cid))
    }

    pub fn get_write_unlock(
        &self,
        gid: ComponentGroupId,
        cid: ComponentTypeId,
    ) -> Result<Option<()>> {
        self.storages
            .get(gid.id())
            .ok_or(ecs_err!(ErrorType::StoragePlanetAccess { id: gid }, self))
            .map(|storage| storage.get_write_unlock(cid))
    }

    pub fn get_read_lock(&self, gid: ComponentGroupId, cid: ComponentTypeId) -> Result<Option<()>> {
        self.storages
            .get(gid.id())
            .ok_or(ecs_err!(ErrorType::StoragePlanetAccess { id: gid }, self))
            .map(|storage| storage.get_read_lock(cid))
    }

    pub fn get_read_unlock(
        &self,
        gid: ComponentGroupId,
        cid: ComponentTypeId,
    ) -> Result<Option<()>> {
        self.storages
            .get(gid.id())
            .ok_or(ecs_err!(ErrorType::StoragePlanetAccess { id: gid }, self))
            .map(|storage| storage.get_read_unlock(cid))
    }

    pub fn get_write(&self, gid: ComponentGroupId, cid: ComponentTypeId) -> Option<*const u8> {
        self.storages.get(gid.id()).unwrap().get_write(cid)
    }

    pub fn get_read(&self, gid: ComponentGroupId, cid: ComponentTypeId) -> Option<*const u8> {
        self.storages.get(gid.id()).unwrap().get_read(cid)
    }

    pub fn get_entities(&self, gid: ComponentGroupId) -> Option<*const Entity> {
        Some(self.storages.get(gid.id())?.get_entities())
    }

    pub fn get_entity_group(&self, entity: Entity) -> Option<ComponentGroupId> {
        self.entities.get(entity.id()).map(|v| *v)
    }

    pub fn get_entity_idx(&self, group: ComponentGroupId, entity: Entity) -> Option<usize> {
        self.storages.get(group.id())?.get_entity_idx(entity)
    }

    pub fn update(&mut self) {
        for (_, bloc) in self.storages.get_mut_dense() {
            bloc.update();
        }
    }
}

#[derive(Debug)]
pub enum StorageTransform {
    Insert(Entity, StorageModifyTransform),
    Modify(Entity, StorageModifyTransform),
    Remove(Entity),
}

#[derive(Debug)]
pub struct StorageModifyTransform {
    inserts: Vec<(ComponentTypeId, TVal)>,
    removes: Vec<ComponentTypeId>,
}

impl StorageModifyTransform {
    pub fn new() -> Self {
        StorageModifyTransform {
            inserts: Vec::new(),
            removes: Vec::new(),
        }
    }

    pub fn insert(&mut self, id: ComponentTypeId, val: TVal) {
        self.inserts.push((id, val));
    }

    pub fn remove(&mut self, id: ComponentTypeId) {
        self.removes.push(id);
    }

    pub fn is_empty(&self) -> bool {
        self.inserts.is_empty() && self.removes.is_empty()
    }
}
