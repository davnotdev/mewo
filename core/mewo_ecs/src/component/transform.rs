use super::{component::ComponentTypeManager, ComponentHash, ComponentTypeId, Entity};
use crate::{data::TVal, unbug::prelude::*};

pub struct EntityTransformer {
    transforms: Vec<EntityTransformBuilder>,
}

impl EntityTransformer {
    pub fn create() -> Self {
        EntityTransformer {
            transforms: Vec::new(),
        }
    }

    pub fn insert(&mut self, transform: EntityTransformBuilder) {
        self.transforms.push(transform);
    }

    pub fn get(&mut self) -> Option<EntityTransformBuilder> {
        if self.transforms.is_empty() {
            None
        } else {
            Some(self.transforms.swap_remove(0))
        }
    }
}

#[derive(Clone, Copy)]
pub enum EntityModifyBuilder {
    Create(Option<Entity>),
    Modify(Entity),
    Destroy(Entity),
}

#[derive(Clone, Copy)]
pub enum EntityModify {
    Create(Entity),
    Modify(Entity),
    Destroy(Entity),
}

pub struct EntityTransformBuilder {
    modify: EntityModifyBuilder,
    inserts: Vec<(ComponentHash, TVal)>,
    removes: Vec<ComponentHash>,
}

impl EntityTransformBuilder {
    pub fn create(modify: EntityModifyBuilder) -> Self {
        EntityTransformBuilder {
            modify,
            inserts: Vec::new(),
            removes: Vec::new(),
        }
    }

    pub fn insert(&mut self, ch: ComponentHash, cval: TVal) {
        self.inserts.push((ch, cval));
    }

    pub fn remove(&mut self, ch: ComponentHash) {
        self.removes.push(ch);
    }

    pub fn get_mut_modify(&mut self) -> &mut EntityModifyBuilder {
        &mut self.modify
    }

    pub fn build(self, ctymgr: &ComponentTypeManager) -> Result<EntityTransform> {
        let mut inserts = Vec::with_capacity(self.inserts.len());
        let mut removes = Vec::with_capacity(self.removes.len());
        for (hash, val) in self.inserts {
            inserts.push((ctymgr.get_id_with_hash(hash)?, val));
        }
        for hash in self.removes {
            removes.push(ctymgr.get_id_with_hash(hash)?);
        }
        Ok(EntityTransform {
            inserts,
            removes,
            modify: match self.modify {
                EntityModifyBuilder::Create(Some(e)) => EntityModify::Create(e),
                EntityModifyBuilder::Modify(e) => EntityModify::Modify(e),
                EntityModifyBuilder::Destroy(e) => EntityModify::Destroy(e),
                _ => {
                    unreachable!("")
                }
            },
        })
    }
}

pub struct EntityTransform {
    modify: EntityModify,
    inserts: Vec<(ComponentTypeId, TVal)>,
    removes: Vec<ComponentTypeId>,
}

impl EntityTransform {
    pub fn get_modify(&self) -> EntityModify {
        self.modify
    }

    pub fn get_inserts(&self) -> &Vec<(ComponentTypeId, TVal)> {
        &self.inserts
    }

    pub fn get_removes(&self) -> &Vec<ComponentTypeId> {
        &self.removes
    }
}
