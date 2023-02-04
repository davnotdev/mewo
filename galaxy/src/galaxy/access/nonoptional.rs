use super::*;

pub trait ComponentAccessNonOptional {
    fn info() -> (ComponentTypeId, QueryAccessType);
    fn data(data: *const u8, idx: usize) -> Self;
    fn component_maybe_insert(ctyp: &RwLock<ComponentTypePlanet>);
}

pub trait ComponentAccessesNonOptional {
    fn infos() -> Vec<(ComponentTypeId, QueryAccessType)>;
    fn datas(datas: &[*const u8], idx: usize) -> Self;
    fn component_maybe_insert(ctyp: &RwLock<ComponentTypePlanet>);
}

impl<C> ComponentAccessNonOptional for &C
where
    C: GenericComponent + 'static,
{
    fn info() -> (ComponentTypeId, QueryAccessType) {
        (C::mewo_component_id(), QueryAccessType::Read)
    }

    fn data(data: *const u8, idx: usize) -> Self {
        unsafe { (data as *const C).add(idx).as_ref().unwrap() }
    }

    fn component_maybe_insert(ctyp: &RwLock<ComponentTypePlanet>) {
        component_maybe_insert::<C>(ctyp)
    }
}

impl<C> ComponentAccessNonOptional for &mut C
where
    C: GenericComponent + 'static,
{
    fn info() -> (ComponentTypeId, QueryAccessType) {
        (C::mewo_component_id(), QueryAccessType::Write)
    }

    fn data(data: *const u8, idx: usize) -> Self {
        unsafe { (data as *const C as *mut C).add(idx).as_mut().unwrap() }
    }

    fn component_maybe_insert(ctyp: &RwLock<ComponentTypePlanet>) {
        component_maybe_insert::<C>(ctyp)
    }
}

impl<C0> ComponentAccessesNonOptional for C0
where
    C0: ComponentAccessNonOptional,
{
    fn infos() -> Vec<(ComponentTypeId, QueryAccessType)> {
        vec![C0::info()]
    }

    fn datas(datas: &[*const u8], idx: usize) -> Self {
        C0::data(datas[0], idx)
    }

    fn component_maybe_insert(ctyp: &RwLock<ComponentTypePlanet>) {
        C0::component_maybe_insert(ctyp);
    }
}

impl<C0, C1> ComponentAccessesNonOptional for (C0, C1)
where
    C0: ComponentAccessNonOptional,
    C1: ComponentAccessNonOptional,
{
    fn infos() -> Vec<(ComponentTypeId, QueryAccessType)> {
        vec![C0::info(), C1::info()]
    }

    fn datas(datas: &[*const u8], idx: usize) -> Self {
        (C0::data(datas[0], idx), C1::data(datas[1], idx))
    }

    fn component_maybe_insert(ctyp: &RwLock<ComponentTypePlanet>) {
        C0::component_maybe_insert(ctyp);
        C1::component_maybe_insert(ctyp);
    }
}

impl<C0, C1, C2> ComponentAccessesNonOptional for (C0, C1, C2)
where
    C0: ComponentAccessNonOptional,
    C1: ComponentAccessNonOptional,
    C2: ComponentAccessNonOptional,
{
    fn infos() -> Vec<(ComponentTypeId, QueryAccessType)> {
        vec![C0::info(), C1::info(), C2::info()]
    }

    fn datas(datas: &[*const u8], idx: usize) -> Self {
        (
            C0::data(datas[0], idx),
            C1::data(datas[1], idx),
            C2::data(datas[2], idx),
        )
    }

    fn component_maybe_insert(ctyp: &RwLock<ComponentTypePlanet>) {
        C0::component_maybe_insert(ctyp);
        C1::component_maybe_insert(ctyp);
        C2::component_maybe_insert(ctyp);
    }
}
