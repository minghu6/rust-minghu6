use std::{sync::Arc};

pub mod bloom_filter;
pub mod bst;


////////////////////////////////////////////////////////////////////////////////
//// Common Traits


/// 1. add a pair to the collection;
/// 2. remove a pair from the collection;
/// 3. modify an existing pair;
/// 4. lookup a value associated with a particular key.
pub trait Dictionary<K, V> {
    // need update or else?
    fn insert(&mut self, key: K, value: V) -> bool;

    // exist or else
    fn remove(&mut self, key: &K) -> Option<V>;

    // exist or else
    fn modify(&mut self, key: &K, value: V) -> bool;

    fn lookup(&self, key: &K) -> Option<&V>;

    fn self_validate(&self);

}


pub trait Adictionary<K, V> {
    // need update or else?
    fn insert(&mut self, key: K, value: V) -> bool;

    // exist or else
    fn remove(&mut self, key: &K) -> Option<Arc<V>>;

    // exist or else
    fn modify(&mut self, key: &K, value: V) -> bool;

    fn lookup(&self, key: &K) -> Option<Arc<V>>;

    fn self_validate(&self);

}


/// Binary Tree
pub trait ABT<K, V> {
    fn left(& self) -> Option<&dyn ABT<K, V>>;


}


pub trait BT<'a, K, V> {
    fn left(&self) -> *mut (dyn BT<'a, K, V> + 'a);
    fn right(&self) -> *mut (dyn BT<'a, K, V> + 'a);
    fn paren(&self) -> *mut (dyn BT<'a, K, V> + 'a);

    fn key(&self) -> &K;
}



pub struct BytesSet {
    storage: [u128;2]
}

impl BytesSet {
    pub fn new() -> Self {
        BytesSet {
            storage: [0; 2]
        }
    }

    #[inline]
    pub fn insert(&mut self, elem: &u8) {
        let (i, shift) = get_i_and_shift(elem);

        self.storage[i] |= 1u128 << shift;
    }

    #[inline]
    pub fn contains(&self, elem: &u8) -> bool {
        let (i, shift) = get_i_and_shift(elem);
        (self.storage[i] & (1u128 << shift)) != 0
    }
}

#[inline]
fn get_i_and_shift(elem: &u8) -> (usize, u8) {
    let i;
    let shift;
    if *elem > 127 {
        shift = elem - 128;
        i = 1;
    } else {
        shift = *elem;
        i = 0;
    }

    (i, shift)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn bytes_set_op_works() {
        let mut bs = BytesSet::new();
        for i in 0..255 {
            bs.insert(&i);

            assert!(bs.contains(&i));
            assert!(!bs.contains(&(i+1)));
        }
    }
}