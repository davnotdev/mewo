use super::drop::ValueDrop;

//  TODO FIX: Fix Valgrand memory errors.

#[derive(Debug)]
pub struct TVal {
    size: usize,
    val: Vec<u8>,
    drop: ValueDrop,
}

impl TVal {
    pub fn new(size: usize, val: *const u8, drop: ValueDrop) -> Self {
        TVal {
            size,
            val: unsafe {
                let mut v = Vec::new();
                if size == 0 {
                    v.resize(1, 0);
                } else {
                    //  Why do these lines cause undefined behavior?
                    //
                    //  v.resize(size, 0);
                    //  std::ptr::copy_nonoverlapping::<u8>(val, v.as_mut_ptr(), size);

                    for s in 0..size {
                        v.push(*val.add(s));
                    }
                }
                v
            },
            drop,
        }
    }

    pub fn get(&self) -> *const u8 {
        self.val.as_ptr()
    }

    pub fn get_size(&self) -> usize {
        self.size
    }

    pub fn take(mut self) {
        self.drop = ValueDrop::empty();
    }
}

impl Drop for TVal {
    fn drop(&mut self) {
        self.drop.call(self.get())
    }
}

#[test]
fn test_tval() {
    let size = std::mem::size_of::<u128>();
    let val = 89238929u128;
    let tval = TVal::new(size, &val as *const u128 as *const u8, ValueDrop::empty());
    unsafe { assert_eq!(val, *(tval.get() as *const u128),) };
}

#[test]
fn test_unsized_tval() {
    #[derive(Debug, PartialEq)]
    struct MyStruct;
    let size = std::mem::size_of::<MyStruct>();
    let tval = TVal::new(
        size,
        &MyStruct as *const MyStruct as *const u8,
        ValueDrop::empty(),
    );
    unsafe { assert_eq!(MyStruct, *(tval.get() as *const MyStruct)) };
}
