use super::{
    ComponentAccessesNormal, ComponentAccessesOptional, Executor, Galaxy, QueryAccess,
    QueryFilterType, QueryId, QueryLockType,
};
use std::marker::PhantomData;

pub struct QueryInfo<'gal, EX, CA> {
    incomplete: QueryAccess,
    galaxy: &'gal Galaxy<EX>,
    phantom: PhantomData<CA>,
}

impl<'gal, EX, CA> QueryInfo<'gal, EX, CA>
where
    CA: ComponentAccessesOptional,
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

    pub fn iter(self) -> QueryIter<'gal, EX, CA> {
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
            group_idx: 0,
            storage_idx: 0,
            phantom: PhantomData,
        }
    }
}

struct QueryStorageGuard<'gal, EX> {
    qid: QueryId,
    group_idx: usize,
    galaxy: &'gal Galaxy<EX>,
}

impl<'gal, EX> QueryStorageGuard<'gal, EX> {
    pub fn new(qid: QueryId, group_idx: usize, galaxy: &'gal Galaxy<EX>) -> Self {
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

impl<'gal, EX> Drop for QueryStorageGuard<'gal, EX> {
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

pub struct QueryIter<'gal, EX, CA> {
    galaxy: &'gal Galaxy<EX>,
    qid: QueryId,
    current_storage: Option<QueryStorageGuard<'gal, EX>>,
    current_datas: Option<Vec<Option<*const u8>>>,
    group_idx: usize,
    storage_idx: usize,
    phantom: PhantomData<CA>,
}

impl<'gal, EX, CA> Iterator for QueryIter<'gal, EX, CA>
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

        if let None = self.current_storage {
            self.current_storage = Some(QueryStorageGuard::new(
                self.qid,
                self.group_idx,
                &self.galaxy,
            ));
            self.current_datas = Some(
                ord.get_components()
                    .iter()
                    .map(|cid| {
                        let lock = locks.get(cid).unwrap();
                        match lock {
                            QueryLockType::Read => self.galaxy.sp.read().get_read(*gid, *cid),
                            QueryLockType::Write => self.galaxy.sp.read().get_write(*gid, *cid),
                        }
                    })
                    .collect(),
            );
            self.storage_idx = 0;
        }

        if self.storage_idx == self.galaxy.sp.read().get_len(*gid) {
            self.group_idx += 1;
            self.current_storage = None;
            self.current_datas = None;
            return self.next();
        }
        let ret = CA::datas(&self.current_datas.as_ref().unwrap(), self.storage_idx);
        self.storage_idx += 1;
        Some(ret)
    }
}

impl<EX> Galaxy<EX>
where
    EX: Executor,
{
    pub fn query<CA: ComponentAccessesOptional>(&self) -> QueryInfo<EX, CA> {
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
