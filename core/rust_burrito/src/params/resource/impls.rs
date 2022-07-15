use super::*;

impl<R> ResourceAccess for &R
where
    R: Resource,
{
    fn data(data: *const u8) -> Self {
        unsafe { &*(data as *const R) }
    }

    fn hash() -> ResourceHash {
        R::resource_hash()
    }

    fn access() -> ResourceQueryAccessType {
        ResourceQueryAccessType::Read
    }
}

impl<R> ResourceAccess for &mut R
where
    R: Resource,
{
    fn data(data: *const u8) -> Self {
        unsafe { &mut *(data as *const R as *mut R) }
    }

    fn hash() -> ResourceHash {
        R::resource_hash()
    }

    fn access() -> ResourceQueryAccessType {
        ResourceQueryAccessType::Write
    }
}

fn lock<RS>(rcmgr: &ResourceManager)
where
    RS: Resources,
{
    for (_, access) in RS::accesses() {
        match access {
            ResourceQueryAccessType::Read => rcmgr.lock(access),
            ResourceQueryAccessType::Write => rcmgr.lock(access),
        }
    }
}

fn unlock<RS>(rcmgr: &ResourceManager)
where
    RS: Resources,
{
    for (_, access) in RS::accesses() {
        match access {
            ResourceQueryAccessType::Read => rcmgr.unlock(access),
            ResourceQueryAccessType::Write => rcmgr.unlock(access),
        }
    }
}

fn val<R>(
    a: &Vec<(ResourceHash, ResourceQueryAccessType)>,
    rcmgr: &ResourceManager,
    idx: usize,
) -> Option<R>
where
    R: ResourceAccess,
{
    if let Some(val) = rcmgr.locked_get(a[idx].0).unwrap() {
        Some(R::data(val.get()))
    } else {
        None
    }
}

impl<R0> Resources for R0
where
    R0: ResourceAccess,
{
    fn accesses() -> Vec<(ResourceHash, ResourceQueryAccessType)> {
        vec![(R0::hash(), R0::access())]
    }
    fn lock(rcmgr: &ResourceManager) {
        lock::<Self>(rcmgr)
    }

    fn unlock(rcmgr: &ResourceManager) {
        unlock::<Self>(rcmgr)
    }

    fn get(rcmgr: &ResourceManager) -> Option<Self>
    where
        Self: Sized,
    {
        let a = Self::accesses();
        Some(val::<R0>(&a, rcmgr, 0)?)
    }
}

impl<R0, R1> Resources for (R0, R1)
where
    R0: ResourceAccess,
    R1: ResourceAccess,
{
    fn accesses() -> Vec<(ResourceHash, ResourceQueryAccessType)> {
        vec![(R0::hash(), R0::access()), (R1::hash(), R1::access())]
    }
    fn lock(rcmgr: &ResourceManager) {
        lock::<Self>(rcmgr)
    }

    fn unlock(rcmgr: &ResourceManager) {
        unlock::<Self>(rcmgr)
    }

    fn get(rcmgr: &ResourceManager) -> Option<Self>
    where
        Self: Sized,
    {
        let a = Self::accesses();
        Some((val::<R0>(&a, rcmgr, 0)?, val::<R1>(&a, rcmgr, 1)?))
    }
}

impl<R0, R1, R2> Resources for (R0, R1, R2)
where
    R0: ResourceAccess,
    R1: ResourceAccess,
    R2: ResourceAccess,
{
    fn accesses() -> Vec<(ResourceHash, ResourceQueryAccessType)> {
        vec![
            (R0::hash(), R0::access()),
            (R1::hash(), R1::access()),
            (R2::hash(), R2::access()),
        ]
    }
    fn lock(rcmgr: &ResourceManager) {
        lock::<Self>(rcmgr)
    }

    fn unlock(rcmgr: &ResourceManager) {
        unlock::<Self>(rcmgr)
    }

    fn get(rcmgr: &ResourceManager) -> Option<Self>
    where
        Self: Sized,
    {
        let a = Self::accesses();
        Some((
            val::<R0>(&a, rcmgr, 0)?,
            val::<R1>(&a, rcmgr, 1)?,
            val::<R2>(&a, rcmgr, 2)?,
        ))
    }
}
