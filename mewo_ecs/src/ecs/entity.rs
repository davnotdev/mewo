use super::error::*;
//  EntityPlanet has one job: keep track of which entity ids exist.

//  Entity(Id, Generation)
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct Entity(usize, usize);

impl Entity {
    pub fn from(id: usize, gen: usize) -> Self {
        Entity(id, gen)
    }

    pub fn id(&self) -> usize {
        self.0
    }

    pub fn generation(&self) -> usize {
        self.1
    }
}

#[derive(Debug)]
pub struct EntityPlanet {
    //  Entity Id -> (generation, exists)
    entities: Vec<(usize, bool)>,
}

//  TODO OPT: Time complexity can be improved by inserting / removing many at a time.
impl EntityPlanet {
    pub fn new() -> Self {
        EntityPlanet {
            entities: Vec::new(),
        }
    }

    pub fn insert(&mut self) -> Entity {
        for (idx, (generation, entity_exists)) in self.entities.iter_mut().enumerate().rev() {
            if !*entity_exists {
                *generation += 1;
                *entity_exists = true;
                return Entity(idx, *generation);
            }
        }
        //  TODO OPT: Change 0 to 128, but for now, it's too annoying.
        self.entities.resize(self.entities.len() + 1, (0, false));
        self.insert()
    }

    pub fn remove(&mut self, entity: Entity) -> Result<()> {
        let err = ecs_err!(ErrorType::EntityPlanetRemove { entity }, self);

        let (generation, exists) = self.entities.get_mut(entity.0).ok_or_else(|| err.clone())?;
        if *generation != entity.1 {
            Err(err)?
        }
        *exists = false;
        *generation += 1;
        Ok(())
    }

    pub fn has_entity(&self, entity: Entity) -> bool {
        self.entities
            .get(entity.id())
            .map(|&(gen, ex)| gen == entity.generation() && ex)
            .unwrap_or(false)
    }

    pub fn get_entities(&self) -> Vec<Entity> {
        self.entities
            .iter()
            .copied()
            .enumerate()
            .filter_map(|(id, (generation, exists))| exists.then_some(Entity(id, generation)))
            .collect()
    }
}
