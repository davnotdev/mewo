use super::{ValueDrop, ValueDuplicate};

#[derive(Debug, Clone)]
pub struct TypeEntry {
    pub size: usize,
    pub name: String,
    pub drop: ValueDrop,
    pub dup: ValueDuplicate,
}
