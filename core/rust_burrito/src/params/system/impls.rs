use super::*;

impl EventAccess for () {
    fn hash() -> EventOption<EventHash> {
        EventOption::Update
    }

    fn data(_: &Option<*const u8>) -> &Self {
        &()
    }
}

impl EventAccess for Startup {
    fn hash() -> EventOption<EventHash> {
        EventOption::Startup
    }

    fn data(_: &Option<*const u8>) -> &Self {
        &Startup
    }
}

impl<E> EventAccess for E
where
    E: Event,
{
    fn hash() -> EventOption<EventHash> {
        EventOption::Event(E::event_hash())
    }

    fn data(ev: &Option<*const u8>) -> &Self {
        unsafe { (ev.unwrap() as *const E).as_ref().unwrap() }
    }
}

impl<C> ComponentAccess for &C
where
    C: Component,
{
    fn access() -> (ComponentHash, ComponentQueryAccessType) {
        (C::component_hash(), ComponentQueryAccessType::Read)
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
        C::component_hash()
    }
}

impl<C> ComponentAccess for &mut C
where
    C: Component,
{
    fn access() -> (ComponentHash, ComponentQueryAccessType) {
        (C::component_hash(), ComponentQueryAccessType::Write)
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
        C::component_hash()
    }
}

impl<C> ComponentAccess for Option<&C>
where
    C: Component,
{
    fn access() -> (ComponentHash, ComponentQueryAccessType) {
        (C::component_hash(), ComponentQueryAccessType::OptionRead)
    }

    fn data(idx: usize, data: &Option<*const u8>) -> Self {
        if let Some(data) = data {
            Some(unsafe { (*data as *const C).offset(idx as isize).as_ref().unwrap() })
        } else {
            None
        }
    }

    fn hash() -> ComponentHash {
        C::component_hash()
    }
}

impl<C> ComponentAccess for Option<&mut C>
where
    C: Component,
{
    fn access() -> (ComponentHash, ComponentQueryAccessType) {
        (C::component_hash(), ComponentQueryAccessType::OptionWrite)
    }

    fn data(idx: usize, data: &Option<*const u8>) -> Self {
        if let Some(data) = data {
            Some(unsafe { (*data as *mut C).offset(idx as isize).as_mut().unwrap() })
        } else {
            None
        }
    }

    fn hash() -> ComponentHash {
        C::component_hash()
    }
}

impl ComponentAccesses for () {
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

impl<C0> ComponentAccesses for C0
where
    C0: ComponentAccess,
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

impl<C0, C1> ComponentAccesses for (C0, C1)
where
    C0: ComponentAccess,
    C1: ComponentAccess,
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

impl<C> ComponentFilter for With<C>
where
    C: Component,
{
    fn filter() -> (ComponentHash, ComponentQueryFilterType) {
        (C::component_hash(), ComponentQueryFilterType::With)
    }
}

impl<C> ComponentFilter for Without<C>
where
    C: Component,
{
    fn filter() -> (ComponentHash, ComponentQueryFilterType) {
        (C::component_hash(), ComponentQueryFilterType::Without)
    }
}

impl ComponentFilters for () {
    fn filters() -> Vec<(ComponentHash, mewo_ecs::ComponentQueryFilterType)> {
        Vec::new()
    }
}

impl<CF0> ComponentFilters for CF0
where
    CF0: ComponentFilter,
{
    fn filters() -> Vec<(ComponentHash, mewo_ecs::ComponentQueryFilterType)> {
        vec![CF0::filter()]
    }
}

impl<CF0, CF1> ComponentFilters for (CF0, CF1)
where
    CF0: ComponentFilter,
    CF1: ComponentFilter,
{
    fn filters() -> Vec<(ComponentHash, mewo_ecs::ComponentQueryFilterType)> {
        vec![CF0::filter(), CF1::filter()]
    }
}
