#[derive(Debug, Clone)]
pub struct SparseSet<T> {
//  (sparse key, data)
    dense: Vec<(usize, T)>,
    sparse: Vec<Option<usize>>,
}

impl<T> SparseSet<T> {
    pub fn create() -> Self {
        Self::create_with_capacity(0) 
    }

    pub fn create_with_capacity(capacity: usize) -> Self {
        let dense = Vec::with_capacity(capacity);
        let sparse = {
            let mut sparse = Vec::new();
            sparse.resize(capacity, None);
            sparse
        };
        SparseSet {
            dense, 
            sparse,
        }
    }

    pub fn insert(&mut self, idx: usize, data: T)  {
        if idx >= self.sparse.len() {
            self.sparse.resize(idx+1, None);
        }
        self.dense.push((idx, data));
        *self.sparse.get_mut(idx).unwrap() = Some(self.dense.len()-1);
    }

    pub fn remove(&mut self, idx: usize) -> Option<T> {
        if idx >= self.sparse.len() {
            None
        } else {
            let dense_idx = if let Some(idx) = 
                std::mem::replace(self.sparse.get_mut(idx).unwrap(), None) {
                    idx
                } else {
                    return None
                };
            if self.dense.len() != 1 {
                let key = self.dense.get(self.dense.len()-1).unwrap().0;
                *self.sparse.get_mut(idx).unwrap() = None;
                *self.sparse.get_mut(key).unwrap() = Some(dense_idx);
            }
            let (_, data) = self.dense.swap_remove(dense_idx);
            Some(data)
        }
    }

    pub fn get_from_sparse(&self, idx: usize) -> Option<usize> {
        if let Some(s) = self.sparse.get(idx) {
            *s
        } else {
            None
        }
    }

    pub fn get(&self, idx: usize) -> Option<&T> {
        if let Some(idx) = self.get_from_sparse(idx) {
            return Some(&self.dense.get(idx)?.1)
        }
        None
    }

    pub fn get_mut(&mut self, idx: usize) -> Option<&mut T> {
        if let Some(idx) = self.get_from_sparse(idx) {
            return Some(&mut self.dense.get_mut(idx)?.1)
        }
        None
    }

    pub fn get_dense(&self) -> &Vec<(usize, T)> {
        &self.dense
    }
}

#[test]
fn test_sparse_set() {
    let mut sparse_set = SparseSet::create();
    assert_eq!(sparse_set.get_dense().len(), 0);
    for i in 0..10 {
        sparse_set.insert(i, i*2);
    }
    assert_eq!(sparse_set.get_dense().len(), 10);
    for i in 0..10 {
        assert_eq!(*sparse_set.get(i).unwrap(), i*2);
    }
    sparse_set.remove(5);
    assert_eq!(sparse_set.get_dense().len(), 9);
    assert_eq!(sparse_set.get(5), None);
    for i in 0..10 {
        if i != 5 {
            assert_eq!(*sparse_set.get(i).unwrap(), i*2);
        } else {
            assert_eq!(sparse_set.get(i), None);
        }
    }
}

