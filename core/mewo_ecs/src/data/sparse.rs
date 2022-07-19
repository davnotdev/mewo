use std::marker::PhantomData;

//  Note that K is only here for code readability.
#[derive(Clone)]
pub struct SparseSet<K, V> {
    //  (K, data)
    dense: Vec<(usize, V)>,
    sparse: Vec<Option<usize>>,
    phantom: PhantomData<K>,
}

impl<K, V> SparseSet<K, V> {
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
            phantom: PhantomData,
        }
    }

    pub fn insert(&mut self, idx: usize, data: V) {
        if idx >= self.sparse.len() {
            self.sparse.resize(idx + 1, None);
        }
        self.dense.push((idx, data));
        *self.sparse.get_mut(idx).unwrap() = Some(self.dense.len() - 1);
    }

    pub fn remove(&mut self, idx: usize) -> Option<V> {
        if idx >= self.sparse.len() {
            None
        } else {
            let dense_idx =
                if let Some(idx) = std::mem::replace(self.sparse.get_mut(idx).unwrap(), None) {
                    idx
                } else {
                    return None;
                };
            if self.dense.len() != 1 {
                let key = self.dense.get(self.dense.len() - 1).unwrap().0;
                *self.sparse.get_mut(idx).unwrap() = None;
                *self.sparse.get_mut(key).unwrap() = Some(dense_idx);
            }
            let (_, data) = self.dense.swap_remove(dense_idx);
            Some(data)
        }
    }

    fn get_from_sparse(&self, idx: usize) -> Option<usize> {
        if let Some(s) = self.sparse.get(idx) {
            *s
        } else {
            None
        }
    }

    pub fn get(&self, idx: usize) -> Option<&V> {
        if let Some(idx) = self.get_from_sparse(idx) {
            return Some(&self.dense.get(idx)?.1);
        }
        None
    }

    pub fn get_mut(&mut self, idx: usize) -> Option<&mut V> {
        if let Some(idx) = self.get_from_sparse(idx) {
            return Some(&mut self.dense.get_mut(idx)?.1);
        }
        None
    }

    pub fn get_dense(&self) -> &Vec<(usize, V)> {
        &self.dense
    }
}

impl<K, V> std::fmt::Debug for SparseSet<K, V>
where
    V: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "SparseSet<{}, {}> {{\n",
            std::any::type_name::<K>(),
            std::any::type_name::<V>()
        )?;
        for (k, v) in self.get_dense() {
            write!(f, "\t{} -> {:?}\n", k, v)?;
        }
        write!(f, "}}")
    }
}

#[test]
fn test_sparse_set() {
    let mut sparse_set = SparseSet::<usize, usize>::create();
    assert_eq!(sparse_set.get_dense().len(), 0);
    for i in 0..10 {
        sparse_set.insert(i, i * 2);
    }
    assert_eq!(sparse_set.get_dense().len(), 10);
    for i in 0..10 {
        assert_eq!(*sparse_set.get(i).unwrap(), i * 2);
    }
    sparse_set.remove(5);
    assert_eq!(sparse_set.get_dense().len(), 9);
    assert_eq!(sparse_set.get(5), None);
    for i in 0..10 {
        if i != 5 {
            assert_eq!(*sparse_set.get(i).unwrap(), i * 2);
        } else {
            assert_eq!(sparse_set.get(i), None);
        }
    }
}
