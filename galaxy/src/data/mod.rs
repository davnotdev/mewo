mod drop;
mod dvec;
mod sparse;
mod typeentry;
//  mod sync;
mod tval;

pub use drop::{CloneFunction, DropFunction, ValueClone, ValueDrop};
pub use dvec::DVec;
pub use sparse::SparseSet;
pub use typeentry::TypeEntry;
//  pub use sync::{CacheLine, Local, LockState};
pub use tval::TVal;

use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

//  The chances of a collision are basically impossible.
//  However, this possiblity still keeps me up at night.
pub fn hash_type<T: 'static>() -> u64 {
    let mut hasher = DefaultHasher::new();
    std::any::TypeId::of::<T>().hash(&mut hasher);
    hasher.finish()
}

pub fn data_drop<T>() -> ValueDrop {
    ValueDrop::new(|ptr| unsafe { drop(std::ptr::read(ptr as *const T as *mut T)) })
}

pub fn data_clone<T: Clone>() -> ValueClone {
    ValueClone::new(|src, dst| unsafe { *(dst as *mut T) = (&*(src as *const T)).clone() })
}
