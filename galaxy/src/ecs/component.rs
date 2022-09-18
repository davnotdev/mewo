use super::error::*;
use crate::data::TypeEntry;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ComponentInfo {
    pub ty: TypeEntry,
    pub storage_ty: ComponentStorageType,
}

#[derive(Debug, Clone)]
pub enum ComponentStorageType {
    Special,
    CopyCat,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ComponentTypeId(u64);

impl ComponentTypeId {
    pub fn from_hash(val: u64) -> Self {
        ComponentTypeId(val)
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct ComponentGroupId(usize);

impl ComponentGroupId {
    pub fn from_id(id: usize) -> Self {
        ComponentGroupId(id)
    }

    pub fn id(&self) -> usize {
        self.0
    }
}

#[derive(Debug)]
pub struct ComponentTypePlanet {
    components: HashMap<ComponentTypeId, ComponentInfo>,
}

impl ComponentTypePlanet {
    pub fn new() -> Self {
        ComponentTypePlanet {
            components: HashMap::new(),
        }
    }

    pub fn insert_type(&mut self, id: ComponentTypeId, ty: ComponentInfo) -> Result<()> {
        if self.components.contains_key(&id) {
            Err(ecs_err!(
                ErrorType::ComponentTypePlanetInsertType { id, ty: ty.clone() },
                self
            ))?
        }
        self.components.insert(id, ty);
        Ok(())
    }

    pub fn get_type(&self, id: ComponentTypeId) -> Result<&ComponentInfo> {
        self.components
            .get(&id)
            .ok_or(ecs_err!(ErrorType::ComponentTypePlanetGetType { id }, self))
    }
}

//  ComponentGroups are always sorted.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct ComponentGroup {
    components: Vec<ComponentTypeId>,
}

impl ComponentGroup {
    pub fn new() -> Self {
        ComponentGroup {
            components: Vec::new(),
        }
    }

    pub fn modify(&mut self) -> ComponentGroupModify {
        ComponentGroupModify { group: self }
    }

    pub fn get_components(&self) -> &Vec<ComponentTypeId> {
        &self.components
    }

    pub fn has(&self, cty: ComponentTypeId) -> bool {
        self.binary_search_recurse(cty, 0, self.components.len())
            .is_some()
    }

    fn binary_search_recurse(
        &self,
        find: ComponentTypeId,
        left: usize,
        right: usize,
    ) -> Option<usize> {
        let middle = (right - left) / 2 + left;
        let &val = self.components.get(middle)?;
        if val == find {
            return Some(middle);
        }
        if middle == right {
            return None;
        }
        if val > find {
            return self.binary_search_recurse(
                find,
                left,
                if let Ok(m) = (middle as isize - 1).try_into() {
                    m
                } else {
                    return None;
                },
            );
        } else {
            return self.binary_search_recurse(find, middle + 1, right);
        }
    }
}

pub struct ComponentGroupModify<'group> {
    group: &'group mut ComponentGroup,
}

impl<'group> ComponentGroupModify<'group> {
    pub fn insert(&mut self, id: ComponentTypeId) {
        self.group.components.push(id);
    }

    pub fn remove(&mut self, id: ComponentTypeId) {
        if let Some(idx) = self
            .group
            .components
            .iter()
            .position(|&component| component == id)
        {
            self.group.components.remove(idx);
        }
    }

    pub fn build(self) {
        self.group.components.sort();

        //  Remove duplicates.
        let mut last = None;
        self.group.components.retain(|&id| {
            if last == Some(id) {
                false
            } else {
                last = Some(id);
                true
            }
        });
    }
}

#[derive(Debug)]
pub struct ComponentGroupPlanet {
    //  ComponentGroupId -> ComponentGroup
    groups: Vec<ComponentGroup>,
    exists: HashMap<ComponentGroup, ComponentGroupId>,
}

impl ComponentGroupPlanet {
    pub fn new() -> ComponentGroupPlanet {
        ComponentGroupPlanet {
            groups: Vec::new(),
            exists: HashMap::new(),
        }
    }

    pub fn insert(&mut self, group: ComponentGroup) -> ComponentGroupId {
        if let Some(gid) = self.exists.get(&group) {
            return *gid;
        }
        let id = ComponentGroupId(self.groups.len());
        self.exists.insert(group.clone(), id);
        self.groups.push(group);
        id
    }

    pub fn get_group_id(&self, group: &ComponentGroup) -> Option<ComponentGroupId> {
        self.exists.get(group).map(|v| *v)
    }

    pub fn get_group(&self, id: ComponentGroupId) -> Option<&ComponentGroup> {
        self.groups.get(id.0)
    }

    pub fn get_groups(&self) -> &Vec<ComponentGroup> {
        &self.groups
    }
}
