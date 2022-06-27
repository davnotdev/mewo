//  SUPER WARNING: TVal does NOT drop its inner value.
//  That's your job now!

#[derive(Debug)]
pub struct TVal {
    size: usize,
    val: Vec<u8>,
}

impl TVal {
    pub fn create(size: usize, val: *const u8) -> Self {
        TVal {
            size,
            val: unsafe {
                let mut v = Vec::new();
                v.resize(size, 0);
                std::ptr::copy_nonoverlapping::<u8>(val, v.as_mut_ptr(), size);
                v
            },
        }
    }

    pub fn get(&self) -> *const u8 {
        self.val.as_ptr()
    }

    pub fn get_size(&self) -> usize {
        self.size
    }
}

pub type TValDropFunction = fn(*const u8);
pub type TValCloneFunction = fn(*const u8) -> TVal;

#[test]
fn test_tval() {
    let size = std::mem::size_of::<u128>();
    let val = 89238929u128;
    let tval = TVal::create(size, &val as *const u128 as *const u8);
    unsafe { assert_eq!(val, *(tval.get() as *const u128),) };
}
