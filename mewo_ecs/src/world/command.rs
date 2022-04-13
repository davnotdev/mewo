use super::resource::ResourceModifyCallback;
use super::entity::Entity;
use super::world::EntityModifyCallback;
use super::mask::BoolMask;

const COMMAND_ENTITY_SPAWN_RESERVE_CONST: usize = 32;
const COMMAND_ENTITY_REMOVE_RESERVE_CONST: usize = 32;
const COMMAND_ENTITY_MODIFY_RESERVE_CONST: usize = 16;
const COMMAND_RESOURCE_MODIFY_RESERVE_CONST: usize = 16;

pub struct WorldCommands {
    entity_spawns: Vec<Option<EntityModifyCallback>>,
    entity_removes: BoolMask,
    entity_modifies: Vec<(Entity, EntityModifyCallback)>,
    resource_modifies: Vec<ResourceModifyCallback>,
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

    pub fn modify_entity(&mut self, entity: Entity, callback: EntityModifyCallback) -> &mut Self {
        self.entity_modifies.push((entity, callback));
        self
    }

    pub fn modify_resources(&mut self, callback: ResourceModifyCallback) -> &mut Self {
        self.resource_modifies.push(callback);
        self
    }

    pub fn spawn_entity(&mut self, callback: Option<EntityModifyCallback>) -> &mut Self {
        self.entity_spawns.push(callback);
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

    pub fn get_entity_commands(&self) -> (&Vec<Option<EntityModifyCallback>>, &BoolMask, &Vec<(Entity, EntityModifyCallback)>) {
        (&self.entity_spawns, &self.entity_removes, &self.entity_modifies)
    }

    pub fn get_resource_commands(&self) -> &Vec<ResourceModifyCallback> {
        &self.resource_modifies
    }

    pub fn flush(&mut self) {
        self.entity_spawns.clear();
        self.entity_removes.flush();
        self.entity_modifies.clear();
        self.resource_modifies.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.entity_spawns.is_empty() && self.entity_removes.is_empty()
    }
}

