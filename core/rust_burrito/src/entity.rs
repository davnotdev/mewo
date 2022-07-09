use super::component::Component;
use mewo_ecs::{
    Entity, EntityModifyBuilder, EntityTransformBuilder, EntityTransformer, TVal, ValueDrop,
};

pub struct EntityBus<'exec> {
    transformer: &'exec mut EntityTransformer,
}

impl<'exec> EntityBus<'exec> {
    pub fn create(transformer: &'exec mut EntityTransformer) -> Self {
        EntityBus { transformer }
    }

    pub fn spawn(&mut self) -> EntityBurrito {
        EntityBurrito::create_insert(self.transformer)
    }

    pub fn despawn(&mut self, entity: Entity) {
        let rm = EntityBurrito::create_remove(entity, self.transformer);
        drop(rm);
    }

    pub fn modify(&mut self, entity: Entity) -> EntityBurrito {
        EntityBurrito::create_modify(entity, self.transformer)
    }
}

pub struct EntityBurrito<'exec> {
    transform: Option<EntityTransformBuilder>,
    transformer: &'exec mut EntityTransformer,
}

impl<'exec> EntityBurrito<'exec> {
    pub fn create_insert(transformer: &'exec mut EntityTransformer) -> Self {
        let transform = Some(EntityTransformBuilder::create(EntityModifyBuilder::Create(
            None,
        )));
        EntityBurrito {
            transform,
            transformer,
        }
    }

    pub fn create_modify(entity: Entity, transformer: &'exec mut EntityTransformer) -> Self {
        let transform = Some(EntityTransformBuilder::create(EntityModifyBuilder::Modify(
            entity,
        )));
        EntityBurrito {
            transform,
            transformer,
        }
    }

    pub fn create_remove(entity: Entity, transformer: &'exec mut EntityTransformer) -> Self {
        let transform = Some(EntityTransformBuilder::create(
            EntityModifyBuilder::Destroy(entity),
        ));
        EntityBurrito {
            transform,
            transformer,
        }
    }

    pub fn insert<C: Component>(mut self, c: C) -> Self {
        self.transform.as_mut().unwrap().insert(
            C::component_hash(),
            TVal::create(
                C::component_size(),
                &c as *const C as *const u8,
                ValueDrop::create(C::component_drop_callback()),
            ),
        );
        self
    }
}

impl<'exec> Drop for EntityBurrito<'exec> {
    fn drop(&mut self) {
        let transform = std::mem::replace(&mut self.transform, None).unwrap();
        self.transformer.insert(transform);
    }
}
