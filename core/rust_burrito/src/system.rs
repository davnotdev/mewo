use super::{component::Component, event::EventBus, resource::ResourceBus, wish::Wish};
use mewo_ecs::{Entity, EntityModifyBuilder, EntityTransformBuilder, EntityTransformer, TVal};

pub type SystemFunction<WE, WA, WF> = fn(SA, Wish<WE, WA, WF>);

//  For lazy people like me.
pub type SA<'exec> = SystemArgs<'exec>;

pub struct SystemArgs<'exec> {
    pub events: EventBus<'exec>,
    pub resources: ResourceBus<'exec>,
    pub transformer: &'exec mut EntityTransformer,
}

impl<'exec> SystemArgs<'exec> {
    pub fn create(
        events: EventBus<'exec>,
        resources: ResourceBus<'exec>,
        transformer: &'exec mut EntityTransformer,
    ) -> Self {
        SystemArgs {
            events,
            resources,
            transformer,
        }
    }

    pub fn spawn<'sa>(&'sa mut self) -> EntityInsertBurrito<'sa, 'exec> {
        EntityInsertBurrito::create(self)
    }
}

pub struct EntityBurrito<'sa, 'exec> {
    args: &'sa mut SystemArgs<'exec>,
    entity: Entity,
    transform: Option<EntityTransformBuilder>,
}

impl<'sa, 'exec> EntityBurrito<'sa, 'exec> {
    pub fn create(args: &'sa mut SA<'exec>, entity: Entity) -> Self {
        let transform = Some(EntityTransformBuilder::create(EntityModifyBuilder::Modify(
            entity,
        )));
        EntityBurrito {
            args,
            entity,
            transform,
        }
    }

    pub fn insert<C: Component>(mut self, c: C) -> Self {
        self.transform.as_mut().unwrap().insert(
            C::hash(),
            TVal::create(std::mem::size_of::<C>(), &c as *const C as *const u8),
        );
        self
    }

    pub fn remove(mut self) {
        *self.transform.as_mut().unwrap().get_mut_modify() =
            EntityModifyBuilder::Destroy(self.entity);
        drop(self)
    }
}

impl<'sa, 'exec> Drop for EntityBurrito<'sa, 'exec> {
    fn drop(&mut self) {
        let transform = std::mem::replace(&mut self.transform, None).unwrap();
        self.args.transformer.insert(transform);
    }
}

pub struct EntityInsertBurrito<'sa, 'exec> {
    args: &'sa mut SystemArgs<'exec>,
    transform: Option<EntityTransformBuilder>,
}

impl<'sa, 'exec> EntityInsertBurrito<'sa, 'exec> {
    pub fn create(args: &'sa mut SA<'exec>) -> Self {
        let transform = Some(EntityTransformBuilder::create(EntityModifyBuilder::Create(
            None,
        )));
        EntityInsertBurrito { args, transform }
    }

    pub fn insert<C: Component>(mut self, c: C) -> Self {
        self.transform.as_mut().unwrap().insert(
            C::hash(),
            TVal::create(std::mem::size_of::<C>(), &c as *const C as *const u8),
        );
        self
    }
}

impl<'sa, 'exec> Drop for EntityInsertBurrito<'sa, 'exec> {
    fn drop(&mut self) {
        let transform = std::mem::replace(&mut self.transform, None).unwrap();
        self.args.transformer.insert(transform);
    }
}
