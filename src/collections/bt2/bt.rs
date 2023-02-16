//! B Tree
//!

use std::fmt::*;

use m6coll::KVEntry;

use super::*;

impl_node!();
impl_tree!(BT {});


macro_rules! attr2 {
    ($node:expr, $attr:ident, $ty:ty) => {
        {
            if let Some(_unr) = $node.clone().0 {
                let _bor = _unr.as_ref().borrow();
                let _attr = (&_bor.$attr) as *const $ty;
                drop(_bor);
                unsafe { &*_attr }
            }
            else {
                panic!("Access {} on None", stringify!($attr));
            }
        }
    };
}

////////////////////////////////////////////////////////////////////////////////
//// Structure

struct Node_<K, V> {
    entries: Vec<KVEntry<K, V>>,
    children: Vec<Node<K, V>>,
    paren: WeakNode<K, V>
}



////////////////////////////////////////////////////////////////////////////////
//// Implementation

impl<K, V> Node<K, V> {
    pub fn keys(&self) -> impl Iterator<Item = &K> {
        // .as_ref().borrow().entries as * const Vec<KVEntry<K, V>>;

        attr2!(self, entries, Vec<KVEntry<K, V>>).iter().map(|entry| &entry.0)
    }
}


impl<K: Debug, V> Debug for Node<K, V> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        // write!(f, "{:?}: {:?}", self.key,)

        Ok(())
    }
}



impl<K, V> Node_<K, V> {

}
