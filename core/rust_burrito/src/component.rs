use mewo_ecs::ComponentHash;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub trait Component: Clone + 'static {
    fn name() -> String {
        format!(
            "{}_{}",
            env!("CARGO_PKG_NAME"),
            std::any::type_name::<Self>(),
        )
    }
    fn hash() -> ComponentHash {
        let mut hasher = DefaultHasher::new();
        std::any::TypeId::of::<Self>().hash(&mut hasher);
        hasher.finish()
    }
    fn is_copy() -> bool;   //  This will come later :)
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

