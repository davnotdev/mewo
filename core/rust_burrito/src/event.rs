use mewo_ecs::{DropFunction, EventHash, EventInsert, TVal, ValueDrop};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub trait Event: Sized + 'static {
    fn event_name() -> String {
        format!(
            "{}_{}",
            env!("CARGO_PKG_NAME"),
            std::any::type_name::<Self>(),
        )
    }

    fn event_hash() -> EventHash {
        let mut hasher = DefaultHasher::new();
        std::any::TypeId::of::<Self>().hash(&mut hasher);
        hasher.finish()
    }

    fn event_size() -> usize {
        std::mem::size_of::<Self>()
    }

    fn event_drop_callback() -> DropFunction {
        |ptr| unsafe { drop(std::ptr::read(ptr as *const Self as *mut Self)) }
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
            E::event_hash(),
            TVal::create(
                E::event_size(),
                &e as *const E as *const u8,
                ValueDrop::create(E::event_drop_callback()),
            ),
        );
        self
    }
}
