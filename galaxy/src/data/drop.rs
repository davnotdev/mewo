pub type DropFunction = fn(*const u8);
pub type CloneFunction = fn(*const u8, *mut u8);

#[derive(Debug, Clone, Copy)]
pub struct ValueDrop(Option<DropFunction>);

#[derive(Debug, Clone, Copy)]
pub enum ValueDuplicate {
    Copy,
    Clone(CloneFunction),
    None,
}

impl ValueDrop {
    pub fn empty() -> Self {
        ValueDrop(None)
    }

    pub fn new(f: DropFunction) -> Self {
        ValueDrop(Some(f))
    }

    pub fn call(&self, val: *const u8) {
        if let Self(Some(f)) = self {
            (f)(val)
        }
    }
}
