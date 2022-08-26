pub use mewo_ecs::hash_type;
use mewo_ecs::{ValueClone, ValueDrop};

pub fn name_type<T: 'static>() -> String {
    format!("{}_{}", env!("CARGO_PKG_NAME"), std::any::type_name::<T>(),)
}

pub fn drop_type<T: 'static>() -> ValueDrop {
    ValueDrop::create(|ptr| unsafe { drop(std::ptr::read(ptr as *const T as *mut T)) })
}

pub fn clone_type<T: 'static + Clone>() -> ValueClone {
    ValueClone::create(|ptr, dst| unsafe {
        let src = std::ptr::read(ptr as *const T);
        let clone = src.clone();
        std::ptr::copy_nonoverlapping(&clone as *const T, dst as *mut T, 1);
        std::mem::forget(src);
        std::mem::forget(clone);
    })
}
