use mewo_ecs::{CloneFunction, ComponentHash, DropFunction};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub trait Component: Clone + 'static {
    fn component_name() -> String {
        format!(
            "{}_{}",
            env!("CARGO_PKG_NAME"),
            std::any::type_name::<Self>(),
        )
    }
    fn component_hash() -> ComponentHash {
        let mut hasher = DefaultHasher::new();
        std::any::TypeId::of::<Self>().hash(&mut hasher);
        hasher.finish()
    }
    fn component_drop_callback() -> DropFunction {
        |ptr| unsafe { drop(std::ptr::read(ptr as *const Self as *mut Self)) }
    }
    fn component_clone_callback() -> CloneFunction {
        |ptr, dst| unsafe {
            let src = std::ptr::read(ptr as *const Self);
            let clone = src.clone();
            std::ptr::copy_nonoverlapping(&clone as *const Self, dst as *mut Self, 1);
            std::mem::forget(src);
            std::mem::forget(clone);
        }
    }
    fn component_size() -> usize {
        std::mem::size_of::<Self>()
    }
    fn component_is_copy() -> bool; //  This will come later :)
}

//  Doesn't work :(
//  pub trait Component : BaseComponent + Clone + 'static {
//      fn is_copy() -> bool {
//          false
//      }
//  }

//  pub trait CopyComponent : BaseComponent + Clone + Copy + 'static {
//      fn is_copy() -> bool {
//          true
//      }
//  }
