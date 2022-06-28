use super::{ComponentGroupId, ComponentTypeId};
use crate::error::*;

pub struct ComponentGroupModify {
    data: Vec<ComponentTypeId>,
}

impl ComponentGroupModify {
    pub fn insert(&mut self, cty: ComponentTypeId) {
        self.data.push(cty);
    }

    pub fn remove(&mut self, cty: ComponentTypeId) -> Result<()> {
        if let Some(idx) = self.data.iter().position(|&icty| icty == cty) {
            self.data.swap_remove(idx);
            Ok(())
        } else {
            Err(RuntimeError::BadComponentType { ctyid: cty })
        }
    }

    pub fn build(mut self) -> Result<ComponentGroup> {
        self.data.sort();
        let mut last = None;
        for &here in self.data.iter() {
            if let Some(last) = last {
                if here == last {
                    return Err(RuntimeError::BadComponentType { ctyid: last });
                }
            }
            last = Some(here);
        }
        Ok(ComponentGroup(self.data))
    }
}

#[derive(Debug, Clone)]
pub struct ComponentGroup(Vec<ComponentTypeId>);

impl ComponentGroup {
    pub fn create() -> ComponentGroup {
        ComponentGroup(Vec::new())
    }

    #[cfg(test)]
    pub fn builder() -> ComponentGroupModify {
        ComponentGroupModify { data: Vec::new() }
    }

    pub fn modify(self) -> ComponentGroupModify {
        ComponentGroupModify { data: self.0 }
    }

    pub fn find(&self, cty: ComponentTypeId) -> Option<usize> {
        if self.get().len() == 0 {
            None
        } else {
            self.binary_search_recurse(cty, 0, self.get().len() - 1)
        }
    }

    pub fn has(&self, cty: ComponentTypeId) -> bool {
        self.find(cty).is_some()
    }

    pub fn compare(&self, other: &ComponentGroup) -> bool {
        if self.get().len() != other.get().len() {
            return false;
        }

        for idx in 0..self.get().len() {
            if self.get().get(idx) != other.get().get(idx) {
                return false;
            }
        }
        true
    }

    pub fn get(&self) -> &Vec<ComponentTypeId> {
        &self.0
    }
}

impl ComponentGroup {
    fn binary_search_recurse(
        &self,
        find: ComponentTypeId,
        left: usize,
        right: usize,
    ) -> Option<usize> {
        let middle = (right - left) / 2 + left;
        let &val = self.get().get(middle)?;
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

pub struct ComponentGroupManager {
    empty: usize, //  just 0
    component_groups: Vec<ComponentGroup>,
}

impl ComponentGroupManager {
    pub fn create() -> Self {
        let mut cgmgr = ComponentGroupManager {
            empty: 0,
            component_groups: Vec::new(),
        };
        let empty = cgmgr.register(ComponentGroup::create()).unwrap();
        cgmgr.empty = empty; //  just in case :)
        cgmgr
    }

    pub fn register(&mut self, group: ComponentGroup) -> Result<ComponentGroupId> {
        for (id, groupi) in self.component_groups.iter().enumerate() {
            if groupi.compare(&group) {
                return Err(RuntimeError::BadComponentGroup { gid: id });
            }
        }

        self.component_groups.push(group);
        Ok(self.component_groups.len() - 1)
    }

    pub fn get(&self, id: ComponentGroupId) -> Result<&ComponentGroup> {
        if let Some(e) = self.component_groups.get(id) {
            Ok(e)
        } else {
            Err(RuntimeError::BadComponentGroup { gid: id })
        }
    }

    pub fn get_id_from_group(&self, group: &ComponentGroup) -> Option<ComponentGroupId> {
        self.component_groups.iter().position(|g| g.compare(group))
    }

    pub fn get_null_group_id(&self) -> ComponentGroupId {
        self.empty
    }
}

#[test]
fn test_component_group() {
    let group = ComponentGroup::create();
    assert!(group.find(0).is_none());
    let mut modify = group.modify();
    assert!(modify.remove(1).is_err());
    modify.insert(1);
    modify.insert(5);
    modify.insert(2);
    modify.insert(3);

    let group = modify.build().unwrap();
    assert!(group.has(1));
    assert!(group.has(2));
    assert!(group.has(3));
    assert!(group.has(5));
    let mut modify = group.modify();
    assert!(modify.remove(3).is_ok());

    let group = modify.build().unwrap();
    assert!(group.has(1));
    assert!(group.has(2));
    assert!(group.has(5));
}
