mod drop;
mod dvec;
mod lock;
mod sparse;
mod tval;

pub use drop::{CloneFunction, DropFunction, ValueClone, ValueDrop};
pub use dvec::DVec;
pub use lock::{CentralLock, IndividualLock, LockState};
pub use sparse::SparseSet;
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
