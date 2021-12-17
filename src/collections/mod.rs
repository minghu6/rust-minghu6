

pub mod bloom_filter;
pub mod bt;
pub mod compact;

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





////////////////////////////////////////////////////////////////////////////////
//// Implments

