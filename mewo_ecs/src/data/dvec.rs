use super::drop::ValueDrop;

#[derive(Debug)]
pub struct DVec {
    data: Vec<u8>,
    len: usize, //  used only for zero sized values
    data_size: usize,
    drop: ValueDrop,
}

impl DVec {
    pub fn new(size: usize, drop: ValueDrop) -> Self {
        DVec::new_with_reserve(size, 0, drop)
    }

    pub fn new_with_reserve(size: usize, reserve: usize, drop: ValueDrop) -> Self {
        DVec {
            data: {
                let mut v = Vec::with_capacity(reserve * size);
                if size == 0 {
                    v.resize(1, 0);
                }
                v
            },
            len: 0,
            data_size: size,
            drop,
        }
    }

    pub unsafe fn resize(&mut self, additional: usize, inplace: *const u8) {
        self.data.reserve(additional * self.data_size);
        for _ in 0..additional {
            for b in 0..self.data_size {
                let offsetb = *inplace.add(b);
                self.data.push(offsetb);
            }
        }
        self.len += additional;
    }

    pub unsafe fn resize_zeroed(&mut self, additional: usize) {
        self.data.reserve(additional * self.data_size);
        for _ in 0..additional {
            for _ in 0..self.data_size {
                self.data.push(0);
            }
        }
        self.len += additional;
    }

    pub unsafe fn unsafe_truncate(&mut self, len: usize) {
        self.data.truncate(self.data_size * len);
        self.len = len;
    }

    pub fn swap_remove(&mut self, idx: usize) -> Option<()> {
        let val = self.get(idx)?;
        if self.data_size != 0 {
            self.drop.call(val);
            for b in (0..self.data_size).rev() {
                let &rm = self.data.last().unwrap();
                *self.data.get_mut(idx * self.data_size + b).unwrap() = rm;
                self.data.pop();
            }
        }
        self.len -= 1;
        Some(())
    }

    pub fn take_swap_remove(&mut self, idx: usize) -> Option<()> {
        if self.data_size != 0 {
            for b in (0..self.data_size).rev() {
                let &rm = self.data.last().unwrap();
                *self.data.get_mut(idx * self.data_size + b).unwrap() = rm;
                self.data.pop();
            }
        }
        self.len -= 1;
        Some(())
    }

    pub fn ptr(&self) -> *const u8 {
        self.data.as_ptr()
    }

    pub fn get(&self, idx: usize) -> Option<*const u8> {
        let idx = if self.data_size == 0 { 0 } else { idx };
        self.data
            .get(idx * self.data_size)
            .map(|data| data as *const u8)
    }

    pub fn clear(&mut self) {
        for idx in 0..self.len() {
            let val = self.get(idx).unwrap();
            self.drop.call(val);
        }
        if self.data_size != 0 {
            self.data.clear();
        }
        self.len = 0;
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn size(&self) -> usize {
        self.data_size
    }
}

impl Drop for DVec {
    fn drop(&mut self) {
        for idx in 0..self.len() {
            let val = self.get(idx).unwrap();
            self.drop.call(val);
        }
    }
}

#[test]
fn test_dvec() {
    let size = std::mem::size_of::<u128>();
    let mut dvec = DVec::new(size, ValueDrop::empty());
    let one = 1u128;
    unsafe { dvec.resize(4, &one as *const u128 as *const u8) };
    assert_eq!(dvec.len(), 4);
    assert_eq!(dvec.data_size, size);
    for i in 0..4u128 {
        unsafe {
            std::ptr::copy_nonoverlapping::<u128>(
                &i as *const u128,
                dvec.get(i as usize).unwrap() as *mut u128,
                1,
            );
        }
    }
    for i in 0..4u128 {
        let val = dvec.get(i as usize).unwrap();
        unsafe {
            assert_eq!(i, *(val as *const u128),);
        }
    }
    dvec.swap_remove(1);
    let expected: [u128; 3] = [0, 3, 2];
    for (i, e) in expected.iter().enumerate() {
        let val = dvec.get(i).unwrap();
        unsafe {
            assert_eq!(*e, *(val as *const u128),);
        }
    }
    assert_eq!(dvec.len(), 3);
    dvec.clear();
    assert_eq!(dvec.len(), 0);
}

#[test]
fn test_unsized_dvec() {
    struct MyStruct;
    let size = std::mem::size_of::<MyStruct>();
    let mut dvec = DVec::new(size, ValueDrop::empty());
    let m = MyStruct;
    unsafe { dvec.resize(2, &m as *const MyStruct as *const u8) };
    assert_eq!(dvec.len(), 2);
    assert_eq!(dvec.data_size, size);
    dvec.swap_remove(0);
    assert_eq!(dvec.len(), 1);
    dvec.clear();
    assert_eq!(dvec.len(), 0);
}
