use super::{
    error::*, ComponentGroup, ComponentGroupId, ComponentGroupPlanet, ComponentStorageType,
    ComponentTypeId, ComponentTypePlanet, Entity, EntityPlanet, QueryPlanet,
};
use crate::data::{DVec, SparseSet, TVal};
use parking_lot::RwLock;
use std::collections::HashMap;

type Column = usize;
type Row = usize;

//      DVec    DVec    DVec
//  e   d       d       d
//  e   d       d       d
#[derive(Debug)]
pub(self) struct StorageBloc {
    datas: Vec<(ComponentTypeId, StorageRow)>,
    entities: Vec<Entity>,
}

impl StorageBloc {
    pub fn new(planet: &ComponentTypePlanet, group: &ComponentGroup) -> Result<Self> {
        Ok(StorageBloc {
            datas: group
                .get_components()
                .iter()
                .map(|&cty| {
                    let info = planet.get_type(cty)?;
                    Ok((
                        cty,
                        match info.storage_ty {
                            ComponentStorageType::Special => StorageRow::Normal(RwLock::new(
                                DVec::new(info.ty.size, info.ty.drop),
                            )),
                            ComponentStorageType::CopyCat => StorageRow::CopyCat(
                                RwLock::new(DVec::new(info.ty.size, info.ty.drop)),
                                DVec::new(info.ty.size, info.ty.drop),
                            ),
                        },
                    ))
                })
                .collect::<Result<_>>()?,
            entities: Vec::new(),
        })
    }

    pub fn get_len(&self) -> usize {
        self.entities.len()
    }

    pub fn get_write_lock(&self, id: ComponentTypeId) -> Option<()> {
        Some(
            self.datas
                .get(self.type_column(id)?)
                .unwrap()
                .1
                .write_lock(),
        )
    }

    pub fn get_write_unlock(&self, id: ComponentTypeId) -> Option<()> {
        Some(
            self.datas
                .get(self.type_column(id)?)
                .unwrap()
                .1
                .write_unlock(),
        )
    }

    pub fn get_read_lock(&self, id: ComponentTypeId) -> Option<()> {
        Some(self.datas.get(self.type_column(id)?).unwrap().1.read_lock())
    }

    pub fn get_read_unlock(&self, id: ComponentTypeId) -> Option<()> {
        Some(
            self.datas
                .get(self.type_column(id)?)
                .unwrap()
                .1
                .read_unlock(),
        )
    }

    pub fn get_write(&self, id: ComponentTypeId) -> Option<*const u8> {
        Some(
            self.datas
                .get(self.type_column(id)?)
                .unwrap()
                .1
                .access_write(),
        )
    }

    pub fn get_read(&self, id: ComponentTypeId) -> Option<*const u8> {
        Some(
            self.datas
                .get(self.type_column(id)?)
                .unwrap()
                .1
                .access_read(),
        )
    }

    pub fn insert_entity(
        &mut self,
        planet: &ComponentTypePlanet,
        entity: Entity,
        ins: StorageBlocInsert,
    ) -> Result<()> {
        assert!(ins.components.len() == self.datas.len());
        let row = self.entity_row(entity);
        for (id, val) in ins.components.into_iter() {
            let column = self.type_column(id).ok_or(ecs_err!(
                ErrorType::StorageBlocInsertComponent {
                    id: id,
                    entity: entity
                },
                self
            ))?;
            if let Some(row) = row {
                if let Some(old_val) = self.datas.get_mut(column).unwrap().1.get_mut(row) {
                    let ty = &planet.get_type(id)?.ty;
                    ty.drop.call(old_val);
                    unsafe { std::ptr::copy_nonoverlapping(val, old_val as *mut u8, ty.size) };
                }
            } else {
                self.datas.get_mut(column).unwrap().1.resize(1, val);
            }
        }
        self.entities.push(entity);
        Ok(())
    }

    pub fn remove_entity(&mut self, entity: Entity) -> Result<()> {
        let row = self
            .entity_row(entity)
            .ok_or(ecs_err!(ErrorType::StorageBlocRemove { entity }, self))?;
        for data in self.datas.iter_mut() {
            data.1.swap_remove(row);
        }
        Ok(())
    }

    //  Don't drop components.
    pub fn take_remove_entity(&mut self, entity: Entity) -> Result<()> {
        let row = self
            .entity_row(entity)
            .ok_or(ecs_err!(ErrorType::StorageBlocRemove { entity }, self))?;
        for data in self.datas.iter_mut() {
            data.1.take_swap_remove(row);
        }
        Ok(())
    }

    fn copy_entity(
        src: &mut Self,
        dst: &mut Self,
        entity: Entity,
        planet: &ComponentTypePlanet,
        mut missings: StorageBlocInsert,
    ) -> Result<()> {
        let src_row = src.entity_row(entity).ok_or(ecs_err!(
            ErrorType::StorageBlocCopyEntity { entity },
            (&src, &dst)
        ))?;
        for (id, data) in src.datas.iter_mut() {
            missings.insert(*id, data.get_mut(src_row).unwrap());
        }
        dst.insert_entity(planet, entity, missings)?;
        src.take_remove_entity(entity)?;

        Ok(())
    }

    pub fn get_entity_idx(&self, entity: Entity) -> Option<usize> {
        self.entity_row(entity)
    }
}

impl StorageBloc {
    fn type_column(&self, cty: ComponentTypeId) -> Option<Column> {
        self.datas.iter().position(|&(p, _)| p == cty)
    }

    fn entity_row(&self, e: Entity) -> Option<Row> {
        self.entities.iter().position(|&p| p == e)
    }
}

pub(self) struct StorageBlocInsert {
    components: HashMap<ComponentTypeId, *const u8>,
}

impl StorageBlocInsert {
    pub fn new() -> Self {
        StorageBlocInsert {
            components: HashMap::new(),
        }
    }

    //  A double insert results in one value being replaced.
    //  Do we want this behavior?
    pub fn insert(&mut self, id: ComponentTypeId, val: *const u8) {
        self.components.insert(id, val);
    }
}

//  TODO FIX: Copy CopyCat

//  Ironic that ComponentStorageType::Special == StorageRow::Normal.
#[derive(Debug)]
enum StorageRow {
    Normal(RwLock<DVec>),
    CopyCat(RwLock<DVec>, DVec),
}

impl StorageRow {
    pub fn access_write(&self) -> *const u8 {
        match self {
            StorageRow::Normal(v) => unsafe { &*v.data_ptr() }.ptr(),
            StorageRow::CopyCat(v, _) => unsafe { &*v.data_ptr() }.ptr(),
        }
    }

    pub fn access_read(&self) -> *const u8 {
        match self {
            StorageRow::Normal(v) => unsafe { &*v.data_ptr() }.ptr(),
            StorageRow::CopyCat(_, v) => v.ptr(),
        }
    }

    pub fn write_lock(&self) {
        match self {
            StorageRow::Normal(v) => std::mem::forget(v.write()),
            StorageRow::CopyCat(v, _) => std::mem::forget(v.write()),
        }
    }

    pub fn write_unlock(&self) {
        match self {
            StorageRow::Normal(v) => unsafe { v.force_unlock_write() },
            StorageRow::CopyCat(v, _) => unsafe { v.force_unlock_write() },
        }
    }

    pub fn read_lock(&self) {
        match self {
            StorageRow::Normal(v) => std::mem::forget(v.read()),
            _ => {}
        }
    }

    pub fn read_unlock(&self) {
        match self {
            StorageRow::Normal(v) => unsafe { v.force_unlock_read() },
            _ => {}
        }
    }

    pub fn swap_remove(&mut self, idx: usize) {
        match self {
            StorageRow::Normal(v) => v.write().swap_remove(idx),
            StorageRow::CopyCat(v, _) => v.write().swap_remove(idx),
        };
    }

    pub fn take_swap_remove(&mut self, idx: usize) {
        match self {
            StorageRow::Normal(v) => v.write().take_swap_remove(idx),
            StorageRow::CopyCat(v, _) => v.write().take_swap_remove(idx),
        };
    }

    pub fn get_mut(&mut self, idx: usize) -> Option<*mut u8> {
        match self {
            StorageRow::Normal(v) => v.write().get(idx).map(|ptr| ptr as *mut u8),
            StorageRow::CopyCat(v, _) => v.write().get(idx).map(|ptr| ptr as *mut u8),
        }
    }

    pub fn resize(&mut self, idx: usize, inplace: *const u8) {
        match self {
            StorageRow::Normal(v) => v.write().resize(idx, inplace),
            StorageRow::CopyCat(v, _) => v.write().resize(idx, inplace),
        }
    }
}

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

        let new_gid = cg_planet.insert(group.clone());
        query_planet.update_with_group(&cg_planet, new_gid)?;
        self.update_with_group(cty_planet, &cg_planet, new_gid)?;
        self.storages
            .insert(new_gid.id(), StorageBloc::new(cty_planet, &group)?);

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
                self.modify(cty_planet, cg_planet, query_planet, entity, modify)?;
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

    pub fn get_entity_group(&self, entity: Entity) -> Option<ComponentGroupId> {
        self.entities.get(entity.id()).map(|v| *v)
    }

    pub fn get_entity_idx(&self, group: ComponentGroupId, entity: Entity) -> Option<usize> {
        self.storages.get(group.id())?.get_entity_idx(entity)
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
}
