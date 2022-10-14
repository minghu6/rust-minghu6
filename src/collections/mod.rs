use std::fmt::Debug;


pub mod bloom_filter;
pub mod bt;
pub mod compact;
pub mod persistent;
pub mod graph;
pub mod easycoll;
pub mod union_find;
pub mod aux;


////////////////////////////////////////////////////////////////////////////////
//// Common Traits


/// 1. add a pair to the collection;
/// 2. remove a pair from the collection;
/// 3. modify an existing pair;
/// 4. lookup a value associated with a particular key.
pub trait Dictionary<K: DictKey, V> {
    // need update or else?
    fn insert(&mut self, key: K, value: V) -> bool;

    // exist or else
    fn remove(&mut self, key: &K) -> Option<V>;

    // exist or else
    fn modify(&mut self, key: &K, value: V) -> bool;

    fn lookup(&self, key: &K) -> Option<&V>;

    fn lookup_mut(&mut self, key: &K) -> Option<&mut V>;


    // check if dict's structure looks like it's expected.
    fn self_validate(&self) -> Result<(), Box<dyn std::error::Error>>;

}

pub trait DictKey = Eq + Ord + std::fmt::Debug;


// pub trait Adictionary<K, V> {
//     // need update or else?
//     fn insert(&mut self, key: K, value: V) -> bool;

//     // exist or else
//     fn remove(&mut self, key: &K) -> Option<Arc<V>>;

//     // exist or else
//     fn modify(&mut self, key: &K, value: V) -> bool;

//     fn lookup(&self, key: &K) -> Option<Arc<V>>;

//     fn self_validate(&self);

// }


pub trait Heap<W: Weight, T> {
    // max for max-heap and min for min-heap respectly.
    fn top(&self) -> Option<&T>;

    fn pop_top(&mut self) -> Option<T>;

    fn insert(&mut self, w: W, item: T);

}

pub trait Weight = Ord + Debug;


pub trait Collection {
    fn len(&self) -> usize;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}


////////////////////////////////////////////////////////////////////////////////
//// Implments



////////////////////////////////////////////////////////////////////////////////
//// Utils

pub fn as_ptr<T>(value: T) -> *mut T {
    Box::into_raw(box value)
}
