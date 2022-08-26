use super::component::Component;
use mewo_ecs::{
    Entity, EntityModifyBuilder, EntityTransformBuilder, EntityTransformer,
    SharedComponentTypeManager, TVal,
};

pub struct EntityBus<'galaxy> {
    ctymgr: &'galaxy SharedComponentTypeManager,
    transformer: &'galaxy mut EntityTransformer,
}

impl<'galaxy> EntityBus<'galaxy> {
    pub fn create(
        ctymgr: &'galaxy SharedComponentTypeManager,
        transformer: &'galaxy mut EntityTransformer,
    ) -> Self {
        EntityBus {
            ctymgr,
            transformer,
        }
    }

    pub fn spawn(&mut self) -> EntityBurrito {
        EntityBurrito::create_insert(self.ctymgr, self.transformer)
    }

    pub fn despawn(&mut self, entity: Entity) {
        let rm = EntityBurrito::create_remove(entity, self.ctymgr, self.transformer);
        drop(rm);
    }

    pub fn modify(&mut self, entity: Entity) -> EntityBurrito {
        EntityBurrito::create_modify(entity, self.ctymgr, self.transformer)
    }
}

pub struct EntityBurrito<'galaxy> {
    ctymgr: &'galaxy SharedComponentTypeManager,
    transform: Option<EntityTransformBuilder>,
    transformer: &'galaxy mut EntityTransformer,
}

impl<'galaxy> EntityBurrito<'galaxy> {
    fn create_insert(
        ctymgr: &'galaxy SharedComponentTypeManager,
        transformer: &'galaxy mut EntityTransformer,
    ) -> Self {
        let transform = Some(EntityTransformBuilder::create(EntityModifyBuilder::Create(
            None,
        )));
        EntityBurrito {
            ctymgr,
            transform,
            transformer,
        }
    }

    fn create_modify(
        entity: Entity,
        ctymgr: &'galaxy SharedComponentTypeManager,
        transformer: &'galaxy mut EntityTransformer,
    ) -> Self {
        let transform = Some(EntityTransformBuilder::create(EntityModifyBuilder::Modify(
            entity,
        )));
        EntityBurrito {
            ctymgr,
            transform,
            transformer,
        }
    }

    fn create_remove(
        entity: Entity,
        ctymgr: &'galaxy SharedComponentTypeManager,
        transformer: &'galaxy mut EntityTransformer,
    ) -> Self {
        let transform = Some(EntityTransformBuilder::create(
            EntityModifyBuilder::Destroy(entity),
        ));
        EntityBurrito {
            ctymgr,
            transform,
            transformer,
        }
    }

    fn maybe_register<C>(&self)
    where
        C: Component,
    {
        let exists = self
            .ctymgr
            .read()
            .unwrap()
            .get_id_with_hash(C::component_trait_hash())
            .is_ok();
        if !exists {
            let _ = self
                .ctymgr
                .write()
                .unwrap()
                .register(C::component_trait_entry());
        }
    }

    pub fn insert<C: Component>(mut self, c: C) -> Self {
        self.maybe_register::<C>();
        self.transform.as_mut().unwrap().insert(
            C::component_trait_hash(),
            TVal::create(
                C::component_trait_entry().size,
                &c as *const C as *const u8,
                C::component_trait_entry().drop,
            ),
        );
        self
    }

    pub fn remove<C: Component>(mut self) -> Self {
        self.maybe_register::<C>();
        self.transform
            .as_mut()
            .unwrap()
            .remove(C::component_trait_hash());
        self
    }
}

impl<'exec> Drop for EntityBurrito<'exec> {
    fn drop(&mut self) {
        let transform = std::mem::replace(&mut self.transform, None).unwrap();
        self.transformer.insert(transform);
    }
}
