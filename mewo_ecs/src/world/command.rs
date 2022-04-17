use super::entity::{
    Entity,
    EntityWrapper
};
use super::world::{
    EntityModifyCallback,
    BoxedEntityModifyCallback,
};
use super::resource::{
    ResourceManager,
    ResourceModifyCallback,
    BoxedResourceModifyCallback,
};
use super::mask::BoolMask;

const COMMAND_ENTITY_SPAWN_RESERVE_CONST: usize = 32;
const COMMAND_ENTITY_REMOVE_RESERVE_CONST: usize = 32;
const COMMAND_ENTITY_MODIFY_RESERVE_CONST: usize = 16;
const COMMAND_RESOURCE_MODIFY_RESERVE_CONST: usize = 16;

pub struct WorldCommands {
    entity_spawns: Vec<Option<BoxedEntityModifyCallback>>,
    entity_removes: BoolMask,
    entity_modifies: Vec<(Entity, BoxedEntityModifyCallback)>,
    resource_modifies: Vec<BoxedResourceModifyCallback>,
}

impl WorldCommands {
    pub fn create() -> Self {
        WorldCommands {
            entity_spawns: {
                let mut vec = Vec::new();
                vec.reserve(COMMAND_ENTITY_SPAWN_RESERVE_CONST);
                vec
            },
            entity_removes: {
                let mut mask = BoolMask::create();
                mask.extend(COMMAND_ENTITY_REMOVE_RESERVE_CONST);
                mask
            },
            entity_modifies: {
                let mut entity_mods = Vec::new();
                entity_mods.reserve(COMMAND_ENTITY_MODIFY_RESERVE_CONST);
                entity_mods
            },
            resource_modifies: {
                let mut resource_mods = Vec::new();
                resource_mods.reserve(COMMAND_RESOURCE_MODIFY_RESERVE_CONST);
                resource_mods
            },
        } 
    }

    pub fn modify_entity<F>(&mut self, entity: Entity, callback: F) -> &mut Self 
        where F: 'static + Fn(EntityWrapper) -> ()
    {
        self.entity_modifies.push((entity, Box::new(EntityModifyCallback(callback))));
        self
    }

    pub fn spawn_entity<F>(&mut self, callback: Option<F>) -> &mut Self 
        where F: 'static + Fn(EntityWrapper) -> ()
    {
        self.entity_spawns.push(
            if let Some(callback) = callback {
                Some(Box::new(EntityModifyCallback(callback)))
            } else {
                None
            }
        );
        self
    }

    pub fn remove_entity(&mut self, e: Entity) -> &mut Self {
        if e.id as usize >= self.entity_removes.get_len() {
            self.entity_removes.extend(COMMAND_ENTITY_SPAWN_RESERVE_CONST);
        }
        match self.entity_removes.set(e.id as usize, true) {
            Err(_) => unreachable!("Only reachable if self.remove_entities fails to resize"),
            _ => {},
        }
        self
    }

    pub fn modify_resources<F>(&mut self, callback: F) -> &mut Self 
        where F: 'static + Fn(&mut ResourceManager) -> ()
    {
        self.resource_modifies.push(Box::new(ResourceModifyCallback(callback)));
        self
    }

    pub fn get_entity_commands(&self) -> (&Vec<Option<BoxedEntityModifyCallback>>, &BoolMask, &Vec<(Entity, BoxedEntityModifyCallback)>) {
        (&self.entity_spawns, &self.entity_removes, &self.entity_modifies)
    }

    pub fn get_resource_commands(&self) -> &Vec<BoxedResourceModifyCallback> {
        &self.resource_modifies
    }

    pub fn flush(&mut self) {
        self.entity_spawns.clear();
        self.entity_removes.flush();
        self.entity_modifies.clear();
        self.resource_modifies.clear();
    }

    pub fn merge(&mut self, mut other: Self) {
        self.entity_spawns.append(&mut other.entity_spawns);
        self.entity_modifies.append(&mut other.entity_modifies);
        self.resource_modifies.append(&mut other.resource_modifies);
        self.entity_removes.extend(self.entity_removes.get_len());
        for i in 0..other.entity_removes.get_len() {
            if other.entity_removes.get(i).unwrap() {
                self.entity_removes.set(i, true).unwrap();
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        self.entity_spawns.is_empty() && self.entity_removes.is_empty()
    }
}

