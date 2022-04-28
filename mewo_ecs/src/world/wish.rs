use super::component::Component;
use super::entity::Entity;
use super::system::{ComponentAccessMode, FilterMode, SystemDataSetInstance};
use super::world::World;
use std::any::TypeId;
use std::iter::Iterator;
use std::marker::PhantomData;

/*

Wish<(          <-- WishTypes
    &C,             <-- WishType
    &mut C,         <-'
    ...
), (            <-- WishFilters
    With<W>,        <-- WishFilter
    Without<W>,     <-'
    ...
)>

*/

#[derive(Debug)]
pub struct WishData {
    pub tyids: Vec<(TypeId, ComponentAccessMode)>,
    pub filters: Vec<(TypeId, FilterMode)>,
}

pub struct Wish<'world, 'setinst, WTypes, WFilters>
where
    WTypes: WishTypes,
    WFilters: WishFilters,
{
    phantom: PhantomData<(WTypes, WFilters)>,
    world: &'world World,
    set_inst: &'setinst SystemDataSetInstance,
}

impl<'world, 'setinst, WTypes, WFilters> Wish<'world, 'setinst, WTypes, WFilters>
where
    WTypes: WishTypes,
    WFilters: WishFilters,
{
    pub fn create(world: &'world World, set_inst: &'setinst SystemDataSetInstance) -> Self {
        Wish {
            world,
            set_inst,
            phantom: PhantomData,
        }
    }

    pub fn get_wish_data() -> WishData {
        let wish_data = WishData {
            tyids: WTypes::get_types(),
            filters: WFilters::get_filters(),
        };
        check_wish(&wish_data);
        wish_data
    }

    pub fn iter(&self) -> WishIter<WTypes> {
        WishIter {
            phantom: PhantomData,
            entity_idx: 0,
            world: self.world,
            set_inst: self.set_inst,
        }
    }
}

pub struct WishIter<'world, 'setinst, WTypes>
where
    WTypes: WishTypes,
{
    phantom: PhantomData<WTypes>,
    entity_idx: usize,
    world: &'world World,
    set_inst: &'setinst SystemDataSetInstance,
}

impl<'world, 'entities, WTypes> Iterator for WishIter<'world, 'entities, WTypes>
where
    WTypes: WishTypes,
{
    type Item = (Entity, WTypes);
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(entity) = self.set_inst.entities.get(self.entity_idx) {
            self.entity_idx += 1;
            Some((*entity, WTypes::datas(self.world, *entity)))
        } else {
            None
        }
    }
}

pub trait WishTypes {
    fn get_types() -> Vec<(TypeId, ComponentAccessMode)>;
    fn datas(world: &World, entity: Entity) -> Self;
}

pub trait WishType {
    fn get_type() -> (TypeId, ComponentAccessMode);
    fn data(data: *const ()) -> Self;
}

pub struct R<C: Component>(*const C);
pub struct W<C: Component>(*const C);

impl<C> WishType for R<C>
where
    C: 'static + Component,
{
    fn get_type() -> (TypeId, ComponentAccessMode) {
        (TypeId::of::<C>(), ComponentAccessMode::Read)
    }

    fn data(data: *const ()) -> Self {
        Self(data as *const C)
    }
}

impl<C> WishType for W<C>
where
    C: 'static + Component,
{
    fn get_type() -> (TypeId, ComponentAccessMode) {
        (TypeId::of::<C>(), ComponentAccessMode::Write)
    }

    fn data(data: *const ()) -> Self {
        Self(data as *const C)
    }
}

impl<C> std::ops::Deref for R<C>
where
    C: 'static + Component,
{
    type Target = C;
    fn deref(&self) -> &Self::Target {
        unsafe { self.0.as_ref() }.unwrap()
    }
}

impl<C> std::ops::Deref for W<C>
where
    C: 'static + Component,
{
    type Target = C;
    fn deref(&self) -> &Self::Target {
        unsafe { self.0.as_ref() }.unwrap()
    }
}

impl<C> std::ops::DerefMut for W<C>
where
    C: 'static + Component,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { (self.0 as *mut C).as_mut() }.unwrap()
    }
}

pub trait WishFilters {
    fn get_filters() -> Vec<(TypeId, FilterMode)>;
}

pub trait WishFilter {
    fn get_filter() -> (TypeId, FilterMode);
}

pub struct With<C: Component>(PhantomData<C>);
pub struct Without<C: Component>(PhantomData<C>);

impl<C> WishFilter for With<C>
where
    C: 'static + Component,
{
    fn get_filter() -> (TypeId, FilterMode) {
        (TypeId::of::<C>(), FilterMode::With)
    }
}

impl<C> WishFilter for Without<C>
where
    C: 'static + Component,
{
    fn get_filter() -> (TypeId, FilterMode) {
        (TypeId::of::<C>(), FilterMode::Without)
    }
}

/*

The Rules of Wishing:
1. No wishing for more wishes
2. No duplicate component types (check_wish)
*/

fn check_wish(wish: &WishData) {
    let mut takens = Vec::new();
    for (ty, _) in wish.tyids.iter() {
        if takens.contains(&ty) {
            wish_rule_panic(wish);
        }
        takens.push(ty);
    }
    for (ty, _) in wish.filters.iter() {
        if takens.contains(&ty) {
            wish_rule_panic(wish);
        }
        takens.push(ty);
    }
}

fn wish_rule_panic(wish: &WishData) {
    //  :o, a fancy error message
    panic!(
        "\n\nWishes cannot contain duplicates of Components. Wish Data (This may or may not be useful).\n|\n`-> {:?}\n\n",
        wish
    );
}
