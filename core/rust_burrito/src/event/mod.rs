use crate::{drop_type, hash_type, name_type};
use mewo_ecs::{EventHash, EventInsert, EventTypeEntry, SharedEventManager, TVal};

mod impls;

pub trait Event: Sized + 'static {
    fn event_trait_entry() -> EventTypeEntry {
        EventTypeEntry {
            size: std::mem::size_of::<Self>(),
            name: name_type::<Self>(),
            hash: hash_type::<Self>(),
            drop: drop_type::<Self>(),
        }
    }

    fn event_trait_hash() -> EventHash {
        hash_type::<Self>()
    }
}

pub struct EventBus<'galaxy, 'exec> {
    evmgr: &'galaxy SharedEventManager,
    insert: &'exec mut EventInsert,
}

impl<'galaxy, 'exec> EventBus<'galaxy, 'exec> {
    pub fn create(evmgr: &'galaxy SharedEventManager, insert: &'exec mut EventInsert) -> Self {
        EventBus { evmgr, insert }
    }

    pub fn get<E: EventAccess>(&self) -> Option<&[E]> {
        E::data(&self.evmgr)
    }

    pub fn event<E: Event>(&mut self, e: E) -> &mut Self {
        let ehash = E::event_trait_hash();
        let exists = self.evmgr.read().unwrap().get(ehash).is_ok();
        if !exists {
            let mut evmgr = self.evmgr.write().unwrap();
            let _ = evmgr.register(E::event_trait_entry());
        }
        self.insert.insert(
            ehash,
            TVal::create(
                E::event_trait_entry().size,
                &e as *const E as *const u8,
                E::event_trait_entry().drop,
            ),
        );
        self
    }
}

pub struct StartupDone;
impl Event for StartupDone {}

pub trait EventAccess {
    fn hash() -> EventHash;
    fn data(evmgr: &SharedEventManager) -> Option<&[Self]>
    where
        Self: Sized;
}
