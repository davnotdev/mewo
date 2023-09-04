use std::{
    ops::{Deref, DerefMut},
    sync::Arc,
};

pub struct Preserve<T> {
    val: T,
    is_borrowed: Arc<()>,
}

impl<T> Preserve<T> {
    pub fn new(val: T) -> Self {
        Self {
            val,
            is_borrowed: Arc::new(()),
        }
    }

    pub fn get_instance(&self) -> PreserveInstance<T> {
        if Arc::strong_count(&self.is_borrowed) != 1 {
            panic!("Ran get_instance on Preservable whilst another instance still out.")
        }
        PreserveInstance {
            ptr: &self.val as *const T as *mut T,
            _is_borrowed: Arc::clone(&self.is_borrowed),
        }
    }
}

pub struct PreserveInstance<T> {
    ptr: *mut T,
    _is_borrowed: Arc<()>,
}

impl<T> Deref for PreserveInstance<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.ptr }
    }
}

impl<T> DerefMut for PreserveInstance<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.ptr }
    }
}
