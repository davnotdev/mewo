use mewo_ecs::{EventHash, EventInsert, TVal};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub trait Event: Clone + 'static {
    fn name() -> String {
        format!(
            "{}_{}",
            env!("CARGO_PKG_NAME"),
            std::any::type_name::<Self>(),
        )
    }

    fn hash() -> EventHash {
        let mut hasher = DefaultHasher::new();
        std::any::TypeId::of::<Self>().hash(&mut hasher);
        hasher.finish()
    }
}

pub struct EventBus<'evinsert> {
    insert: &'evinsert mut EventInsert,
}

impl<'evinsert> EventBus<'evinsert> {
    pub fn create(insert: &'evinsert mut EventInsert) -> Self {
        EventBus { insert }
    }

    pub fn event<E: Event>(&mut self, e: E) -> &mut Self {
        self.insert.insert(
            E::hash(),
            TVal::create(std::mem::size_of::<E>(), &e as *const E as *const u8),
        );
        self
    }
}
