use super::component::ComponentTypeId;
use super::world::World;
use crate::error::{ComponentErrorIdentifier, ECSError, Result};
use crate::BoolMask;

#[derive(Debug, Clone)]
pub struct ComponentStamp {
    len: usize,
    stamp: BoolMask,
}

impl ComponentStamp {
    pub fn create(world: &World) -> ComponentStamp {
        Self::create_from_len(world.get_component_manager().get_component_type_count())
    }

    pub fn create_from_len(len: usize) -> ComponentStamp {
        ComponentStamp {
            len,
            stamp: BoolMask::create_with_capacity(len),
        }
    }

    pub fn stamp(&mut self, id: ComponentTypeId) -> Result<()> {
        if id >= self.len {
            Err(ECSError::ComponentTypeDoesNotExist(
                ComponentErrorIdentifier::Id(id),
            ))
        } else {
            self.stamp.set(id, true);
            Ok(())
        }
    }

    pub fn unstamp(&mut self, id: ComponentTypeId) -> Result<()> {
        if id >= self.len {
            Err(ECSError::ComponentTypeDoesNotExist(
                ComponentErrorIdentifier::Id(id),
            ))
        } else {
            self.stamp.set(id, false);
            Ok(())
        }
    }

    pub fn get(&self, id: ComponentTypeId) -> Result<bool> {
        if id >= self.len {
            Err(ECSError::ComponentTypeDoesNotExist(
                ComponentErrorIdentifier::Id(id),
            ))
        } else {
            Ok(self.stamp.get(id))
        }
    }

    pub fn get_len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.stamp.is_empty()
    }

    pub fn merge(&mut self, other: &ComponentStamp) -> Result<()> {
        if self.stamp.merge(&other.stamp).is_none() {
            unreachable!("Two Component Stamps should ALWAYS have the same # of Component Types")
        }
        Ok(())
    }

    //  function specially tailored for use in system.rs
    pub fn system_match(
        entity_dep: &ComponentStamp,
        total_withs: &ComponentStamp,
        without: &Option<ComponentStamp>,
    ) -> bool {
        let with_res = if let Some(res) = entity_dep.stamp.bitwise_and(&total_withs.stamp) {
            res
        } else {
            unreachable!("All ComponentStamps should be of identical length (With)")
        };
        if !with_res.compare(&total_withs.stamp) {
            return false;
        }
        if let Some(without) = without {
            let without_res = if let Some(res) = entity_dep.stamp.bitwise_and(&without.stamp) {
                res
            } else {
                unreachable!("All ComponentStamps should be of identical length (Without)")
            };
            if without_res.compare(&without.stamp) {
                return false;
            }
        }
        true
    }
}
