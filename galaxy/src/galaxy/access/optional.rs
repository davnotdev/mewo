use super::*;

pub trait ComponentAccessOptional {
    fn info() -> (ComponentTypeId, QueryAccessType);
    fn data(data: Option<*const u8>, idx: usize) -> Self;
    fn component_maybe_insert(ctyp: &RwLock<ComponentTypePlanet>);
}

pub trait ComponentAccessesOptional {
    fn infos() -> Vec<(ComponentTypeId, QueryAccessType)>;
    fn datas(datas: &[Option<*const u8>], idx: usize) -> Self;
    fn component_maybe_insert(ctyp: &RwLock<ComponentTypePlanet>);
}

impl<C> ComponentAccessOptional for &C
where
    C: GenericComponent + 'static,
{
    fn info() -> (ComponentTypeId, QueryAccessType) {
        (C::mewo_component_id(), QueryAccessType::Read)
    }

    fn data(data: Option<*const u8>, idx: usize) -> Self {
        unsafe { (data.unwrap() as *const C).add(idx).as_ref().unwrap() }
    }

    fn component_maybe_insert(ctyp: &RwLock<ComponentTypePlanet>) {
        component_maybe_insert::<C>(ctyp)
    }
}

impl<C> ComponentAccessOptional for &mut C
where
    C: GenericComponent + 'static,
{
    fn info() -> (ComponentTypeId, QueryAccessType) {
        (C::mewo_component_id(), QueryAccessType::Write)
    }

    fn data(data: Option<*const u8>, idx: usize) -> Self {
        unsafe {
            (data.unwrap() as *const C as *mut C)
                .add(idx)
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
    C: GenericComponent + 'static,
{
    fn info() -> (ComponentTypeId, QueryAccessType) {
        (C::mewo_component_id(), QueryAccessType::OptionRead)
    }

    fn data(data: Option<*const u8>, idx: usize) -> Self {
        unsafe { (data? as *const C).add(idx).as_ref() }
    }

    fn component_maybe_insert(ctyp: &RwLock<ComponentTypePlanet>) {
        component_maybe_insert::<C>(ctyp)
    }
}

impl<C> ComponentAccessOptional for Option<&mut C>
where
    C: GenericComponent + 'static,
{
    fn info() -> (ComponentTypeId, QueryAccessType) {
        (C::mewo_component_id(), QueryAccessType::OptionWrite)
    }

    fn data(data: Option<*const u8>, idx: usize) -> Self {
        unsafe { (data? as *const C as *mut C).add(idx).as_mut() }
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

    fn datas(datas: &[Option<*const u8>], idx: usize) -> Self {
        C0::data(datas[0], idx)
    }

    fn component_maybe_insert(ctyp: &RwLock<ComponentTypePlanet>) {
        C0::component_maybe_insert(ctyp);
    }
}

impl<C0, C1> ComponentAccessesOptional for (C0, C1)
where
    C0: ComponentAccessOptional,
    C1: ComponentAccessOptional,
{
    fn infos() -> Vec<(ComponentTypeId, QueryAccessType)> {
        vec![C0::info(), C1::info()]
    }

    fn datas(datas: &[Option<*const u8>], idx: usize) -> Self {
        (C0::data(datas[0], idx), C1::data(datas[1], idx))
    }

    fn component_maybe_insert(ctyp: &RwLock<ComponentTypePlanet>) {
        C0::component_maybe_insert(ctyp);
        C1::component_maybe_insert(ctyp);
    }
}

impl<C0, C1, C2> ComponentAccessesOptional for (C0, C1, C2)
where
    C0: ComponentAccessOptional,
    C1: ComponentAccessOptional,
    C2: ComponentAccessOptional,
{
    fn infos() -> Vec<(ComponentTypeId, QueryAccessType)> {
        vec![C0::info(), C1::info(), C2::info()]
    }

    fn datas(datas: &[Option<*const u8>], idx: usize) -> Self {
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
