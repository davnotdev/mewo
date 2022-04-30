#![allow(unused_parens)]
use super::entity::Entity;
use super::system::{ComponentAccessMode, FilterMode, SystemDataSetInstance};
use super::wish::*;
use super::world::World;
use std::any::TypeId;

//  TODO: use macros silly!

//  Wishes

impl Wishes for () {
    fn create(_world: *const World, _set_insts: *const Vec<SystemDataSetInstance>) -> Self {
        ()
    }
    fn get_wish_datas() -> Vec<WishData> {
        Vec::new()
    }
}

impl<WT0, WF0> Wishes for (Wish<WT0, WF0>)
where
    WT0: WishTypes,
    WF0: WishFilters,
{
    fn create(world: *const World, set_insts: *const Vec<SystemDataSetInstance>) -> Self {
        let set_insts = unsafe { set_insts.as_ref().unwrap() };
        (Wish::<WT0, WF0>::create(world, set_insts.get(0).unwrap()))
    }
    fn get_wish_datas() -> Vec<WishData> {
        vec![Wish::<WT0, WF0>::get_wish_data()]
    }
}

impl<WT0, WF0, WT1, WF1> Wishes for (Wish<WT0, WF0>, Wish<WT1, WF1>)
where
    WT0: WishTypes,
    WF0: WishFilters,
    WT1: WishTypes,
    WF1: WishFilters,
{
    fn create(world: *const World, set_insts: *const Vec<SystemDataSetInstance>) -> Self {
        let set_insts = unsafe { set_insts.as_ref().unwrap() };
        (
            Wish::create(world, set_insts.get(0).unwrap()),
            Wish::create(world, set_insts.get(1).unwrap()),
        )
    }
    fn get_wish_datas() -> Vec<WishData> {
        vec![
            Wish::<WT0, WF0>::get_wish_data(),
            Wish::<WT1, WF1>::get_wish_data(),
        ]
    }
}

impl<WT0, WF0, WT1, WF1, WT2, WF2> Wishes for (Wish<WT0, WF0>, Wish<WT1, WF1>, Wish<WT2, WF2>)
where
    WT0: WishTypes,
    WF0: WishFilters,
    WT1: WishTypes,
    WF1: WishFilters,
    WT2: WishTypes,
    WF2: WishFilters,
{
    fn create(world: *const World, set_insts: *const Vec<SystemDataSetInstance>) -> Self {
        let set_insts = unsafe { set_insts.as_ref().unwrap() };
        (
            Wish::create(world, set_insts.get(0).unwrap()),
            Wish::create(world, set_insts.get(1).unwrap()),
            Wish::create(world, set_insts.get(2).unwrap()),
        )
    }
    fn get_wish_datas() -> Vec<WishData> {
        vec![
            Wish::<WT0, WF0>::get_wish_data(),
            Wish::<WT1, WF1>::get_wish_data(),
            Wish::<WT2, WF2>::get_wish_data(),
        ]
    }
}

//  WishTypes

impl WishTypes for () {
    fn get_types() -> Vec<(TypeId, ComponentAccessMode)> {
        Vec::new()
    }

    fn datas(_world: &World, _entity: Entity) -> Self {
        ()
    }
}

impl<W0> WishTypes for (W0)
where
    W0: WishType,
{
    fn get_types() -> Vec<(TypeId, ComponentAccessMode)> {
        vec![W0::get_type()]
    }

    fn datas(world: &World, entity: Entity) -> Self {
        (W0::data(
            world
                .get_raw_component_with_type_id_and_entity(W0::get_type().0, entity)
                .unwrap(),
        ))
    }
}

impl<W0, W1> WishTypes for (W0, W1)
where
    W0: WishType,
    W1: WishType,
{
    fn get_types() -> Vec<(TypeId, ComponentAccessMode)> {
        let ret = vec![W0::get_type(), W1::get_type()];
        println!("many types? {:?}", ret);
        ret
    }

    fn datas(world: &World, entity: Entity) -> Self {
        (
            W0::data(
                world
                    .get_raw_component_with_type_id_and_entity(W0::get_type().0, entity)
                    .unwrap(),
            ),
            W1::data(
                world
                    .get_raw_component_with_type_id_and_entity(W1::get_type().0, entity)
                    .unwrap(),
            ),
        )
    }
}

//  WishFilters
impl WishFilters for () {
    fn get_filters() -> Vec<(TypeId, FilterMode)> {
        Vec::new()
    }
}

impl<F0> WishFilters for (F0)
where
    F0: WishFilter,
{
    fn get_filters() -> Vec<(TypeId, FilterMode)> {
        vec![F0::get_filter()]
    }
}

impl<F0, F1> WishFilters for (F0, F1)
where
    F0: WishFilter,
    F1: WishFilter,
{
    fn get_filters() -> Vec<(TypeId, FilterMode)> {
        vec![F0::get_filter(), F1::get_filter()]
    }
}

impl<F0, F1, F2> WishFilters for (F0, F1, F2)
where
    F0: WishFilter,
    F1: WishFilter,
    F2: WishFilter,
{
    fn get_filters() -> Vec<(TypeId, FilterMode)> {
        vec![F0::get_filter(), F1::get_filter(), F2::get_filter()]
    }
}

impl<F0, F1, F2, F3> WishFilters for (F0, F1, F2, F3)
where
    F0: WishFilter,
    F1: WishFilter,
    F2: WishFilter,
    F3: WishFilter,
{
    fn get_filters() -> Vec<(TypeId, FilterMode)> {
        vec![
            F0::get_filter(),
            F1::get_filter(),
            F2::get_filter(),
            F3::get_filter(),
        ]
    }
}
