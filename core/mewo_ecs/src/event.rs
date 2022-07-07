use crate::{
    data::{DVec, TVal, TValCloneFunction, TValDropFunction},
    error::*,
    HashType,
};
use std::collections::HashMap;
use std::hash::Hash;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum EventOption<T>
where
    T: Clone + Copy + Eq + PartialEq + Hash,
{
    Startup,
    Update,
    Event(T),
}

pub type EventHash = HashType;

pub struct EventTypeEntry {
    pub size: usize,
    pub name: String,
    pub hash: EventHash,
    pub drop: TValDropFunction,
    pub clone: TValCloneFunction,
}

pub struct EventManager {
    hash_map: HashMap<EventHash, (EventTypeEntry, EventStorage)>,
}

impl EventManager {
    pub fn create() -> Self {
        EventManager {
            hash_map: HashMap::new(),
        }
    }

    pub fn register(&mut self, entry: EventTypeEntry) -> Result<()> {
        if self.hash_map.contains_key(&entry.hash) {
            Err(RuntimeError::DuplicateEventTypeHash { hash: entry.hash })?
        }
        let storage = EventStorage::create(entry.size, entry.drop, entry.clone);
        self.hash_map.insert(entry.hash, (entry, storage));
        Ok(())
    }

    pub fn get(&self, hash: EventHash) -> Result<&EventTypeEntry> {
        Ok(&self
            .hash_map
            .get(&hash)
            .ok_or(RuntimeError::BadEventTypeHash { hash })?
            .0)
    }

    pub fn get_event(&self, hash: EventHash, idx: usize) -> Result<*const u8> {
        self.hash_map
            .get(&hash)
            .ok_or(RuntimeError::BadEventTypeHash { hash })?
            .1
            .get(idx)
    }

    pub fn get_event_count(&self, hash: EventHash) -> Result<usize> {
        Ok(self
            .hash_map
            .get(&hash)
            .ok_or(RuntimeError::BadEventTypeHash { hash })?
            .1
            .len())
    }

    pub fn flush(&mut self, inserts: &mut EventInsert) -> Result<()> {
        for (hash, insert) in inserts.get() {
            let (_entry, storage) = self
                .hash_map
                .get_mut(hash)
                .ok_or(RuntimeError::BadEventTypeHash { hash: *hash })?;
            storage.push(insert.get());
        }
        inserts.flush();
        Ok(())
    }
}

//  Just as in archetype/storage, clone is never used.
//  `--,
//     v
#[allow(dead_code)]
struct EventStorage {
    drop: TValDropFunction,
    clone: TValCloneFunction,
    datas: DVec,
}

impl EventStorage {
    pub fn create(size: usize, drop: TValDropFunction, clone: TValCloneFunction) -> Self {
        EventStorage {
            drop,
            clone,
            datas: DVec::create(size),
        }
    }

    pub fn push(&mut self, data: *const u8) {
        self.datas.resize(1, data);
    }

    pub fn len(&self) -> usize {
        self.datas.len()
    }

    pub fn get(&self, idx: usize) -> Result<*const u8> {
        self.datas
            .get(idx)
            .ok_or(RuntimeError::BadEventStorageGetIndex { idx })
    }
}

pub struct EventInsert {
    events: Vec<(EventHash, TVal)>,
}

impl EventInsert {
    pub fn create() -> EventInsert {
        EventInsert { events: Vec::new() }
    }

    pub fn insert(&mut self, ev_hash: EventHash, val: TVal) {
        self.events.push((ev_hash, val));
    }

    pub(self) fn get(&self) -> &Vec<(EventHash, TVal)> {
        &self.events
    }

    fn flush(&mut self) {
        self.events.clear();
    }
}
