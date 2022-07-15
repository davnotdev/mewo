//  TODO Use condvar, mutexes, or something other than spin lock.

mod central;
mod individual;

pub use central::CentralLock;
pub use individual::{IndividualLock, LockState};
