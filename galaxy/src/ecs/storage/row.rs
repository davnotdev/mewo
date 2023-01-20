use super::*;

#[derive(Debug)]
pub(super) enum StorageRow {
    Normal(RwLock<DVec>),
    CopyCat(Mutex<DVec>, DVec),
}

impl StorageRow {
    pub fn update(&mut self) {
        if let StorageRow::CopyCat(write, read) = self {
            //  Assumes that the size of write is always >= read.
            let write = write.lock();
            unsafe {
                if write.len() < read.len() {
                    read.unsafe_truncate(write.len());
                } else {
                    read.resize_zeroed(write.len() - read.len());
                }
                std::ptr::copy_nonoverlapping(
                    write.ptr(),
                    read.ptr() as *mut u8,
                    write.len() * write.size(),
                );
            };
        }
    }

    pub fn access_write(&self) -> *const u8 {
        match self {
            StorageRow::Normal(v) => unsafe { &*v.data_ptr() }.ptr(),
            StorageRow::CopyCat(v, _) => unsafe { &*v.data_ptr() }.ptr(),
        }
    }

    pub fn access_read(&self) -> *const u8 {
        match self {
            StorageRow::Normal(v) => unsafe { &*v.data_ptr() }.ptr(),
            StorageRow::CopyCat(_, v) => v.ptr(),
        }
    }

    pub fn write_lock(&self) {
        match self {
            StorageRow::Normal(v) => std::mem::forget(v.write()),
            StorageRow::CopyCat(v, _) => std::mem::forget(v.lock()),
        }
    }

    pub fn write_unlock(&self) {
        match self {
            StorageRow::Normal(v) => unsafe { v.force_unlock_write() },
            StorageRow::CopyCat(v, _) => unsafe { v.force_unlock() },
        }
    }

    pub fn read_lock(&self) {
        if let StorageRow::Normal(v) = self {
            std::mem::forget(v.read())
        };
    }

    pub fn read_unlock(&self) {
        if let StorageRow::Normal(v) = self {
            unsafe { v.force_unlock_read() }
        };
    }

    pub fn swap_remove(&mut self, idx: usize) {
        match self {
            StorageRow::Normal(v) => v.write().swap_remove(idx),
            StorageRow::CopyCat(v, _) => v.lock().swap_remove(idx),
        };
    }

    pub fn take_swap_remove(&mut self, idx: usize) {
        match self {
            StorageRow::Normal(v) => v.write().take_swap_remove(idx),
            StorageRow::CopyCat(v, _) => v.lock().take_swap_remove(idx),
        };
    }

    pub fn get_mut(&mut self, idx: usize) -> Option<*mut u8> {
        match self {
            StorageRow::Normal(v) => v.write().get(idx).map(|ptr| ptr as *mut u8),
            StorageRow::CopyCat(v, _) => v.lock().get(idx).map(|ptr| ptr as *mut u8),
        }
    }

    pub fn resize(&mut self, idx: usize, inplace: *const u8) {
        match self {
            StorageRow::Normal(v) => v.write().resize(idx, inplace),
            StorageRow::CopyCat(v, _) => v.lock().resize(idx, inplace),
        }
    }

    //  For copycat, the len of copy is returned.
    pub fn get_len(&self) -> usize {
        let r = match self {
            StorageRow::Normal(v) => v.read().len(),
            StorageRow::CopyCat(_, v) => v.len(),
        };
        r
    }
}
