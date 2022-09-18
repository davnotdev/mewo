use super::*;

pub trait ComponentAccessOptional {
    fn info() -> (ComponentTypeId, QueryAccessType);
    fn data(data: Option<*const u8>, idx: usize) -> Self;
    fn component_maybe_insert(ctyp: &RwLock<ComponentTypePlanet>);
}

pub trait ComponentAccessesOptional {
    fn infos() -> Vec<(ComponentTypeId, QueryAccessType)>;
    fn datas(datas: &Vec<Option<*const u8>>, idx: usize) -> Self;
    fn component_maybe_insert(ctyp: &RwLock<ComponentTypePlanet>);
}

impl<C> ComponentAccessOptional for &C
where
    C: Component + 'static,
{
    fn info() -> (ComponentTypeId, QueryAccessType) {
        (C::mewo_component_id(), QueryAccessType::Read)
    }

    fn data(data: Option<*const u8>, idx: usize) -> Self {
        unsafe {
            (data.unwrap() as *const C)
                .offset(idx as isize)
                .as_ref()
                .unwrap()
        }
    }

    fn component_maybe_insert(ctyp: &RwLock<ComponentTypePlanet>) {
        component_maybe_insert::<C>(ctyp)
    }
}

impl<C> ComponentAccessOptional for &mut C
where
    C: Component + 'static,
{
    fn info() -> (ComponentTypeId, QueryAccessType) {
        (C::mewo_component_id(), QueryAccessType::Write)
    }

    fn data(data: Option<*const u8>, idx: usize) -> Self {
        unsafe {
            (data.unwrap() as *const C as *mut C)
                .offset(idx as isize)
                .as_mut()
                .unwrap()
        }
    }

    fn component_maybe_insert(ctyp: &RwLock<ComponentTypePlanet>) {
        component_maybe_insert::<C>(ctyp)
    }
}

impl<C> ComponentAccessOptional for Option<&C>
where
    C: Component + 'static,
{
    fn info() -> (ComponentTypeId, QueryAccessType) {
        (C::mewo_component_id(), QueryAccessType::OptionRead)
    }

    fn data(data: Option<*const u8>, idx: usize) -> Self {
        unsafe { (data? as *const C).offset(idx as isize).as_ref() }
    }

    fn component_maybe_insert(ctyp: &RwLock<ComponentTypePlanet>) {
        component_maybe_insert::<C>(ctyp)
    }
}

impl<C> ComponentAccessOptional for Option<&mut C>
where
    C: Component + 'static,
{
    fn info() -> (ComponentTypeId, QueryAccessType) {
        (C::mewo_component_id(), QueryAccessType::OptionWrite)
    }

    fn data(data: Option<*const u8>, idx: usize) -> Self {
        unsafe { (data? as *const C as *mut C).offset(idx as isize).as_mut() }
    }

    fn component_maybe_insert(ctyp: &RwLock<ComponentTypePlanet>) {
        component_maybe_insert::<C>(ctyp)
    }
}

//
//  ---
//

impl<C0> ComponentAccessesOptional for C0
where
    C0: ComponentAccessOptional,
{
    fn infos() -> Vec<(ComponentTypeId, QueryAccessType)> {
        vec![C0::info()]
    }

    fn datas(datas: &Vec<Option<*const u8>>, idx: usize) -> Self {
        C0::data(datas[0], idx)
    }

    fn component_maybe_insert(ctyp: &RwLock<ComponentTypePlanet>) {
        C0::component_maybe_insert(ctyp);
    }
}
