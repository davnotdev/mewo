use super::*;
use mewo_ecs::Entity;

pub struct ComponentsIter<CA> {
    idx: usize,
    len: usize,
    datas: Vec<Option<*const u8>>,
    phantom: PhantomData<CA>,
}

impl<CA> ComponentsIter<CA> {
    pub fn create(len: usize, datas: Vec<Option<*const u8>>) -> Self {
        ComponentsIter {
            idx: 0,
            len,
            datas,
            phantom: PhantomData,
        }
    }
}

impl<CA> Iterator for ComponentsIter<CA>
where
    CA: ComponentAccesses,
{
    type Item = CA;
    fn next(&mut self) -> Option<Self::Item> {
        if self.idx == self.len {
            None?
        }
        self.idx += 1;

        let data = CA::datas(self.idx - 1, &self.datas);
        let ret = Some(data);
        ret
    }
}

pub struct ComponentsEIter<'sys, CA> {
    idx: usize,
    len: usize,
    datas: Vec<Option<*const u8>>,
    access: &'sys Option<ArchetypeAccess<'sys>>,
    phantom: PhantomData<CA>,
}

impl<'sys, CA> ComponentsEIter<'sys, CA> {
    pub fn create(
        len: usize,
        datas: Vec<Option<*const u8>>,
        access: &'sys Option<ArchetypeAccess<'sys>>,
    ) -> Self {
        ComponentsEIter {
            idx: 0,
            len,
            datas,
            access,
            phantom: PhantomData,
        }
    }
}

impl<'sys, CA> Iterator for ComponentsEIter<'sys, CA>
where
    CA: ComponentAccesses,
{
    type Item = (Entity, CA);
    fn next(&mut self) -> Option<Self::Item> {
        if self.idx == self.len {
            None?
        }
        self.idx += 1;
        Some((
            self.access.as_ref().unwrap().get_iter_entity(self.idx - 1),
            CA::datas(self.idx - 1, &self.datas),
        ))
    }
}
