use super::{*, storage::ArchetypeStorage, locker::LockState};

pub struct ArchetypeAccessKeyManager {
    keys: Vec<(ArchetypeAccessKeyEntry, ComponentGroupQuery)>,
}

impl ArchetypeAccessKeyManager {
    pub fn create() -> Self {
        ArchetypeAccessKeyManager { keys: Vec::new() }
    }

    //  This function depends on the fact that component
    //  groups are created ONLY when an entity transforms.
    pub fn register(&mut self, query: ComponentGroupQuery) -> ArchetypeAccessKey {
        let ak = ArchetypeAccessKeyEntry {
            accesses: Vec::new(),
        };
        self.keys.push((ak, query));
        self.keys.len() - 1
    }

    pub fn update(&mut self, gid: ComponentGroupId, group: &ComponentGroup) {
        for (ak, query) in self.keys.iter_mut() {
            if let Some(access) = Self::filter(group, query.get_accesses(), query.get_filters()) {
                ak.accesses.push((gid, access));
            }
        }
    }

    pub fn get(&self, akid: ArchetypeAccessKey) -> Option<&ArchetypeAccessKeyEntry> {
        self.keys.get(akid).map(|(entry, _)| entry)
    }
}

impl ArchetypeAccessKeyManager {
    fn query_type_to_lock_state(ctq: ComponentQueryAccessType) -> LockState {
        match ctq {
            ComponentQueryAccessType::Read | ComponentQueryAccessType::OptionRead => {
                LockState::Shared
            }
            ComponentQueryAccessType::Write | ComponentQueryAccessType::OptionWrite => {
                LockState::Exclusive
            }
        }
    }

    //  Cases:
    //  1. &C,          4. &mut C
    //  2. Option<&C>   5. Option<&mut C>
    //  3. With<C>      6. Without<C>
    fn filter(
        group: &ComponentGroup,
        accesses: &Vec<(ComponentTypeId, ComponentQueryAccessType)>,
        filters: &Vec<(ComponentTypeId, ComponentQueryFilterType)>,
    ) -> Option<SparseSet<ComponentTypeId, LockState>> {
        let mut lock_map = SparseSet::create();
        for &(cty, ctf) in filters.iter() {
            match ctf {
                ComponentQueryFilterType::With => {
                    if !group.has(cty) {
                        return None;
                    }
                }
                ComponentQueryFilterType::Without => {
                    if group.has(cty) {
                        return None;
                    }
                }
            }
        }
        for &(cty, ctq) in accesses.iter() {
            match ctq {
                ComponentQueryAccessType::Read | ComponentQueryAccessType::Write => {
                    if !group.has(cty) {
                        return None;
                    }
                }
                ComponentQueryAccessType::OptionRead | ComponentQueryAccessType::OptionWrite => {
                    if !group.has(cty) {
                        continue;
                    }
                }
            }
            lock_map.insert(cty, Self::query_type_to_lock_state(ctq));
        }

        Some(lock_map)
    }
}

pub struct ArchetypeAccessKeyEntry {
    accesses: Vec<(ComponentGroupId, SparseSet<ComponentTypeId, LockState>)>,
}

impl ArchetypeAccessKeyEntry {
    pub fn get_count(&self) -> usize {
        self.accesses.len()
    }
}

pub struct ArchetypeAccess<'amgr> {
    idx: usize,
    lock_count: &'amgr AtomicU32,
    storage: &'amgr ArchetypeStorage,
    archetype_access_key: &'amgr ArchetypeAccessKeyEntry,
}

impl<'amgr> ArchetypeAccess<'amgr> {
    pub fn get_iter(&self, ctyid: ComponentTypeId) -> Result<*const u8> {
        self.storage.get_iter(ctyid)
    }

    pub fn get_iter_count(&self) -> usize {
        self.storage.get_iter_count()
    }

    pub fn get_iter_entity(&self, idx: usize) -> Entity {
        self.storage.get_iter_entity(idx)
    }
}

impl<'amgr> Drop for ArchetypeAccess<'amgr> {
    fn drop(&mut self) {
        let (_gid, lock_map) = self.archetype_access_key.accesses.get(self.idx).unwrap();
        for &(cty, lock_state) in lock_map.get_dense() {
            self.storage.unlock_component(cty, lock_state).unwrap();
            self.lock_count.fetch_sub(1, Ordering::Relaxed);
        }
    }
}

impl ArchetypeManager {
    pub fn create_archetype_access_key(
        &mut self,
        q: ComponentGroupQuery,
    ) -> Result<ArchetypeAccessKey> {
        self.yeet_locked()?;
        Ok(self.akmgr.register(q))
    }

    pub fn try_access(
        &self,
        akid: ArchetypeAccessKey,
        idx: usize,
    ) -> Result<Option<ArchetypeAccess<'_>>> {
        if let Some(access) = self.akmgr.get(akid) {
            if let Some((gid, lock_map)) = access.accesses.get(idx) {
                let gid = *gid;
                if let Some(storage) = self.storages.get(gid) {
                    let group = self.cgmgr.get(gid).unwrap();
                    for &cty in group.get() {
                        if let Some(&lock_state) = lock_map.get(cty) {
                            while !storage.try_lock_component(cty, lock_state)? {
                                std::hint::spin_loop()
                            }
                            self.lock_count.fetch_add(1, Ordering::SeqCst);
                        }
                    }
                    let access = ArchetypeAccess {
                        idx,
                        storage,
                        lock_count: &self.lock_count,
                        archetype_access_key: access,
                    };
                    Ok(Some(access))
                } else {
                    Err(RuntimeError::BadComponentGroup { gid })
                }
            } else {
                Err(RuntimeError::BadArchetypeManagerAccessIndex {
                    idx,
                    max: access.get_count(),
                })
            }
        } else {
            Err(RuntimeError::BadArchetypeAccessKey { akid })
        }
    }

    pub fn get_access_count(&self, akid: ArchetypeAccessKey) -> usize {
        self.akmgr.get(akid).unwrap().get_count()
    }
}
