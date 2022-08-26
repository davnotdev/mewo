use crate::util::{clone_type, drop_type, hash_type, name_type};
use mewo_ecs::{
    ArchetypeAccess, ArchetypeAccessKey, ArchetypeManager, ComponentHash, ComponentQueryAccessType,
    ComponentQueryFilterType, ComponentTypeEntry, ComponentTypeManager, Entity,
    SharedComponentTypeManager,
};
use std::marker::PhantomData;

mod impls;
mod iter;

pub use iter::Components;

pub trait Component: Clone + 'static {
    fn component_trait_entry() -> ComponentTypeEntry {
        ComponentTypeEntry {
            size: std::mem::size_of::<Self>(),
            name: name_type::<Self>(),
            hash: hash_type::<Self>(),
            drop: drop_type::<Self>(),
            clone: clone_type::<Self>(),
        }
    }

    fn component_trait_hash() -> ComponentHash {
        hash_type::<Self>()
    }
}

pub trait ComponentAccesses {
    fn accesses() -> Vec<(ComponentHash, ComponentQueryAccessType)>;
    fn hashes() -> Vec<ComponentHash>;
    fn datas(idx: usize, datas: &Vec<Option<*const u8>>) -> Self;
    fn maybe_register(ctymgr: &mut ComponentTypeManager);
}

trait ComponentAccess {
    fn access() -> (ComponentHash, ComponentQueryAccessType);
    fn data(idx: usize, data: &Option<*const u8>) -> Self;
    fn hash() -> ComponentHash;
    fn maybe_register(ctymgr: &mut ComponentTypeManager);
}

pub trait ComponentFilters {
    fn filters() -> Vec<(ComponentHash, ComponentQueryFilterType)>;
    fn maybe_register(ctymgr: &mut ComponentTypeManager);
}

trait ComponentFilter {
    fn filter() -> (ComponentHash, ComponentQueryFilterType);
    fn maybe_register(ctymgr: &mut ComponentTypeManager);
}

pub struct With<C: Component>(PhantomData<C>);
pub struct Without<C: Component>(PhantomData<C>);
