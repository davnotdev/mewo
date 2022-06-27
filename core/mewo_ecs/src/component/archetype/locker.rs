use std::sync::atomic::{AtomicU8, Ordering};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LockState {
    Shared,
    Exclusive,
}

//  00010 ~ Shared by 2
//  00000 ~ Open
//  11111 ~ Exclusive
type AtomicLockState = AtomicU8;
const OPEN_STATE: u8 = 0u8;
const EXCLUSIVE_STATE: u8 = u8::MAX;

#[derive(Debug)]
pub struct Locker {
    lock: AtomicLockState,
}

//  All these Orderings may be suboptimal ¯\_(ツ)_/¯
impl Locker {
    pub fn create() -> Self {
        Locker {
            lock: AtomicU8::new(OPEN_STATE),
        }
    }

    pub fn try_obtain_lock(&self, state: LockState) -> bool {
        match state {
            LockState::Exclusive => {
                if let Ok(_) = self.lock.compare_exchange(
                    OPEN_STATE,
                    EXCLUSIVE_STATE,
                    Ordering::AcqRel,
                    Ordering::Relaxed,
                ) {
                    true
                } else {
                    false
                }
            }
            LockState::Shared => {
                //  https://stackoverflow.com/questions/47753528/how-to-compare-and-increment-an-atomic-variable
                let val = self.lock.load(Ordering::SeqCst);
                let new = match val {
                    EXCLUSIVE_STATE => return false,
                    _ => val + 1,
                };
                if let Ok(_) =
                    self.lock
                        .compare_exchange(val, new, Ordering::AcqRel, Ordering::Relaxed)
                {
                    true
                } else {
                    false
                }
            }
        }
    }

    //  Will absolutely corrupt lock if used inappropriately.
    pub fn release_lock(&self, lock_state: LockState) {
        match lock_state {
            LockState::Shared => self.lock.fetch_sub(1, Ordering::Relaxed),
            LockState::Exclusive => self.lock.fetch_add(1, Ordering::Relaxed),
        };
    }
}

//  Doesn't test for race conditions
#[test]
fn test_locker() {
    let locker = Locker::create();

    assert_eq!(locker.try_obtain_lock(LockState::Exclusive), true);
    assert_eq!(locker.try_obtain_lock(LockState::Exclusive), false);
    assert_eq!(locker.try_obtain_lock(LockState::Shared), false);
    locker.release_lock(LockState::Exclusive);
    assert_eq!(locker.lock.load(Ordering::SeqCst), OPEN_STATE);

    assert_eq!(locker.try_obtain_lock(LockState::Shared), true);
    assert_eq!(locker.try_obtain_lock(LockState::Shared), true);
    assert_eq!(locker.try_obtain_lock(LockState::Shared), true);
    assert_eq!(locker.try_obtain_lock(LockState::Exclusive), false);
    assert_eq!(locker.lock.load(Ordering::SeqCst), 3u8);
    locker.release_lock(LockState::Shared);
    locker.release_lock(LockState::Shared);
    locker.release_lock(LockState::Shared);
    assert_eq!(locker.lock.load(Ordering::SeqCst), OPEN_STATE);
}
