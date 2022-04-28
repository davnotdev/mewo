#![allow(unused_parens)]
use super::entity::Entity;
use super::system::{ComponentAccessMode, FilterMode};
use super::wish::*;
use super::world::World;
use std::any::TypeId;

//  TODO: use macros silly!

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
