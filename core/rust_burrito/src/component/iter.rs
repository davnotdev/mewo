use super::*;

pub struct Components<'sys, CA, CF> {
    ctymgr: &'sys SharedComponentTypeManager,
    amgr: &'sys ArchetypeManager,
    akid: ArchetypeAccessKey,
    phantom: PhantomData<(CA, CF)>,
}

impl<'sys, CA, CF> Components<'sys, CA, CF>
where
    CA: ComponentAccesses,
    CF: ComponentFilters,
{
    pub fn create(
        ctymgr: &'sys SharedComponentTypeManager,
        amgr: &'sys ArchetypeManager,
        akid: ArchetypeAccessKey,
    ) -> Self {
        Components {
            ctymgr,
            amgr,
            akid,
            phantom: PhantomData,
        }
    }

    pub fn iter(&self) -> ComponentsIter<CA> {
        ComponentsIter::create(self.amgr, self.ctymgr, self.akid)
    }
    pub fn eiter(&self) -> ComponentsEIter<CA> {
        ComponentsEIter::create(self.amgr, self.ctymgr, self.akid)
    }
}

#[derive(Debug)]
pub struct ComponentsIter<'sys, CA> {
    amgr: &'sys ArchetypeManager,
    ctymgr: &'sys SharedComponentTypeManager,
    akid: ArchetypeAccessKey,
    current_idx: usize,
    current_access: Option<ArchetypeAccess<'sys>>,
    current_datas: Vec<Option<*const u8>>,
    all_idx: usize,
    all_count: usize,
    phantom: PhantomData<CA>,
}

impl<'sys, CA> ComponentsIter<'sys, CA> {
    pub fn create(
        amgr: &'sys ArchetypeManager,
        ctymgr: &'sys SharedComponentTypeManager,
        akid: ArchetypeAccessKey,
    ) -> Self {
        ComponentsIter {
            amgr,
            ctymgr,
            akid,
            current_idx: 0,
            current_access: None,
            current_datas: Vec::new(),
            all_idx: 0,
            all_count: amgr.get_access_count(akid),
            phantom: PhantomData,
        }
    }
}

impl<'sys, CA> Iterator for ComponentsIter<'sys, CA>
where
    CA: ComponentAccesses,
{
    type Item = CA;
    fn next(&mut self) -> Option<Self::Item> {
        if self.all_count == 0 {
            None?
        }
        loop {
            if self.current_idx
                == self
                    .current_access
                    .as_ref()
                    .map(|v| v.get_iter_count())
                    .unwrap_or(self.current_idx)
            {
                if self.all_idx == self.all_count {
                    None?
                }
                loop {
                    if let Some(access) = self.amgr.try_access(self.akid, self.all_idx).unwrap() {
                        self.current_datas = CA::hashes()
                            .into_iter()
                            .map(|hash| {
                                let cty =
                                    self.ctymgr.read().unwrap().get_id_with_hash(hash).unwrap();
                                access.get_iter(cty).unwrap()
                            })
                            .collect();
                        self.current_access = Some(access);
                        break;
                    }
                    std::hint::spin_loop();
                }
                self.all_idx += 1;
                self.current_idx = 0;
            } else {
                break;
            }
        }

        self.current_idx += 1;
        Some(CA::datas(self.current_idx - 1, &self.current_datas))
    }
}

//  TODO Maybe we should reuse instead of copying...

pub struct ComponentsEIter<'sys, CA> {
    amgr: &'sys ArchetypeManager,
    ctymgr: &'sys SharedComponentTypeManager,
    akid: ArchetypeAccessKey,
    current_idx: usize,
    current_access: Option<ArchetypeAccess<'sys>>,
    current_datas: Vec<Option<*const u8>>,
    all_idx: usize,
    all_count: usize,
    phantom: PhantomData<CA>,
}

impl<'sys, CA> ComponentsEIter<'sys, CA> {
    pub fn create(
        amgr: &'sys ArchetypeManager,
        ctymgr: &'sys SharedComponentTypeManager,
        akid: ArchetypeAccessKey,
    ) -> Self {
        ComponentsEIter {
            amgr,
            ctymgr,
            akid,
            current_idx: 0,
            current_access: None,
            current_datas: Vec::new(),
            all_idx: 0,
            all_count: amgr.get_access_count(akid),
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
        if self.all_count == 0 {
            None?
        }
        loop {
            if self.current_idx
                == self
                    .current_access
                    .as_ref()
                    .map(|v| v.get_iter_count())
                    .unwrap_or(self.current_idx)
            {
                if self.all_idx == self.all_count {
                    None?
                }
                loop {
                    if let Some(access) = self.amgr.try_access(self.akid, self.all_idx).unwrap() {
                        self.current_datas = CA::hashes()
                            .into_iter()
                            .map(|hash| {
                                let cty =
                                    self.ctymgr.read().unwrap().get_id_with_hash(hash).unwrap();
                                access.get_iter(cty).unwrap()
                            })
                            .collect();
                        self.current_access = Some(access);
                        break;
                    }
                    std::hint::spin_loop();
                }
                self.all_idx += 1;
                self.current_idx = 0;
            } else {
                break;
            }
        }

        self.current_idx += 1;
        Some((
            self.current_access
                .as_ref()
                .unwrap()
                .get_iter_entity(self.current_idx - 1),
            CA::datas(self.current_idx - 1, &self.current_datas),
        ))
    }
}
