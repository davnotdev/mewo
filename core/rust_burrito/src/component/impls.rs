use super::*;

impl<C> ComponentAccess for &C
where
    C: Component,
{
    fn access() -> (ComponentHash, ComponentQueryAccessType) {
        (C::component_trait_hash(), ComponentQueryAccessType::Read)
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
        C::component_trait_hash()
    }

    fn maybe_register(ctymgr: &mut ComponentTypeManager) {
        let _ = ctymgr.register(C::component_trait_entry());
    }
}

impl<C> ComponentAccess for &mut C
where
    C: Component,
{
    fn access() -> (ComponentHash, ComponentQueryAccessType) {
        (C::component_trait_hash(), ComponentQueryAccessType::Write)
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
        C::component_trait_hash()
    }

    fn maybe_register(ctymgr: &mut ComponentTypeManager) {
        let _ = ctymgr.register(C::component_trait_entry());
    }
}

impl<C> ComponentAccess for Option<&C>
where
    C: Component,
{
    fn access() -> (ComponentHash, ComponentQueryAccessType) {
        (
            C::component_trait_hash(),
            ComponentQueryAccessType::OptionRead,
        )
    }

    fn data(idx: usize, data: &Option<*const u8>) -> Self {
        if let Some(data) = data {
            Some(unsafe { (*data as *const C).offset(idx as isize).as_ref().unwrap() })
        } else {
            None
        }
    }

    fn hash() -> ComponentHash {
        C::component_trait_hash()
    }

    fn maybe_register(ctymgr: &mut ComponentTypeManager) {
        let _ = ctymgr.register(C::component_trait_entry());
    }
}

impl<C> ComponentAccess for Option<&mut C>
where
    C: Component,
{
    fn access() -> (ComponentHash, ComponentQueryAccessType) {
        (
            C::component_trait_hash(),
            ComponentQueryAccessType::OptionWrite,
        )
    }

    fn data(idx: usize, data: &Option<*const u8>) -> Self {
        if let Some(data) = data {
            Some(unsafe { (*data as *mut C).offset(idx as isize).as_mut().unwrap() })
        } else {
            None
        }
    }

    fn hash() -> ComponentHash {
        C::component_trait_hash()
    }

    fn maybe_register(ctymgr: &mut ComponentTypeManager) {
        let _ = ctymgr.register(C::component_trait_entry());
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

    fn maybe_register(_ctymgr: &mut ComponentTypeManager) {}
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

    fn maybe_register(ctymgr: &mut ComponentTypeManager) {
        C0::maybe_register(ctymgr);
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

    fn maybe_register(ctymgr: &mut ComponentTypeManager) {
        C0::maybe_register(ctymgr);
        C1::maybe_register(ctymgr);
    }
}

impl<C0, C1, C2> ComponentAccesses for (C0, C1, C2)
where
    C0: ComponentAccess,
    C1: ComponentAccess,
    C2: ComponentAccess,
{
    fn accesses() -> Vec<(ComponentHash, ComponentQueryAccessType)> {
        vec![C0::access(), C1::access(), C2::access()]
    }

    fn hashes() -> Vec<ComponentHash> {
        vec![C0::hash(), C1::hash(), C2::hash()]
    }

    fn datas(idx: usize, datas: &Vec<Option<*const u8>>) -> Self {
        (
            C0::data(idx, datas.get(0).unwrap()),
            C1::data(idx, datas.get(1).unwrap()),
            C2::data(idx, datas.get(2).unwrap()),
        )
    }

    fn maybe_register(ctymgr: &mut ComponentTypeManager) {
        C0::maybe_register(ctymgr);
        C1::maybe_register(ctymgr);
        C2::maybe_register(ctymgr);
    }
}

impl<C> ComponentFilter for With<C>
where
    C: Component,
{
    fn filter() -> (ComponentHash, ComponentQueryFilterType) {
        (C::component_trait_hash(), ComponentQueryFilterType::With)
    }

    fn maybe_register(ctymgr: &mut ComponentTypeManager) {
        let _ = ctymgr.register(C::component_trait_entry());
    }
}

impl<C> ComponentFilter for Without<C>
where
    C: Component,
{
    fn filter() -> (ComponentHash, ComponentQueryFilterType) {
        (C::component_trait_hash(), ComponentQueryFilterType::Without)
    }

    fn maybe_register(ctymgr: &mut ComponentTypeManager) {
        let _ = ctymgr.register(C::component_trait_entry());
    }
}

impl ComponentFilters for () {
    fn filters() -> Vec<(ComponentHash, mewo_ecs::ComponentQueryFilterType)> {
        Vec::new()
    }

    fn maybe_register(_ctymgr: &mut ComponentTypeManager) {}
}

impl<CF0> ComponentFilters for CF0
where
    CF0: ComponentFilter,
{
    fn filters() -> Vec<(ComponentHash, mewo_ecs::ComponentQueryFilterType)> {
        vec![CF0::filter()]
    }

    fn maybe_register(ctymgr: &mut ComponentTypeManager) {
        CF0::maybe_register(ctymgr);
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

    fn maybe_register(ctymgr: &mut ComponentTypeManager) {
        CF0::maybe_register(ctymgr);
        CF1::maybe_register(ctymgr);
    }
}
