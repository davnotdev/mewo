use super::{ComponentInfo, ComponentStorageType, ComponentTypeId};
use crate::data::{data_clone, data_drop, hash_type, TypeEntry, ValueClone, ValueDrop};

pub trait Component: Clone {
    fn mewo_component_id() -> ComponentTypeId
    where
        Self: 'static,
    {
        ComponentTypeId::from_hash(hash_type::<Self>())
    }

    fn mewo_component_type_entry() -> TypeEntry {
        TypeEntry {
            size: Self::mewo_component_size(),
            name: String::from(std::any::type_name::<Self>()),
            drop: Self::mewo_component_drop(),
            clone: Self::mewo_component_clone(),
        }
    }

    fn mewo_component_info() -> ComponentInfo {
        ComponentInfo {
            ty: Self::mewo_component_type_entry(),
            storage_ty: Self::mewo_component_storage_type(),
        }
    }

    fn mewo_component_storage_type() -> ComponentStorageType;

    fn mewo_component_size() -> usize {
        std::mem::size_of::<Self>()
    }

    fn mewo_component_drop() -> ValueDrop {
        data_drop::<Self>()
    }

    fn mewo_component_clone() -> ValueClone {
        data_clone::<Self>()
    }
}
