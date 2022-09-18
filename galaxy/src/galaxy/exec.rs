use super::{EventModify, StorageTransform};

pub trait Executor {
    fn new() -> Self
    where
        Self: Sized;
    fn get_event_modify(&self) -> &mut EventModify;
    fn get_storage_transforms(&self) -> &mut Vec<StorageTransform>;

    fn get_all_event_modify(&mut self) -> &mut [EventModify];
    fn get_all_storage_transforms(&mut self) -> &mut [Vec<StorageTransform>];

    fn clear_all_storage_transforms(&mut self);
}
