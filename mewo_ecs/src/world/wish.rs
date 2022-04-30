use super::component::Component;
use super::entity::Entity;
use super::system::{ComponentAccessMode, FilterMode, SystemDataSetInstance};
use super::world::World;
use std::any::TypeId;
use std::iter::Iterator;
use std::marker::PhantomData;

/*

(           <-- Wishes
Wish<(          <-- WishTypes
    &C,             <-- WishType
    &mut C,         <-'
    ...
), (            <-- WishFilters
    With<W>,        <-- WishFilter
    Without<W>,     <-'
    ...
)>
)

*/

pub trait Wishes {
    fn create(world: *const World, set_insts: *const Vec<SystemDataSetInstance>) -> Self;
    fn get_wish_datas() -> Vec<WishData>;
}

#[derive(Debug)]
pub struct WishData {
    pub tyids: Vec<(TypeId, ComponentAccessMode)>,
    pub filters: Vec<(TypeId, FilterMode)>,
}

pub struct Wish<WTypes, WFilters>
where
    WTypes: WishTypes,
    WFilters: WishFilters,
{
    phantom: PhantomData<(WTypes, WFilters)>,
    world: *const World,
    set_inst: *const SystemDataSetInstance,
}

impl<WTypes, WFilters> Wish<WTypes, WFilters>
where
    WTypes: WishTypes,
    WFilters: WishFilters,
{
    pub fn create(world: *const World, set_inst: *const SystemDataSetInstance) -> Self {
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

    pub fn eiter(&self) -> WishEIter<WTypes> {
        WishEIter {
            phantom: PhantomData,
            entity_idx: 0,
            world: self.world,
            set_inst: self.set_inst,
        }
    }
}

//  should be safe bc both world and set_inst are
//  never borrowed exclusively
//  until the end of system execution
pub struct WishIter<WTypes>
where
    WTypes: WishTypes,
{
    phantom: PhantomData<WTypes>,
    entity_idx: usize,
    world: *const World,
    set_inst: *const SystemDataSetInstance,
}

impl<WTypes> Iterator for WishIter<WTypes>
where
    WTypes: WishTypes,
{
    type Item = WTypes;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(entity) = unsafe { self.set_inst.as_ref().unwrap() }
            .entities
            .get(self.entity_idx)
        {
            self.entity_idx += 1;
            Some(WTypes::datas(
                unsafe { self.world.as_ref().unwrap() },
                *entity,
            ))
        } else {
            None
        }
    }
}

pub struct WishEIter<WTypes>
where
    WTypes: WishTypes,
{
    phantom: PhantomData<WTypes>,
    entity_idx: usize,
    world: *const World,
    set_inst: *const SystemDataSetInstance,
}

impl<WTypes> Iterator for WishEIter<WTypes>
where
    WTypes: WishTypes,
{
    type Item = (Entity, WTypes);
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(entity) = unsafe { self.set_inst.as_ref().unwrap() }
            .entities
            .get(self.entity_idx)
        {
            self.entity_idx += 1;
            Some((
                *entity,
                WTypes::datas(unsafe { self.world.as_ref().unwrap() }, *entity),
            ))
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

impl<C> WishType for &C
where
    C: 'static + Component,
{
    fn get_type() -> (TypeId, ComponentAccessMode) {
        (TypeId::of::<C>(), ComponentAccessMode::Read)
    }

    fn data(data: *const ()) -> Self {
        unsafe { (data as *const C).as_ref().unwrap() }
    }
}

impl<C> WishType for &mut C
where
    C: 'static + Component,
{
    fn get_type() -> (TypeId, ComponentAccessMode) {
        (TypeId::of::<C>(), ComponentAccessMode::Write)
    }

    fn data(data: *const ()) -> Self {
        unsafe { (data as *mut C).as_mut().unwrap() }
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
