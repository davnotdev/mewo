use super::entity::{
    BoxedEntityModifyCallback, Entity, EntityModifyCallback, EntityModifyHandle, EntityWrapper,
};
use super::resource::{BoxedResourceModifyCallback, ResourceManager, ResourceModifyCallback};

const COMMAND_ENTITY_SPAWN_RESERVE_CONST: usize = 16;
const COMMAND_ENTITY_REMOVE_RESERVE_CONST: usize = 16;
const COMMAND_RESOURCE_MODIFY_RESERVE_CONST: usize = 16;

pub struct WorldCommands {
    pub entity_cmds: Vec<(EntityModifyHandle, Option<BoxedEntityModifyCallback>)>,
    pub entity_removes: Vec<Entity>,
    pub resource_modifies: Vec<BoxedResourceModifyCallback>,
}

impl WorldCommands {
    pub fn create() -> Self {
        WorldCommands {
            entity_cmds: Vec::with_capacity(COMMAND_ENTITY_SPAWN_RESERVE_CONST),
            entity_removes: Vec::with_capacity(COMMAND_ENTITY_REMOVE_RESERVE_CONST),
            resource_modifies: Vec::with_capacity(COMMAND_RESOURCE_MODIFY_RESERVE_CONST),
        }
    }

    pub fn spawn_entity<F>(&mut self, callback: Option<F>)
    where
        F: 'static + Fn(&mut EntityWrapper),
    {
        if let Some(callback) = callback {
            self.entity_cmds.push((
                EntityModifyHandle::Spawn,
                Some(Box::new(EntityModifyCallback(callback))),
            ))
        } else {
            self.entity_cmds.push((EntityModifyHandle::Spawn, None));
        }
    }

    pub fn modify_entity<F>(&mut self, entity: Entity, callback: F)
    where
        F: 'static + Fn(&mut EntityWrapper),
    {
        self.entity_cmds.push((
            EntityModifyHandle::Entity(entity),
            Some(Box::new(EntityModifyCallback(callback))),
        ));
    }

    pub fn remove_entity(&mut self, entity: Entity) {
        self.entity_removes.push(entity);
    }

    pub fn modify_resources<F>(&mut self, callback: F)
    where
        F: 'static + Fn(&mut ResourceManager) -> (),
    {
        self.resource_modifies
            .push(Box::new(ResourceModifyCallback(callback)));
    }

    pub fn flush(&mut self) {
        self.entity_cmds.clear();
        self.entity_removes.clear();
        self.resource_modifies.clear();
    }

    pub fn merge(&mut self, mut other: Self) {
        self.entity_cmds.append(&mut other.entity_cmds);
        self.entity_removes.append(&mut other.entity_removes);
        self.resource_modifies.append(&mut other.resource_modifies);
    }
}
