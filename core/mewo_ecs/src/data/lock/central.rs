use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicIsize, Ordering},
        RwLock,
    },
    thread::{current, ThreadId},
};

//  A `CentralLock` is implemented such that a multiple threads can read, but when a thread writes,
//  it is allowed to additionally have multiple writes and reads while other threads wait.
pub struct CentralLock {
    insert_lock: RwLock<()>,
    total_refs: AtomicIsize,
    thread_refs: HashMap<ThreadId, (usize, usize)>, //  (Mutable References, References)
}

//  There may be an edge case here that my tiny brain cannot fathom.
impl CentralLock {
    pub fn create() -> Self {
        CentralLock {
            insert_lock: RwLock::new(()),
            total_refs: AtomicIsize::new(0),
            thread_refs: HashMap::new(),
        }
    }

    fn potentially_new_thread(&self) {
        let read = self.insert_lock.read().unwrap();
        let current_id = current().id();
        if let None = self.thread_refs.get(&current_id) {
            drop(read);
            let _w = self.insert_lock.write().unwrap();
            self.unsafe_get_mut_map().insert(current_id, (0, 0));
        }
    }

    fn unsafe_get_mut_map(&self) -> &mut HashMap<ThreadId, (usize, usize)> {
        unsafe {
            &mut *(&self.thread_refs as *const HashMap<ThreadId, (usize, usize)>
                as *mut HashMap<ThreadId, (usize, usize)>)
        }
    }

    pub fn lock_read(&self) {
        self.potentially_new_thread();
        let _r = self.insert_lock.read();
        let (mut_count, ref_count) = self.unsafe_get_mut_map().get_mut(&current().id()).unwrap();
        let run = |count: &mut usize| {
            if *mut_count == 0 {
                let val = self.total_refs.load(Ordering::Acquire);
                if val != -1 {
                    if let Err(_) = self.total_refs.compare_exchange(
                        val,
                        val + 1,
                        Ordering::Release,
                        Ordering::Relaxed,
                    ) {
                        return false;
                    }
                }
            }
            *count += 1;
            true
        };
        while !run(ref_count) {
            std::hint::spin_loop()
        }
    }

    pub fn unlock_read(&self) {
        let _r = self.insert_lock.read();
        let (_, ref_count) = self.unsafe_get_mut_map().get_mut(&current().id()).unwrap();
        assert!(*ref_count != 0);
        *ref_count -= 1;
        self.total_refs.fetch_sub(1, Ordering::Release);
    }

    pub fn lock_write(&self) {
        self.potentially_new_thread();
        let _r = self.insert_lock.read();
        let (mut_count, ref_count) = self.unsafe_get_mut_map().get_mut(&current().id()).unwrap();

        //  If the thread already has mutable access, great!
        if *mut_count > 0 {
            *mut_count += 1;
            return;
        }
        let current_total = self.total_refs.load(Ordering::SeqCst);
        if current_total - *ref_count as isize == 0 {
            if let Err(_) = self.total_refs.compare_exchange(
                current_total,
                -1,
                Ordering::Release,
                Ordering::Relaxed,
            ) {
                std::hint::spin_loop();
            }
            *mut_count += 1;
        } else {
            std::hint::spin_loop();
        }
    }

    pub fn unlock_write(&self) {
        self.potentially_new_thread();
        let _r = self.insert_lock.read();
        let (mut_count, ref_count) = self.unsafe_get_mut_map().get_mut(&current().id()).unwrap();
        assert!(*mut_count != 0);
        *mut_count -= 1;
        if *mut_count == 0 {
            self.total_refs
                .store(*(ref_count) as isize, Ordering::SeqCst);
        }
    }
}

//  TODO Unit test!
