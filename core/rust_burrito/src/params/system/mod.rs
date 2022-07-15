use super::super::{component::Component, event::Event, system::SystemBus};
use mewo_ecs::{
    ArchetypeAccess, ComponentHash, ComponentQueryAccessType, ComponentQueryFilterType,
    ComponentTypeManager, EventHash, EventOption,
};
use std::marker::PhantomData;

mod impls;
mod iter;
mod params;

pub use iter::{ComponentsEIter, ComponentsIter};
pub use params::{Components, Events};

//  fn my_system(
//      ev: Events<Event>,
//      w: Components<
//          (&A, &mut B, Option<&C>, Option<&mut D>),
//          (With<A>, Without<B>),
//      >
//  )

//  Maybe move this to system.rs?
//  E=EventAccess, CA=ComponentAccesses, CF=ComponentFilters
pub type SystemFunction<E, CA, CF> = fn(SystemBus, Events<E>, Components<CA, CF>);

pub trait EventAccess {
    fn hash() -> EventOption<EventHash>;
    fn data(ev: &Option<*const u8>) -> &Self;
}

pub trait ComponentAccesses {
    fn accesses() -> Vec<(ComponentHash, ComponentQueryAccessType)>;
    fn hashes() -> Vec<ComponentHash>;
    fn datas(idx: usize, datas: &Vec<Option<*const u8>>) -> Self;
}

trait ComponentAccess {
    fn access() -> (ComponentHash, ComponentQueryAccessType);
    fn data(idx: usize, data: &Option<*const u8>) -> Self;
    fn hash() -> ComponentHash;
}

pub trait ComponentFilters {
    fn filters() -> Vec<(ComponentHash, ComponentQueryFilterType)>;
}

trait ComponentFilter {
    fn filter() -> (ComponentHash, ComponentQueryFilterType);
}

pub struct Startup;

pub struct With<C: Component>(PhantomData<C>);
pub struct Without<C: Component>(PhantomData<C>);
