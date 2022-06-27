#[derive(Debug)]
pub struct DVec {
    data: Vec<u8>,
    data_size: usize,
}

impl DVec {
    pub fn create(size: usize) -> Self {
        DVec::create_with_reserve(size, 0)
    }

    pub fn create_with_reserve(size: usize, reserve: usize) -> Self {
        DVec {
            data: Vec::with_capacity(reserve * size),
            data_size: size,
        }
    }

    pub fn resize(&mut self, additional: usize, inplace: *const u8) {
        if self.data_size == 0 {
            return 
        }
        self.data.reserve(additional * self.data_size);
        for _ in 0..additional {
            for b in 0..self.data_size {
                let offsetb = unsafe { *inplace.offset(b as isize) };
                self.data.push(offsetb);
            }
        }
    }

    pub fn swap_remove(&mut self, idx: usize) -> Option<()> {
        if self.len() == 0 || idx >= self.len() {
            None?
        }
        for b in (0..self.data_size).rev() {
            let &rm = self.data.get(self.data.len()-1).unwrap();
            *self.data.get_mut(idx * self.data_size + b).unwrap() = rm;
            self.data.pop();
        }
        Some(())
    }

    pub fn ptr(&self) -> *const u8 {
        self.data.as_ptr()
    }

    pub fn get(&self, idx: usize) -> Option<*const u8> {
        self.data
            .get(idx * self.data_size)
            .map(|data| data as *const u8)
    }

    pub fn len(&self) -> usize {
        self.data.len() / self.data_size
    }
}

#[test]
fn test_dvec() {
    let size = std::mem::size_of::<u128>();
    let mut dvec = DVec::create(size);
    let one = 1u128;
    dvec.resize(4, &one as *const u128 as *const u8);
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
        let val = dvec.get(i as usize).unwrap();
        unsafe {
            assert_eq!(*e, *(val as *const u128),);
        }
    }
    assert_eq!(dvec.len(), 3);
}
