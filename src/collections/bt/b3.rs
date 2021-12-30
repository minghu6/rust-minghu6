//! AKA 2-3 tree, order 3 of B Tree, so call it B3.

use std::{
    collections::VecDeque,
    ptr::{null, null_mut},
};

use either::Either;
use itertools::Itertools;

use super::{bst::rawst::RawSTNode, BTItem, BTNode, BT};
use crate::collections::{DictKey, Dictionary};
use crate::*;


////////////////////////////////////////////////////////////////////////////////
//// Structs

/// 2-3 Tree
pub struct B3<K, V> {
    root: *mut B3Node<K, V>,
}

pub struct B3Node<K, V> {
    keys: VecDeque<*mut K>,   // len: 1~2,
    values: VecDeque<*mut V>, // co-variant with keys: 1~2

    children: VecDeque<*mut Self>,
    paren: *mut Self,
}



////////////////////////////////////////////////////////////////////////////////
//// Implement
///


impl<'a, K: DictKey + 'a, V: 'a> B3Node<K, V> {
    pub fn new_value(key: K, value: V) -> *mut Self {
        let key = Box::into_raw(box key);
        let value = Box::into_raw(box value);

        Self::new_ptr(key, value)
    }

    pub fn new_ptr(key: *mut K, value: *mut V) -> *mut Self {
        Box::into_raw(box Self {
            keys: vecdeq![key],
            values: vecdeq![value],
            paren: null_mut(),
            children: vecdeq![],
        })
    }

    fn is_leaf(&self) -> bool {
        self.children.is_empty()
    }

    fn node_insert(&mut self, key: *mut K, value: *mut V) {
        let insert_idx = if let Some((i, _)) = self
            .keys
            .iter()
            .find_position(|&&here_k| unsafe { *key < *here_k })
        {
            i
        } else {
            self.keys.len()
        };

        self.keys.insert(insert_idx, key);
        self.values.insert(insert_idx, value);
    }

    unsafe fn connect_child_append(&mut self, child: *mut B3Node<K, V>) {
        if !child.is_null() {
            (*child).paren = self as *mut Self;
        }

        self.children.push_back(child);
    }

    unsafe fn connect_child_insert(
        &mut self,
        child: *mut B3Node<K, V>,
        idx: usize,
    ) {
        if !child.is_null() {
            (*child).paren = self as *mut Self;
        }

        self.children.insert(idx, child);
    }

    /// Result is Invalid B3 Node.
    unsafe fn remove_node(&mut self, remove_idx: usize) -> *mut B3Node<K, V> {
        let (key, val) = (
            self.keys.remove(remove_idx).unwrap(),
            self.values.remove(remove_idx).unwrap(),
        );

        B3Node::new_ptr(key, val)
    }

    unsafe fn merge_node(&mut self, income_node: *mut B3Node<K, V>) {
        let income_item_len = (*income_node).node_size();

        for _ in 0..income_item_len {
            self.node_insert(
                (*income_node).keys.pop_front().unwrap(),
                (*income_node).values.pop_front().unwrap()
            )
        }
    }
}



impl<'a, K: DictKey + 'a, V: 'a> BTNode<'a, K, V> for B3Node<K, V> {
    fn itself(&self) -> *const (dyn BTNode<'a, K, V> + 'a) {
        self as *const Self
    }

    fn null(&self) -> *const (dyn BTNode<'a, K, V> + 'a) {
        null::<Self>()
    }

    fn try_as_bst(
        &self,
    ) -> Result<*const (dyn super::bst::BSTNode<'a, K, V> + 'a), ()> {
        Err(())
    }

    fn order(&self) -> usize {
        3
    }

    fn child(&self, idx: usize) -> *mut (dyn BTNode<'a, K, V> + 'a) {
        if idx < self.children.len() {
            self.children[idx]
        } else {
            null_mut::<Self>()
        }
    }

    fn assign_child(
        &mut self,
        child: *mut (dyn BTNode<'a, K, V> + 'a),
        idx: usize,
    ) {
        if idx < self.children.len() {
            self.children[idx] = child as *mut Self;
        } else if idx == self.children.len() {
            self.children.push_back(child as *mut Self);
        } else {
            unreachable!()
        }
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

    fn key_ptr(&self, idx: usize) -> *mut K {
        if idx < self.keys.len() {
            self.keys[idx]
        } else {
            null_mut::<K>()
        }
    }

    fn assign_key_ptr(&mut self, idx: usize, key_ptr: *mut K) {
        if idx < self.keys.len() {
            self.keys[idx] = key_ptr;
        }
    }

    fn val_ptr(&self, idx: usize) -> *mut V {
        if idx < self.values.len() {
            self.values[idx]
        } else {
            null_mut::<V>()
        }
    }

    fn assign_val_ptr(&mut self, idx: usize, val_ptr: *mut V) {
        if idx < self.values.len() {
            self.values[idx] = val_ptr;
        }
    }

    fn height(&self) -> i32 {
        self.calc_height()
    }
}


impl<'a, K: DictKey + 'a, V: 'a> B3<K, V> {
    pub fn new() -> Self {
        Self { root: null_mut() }
    }

    /// Ordered Sequence
    pub fn bulk_load(seq: &mut dyn Iterator<Item = (K, V)>) -> Self {
        let mut b3 = Self::new();

        let mut seq =
            seq.map(|(k, v)| (Box::into_raw(box k), Box::into_raw(box v)));

        if let Some((k, v)) = seq.next() {
            b3.root = B3Node::new_ptr(k, v);
        }

        for (k, v) in seq.into_iter() {
            unsafe {
                let target_node = b3.maximum() as *mut B3Node<K, V>;

                (*target_node).node_insert(k, v);

                b3.promote(target_node);
            }
        }

        b3
    }

    /// Assume that key is unique from node's keys.
    ///
    /// this node is fullfilled (2 key)
    ///
    /// There are 4 case for its parent:
    ///
    /// (0), 0 key, root, -> 1 key 2 br (split)
    ///
    /// (1), 1 key 2 br,  -> 2 key 3 br (no-split)
    ///
    /// (2), 2 key 3 br,  -> 3 key 4 br (split)
    ///
    /// (3), 3 key 4 br (tmporary)      (split)
    unsafe fn promote(&mut self, x: *mut B3Node<K, V>) {
        if x.is_null() || !(*x).node_is_overfilled() {
            return;
        }

        let x_mid_key = (*x).keys.remove(1).unwrap();
        let x_mid_val = (*x).values.remove(1).unwrap();
        let left_sibling = (*x).remove_node(0);

        if !(*x).is_leaf() {
            (*left_sibling).connect_child_append((*x).children.pop_front().unwrap());
            (*left_sibling).connect_child_append((*x).children.pop_front().unwrap());
        }

        if (*x).paren.is_null() {
            self.root = B3Node::new_ptr(x_mid_key, x_mid_val);

            (*self.root).connect_child_append(left_sibling);
            (*self.root).connect_child_append(x);

        } else {
            let x_idx = (*(*x).paren).index_of_child(x);
            (*(*x).paren).connect_child_insert(left_sibling, x_idx);
            (*(*x).paren).node_insert(x_mid_key, x_mid_val);

            self.promote((*x).paren);
        };

        // case-0, root is null
        // case-1,
        //     8              (3,8)
        //   /    \   ==>    /  |  \
        // (2,3,5) 9        2   5   9
        //
        // case-2, 2 key 3 br => 3 key 4 br, split and continue to promote.
        //   3,    8            3,5,8           5
        //  /   |   \   ==>    / | | \   ==>   / \
        // 2 (4,5,7) 9        2  4 7  9       3   8
        //                                   / \ / \
        //                                  2  4 7  9

    }

    unsafe fn unpromote(&mut self, leaf: *mut B3Node<K, V>) {
        debug_assert!(!leaf.is_null());

        if (*leaf).node_size() == 0 {
            let paren = (*leaf).paren;

            if paren.is_null() {
                self.root = null_mut();
            } else {
                let leaf_idx = (*paren).index_of_child(leaf);
                self.unpromote_(paren, leaf_idx);
            }
        }
    }

    unsafe fn unpromote_(&mut self, paren: *mut B3Node<K, V>, leaf_idx: usize) {
        debug_assert!(!paren.is_null());

        // First check 2-key-val sibling
        // Split && Redistribute
        if !(*paren).child(leaf_idx + 1).is_null() {
            if (*(*paren).child(leaf_idx + 1)).node_size() > 1 {
                // split
                let sibling = (*paren).child(leaf_idx + 1) as *mut B3Node<K, V>;
                let split_sibling = (*sibling).remove_node(0);
                (*paren).children.remove(leaf_idx);
                (*paren).connect_child_insert(split_sibling, leaf_idx);

                // redistribute
                let mut paren_item = BTItem::new(paren, leaf_idx);
                let mut split_sibling_item = BTItem::new(split_sibling, 0);
                BTItem::swap(&mut paren_item, &mut split_sibling_item);

                if !(*sibling).is_leaf() {
                    let sibling_child =  (*sibling).children.pop_front().unwrap();
                    (*split_sibling).children.push_back(null_mut());
                    (*split_sibling).connect_child_append(sibling_child);

                    self.unpromote_(split_sibling, 0)
                }

                return;
            }
        }

        if leaf_idx > 0 && !(*paren).child(leaf_idx - 1).is_null() {
            if (*(*paren).child(leaf_idx - 1)).node_size() > 1 {
                // split
                let sibing = (*paren).child(leaf_idx - 1) as *mut B3Node<K, V>;
                let split_sibling = (*sibing).remove_node((*sibing).node_size() - 1);
                (*paren).children.remove(leaf_idx);
                (*paren).connect_child_insert(split_sibling, leaf_idx);

                // redistribute (including subtree)
                let mut paren_item = BTItem::new(paren, leaf_idx - 1);
                let mut split_sibling_item = BTItem::new(split_sibling, 0);
                BTItem::swap(&mut paren_item, &mut split_sibling_item);

                if !(*sibing).is_leaf() {
                    let sibling_child = (*sibing).children.pop_back().unwrap();
                    (*split_sibling).connect_child_append(sibling_child);
                    (*split_sibling).children.push_back(null_mut());

                    self.unpromote_(split_sibling, 1)
                }

                return;
            }
        }


        // For 1-key-val node
        // Move down && Merge (Recursive)
        let sibling;
        if !(*paren).child(leaf_idx + 1).is_null() {
            // move down
            sibling = (*paren).child(leaf_idx + 1) as *mut B3Node<K, V>;
            (*paren).children.remove(leaf_idx);
            let mvd_sibling = (*paren).remove_node(leaf_idx);

            // merge
            (*sibling).merge_node(mvd_sibling);

            if (*paren).node_size() == 0 {
                self.subtree_shift(paren, sibling);
            }

            if !(*sibling).is_leaf() {
                (*sibling).children.insert(0, null_mut());
                self.unpromote_(sibling, 0 );
            }

            return;
        }

        if leaf_idx > 0 && !(*paren).child(leaf_idx - 1).is_null() {
            // move down
            sibling = (*paren).child(leaf_idx - 1) as *mut B3Node<K, V>;
            (*paren).children.remove(leaf_idx);
            let mvd_sibling = (*paren).remove_node(leaf_idx - 1);

            // merge
            (*sibling).merge_node(mvd_sibling);

            if (*paren).node_size() == 0 {
                self.subtree_shift(paren, sibling);
            }

            if !(*sibling).is_leaf() {
                (*sibling).children.push_back(null_mut());
                self.unpromote_(sibling, (*sibling).children.len() - 1);
            }

            return;
        }

        unreachable!()
    }
}


impl<'a, K: DictKey + 'a, V: 'a> Dictionary<K, V> for B3<K, V> {
    fn insert(&mut self, key: K, value: V) -> bool {
        if self.root().is_null() {
            self.assign_root(B3Node::new_value(key, value));
            return true;
        }

        unsafe {
            let x = (*self.root()).search_approximately(&key);

            if (*x).node_contains(&key) {
                return false;
            }

            // box key and value
            let key = Box::into_raw(box key);
            let value = Box::into_raw(box value);

            let x_self = x as *mut B3Node<K, V>;
            (*x_self).node_insert(key, value);

            self.promote(x_self);
        }

        true
    }

    fn remove(&mut self, key: &K) -> Option<V> {
        let res = self.search_approximately(key) as *mut B3Node<K, V>;

        if res.is_null() {
            return None;
        }

        unsafe {
            if let Some(idx) = (*res).find_pos_of_key(key) {
                // if !(*res).paren.is_null() {
                //     println!("REMOVE RES: {}", (*res).format_keys());

                //     println!("REMOVE RES PAREN:", );
                //     (*(*res).paren).just_echo_stdout();
                // }

                let leaf_item = (*res).swap_to_leaf(idx);
                let leaf = leaf_item.node as *mut B3Node<K, V>;

                let _key = (*leaf).keys.remove(leaf_item.idx).unwrap();
                let val = (*leaf).values.remove(leaf_item.idx).unwrap();

                self.unpromote(leaf);

                Some(*Box::from_raw(val))
            } else {
                None
            }
        }
    }

    fn modify(&mut self, key: &K, value: V) -> bool {
        let res = self.search_approximately(key) as *mut B3Node<K, V>;

        if res.is_null() {
            false
        } else {
            unsafe {
                if let Some((idx, _)) = (*res)
                    .keys
                    .iter_mut()
                    .find_position(|&&mut here_key| *here_key == *key)
                {
                    (*res).values[idx] = Box::into_raw(box value);
                    true
                } else {
                    false
                }
            }
        }
    }

    fn lookup(&self, key: &K) -> Option<&V> {
        let res = self.search_approximately(key) as *const B3Node<K, V>;

        if res.is_null() {
            None
        } else {
            unsafe {
                if let Some((idx, _)) = (*res)
                    .keys
                    .iter()
                    .find_position(|&&here_key| *here_key == *key)
                {
                    Some(&*(*res).values[idx])
                } else {
                    None
                }
            }
        }
    }

    fn lookup_mut(&mut self, key: &K) -> Option<&mut V> {
        let res = self.search_approximately(key) as *mut B3Node<K, V>;

        if res.is_null() {
            None
        } else {
            unsafe {
                if let Some((idx, _)) = (*res)
                    .keys
                    .iter_mut()
                    .find_position(|&&mut here_key| *here_key == *key)
                {
                    Some(&mut *(*res).values[idx])
                } else {
                    None
                }
            }
        }
    }

    fn self_validate(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.basic_self_validate()
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



#[cfg(test)]
mod tests {
    use rand::{prelude::SliceRandom, thread_rng};

    use crate::{
        collections::{
            bt::{b3::B3, BT},
            Dictionary,
        },
        test::{
            dict::{DictProvider, GetKey, Inode, InodeProvider},
            Provider,
        },
    };


    #[test]
    fn test_b3_fixeddata_case_0() {
        let mut b3 = B3::<i32, ()>::new();

        let dict = &mut b3 as &mut dyn Dictionary<i32, ()>;

        dict.insert(92, ());
        dict.insert(917, ());
        dict.insert(765, ());
        dict.insert(901, ());
        dict.insert(345, ());
        dict.insert(645, ());
        dict.insert(794, ());
        dict.insert(643, ());
        dict.insert(540, ());
        dict.insert(81, ());
        dict.insert(174, ());
        dict.insert(340, ());
        dict.insert(923, ());
        dict.insert(88, ());
        dict.insert(226, ());
        dict.insert(126, ());
        dict.insert(784, ());
        dict.insert(943, ());
        dict.insert(332, ());
        dict.insert(885, ());


        assert!(dict.remove(&794).is_some());
        assert!(dict.self_validate().is_ok());

        assert!(dict.remove(&81).is_some());
        dict.self_validate().unwrap();

        assert!(dict.remove(&901).is_some());
        dict.self_validate().unwrap();

        assert!(dict.remove(&643).is_some());
        dict.self_validate().unwrap();

        assert!(dict.remove(&345).is_some());
        dict.self_validate().unwrap();

        assert!(dict.remove(&645).is_some());
        dict.self_validate().unwrap();

        assert!(dict.remove(&92).is_some());
        dict.self_validate().unwrap();

        assert!(dict.remove(&784).is_some());
        dict.self_validate().unwrap();

        assert!(dict.remove(&885).is_some());
        dict.self_validate().unwrap();

        assert!(dict.remove(&332).is_some());
        dict.self_validate().unwrap();

        b3.just_echo_stdout();
    }

    #[test]
    fn test_b3_fixeddata_case_1() {
        let mut b3 = B3::<i32, ()>::new();

        let dict = &mut b3 as &mut dyn Dictionary<i32, ()>;

        dict.insert(75, ());
        dict.insert(60, ());
        dict.insert(98, ());
        dict.insert(91, ());
        dict.insert(59, ());
        dict.insert(92, ());
        dict.insert(45, ());
        dict.insert(2, ());
        dict.insert(4, ());
        dict.insert(13, ());

        assert!(dict.lookup(&75).is_some());
        assert!(dict.remove(&75).is_some());
        assert!(dict.lookup(&75).is_none());
        dict.self_validate().unwrap();

        assert!(dict.remove(&60).is_some());
        dict.self_validate().unwrap();

        assert!(dict.remove(&98).is_some());
        dict.self_validate().unwrap();

        assert!(dict.remove(&59).is_some());
        dict.self_validate().unwrap();

        assert!(dict.remove(&92).is_some());
        dict.self_validate().unwrap();

        assert!(dict.remove(&45).is_some());
        dict.self_validate().unwrap();

        assert!(dict.remove(&2).is_some());
        dict.self_validate().unwrap();

        assert!(dict.remove(&4).is_some());
        dict.self_validate().unwrap();

        assert!(dict.remove(&13).is_some());
        dict.self_validate().unwrap();


        b3.just_echo_stdout();
    }


    #[test]
    pub(crate) fn test_b3_randomdata() {
        let provider = InodeProvider {};

        (&provider as &dyn DictProvider<u32, Inode>)
            .test_dict(|| box B3::new());
    }

    #[test]
    fn test_b3_bulk_load() {
        let mut seq = (10..110).step_by(10).map(|n| (n, ()));

        let b3 = B3::<i32, ()>::bulk_load(&mut seq);

        b3.self_validate().unwrap();
        b3.just_echo_stdout();
    }


    ///
    /// Debug B3 entry
    ///
    #[ignore]
    #[test]
    fn hack_b3() {
        for _ in 0..50 {
        let batch_num = 20;
        let mut collected_elems = vec![];
        let mut keys = vec![];
        let provider = InodeProvider {};
        let mut dict = B3::new();

        // Create
        let mut i = 0;
        while i < batch_num {
            let e = provider.get_one();
            let k = e.get_key();
            if keys.contains(&k) {
                continue;
            }

            keys.push(k.clone());
            collected_elems.push(e.clone());

            assert!(dict.insert(k, e));

            println!("insert {}: {:?}", i, k);

            assert!(dict.lookup(&keys.last().unwrap()).is_some());

            dict.self_validate().unwrap();

            i += 1;
        }

        collected_elems.shuffle(&mut thread_rng());

        // Remove-> Verify
        for i in 0..batch_num {
            let e = &collected_elems[i];
            let k = &e.get_key();

            println!("remove i: {}, k: {}", i, k);
            assert!(dict.lookup(k).is_some());
            let res = dict.remove(k);
            if res.is_none() {
                assert!(dict.lookup(k).is_some());
                dict.remove(k);
            }

            // assert!(dict.remove(k).is_some());
            assert!(!dict.lookup(k).is_some());
            dict.self_validate().unwrap();

        }
    }
    }
}
