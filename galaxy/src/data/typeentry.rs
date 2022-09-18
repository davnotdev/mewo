use super::{ValueClone, ValueDrop};

#[derive(Debug, Clone)]
pub struct TypeEntry {
    pub size: usize,
    pub name: String,
    pub drop: ValueDrop,
    pub clone: ValueClone,
}
