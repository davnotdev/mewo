type InnerMaskType = u128;
const INNER_MASK_LEN: usize = std::mem::size_of::<InnerMaskType>() * 8;

#[derive(Debug, Clone, PartialEq)]
pub struct BoolMask {
    mask: Vec<InnerMaskType>,
}

impl BoolMask {
    pub fn create() -> BoolMask {
        Self::create_with_capacity(0)
    }

    pub fn create_with_capacity(capacity: usize) -> BoolMask {
        let capacity = capacity as f32 / INNER_MASK_LEN as f32;
        BoolMask {
            mask: {
                let mut mask = Vec::new();
                mask.resize(capacity.ceil() as usize, 0);
                mask
            },
        }
    }

    pub fn get(&self, idx: usize) -> bool {
        if let Some(byte) = self.mask.get(idx / INNER_MASK_LEN) {
            *byte & (1 << (idx % INNER_MASK_LEN)) != 0
        } else {
            false
        }
    }

    pub fn set(&mut self, idx: usize, val: bool) {
        if let Some(byte) = self.mask.get_mut(idx / INNER_MASK_LEN) {
            *byte ^=
                (-(val as i128) ^ *byte as i128) as InnerMaskType & (1 << idx % INNER_MASK_LEN);
        } else {
            let new_capacity = idx as f32 / INNER_MASK_LEN as f32;
            self.mask.resize(new_capacity.ceil() as usize, 0);
            self.set(idx, val);
        }
    }

    pub fn merge(&mut self, other: &BoolMask) -> Option<()> {
        if self.mask.len() != other.mask.len() {
            None
        } else {
            for (self_byte, other_byte) in self.mask.iter_mut().zip(other.mask.iter()) {
                *self_byte = *self_byte | *other_byte;
            }
            Some(())
        }
    }

    pub fn bitwise_and(&self, other: &BoolMask) -> Option<BoolMask> {
        if self.mask.len() != other.mask.len() {
            None
        } else {
            let mut res = BoolMask::create_with_capacity(self.mask.len() * INNER_MASK_LEN);
            for (i, (self_byte, other_byte)) in self.mask.iter().zip(other.mask.iter()).enumerate()
            {
                res.mask[i] = *self_byte & *other_byte;
            }
            Some(res)
        }
    }

    pub fn compare(&self, other: &BoolMask) -> bool {
        if self.mask.len() != other.mask.len() {
            false
        } else {
            for (self_byte, other_byte) in self.mask.iter().zip(other.mask.iter()) {
                if *self_byte != *other_byte {
                    return false;
                }
            }
            true
        }
    }

    pub fn is_empty(&self) -> bool {
        for byte in self.mask.iter() {
            if *byte != 0 {
                return false;
            }
        }
        true
    }
}

#[test]
fn test_mask() {
    let mask = BoolMask::create_with_capacity(0);
    let mut test_mask_a = mask.clone();
    let sets = [2, 4, 33, 36, 420];
    for i in sets {
        test_mask_a.set(i, true);
    }
    for i in 0..=420 {
        assert_eq!(test_mask_a.get(i), sets.contains(&i))
    }

    let mut test_mask_b = mask.clone();
    let sets = [2, 4, 24, 36, 420];
    for i in sets {
        test_mask_b.set(i, true);
    }
    for i in 0..=421 {
        assert_eq!(test_mask_b.get(i), sets.contains(&i))
    }

    let mut expected_mask = mask.clone();
    expected_mask.set(2, true);
    expected_mask.set(4, true);
    expected_mask.set(36, true);
    expected_mask.set(420, true);

    let res = test_mask_a.bitwise_and(&test_mask_b).unwrap();
    assert_eq!(res, expected_mask);
    assert_eq!(res.compare(&test_mask_a), false);
    assert_eq!(res.compare(&expected_mask), true);
}
