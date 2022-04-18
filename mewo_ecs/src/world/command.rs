use super::entity::{
    Entity,
    EntityModifier,
    EntityModifierStore,
    EntityModifierHandle,
};
use super::resource::{
    ResourceManager,
    ResourceModifyCallback,
    BoxedResourceModifyCallback,
};
use super::World;

const COMMAND_ENTITY_MOD_RESERVE_CONST: usize = 16;
const COMMAND_ENTITY_REMOVE_RESERVE_CONST: usize = 16;
const COMMAND_RESOURCE_MODIFY_RESERVE_CONST: usize = 16;

pub struct WorldCommandsStore {
    pub entity_removes: Vec<Entity>,
    pub entity_cmds: Vec<EntityModifierStore>,
    pub resource_modifies: Vec<BoxedResourceModifyCallback>,
}

impl WorldCommandsStore {
    pub fn create() -> Self {
        WorldCommandsStore {
            entity_cmds: Vec::with_capacity(COMMAND_ENTITY_MOD_RESERVE_CONST),
            resource_modifies: Vec::with_capacity(COMMAND_RESOURCE_MODIFY_RESERVE_CONST),
            entity_removes: Vec::with_capacity(COMMAND_ENTITY_REMOVE_RESERVE_CONST),
        } 
    }

    pub fn modify<'world, 'store>(&'store mut self, world: &'world World) -> WorldCommands<'world, 'store> {
        WorldCommands {
            world,
            entity_cmds: &mut self.entity_cmds,
            resource_modifies: &mut self.resource_modifies,
            entity_removes: &mut self.entity_removes,
        }
    }

    pub fn flush(&mut self) {
        self.entity_cmds.clear();
        self.resource_modifies.clear();
        self.entity_removes.clear()
    }

    pub fn merge(&mut self, mut other: Self) {
        self.entity_cmds.append(&mut other.entity_cmds);
        self.resource_modifies.append(&mut other.resource_modifies);
        self.entity_removes.append(&mut other.entity_removes);
    }
}

pub struct WorldCommands<'world, 'store> {
    world: &'world World,
    entity_cmds: &'store mut Vec<EntityModifierStore>,
    resource_modifies: &'store mut Vec<BoxedResourceModifyCallback>,
    entity_removes: &'store mut Vec<Entity>,
}

impl<'world, 'store> WorldCommands<'world, 'store> {
    pub fn modify_entity(&mut self, entity: Entity) -> EntityModifier {
        let store = EntityModifierStore::create(EntityModifierHandle::Modify(entity), self.world);
        self.entity_cmds.push(store);
        let len = self.entity_cmds.len()-1;
        let store = self.entity_cmds.get_mut(len).unwrap();
        let modifier = store.modify(self.world);
        modifier
    }

    pub fn spawn_entity(&mut self) -> EntityModifier {
        let store = EntityModifierStore::create(EntityModifierHandle::Spawn, self.world);
        self.entity_cmds.push(store);
        let len = self.entity_cmds.len()-1;
        let store = self.entity_cmds.get_mut(len).unwrap();
        let modifier = store.modify(self.world);
        modifier
    }

    pub fn remove_entity(&mut self, e: Entity) {
        self.entity_removes.push(e);
    }

    pub fn modify_resources<F>(&mut self, callback: F)  
        where F: 'static + Fn(&mut ResourceManager) -> ()
    {
        self.resource_modifies.push(Box::new(ResourceModifyCallback(callback)));
    }
}

