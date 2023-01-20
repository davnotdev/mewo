use super::error::*;
use crate::data::{DVec, TVal, TypeEntry};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct EventId(pub u64);

impl EventId {
    pub fn from_hash(val: u64) -> Self {
        EventId(val)
    }
}

#[derive(Debug)]
pub struct EventPlanet {
    events: HashMap<EventId, (TypeEntry, DVec)>,
}

impl EventPlanet {
    pub fn new() -> Self {
        EventPlanet {
            events: HashMap::new(),
        }
    }

    pub fn insert_type(&mut self, id: EventId, ty: TypeEntry) -> Result<()> {
        if self.events.contains_key(&id) {
            Err(ecs_err!(
                ErrorType::EventPlanetInsert { id, ty: ty.clone() },
                self
            ))?
        }
        let data = DVec::new(ty.size, ty.drop);
        self.events.insert(id, (ty, data));
        Ok(())
    }

    pub fn get_type(&self, id: EventId) -> Option<&TypeEntry> {
        self.events.get(&id).as_ref().map(|(ty, _)| ty)
    }

    pub fn get_events(&self, id: EventId) -> Result<&DVec> {
        self.events
            .get(&id)
            .ok_or(ecs_err!(ErrorType::EventPlanetGetEvents { id }, self))
            .map(|(_, v)| v)
    }

    pub fn modify(&mut self, modify: &mut EventModify) -> Result<()> {
        for (_, (_, data)) in self.events.iter_mut() {
            data.clear();
        }
        let events = std::mem::take(&mut modify.events);
        let err = ecs_err!(ErrorType::EventPlanetModify, (&self, &events));
        for (id, val) in events.into_iter() {
            let ptr = val.get();
            self.events
                .get_mut(&id)
                .ok_or_else(|| err.clone())?
                .1
                .resize(1, ptr);
            val.take();
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct EventModify {
    events: Vec<(EventId, TVal)>,
}

impl EventModify {
    pub fn new() -> Self {
        EventModify { events: Vec::new() }
    }

    pub fn insert(&mut self, id: EventId, val: TVal) {
        self.events.push((id, val));
    }
}

impl Default for EventModify {
    fn default() -> Self {
        Self::new()
    }
}
