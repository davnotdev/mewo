#[derive(Debug, PartialEq)]
pub struct BoolMask {
    mask: Vec<u32>,
    len: usize,
    res: usize,
}

impl BoolMask {
    pub fn create() -> BoolMask {
        BoolMask {
            mask: vec![0],
            len: 0,
            res: 32,
        } 
    }

    pub fn get_len(&self) -> usize {
        self.len
    }

    pub fn get_reserve(&self) -> usize {
        self.res
    }

    pub fn extend(&mut self, i: usize) {
        self.len += i;
        self.res += i;
        self.mask.resize((self.res+32)/32 - self.mask.len(), 0)
    }

    pub fn get(&self, i: usize) -> Result<bool, ()> {
        if i < self.res {
            let local = i%32;
            Ok((self.mask.get(i/32).unwrap() & (1 << local)) != 0)
        } else {
            Err(())
        }
    }

    pub fn set(&mut self, i: usize, val: bool) -> Result<(), ()> {
        if i < self.res {
            let local = i%32;
            let get = self.mask.get_mut(i/32).unwrap();
            *get ^= (-(val as i32) ^ *get as i32) as u32 & (1 << local);
            Ok(())
        } else {
            Err(())
        }
    }

    pub fn is_empty(&self) -> bool {
        for i in 0..self.get_reserve()/32 {
            if i != 0 {
                return false
            } 
        }
        true
    }

    pub fn compare(&self, other: &BoolMask) -> Result<bool, ()> {
        if self.get_len() != other.get_len() {
            return Err(())
        }
        for i in 0..self.get_reserve()/32 {
            if self.mask[i] != other.mask[i] {
                return Ok(false)
            }
        }
        Ok(true)
    }

    pub fn bitwise_and(&self, other: &BoolMask) -> Result<BoolMask, ()> {
        if self.get_len() != other.get_len() {
            return Err(())
        }
        let mut res = BoolMask::create();
        res.extend(self.get_len()-res.get_len());
        for i in 0..self.get_reserve()/32 {
            res.mask[i] = self.mask[i] & other.mask[i];
        }
        Ok(res)
    }

    pub fn flush(&mut self) {
        for i in 0..self.get_reserve()/32 {
            self.mask[i] = 0;
        }
    }
}

#[test]
fn test_bool_mask() {
    let mut mask = BoolMask::create();
    assert_eq!(mask.get_len(), 0);
    assert_eq!(mask.get_reserve(), 32);
    assert_eq!(mask.get(32), Err(()));
    mask.extend(63);
    assert_eq!(mask.mask.len(), 2);
    mask.set(0, true).unwrap();
    assert_eq!(mask.get(0), Ok(true));
    assert_eq!(*mask.mask.get(0).unwrap(), (1 << 0));
}

#[test]
fn test_bool_mask_operations() {
    let mut mask = BoolMask::create();
    mask.extend(32);
    mask.set(2, true).unwrap();
    mask.set(4, true).unwrap();
    mask.set(33, true).unwrap();
    mask.set(36, true).unwrap();
    let mut test_mask = BoolMask::create();
    test_mask.extend(32);
    test_mask.set(2, true).unwrap();
    test_mask.set(4, true).unwrap();
    test_mask.set(34, true).unwrap();
    test_mask.set(36, true).unwrap();
    let mut expected_mask = BoolMask::create();
    expected_mask.extend(32);
    expected_mask.set(2, true).unwrap();
    expected_mask.set(4, true).unwrap();
    expected_mask.set(36, true).unwrap();

//  and + compare
    let res = mask.bitwise_and(&test_mask).unwrap();
    assert_eq!(res, expected_mask);
    assert_eq!(res.compare(&expected_mask), Ok(true));
    assert_eq!(res.compare(&test_mask), Ok(false));

//  is_empty
    let empty_mask = BoolMask::create();
    assert_eq!(test_mask.is_empty(), false);
    assert_eq!(empty_mask.is_empty(), true);
}

