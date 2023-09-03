use super::{
    ComponentAccessesOptional, ComponentGroupId, Entity, Galaxy, GenericComponent, QueryAccessType,
    StorageModifyTransform, StorageTransform,
};
use crate::data::TVal;
use std::marker::PhantomData;

pub trait EntityModifyOnly {}
pub struct EntityModifyOnlyImpl;
impl EntityModifyOnly for EntityModifyOnlyImpl {}

pub struct EntityGetter<'gal, T> {
    galaxy: &'gal Galaxy,
    trans: Option<StorageTransform>,
    phantom: PhantomData<T>,
}

impl<'gal, T> EntityGetter<'gal, T> {
    pub fn get_entity(&self) -> Entity {
        match self.trans.as_ref().unwrap() {
            StorageTransform::Insert(e, _) | StorageTransform::Modify(e, _) => *e,
            _ => unreachable!(),
        }
    }

    pub fn insert<C: GenericComponent + 'static>(&mut self, c: C) -> &mut Self {
        self.component_maybe_insert::<C>();
        match self.trans.as_mut().unwrap() {
            StorageTransform::Insert(_, modify) | StorageTransform::Modify(_, modify) => {
                modify.insert(C::mewo_component_id(), unsafe {
                    TVal::new(
                        C::mewo_component_size(),
                        &c as *const C as *const u8,
                        C::mewo_component_drop(),
                    )
                });
            }
            _ => unreachable!(),
        }
        std::mem::forget(c);
        self
    }

    pub fn remove<C: GenericComponent + 'static>(&mut self) -> &mut Self {
        self.component_maybe_insert::<C>();
        match self.trans.as_mut().unwrap() {
            StorageTransform::Insert(_, modify) | StorageTransform::Modify(_, modify) => {
                modify.remove(C::mewo_component_id());
            }
            _ => unreachable!(),
        }
        self
    }

    fn component_maybe_insert<C: GenericComponent + 'static>(&self) {
        let id = C::mewo_component_id();
        if self.galaxy.ctyp.read().get_type(id).is_err() {
            let mut ctyp = self.galaxy.ctyp.write();
            ctyp.insert_type(id, C::mewo_component_type_entry())
                .unwrap();
        }
    }
}

impl<'gal, T> EntityGetter<'gal, T>
where
    T: EntityModifyOnly,
{
    pub fn get<CA: ComponentAccessesOptional>(
        &mut self,
    ) -> Option<EntityComponentGetter<'gal, CA>> {
        Some(EntityComponentGetter::new(
            self.galaxy,
            *match self.trans.as_ref().unwrap() {
                StorageTransform::Modify(e, _) => e,
                _ => return None?,
            },
        ))
    }
}

impl<'gal, T> Drop for EntityGetter<'gal, T> {
    fn drop(&mut self) {
        self.galaxy
            .get_storage_transforms()
            .push(std::mem::replace(&mut self.trans, None).unwrap());
    }
}

pub struct EntityComponentGetter<'gal, CA: ComponentAccessesOptional> {
    galaxy: &'gal Galaxy,
    group_id: ComponentGroupId,
    entity_idx: usize,
    datas: Vec<Option<*const u8>>,
    phantom: PhantomData<CA>,
}

impl<'gal, CA> EntityComponentGetter<'gal, CA>
where
    CA: ComponentAccessesOptional,
{
    pub fn new(galaxy: &'gal Galaxy, entity: Entity) -> Self {
        CA::component_maybe_insert(&galaxy.ctyp);
        let sp = galaxy.sp.read();
        let cgp = galaxy.cgp.read();
        let gid = sp.get_entity_group(entity).unwrap();
        let group = cgp.get_group(gid).unwrap();
        let query = CA::infos();
        let mut datas: Vec<Option<*const u8>> = query.iter().map(|_| None).collect();
        for &cty in group.get_components() {
            for (idx, &(qcty, qlock)) in query.iter().enumerate() {
                if qcty == cty {
                    match qlock {
                        QueryAccessType::Read | QueryAccessType::OptionRead => {
                            sp.get_read_lock(gid, cty).unwrap();
                            *datas.get_mut(idx).unwrap() = sp.get_read(gid, cty);
                        }
                        QueryAccessType::Write | QueryAccessType::OptionWrite => {
                            sp.get_write_lock(gid, cty).unwrap();
                            *datas.get_mut(idx).unwrap() = sp.get_read(gid, cty);
                        }
                    };
                }
            }
        }
        EntityComponentGetter {
            galaxy,
            entity_idx: sp.get_entity_idx(gid, entity).unwrap(),
            group_id: gid,
            datas: datas.into_iter().collect(),
            phantom: PhantomData,
        }
    }

    pub fn get(&self) -> CA {
        CA::datas(&self.datas, self.entity_idx)
    }
}

impl<'gal, CA> Drop for EntityComponentGetter<'gal, CA>
where
    CA: ComponentAccessesOptional,
{
    fn drop(&mut self) {
        let sp = self.galaxy.sp.read();
        let cgp = self.galaxy.cgp.read();
        let group = cgp.get_group(self.group_id).unwrap();
        let query = CA::infos();

        //  Maybe it's safer to drop locks in order?
        for &cty in group.get_components() {
            for &(qcty, qlock) in query.iter() {
                if qcty == cty {
                    match qlock {
                        //  Some(whatever.unwrap()) is intentional.
                        QueryAccessType::Read | QueryAccessType::OptionRead => {
                            sp.get_read_unlock(self.group_id, cty).unwrap()
                        }
                        QueryAccessType::Write | QueryAccessType::OptionWrite => {
                            sp.get_write_unlock(self.group_id, cty).unwrap()
                        }
                    };
                }
            }
        }
    }
}

impl Galaxy {
    pub fn insert_entity(&self) -> EntityGetter<()> {
        let e = self.ep.write().insert();
        EntityGetter {
            galaxy: self,
            trans: Some(StorageTransform::Insert(e, StorageModifyTransform::new())),
            phantom: PhantomData,
        }
    }

    pub fn get_entity(&self, entity: Entity) -> Option<EntityGetter<EntityModifyOnlyImpl>> {
        self.ep.read().has_entity(entity).then_some(EntityGetter {
            galaxy: self,
            trans: Some(StorageTransform::Modify(
                entity,
                StorageModifyTransform::new(),
            )),
            phantom: PhantomData,
        })
    }

    pub fn remove_entity(&self, e: Entity) {
        self.get_storage_transforms()
            .push(StorageTransform::Remove(e));
    }
}
