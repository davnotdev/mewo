use super::{
    component::{ComponentAccesses, ComponentFilters, Components},
    entity::EntityBus,
    event::EventBus,
    resource::ResourceBus,
};

pub struct SystemBus<'galaxy, 'exec, CA, CF> {
    pub entities: EntityBus<'exec>,
    pub events: EventBus<'galaxy, 'exec>,
    pub resources: ResourceBus<'exec>,
    pub components: Components<'exec, CA, CF>,
}

impl<'galaxy, 'exec, CA, CF> SystemBus<'galaxy, 'exec, CA, CF>
where
    CA: ComponentAccesses,
    CF: ComponentFilters,
{
    pub fn create(
        entities: EntityBus<'exec>,
        events: EventBus<'galaxy, 'exec>,
        resources: ResourceBus<'exec>,
        components: Components<'exec, CA, CF>,
    ) -> Self {
        SystemBus {
            entities,
            events,
            components,
            resources,
        }
    }
}

//  fn my_system(
//      sb: SystemBus<
//          (&A, &mut B, Option<&C>, Option<&mut D>),
//          (With<A>, Without<B>),
//      >,
//  ) -> () | Option<T>
//  CA=ComponentAccesses, CF=ComponentFilters

pub type SystemFunction<CA, CF, O> = fn(SystemBus<CA, CF>) -> O;
pub type EarlySystemFunction<CA, CF> = fn(SystemBus<CA, CF>) -> Option<()>;
