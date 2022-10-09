use super::{
    error::*, ComponentGroup, ComponentGroupId, ComponentGroupPlanet, ComponentTypeId,
    ComponentTypePlanet, StoragePlanet,
};
use parking_lot::RwLock;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy)]
pub struct QueryId(usize);

impl QueryId {
    pub fn id(&self) -> usize {
        self.0
    }
}

#[derive(Debug, Clone, Copy)]
pub enum QueryLockType {
    Read,
    Write,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum QueryAccessType {
    Read,
    Write,
    OptionRead,
    OptionWrite,
}

impl QueryAccessType {
    pub fn into_lock(self) -> QueryLockType {
        match self {
            QueryAccessType::Read | QueryAccessType::OptionRead => QueryLockType::Read,
            QueryAccessType::Write | QueryAccessType::OptionWrite => QueryLockType::Write,
        }
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum QueryFilterType {
    With,
    Without,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct QueryAccess {
    pub accesses: Vec<(ComponentTypeId, QueryAccessType)>,
    pub filters: Vec<(ComponentTypeId, QueryFilterType)>,
}

#[derive(Debug)]
pub struct StorageAccess {
    pub groups: Vec<(
        ComponentGroupId,
        ComponentGroup,
        HashMap<ComponentTypeId, QueryLockType>,
    )>,
}

#[derive(Debug)]
pub struct QueryPlanet {
    queries: Vec<StorageAccess>,
    accesses: HashMap<QueryAccess, QueryId>,
}

impl QueryPlanet {
    pub fn new() -> Self {
        QueryPlanet {
            queries: Vec::new(),
            accesses: HashMap::new(),
        }
    }

    pub fn insert_access(
        &mut self,
        type_planet: &RwLock<ComponentTypePlanet>,
        group_planet: &RwLock<ComponentGroupPlanet>,
        storage_planet: &RwLock<StoragePlanet>,
        access: QueryAccess,
    ) -> Result<QueryId> {
        self.group_maybe_insert(type_planet, group_planet, storage_planet, &access)?;
        let group_planet = group_planet.read();
        let storage_query = storage_query_from_access(&group_planet, &access);
        if self.accesses.get(&access).is_some() {
            Err(ecs_err!(
                ErrorType::QueryPlanetInsertAccess { access },
                self
            ))
        } else {
            let id = QueryId(self.queries.len());
            self.accesses.insert(access, id);
            self.queries.push(storage_query);
            Ok(id)
        }
    }

    //  When inserting an access, we may need to create a new group which
    //  also requires updating the storage and query planets.
    fn group_maybe_insert(
        &mut self,
        type_planet: &RwLock<ComponentTypePlanet>,
        group_planet: &RwLock<ComponentGroupPlanet>,
        storage_planet: &RwLock<StoragePlanet>,
        access: &QueryAccess,
    ) -> Result<()> {
        let mut group = ComponentGroup::new();
        let mut group_modify = group.modify();
        access
            .accesses
            .iter()
            .for_each(|(cty, _)| group_modify.insert(*cty));
        group_modify.build();
        let group_planet_read = group_planet.read();
        if group_planet_read.get_group_id(&group).is_none() {
            drop(group_planet_read);
            let mut group_planet = group_planet.write();
            let gid = group_planet.insert(group.clone());
            self.update_with_group(&group_planet, gid)?;
            storage_planet
                .write()
                .update_with_group(&type_planet.read(), &group_planet, gid)?;
        }
        Ok(())
    }

    pub fn get_query_id(&self, access: &QueryAccess) -> Option<QueryId> {
        self.accesses.get(access).map(|v| *v)
    }

    pub fn get_access(&self, id: QueryId) -> Result<&StorageAccess> {
        self.queries
            .get(id.id())
            .ok_or(ecs_err!(ErrorType::QueryPlanetGetAccess { id }, self))
    }

    pub fn update_with_group(
        &mut self,
        planet: &ComponentGroupPlanet,
        group: ComponentGroupId,
    ) -> Result<()> {
        let update_group = planet
            .get_group(group)
            .ok_or(ecs_err!(ErrorType::QueryPlanetUpdate { id: group }, planet))?;
        for (access, id) in self.accesses.iter() {
            if let Some(storage_access) =
                access_filter(update_group, &access.accesses, &access.filters)
            {
                self.queries.get_mut(id.0).unwrap().groups.push((
                    group,
                    group_from_access(&storage_access),
                    storage_access,
                ));
            }
        }
        Ok(())
    }
}

//  TODO FIX: from_access? What access? Fix ambiguity.

fn storage_query_from_access(planet: &ComponentGroupPlanet, access: &QueryAccess) -> StorageAccess {
    let mut groups = Vec::new();
    for (gid, group) in planet.get_groups().iter().enumerate() {
        if let Some(access) = access_filter(group, &access.accesses, &access.filters) {
            groups.push((
                ComponentGroupId::from_id(gid),
                group_from_access(&access),
                access,
            ));
        }
    }
    StorageAccess { groups }
}

fn group_from_access(access: &HashMap<ComponentTypeId, QueryLockType>) -> ComponentGroup {
    let mut group = ComponentGroup::new();
    let mut group_modify = group.modify();
    access.iter().for_each(|(&cty, _)| {
        group_modify.insert(cty);
    });
    group_modify.build();
    group
}

//  Cases:
//  1. &C,          4. &mut C
//  2. Option<&C>   5. Option<&mut C>
//  3. With<C>      6. Without<C>
fn access_filter(
    group: &ComponentGroup,
    accesses: &Vec<(ComponentTypeId, QueryAccessType)>,
    filters: &Vec<(ComponentTypeId, QueryFilterType)>,
) -> Option<HashMap<ComponentTypeId, QueryLockType>> {
    let mut lock_map = HashMap::new();
    for &(cty, ctf) in filters.iter() {
        match ctf {
            QueryFilterType::With => {
                if !group.has(cty) {
                    return None;
                }
            }
            QueryFilterType::Without => {
                if group.has(cty) {
                    return None;
                }
            }
        }
    }
    for &(cty, ctq) in accesses.iter() {
        match ctq {
            QueryAccessType::Read | QueryAccessType::Write => {
                if !group.has(cty) {
                    return None;
                }
            }
            QueryAccessType::OptionRead | QueryAccessType::OptionWrite => {
                if !group.has(cty) {
                    continue;
                }
            }
        }
        lock_map.insert(cty, ctq.into_lock());
    }

    if lock_map.len() == 0 {
        None?
    }
    Some(lock_map)
}
