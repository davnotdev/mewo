use crate::error::{ 
    Result, 
    ECSError 
};
use super::entity::{ 
    Id, 
    Entity 
};

const ENTITY_MANAGER_EXTEND_CONST: usize = 128;

pub struct EntityManager {
    entities: Vec<bool>, 
    entity_count: Id,
}

impl EntityManager {
    pub fn create() -> EntityManager {
        EntityManager {
            entities: Vec::with_capacity(ENTITY_MANAGER_EXTEND_CONST),
            entity_count: 0,
        } 
    }

    pub fn register_entity(&mut self) -> Entity {
        if self.entities.len() >= self.entity_count as usize {
            self.entities.resize(ENTITY_MANAGER_EXTEND_CONST, false)
        };
        for (ei, taken) in self.entities.iter_mut().enumerate() {
            if !*taken {
                *taken = true;
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
            *self.entities.get_mut(entity.as_index()).unwrap() = false;
            self.entity_count -= 1; 
            return Ok(())
        }
        Err(ECSError::EntityDoesNotExist(entity))
    }

    pub fn entity_exists(&self, entity: Entity) -> bool {
        if let Some(exists) = self.entities.get(entity.as_index()) {
            *exists
        } else {
            false
        }
    }

    pub fn get_entity_count(&self) -> usize {
        self.entity_count as usize
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

