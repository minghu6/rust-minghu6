//! AKA 2-3 tree, order 3 of B Tree, so call it B3.

use std::ptr::{null, null_mut};

use crate::collections::{DictKey, Dictionary};

use super::{BT, BTNode, bst::rawst::RawSTNode};


////////////////////////////////////////////////////////////////////////////////
//// Structs

/// 2-3 Tree
pub struct B3<K, V> {
    root: *mut B3Node<K, V>
}

pub struct B3Node<K, V> {
    keys: [*mut K; 2],
    values: [*mut V; 2],

    children: [*mut Self; 3],
    paren: *mut Self
}




////////////////////////////////////////////////////////////////////////////////
//// Implement

impl<'a, K: DictKey + 'a, V: 'a> B3Node<K, V> {
    pub fn new(key: K, value: V) -> *mut Self {
        let key = Box::into_raw(box key);
        let value = Box::into_raw(box value);

        Box::into_raw(box Self {
            keys: [key, null_mut()],
            values: [value, null_mut()],
            paren: null_mut(),
            children: [null_mut(); 3],
        })
    }
}



impl<'a, K: DictKey + 'a, V: 'a> BTNode<'a, K, V> for B3Node<K, V> {
    fn itself(&self) -> *const (dyn BTNode<'a, K, V> + 'a) {
        self as *const Self
    }

    fn null(&self) -> *const (dyn BTNode<'a, K, V> + 'a) {
        null::<Self>()
    }

    fn try_as_bst(&self) -> Result<*const (dyn super::bst::BSTNode<'a, K, V> + 'a), ()> {
        Err(())
    }

    fn order(&self) -> usize {
        3
    }

    fn child(&self, idx: usize) -> *mut (dyn BTNode<'a, K, V> + 'a) {
        self.children[idx]
    }

    fn assign_child(&mut self, child: *mut (dyn BTNode<'a, K, V> + 'a), idx: usize) {
        self.children[idx] = child as *mut Self;
    }

    fn assign_value(&mut self, value: V, idx: usize) {
        self.values[idx] = Box::into_raw(box value)
    }

    fn assign_paren(&mut self, paren: *mut (dyn BTNode<'a, K, V> + 'a)) {
        self.paren = paren as *mut Self;
    }

    fn paren(&self) -> *mut (dyn BTNode<'a, K, V> + 'a) {
        self.paren
    }

    fn key(&self, idx: usize) -> Option<&K> {
        if idx > 2 {
            return None;
        }

        if !self.keys[idx].is_null() {
            Some(unsafe{ &* self.keys[idx] })
        } else {
            None
        }
    }

    fn value(&self, idx: usize) -> &V {
        unsafe{ &*self.values[idx] }
    }

    fn value_mut(&self, idx: usize) -> &mut V {
        unsafe{ &mut *self.values[idx] }
    }

    fn height(&self) -> i32 {
        self.calc_height()
    }
}


impl<'a, K: DictKey + 'a, V: 'a> B3<K, V> {

}


impl<'a, K: DictKey + 'a, V: 'a> Dictionary<K, V> for B3<K, V> {
    fn insert(&mut self, key: K, value: V) -> bool {
        let new_node = B3Node::new(key, value);

        if self.root.is_null() {

        }

        todo!()
    }

    fn remove(&mut self, key: &K) -> Option<V> {
        todo!()
    }

    fn modify(&mut self, key: &K, value: V) -> bool {
        todo!()
    }

    fn lookup(&self, key: &K) -> Option<&V> {
        todo!()
    }

    fn lookup_mut(&mut self, key: &K) -> Option<&mut V> {
        todo!()
    }

    fn self_validate(&self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}


impl<'a, K: DictKey + 'a, V: 'a> BT<'a, K, V> for B3<K, V> {
    fn order(&self) -> usize {
        3
    }

    fn root(&self) -> *mut (dyn BTNode<'a, K, V> + 'a) {
        self.root
    }

    fn assign_root(&mut self, root: *mut (dyn BTNode<'a, K, V> + 'a)) {
        self.root = root as *mut B3Node<K, V>;
    }
}

