use super::{
    ComponentAccessOptional, ComponentAccessesNormal, ComponentAccessesOptional, Entity, Galaxy,
    QueryAccess, QueryFilterType, QueryId, QueryLockType,
};
use std::marker::PhantomData;

//  TODO OPT: Don't write lock storages if they are len == 0.

pub struct QueryInfo<'gal, CA> {
    incomplete: QueryAccess,
    galaxy: &'gal Galaxy,
    phantom: PhantomData<CA>,
}

impl<'gal, CA> QueryInfo<'gal, CA>
where
    CA: ComponentAccessOptional,
{
    pub fn with<CF: ComponentAccessesNormal>(mut self) -> Self {
        CF::component_maybe_insert(&self.galaxy.ctyp);
        self.incomplete.filters.append(
            &mut CF::hashes()
                .into_iter()
                .map(|cty| (cty, QueryFilterType::With))
                .collect(),
        );
        self
    }

    pub fn without<CF: ComponentAccessesNormal>(mut self) -> Self {
        CF::component_maybe_insert(&self.galaxy.ctyp);
        self.incomplete.filters.append(
            &mut CF::hashes()
                .into_iter()
                .map(|cty| (cty, QueryFilterType::Without))
                .collect(),
        );
        self
    }

    pub fn iter(self) -> QueryIter<'gal, CA> {
        let Self {
            incomplete,
            galaxy,
            phantom,
        } = self;
        let _ = phantom;
        QueryIter {
            galaxy,
            qid: {
                let maybe_qid = galaxy.qp.read().get_query_id(&incomplete);
                if let Some(qid) = maybe_qid {
                    qid
                } else {
                    let mut qp = self.galaxy.qp.write();
                    qp.insert_access(
                        &self.galaxy.ctyp,
                        &self.galaxy.cgp,
                        &self.galaxy.sp,
                        incomplete,
                    )
                    .unwrap()
                }
            },
            current_storage: None,
            current_datas: None,
            current_entities: None,
            current_storage_len: None,
            group_idx: 0,
            storage_idx: 0,
            phantom: PhantomData,
        }
    }

    pub fn eiter(self) -> QueryEIter<'gal, CA> {
        QueryEIter { qiter: self.iter() }
    }
}

struct QueryStorageGuard<'gal> {
    qid: QueryId,
    group_idx: usize,
    galaxy: &'gal Galaxy,
}

impl<'gal> QueryStorageGuard<'gal> {
    pub fn new(qid: QueryId, group_idx: usize, galaxy: &'gal Galaxy) -> Self {
        let qp = galaxy.qp.read();
        let access = qp.get_access(qid).unwrap();
        let (gid, ord, locks) = &access.groups[group_idx];
        for cty in ord.get_components().iter() {
            let lock = locks.get(cty).unwrap();
            match lock {
                QueryLockType::Read => galaxy.sp.read().get_read_lock(*gid, *cty),
                QueryLockType::Write => galaxy.sp.read().get_write_lock(*gid, *cty),
            }
            .unwrap();
        }
        QueryStorageGuard {
            qid,
            group_idx,
            galaxy,
        }
    }
}

impl<'gal> Drop for QueryStorageGuard<'gal> {
    fn drop(&mut self) {
        let qp = self.galaxy.qp.read();
        let access = qp.get_access(self.qid).unwrap();
        //  Order doesn't matter does it?
        let (gid, _, locks) = &access.groups[self.group_idx];
        for (cty, lock) in locks {
            match lock {
                QueryLockType::Read => self.galaxy.sp.read().get_read_unlock(*gid, *cty),
                QueryLockType::Write => self.galaxy.sp.read().get_write_unlock(*gid, *cty),
            }
            .unwrap();
        }
    }
}

pub struct QueryIter<'gal, CA> {
    galaxy: &'gal Galaxy,
    qid: QueryId,
    current_storage: Option<QueryStorageGuard<'gal>>,
    current_datas: Option<Vec<Option<*const u8>>>,
    current_entities: Option<*const Entity>,
    current_storage_len: Option<usize>,
    group_idx: usize,
    storage_idx: usize,
    phantom: PhantomData<CA>,
}

impl<'gal, CA> QueryIter<'gal, CA> {
    //  Call after next.
    pub fn get_current_entity(&self) -> Entity {
        unsafe { *self.current_entities.unwrap().add(self.storage_idx - 1) }
    }
}

impl<'gal, CA> Iterator for QueryIter<'gal, CA>
where
    CA: ComponentAccessesOptional,
{
    type Item = CA;

    fn next(&mut self) -> Option<Self::Item> {
        let qp = self.galaxy.qp.read();
        let access = qp.get_access(self.qid).unwrap();

        if access.groups.len() == self.group_idx {
            None?
        }

        let (gid, ord, locks) = &access.groups[self.group_idx];

        if self.current_storage.is_none() {
            self.current_storage_len = Some(self.galaxy.sp.read().get_len(*gid));
            self.current_storage = Some(QueryStorageGuard::new(
                self.qid,
                self.group_idx,
                self.galaxy,
            ));
            let sp = self.galaxy.sp.read();
            self.current_datas = Some(
                ord.get_components()
                    .iter()
                    .map(|cid| {
                        let lock = locks.get(cid).unwrap();
                        match lock {
                            QueryLockType::Read => sp.get_read(*gid, *cid),
                            QueryLockType::Write => sp.get_write(*gid, *cid),
                        }
                    })
                    .collect(),
            );
            self.current_entities = Some(sp.get_entities(*gid).unwrap());
            self.storage_idx = 0;
        }

        if self.storage_idx == self.current_storage_len.unwrap() {
            self.group_idx += 1;
            self.current_storage = None;
            self.current_datas = None;
            self.current_storage_len = None;
            return self.next();
        }
        let ret = CA::datas(self.current_datas.as_ref().unwrap(), self.storage_idx);
        self.storage_idx += 1;
        Some(ret)
    }
}

pub struct QueryEIter<'gal, CA> {
    qiter: QueryIter<'gal, CA>,
}

impl<'gal, CA> Iterator for QueryEIter<'gal, CA>
where
    CA: ComponentAccessesOptional,
{
    type Item = (Entity, CA);

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.qiter.next()?;
        let entity = self.qiter.get_current_entity();
        Some((entity, next))
    }
}

impl Galaxy {
    pub fn query<CA: ComponentAccessOptional>(&self) -> QueryInfo<CA> {
        CA::component_maybe_insert(&self.ctyp);
        QueryInfo {
            incomplete: QueryAccess {
                accesses: CA::infos(),
                filters: Vec::new(),
            },
            galaxy: self,
            phantom: PhantomData,
        }
    }
}
