use std::fmt::Debug;


pub mod bloom_filter;
pub mod bt;
pub mod compact;
pub mod persistent;
pub mod graph;
pub mod easycoll;
pub mod union_find;
pub mod heap;
pub mod aux;
pub mod bst2;
pub mod bt2;


////////////////////////////////////////////////////////////////////////////////
//// Common Trait


/// 1. add a pair to the collection;
/// 2. remove a pair from the collection;
/// 3. modify an existing pair;
/// 4. lookup a value associated with a particular key.
pub trait Dictionary<K: CollKey, V> {
    /// need update or else?
    ///
    /// , return instead of replace to be friendly for BST
    ///
    /// loopup is often cheap moreover
    fn insert(&mut self, key: K, value: V) -> bool;

    /// exist or else
    fn remove(&mut self, key: &K) -> Option<V>;

    /// exist or else
    fn modify(&mut self, key: &K, value: V) -> bool;

    fn get(&self, key: &K) -> Option<&V>;

    fn get_mut(&mut self, key: &K) -> Option<&mut V>;

    // check if dict's structure looks like it's expected.
    fn self_validate(&self) -> Result<(), Box<dyn std::error::Error>>;
}

pub trait CollKey = Ord + Debug;


pub trait Heap<K: CollKey, T: CollKey>: Coll {
    // max for max-heap and min for min-heap respectly.
    fn top(&self) -> Option<&T>;

    fn pop(&mut self) -> Option<T>;

    fn push(&mut self, key: K, val: T);
}


pub trait AdvHeap<K: CollKey, T: CollKey>: Heap<K, T> {
    // decrease key
    fn update(&mut self, index: K, val: T) -> Option<T>;
}


pub trait Coll {
    fn len(&self) -> usize;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}


////////////////////////////////////////////////////////////////////////////////
//// Utils
