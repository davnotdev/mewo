use parking_lot::RwLock;
use std::{
    cell::UnsafeCell,
    collections::HashMap,
    ops::{Deref, DerefMut},
    thread::{current, ThreadId},
};

pub struct ThreadLocalGuard<'a, T> {
    id: ThreadId,
    thread_local: &'a ThreadLocal<T>,
}

impl<'a, T> ThreadLocalGuard<'a, T> {
    //  The whole point of this fn is to get a mut from ref.
    #[allow(clippy::mut_from_ref)]
    fn get(&self) -> &mut T {
        let hashes = self.thread_local.hashes.read();
        let idx = hashes.get(&self.id).unwrap();
        unsafe {
            self.thread_local
                .datas
                .data_ptr()
                .as_mut()
                .unwrap()
                .get()
                .as_mut()
                .unwrap()
                .get_mut(*idx)
                .unwrap()
        }
    }
}

impl<'a, T> Drop for ThreadLocalGuard<'a, T> {
    fn drop(&mut self) {
        unsafe { self.thread_local.datas.force_unlock_read() }
    }
}

impl<'a, T> Deref for ThreadLocalGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.get()
    }
}

impl<'a, T> DerefMut for ThreadLocalGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.get()
    }
}

//  Just in case.
#[repr(align(64))]
pub struct ThreadLocal<T> {
    hashes: RwLock<HashMap<ThreadId, usize>>,
    datas: RwLock<UnsafeCell<Vec<T>>>,
}

impl<T> ThreadLocal<T> {
    pub fn new() -> Self {
        ThreadLocal {
            hashes: RwLock::new(HashMap::new()),
            datas: RwLock::new(UnsafeCell::new(Vec::new())),
        }
    }

    pub fn get(&self) -> Option<ThreadLocalGuard<T>> {
        let id = current().id();
        self.hashes.read().get(&id)?;
        std::mem::forget(self.datas.read());
        Some(ThreadLocalGuard {
            id,
            thread_local: self,
        })
    }

    pub fn get_or<F: FnOnce() -> T>(&self, f: F) -> ThreadLocalGuard<T> {
        let id = current().id();
        if let Some(val) = self.get() {
            val
        } else {
            {
                let datas = unsafe { self.datas.write().get().as_mut().unwrap() };
                self.hashes.write().insert(id, datas.len());
                datas.push((f)());
            }
            self.get().unwrap()
        }
    }

    //  Assumes that no references from get or get_or are out.
    pub unsafe fn get_inner(&mut self) -> &mut Vec<T> {
        self.datas
            .data_ptr()
            .as_mut()
            .unwrap()
            .get()
            .as_mut()
            .unwrap()
    }
}

unsafe impl<T> Send for ThreadLocal<T> {}
unsafe impl<T> Sync for ThreadLocal<T> {}

#[test]
fn test_thread_local() {
    use std::sync::Arc;

    let local = Arc::new(ThreadLocal::new());

    local.get_or(|| 10);

    let tlocal = Arc::clone(&local);
    assert_eq!(
        std::thread::spawn(move || {
            let local = tlocal;
            local.get_or(|| 12);
            local.get().map(|v| *v)
        })
        .join()
        .unwrap(),
        Some(12)
    );
    assert_eq!(local.get().map(|v| *v), Some(10))
}
