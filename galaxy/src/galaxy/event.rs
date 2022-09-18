use super::{EventId, Executor, Galaxy};
use crate::data::{data_clone, data_drop, hash_type, TVal, TypeEntry, ValueClone, ValueDrop};

pub trait Event: Clone {
    fn mewo_event_id() -> EventId
    where
        Self: 'static,
    {
        EventId::from_hash(hash_type::<Self>())
    }

    fn mewo_event_type_entry() -> TypeEntry {
        TypeEntry {
            size: Self::mewo_event_size(),
            name: String::from(std::any::type_name::<Self>()),
            drop: Self::mewo_event_drop(),
            clone: Self::mewo_event_clone(),
        }
    }

    fn mewo_event_size() -> usize {
        std::mem::size_of::<Self>()
    }

    fn mewo_event_drop() -> ValueDrop {
        data_drop::<Self>()
    }

    fn mewo_event_clone() -> ValueClone {
        data_clone::<Self>()
    }
}

impl<EX> Galaxy<EX>
where
    EX: Executor,
{
    pub fn insert_event<E: Event + 'static>(&self, e: E) -> &Self {
        self.event_maybe_insert::<E>();
        self.exec.get_event_modify().insert(
            E::mewo_event_id(),
            TVal::new(
                E::mewo_event_size(),
                &e as *const E as *const u8,
                E::mewo_event_drop(),
            ),
        );
        std::mem::forget(e);
        self
    }

    pub fn get_events<E: Event + 'static>(&self) -> &[E] {
        let evp = self.evp.read();
        let events = evp.get_events(E::mewo_event_id()).unwrap();
        //  TODO CHK: Test for zero sized values.
        unsafe { std::slice::from_raw_parts(events.ptr() as *const E, events.len()) }
    }

    fn event_maybe_insert<E: Event + 'static>(&self) {
        let id = E::mewo_event_id();
        if self.evp.read().get_type(id).is_none() {
            let mut evp = self.evp.write();
            evp.insert_type(id, E::mewo_event_type_entry()).unwrap();
        }
    }
}
