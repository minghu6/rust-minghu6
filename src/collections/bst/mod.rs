pub mod avl;


use std::fmt::Debug;

use super::Dictionary;


/// LF(key) < MID(key) < RH(key)
pub trait BST<K: BSTKey, V>: Dictionary<K, V> {

}

pub trait BSTKey = Eq + Ord + Debug;
