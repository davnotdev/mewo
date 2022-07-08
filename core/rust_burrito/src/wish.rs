use super::component::Component;
use mewo_ecs::{
    ArchetypeAccess, ComponentHash, ComponentQueryAccessType, ComponentQueryFilterType,
    ComponentTypeManager, Entity, EventHash, EventOption,
};
use std::marker::PhantomData;

/*

query: Wish<
Event,
(
    &C, &mut C,
    Option<&C>, Option<&mut C>,
), (
    With<C>, Without<C>
)>

*/

pub struct Wish<'access, 'ctymgr, 'amgr, WE: WishEvent, WA: WishAccesses, WF: WishFilters> {
    phantom: PhantomData<(WE, WA, WF)>,
    ev: Option<*const u8>,
    access: Option<&'access ArchetypeAccess<'amgr>>,
    ctymgr: &'ctymgr ComponentTypeManager,
}

impl<'access, 'ctymgr, 'amgr, WE, WA, WF> Wish<'access, 'ctymgr, 'amgr, WE, WA, WF>
where
    WE: WishEvent,
    WA: WishAccesses,
    WF: WishFilters,
{
    pub fn create(
        ev: Option<*const u8>,
        ctymgr: &'ctymgr ComponentTypeManager,
        access: Option<&'access ArchetypeAccess<'amgr>>,
    ) -> Self {
        Wish {
            ev,
            ctymgr,
            access,
            phantom: PhantomData,
        }
    }

    pub fn is_empty() -> bool {
        WA::accesses().len() == 0
    }

    pub fn event(&self) -> &WE {
        unsafe { (self.ev.unwrap() as *const WE).as_ref().unwrap() }
    }

    pub fn iter(&self) -> WishIter<WA> {
        assert!(!Self::is_empty());
        let access = self.access.unwrap();
        WishIter {
            idx: 0,
            len: access.get_iter_count(),
            datas: WA::hashes()
                .into_iter()
                .map(|hash| {
                    let cty = self.ctymgr.get_id_with_hash(hash).unwrap();
                    access.get_iter(cty).map(|data| Some(data)).unwrap_or(None)
                })
                .collect(),
            phantom: PhantomData,
        }
    }

    pub fn eiter(&self) -> WishEIter<WA> {
        assert!(!Self::is_empty());
        let access = self.access.unwrap();
        WishEIter {
            idx: 0,
            len: access.get_iter_count(),
            datas: WA::hashes()
                .into_iter()
                .map(|hash| {
                    let cty = self.ctymgr.get_id_with_hash(hash).unwrap();
                    access.get_iter(cty).map(|data| Some(data)).unwrap_or(None)
                })
                .collect(),
            access,
            phantom: PhantomData,
        }
    }
}

pub struct WishIter<WA> {
    idx: usize,
    len: usize,
    datas: Vec<Option<*const u8>>,
    phantom: PhantomData<WA>,
}

impl<WA> Iterator for WishIter<WA>
where
    WA: WishAccesses,
{
    type Item = WA;
    fn next(&mut self) -> Option<Self::Item> {
        if self.idx == self.len {
            None?
        }
        self.idx += 1;

        let data = WA::datas(self.idx - 1, &self.datas);
        let ret = Some(data);
        ret
    }
}

pub struct WishEIter<'access, 'amgr, WA> {
    idx: usize,
    len: usize,
    datas: Vec<Option<*const u8>>,
    access: &'access ArchetypeAccess<'amgr>,
    phantom: PhantomData<WA>,
}

impl<'access, 'amgr, WA> Iterator for WishEIter<'access, 'amgr, WA>
where
    WA: WishAccesses,
{
    type Item = (Entity, WA);
    fn next(&mut self) -> Option<Self::Item> {
        if self.idx == self.len {
            None?
        }
        self.idx += 1;
        Some((
            self.access.get_iter_entity(self.idx - 1),
            WA::datas(self.idx - 1, &self.datas),
        ))
    }
}

pub trait WishEvent {
    fn hash() -> EventOption<EventHash>;
}

pub struct Startup;

pub trait WishAccesses {
    fn accesses() -> Vec<(ComponentHash, ComponentQueryAccessType)>;
    fn hashes() -> Vec<ComponentHash>;
    fn datas(idx: usize, datas: &Vec<Option<*const u8>>) -> Self;
}

pub trait WishFilters {
    fn filters() -> Vec<(ComponentHash, ComponentQueryFilterType)>;
}

pub trait WishAccess {
    fn access() -> (ComponentHash, ComponentQueryAccessType);
    fn data(idx: usize, data: &Option<*const u8>) -> Self;
    fn hash() -> ComponentHash;
}

pub struct With<C: Component>(PhantomData<C>);
pub struct Without<C: Component>(PhantomData<C>);

pub trait WishFilter {
    fn filter() -> (ComponentHash, ComponentQueryFilterType);
}

impl<C> WishFilter for With<C>
where
    C: Component,
{
    fn filter() -> (ComponentHash, ComponentQueryFilterType) {
        (C::component_hash(), ComponentQueryFilterType::With)
    }
}

impl<C> WishFilter for Without<C>
where
    C: Component,
{
    fn filter() -> (ComponentHash, ComponentQueryFilterType) {
        (C::component_hash(), ComponentQueryFilterType::Without)
    }
}
