use crate::*;

pub trait Plugin {
    fn name() -> &'static str;
    fn plugin(pb: &mut App);
}
