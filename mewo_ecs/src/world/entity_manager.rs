use super::error::{ Result, ECSError };
use super::entity::{ Id, Entity };
use super::mask::BoolMask;

const ENTITY_MANAGER_EXTEND_CONST: usize = 128;

pub struct EntityManager {
    entities: BoolMask, 
    entity_count: Id,
}

impl EntityManager {
    pub fn create() -> EntityManager {
        EntityManager {
            entities: BoolMask::create(),
            entity_count: 0,
        } 
    }

    pub fn register_entity(&mut self) -> Entity {
        if self.entities.get_len() >= self.entity_count as usize {
            self.entities.extend(ENTITY_MANAGER_EXTEND_CONST);
        };
        for ei in 0..self.entities.get_len() {
            let taken = self.entities.get(ei).unwrap();
            if !taken {
                self.entities.set(ei, true).unwrap();
                self.entity_count += 1;
                return Entity::from_id(ei as Id);
            }
        }
        self.entity_count += 1;
        Entity {
            id: self.entity_count
        }
    }

    pub fn deregister_entity(&mut self, entity: Entity) -> Result<()> {
        if self.entity_exists(entity) {
            self.entities.set(entity.id as usize, false).unwrap();
            self.entity_count -= 1; 
            return Ok(())
        }
        Err(ECSError::EntityDoesNotExist(entity))
    }

    pub fn entity_exists(&self, entity: Entity) -> bool {
        if let Ok(exists) = self.entities.get(entity.id as usize) {
            if exists {
                return true
            }
        }
        false
    }
}

#[test]
fn test_entity_manager() {
    let mut entity_manager = EntityManager::create();
    let mut entities = Vec::new();
    for _ in 0..10 {
        entities.push(entity_manager.register_entity());
    } 
    for i in 0..10 {
        assert_eq!(entity_manager.entity_exists(entities[i]), true);
    }
    let rm = 3;
    entity_manager.deregister_entity(entities[rm]).unwrap();
    for i in 0..10 {
        assert_eq!(entity_manager.entity_exists(entities[i]), i != rm);
    }
    for i in 0..10 {
        if i != rm {
            entity_manager.deregister_entity(entities[i]).unwrap();
        }
    }
    for i in 0..10 {
        assert_eq!(entity_manager.entity_exists(entities[i]), false);
    }
}

