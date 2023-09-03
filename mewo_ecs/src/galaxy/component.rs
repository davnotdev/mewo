use super::ComponentTypeId;
use crate::data::{data_clone, data_drop, hash_type, TypeEntry, ValueDrop, ValueDuplicate};

pub trait CheapComponent: Copy {
    fn mewo_component_duplicate() -> ValueDuplicate {
        ValueDuplicate::Copy
    }
}

pub trait UniqueComponent {
    fn mewo_component_duplicate() -> ValueDuplicate {
        ValueDuplicate::None
    }
}

pub trait Component: Clone {
    fn mewo_component_duplicate() -> ValueDuplicate {
        data_clone::<Self>()
    }
}

pub trait GenericComponent {
    fn mewo_component_id() -> ComponentTypeId
    where
        Self: 'static + Sized,
    {
        ComponentTypeId::from_hash(hash_type::<Self>())
    }

    fn mewo_component_type_entry() -> TypeEntry
    where
        Self: Sized,
    {
        TypeEntry {
            size: Self::mewo_component_size(),
            name: String::from(std::any::type_name::<Self>()),
            drop: Self::mewo_component_drop(),
            dup: Self::mewo_component_duplicate(),
        }
    }

    fn mewo_component_size() -> usize
    where
        Self: Sized,
    {
        std::mem::size_of::<Self>()
    }

    fn mewo_component_drop() -> ValueDrop
    where
        Self: Sized,
    {
        data_drop::<Self>()
    }

    fn mewo_component_duplicate() -> ValueDuplicate;
}
