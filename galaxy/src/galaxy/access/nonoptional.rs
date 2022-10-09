use super::*;

pub trait ComponentAccessNonOptional {
    fn info() -> (ComponentTypeId, QueryAccessType);
    fn data(data: *const u8, idx: usize) -> Self;
    fn component_maybe_insert(ctyp: &RwLock<ComponentTypePlanet>);
}

pub trait ComponentAccessesNonOptional {
    fn infos() -> Vec<(ComponentTypeId, QueryAccessType)>;
    fn datas(datas: &Vec<*const u8>, idx: usize) -> Self;
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
        unsafe { (data as *const C).offset(idx as isize).as_ref().unwrap() }
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
        unsafe {
            (data as *const C as *mut C)
                .offset(idx as isize)
                .as_mut()
                .unwrap()
        }
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

    fn datas(datas: &Vec<*const u8>, idx: usize) -> Self {
        C0::data(datas[0], idx)
    }

    fn component_maybe_insert(ctyp: &RwLock<ComponentTypePlanet>) {
        C0::component_maybe_insert(ctyp);
    }
}
