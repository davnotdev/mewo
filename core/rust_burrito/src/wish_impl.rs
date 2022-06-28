use super::{
    component::Component,
    event::Event,
    wish::{Startup, WishAccess, WishAccesses, WishEvent, WishFilter, WishFilters},
};
use mewo_ecs::{ComponentHash, ComponentQueryAccessType, EventHash, EventOption};

//  TODO: use macros silly!

impl WishEvent for () {
    fn hash() -> EventOption<EventHash> {
        EventOption::Update
    }
}

impl WishEvent for Startup {
    fn hash() -> EventOption<EventHash> {
        EventOption::Startup
    }
}

impl<E> WishEvent for E
where
    E: Event,
{
    fn hash() -> EventOption<EventHash> {
        EventOption::Event(E::hash())
    }
}

impl<C> WishAccess for &C
where
    C: Component,
{
    fn access() -> (ComponentHash, ComponentQueryAccessType) {
        (C::hash(), ComponentQueryAccessType::Read)
    }

    fn data(idx: usize, data: &Option<*const u8>) -> Self {
        unsafe {
            (data.unwrap() as *const C)
                .offset(idx as isize)
                .as_ref()
                .unwrap()
        }
    }

    fn hash() -> ComponentHash {
        C::hash()
    }
}

impl<C> WishAccess for &mut C
where
    C: Component,
{
    fn access() -> (ComponentHash, ComponentQueryAccessType) {
        (C::hash(), ComponentQueryAccessType::Write)
    }

    fn data(idx: usize, data: &Option<*const u8>) -> Self {
        unsafe {
            (data.unwrap() as *mut C)
                .offset(idx as isize)
                .as_mut()
                .unwrap()
        }
    }

    fn hash() -> ComponentHash {
        C::hash()
    }
}

impl<C> WishAccess for Option<&C>
where
    C: Component,
{
    fn access() -> (ComponentHash, ComponentQueryAccessType) {
        (C::hash(), ComponentQueryAccessType::Write)
    }

    fn data(idx: usize, data: &Option<*const u8>) -> Self {
        if let Some(data) = data {
            Some(unsafe { (data as *const C).offset(idx as isize).as_ref().unwrap() })
        } else {
            None
        }
    }

    fn hash() -> ComponentHash {
        C::hash()
    }
}

impl<C> WishAccess for Option<&mut C>
where
    C: Component,
{
    fn access() -> (ComponentHash, ComponentQueryAccessType) {
        (C::hash(), ComponentQueryAccessType::Write)
    }

    fn data(idx: usize, data: &Option<*const u8>) -> Self {
        if let Some(data) = data {
            Some(unsafe { (data as *mut C).offset(idx as isize).as_mut().unwrap() })
        } else {
            None
        }
    }

    fn hash() -> ComponentHash {
        C::hash()
    }
}

impl WishAccesses for () {
    fn accesses() -> Vec<(ComponentHash, ComponentQueryAccessType)> {
        vec![]
    }

    fn hashes() -> Vec<ComponentHash> {
        vec![]
    }

    fn datas(_: usize, _: &Vec<Option<*const u8>>) -> Self {
        ()
    }
}

impl<C0> WishAccesses for C0
where
    C0: WishAccess,
{
    fn accesses() -> Vec<(ComponentHash, ComponentQueryAccessType)> {
        vec![C0::access()]
    }

    fn hashes() -> Vec<ComponentHash> {
        vec![C0::hash()]
    }

    fn datas(idx: usize, datas: &Vec<Option<*const u8>>) -> Self {
        C0::data(idx, datas.get(0).unwrap())
    }
}

impl<C0, C1> WishAccesses for (C0, C1)
where
    C0: WishAccess,
    C1: WishAccess,
{
    fn accesses() -> Vec<(ComponentHash, ComponentQueryAccessType)> {
        vec![C0::access(), C1::access()]
    }

    fn hashes() -> Vec<ComponentHash> {
        vec![C0::hash(), C1::hash()]
    }

    fn datas(idx: usize, datas: &Vec<Option<*const u8>>) -> Self {
        (
            C0::data(idx, datas.get(0).unwrap()),
            C1::data(idx, datas.get(1).unwrap()),
        )
    }
}

impl WishFilters for () {
    fn filters() -> Vec<(ComponentHash, mewo_ecs::ComponentQueryFilterType)> {
        Vec::new()
    }
}

impl<WF0> WishFilters for WF0
where
    WF0: WishFilter,
{
    fn filters() -> Vec<(ComponentHash, mewo_ecs::ComponentQueryFilterType)> {
        vec![WF0::filter()]
    }
}

impl<WF0, WF1> WishFilters for (WF0, WF1)
where
    WF0: WishFilter,
    WF1: WishFilter,
{
    fn filters() -> Vec<(ComponentHash, mewo_ecs::ComponentQueryFilterType)> {
        vec![WF0::filter(), WF1::filter()]
    }
}
