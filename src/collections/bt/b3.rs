//! AKA 2-3 tree, order 3 of B Tree, so call it B3.

use std::{
    collections::VecDeque,
    ptr::{null, null_mut},
};

use itertools::Itertools;

use super::{bst::rawst::RawSTNode, BTNode, BT};
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

    /// 3key, 4br => 2x2 + 1
    unsafe fn inherrit_children(
        heir: *mut B3Node<K, V>,
        left: *mut B3Node<K, V>,
        right: *mut B3Node<K, V>,
    ) {
        if !(*heir).is_leaf() {
            (*left).children.clear();
            (*left).connect_child_append((*heir).child(0) as *mut Self);
            (*left).connect_child_append((*heir).child(1) as *mut Self);

            (*right).children.clear();
            (*right).connect_child_append((*heir).child(2) as *mut Self);
            (*right).connect_child_append((*heir).child(3) as *mut Self);
        }
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

    fn key(&self, idx: usize) -> Option<&K> {
        if idx >= self.keys.len() {
            return None;
        }

        Some(unsafe { &*self.keys[idx] })
    }

    fn value(&self, idx: usize) -> &V {
        unsafe { &*self.values[idx] }
    }

    fn value_mut(&mut self, idx: usize) -> &mut V {
        unsafe { &mut *self.values[idx] }
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

        let x_left_key = (*x).keys.pop_front().unwrap();
        let x_left_val = (*x).values.pop_front().unwrap();
        let xlv_left = B3Node::new_ptr(x_left_key, x_left_val);

        // let x_mid_key = (*x).keys[0];
        // let x_mid_val = (*x).values[0];

        let x_right_key = (*x).keys.pop_back().unwrap();
        let x_right_val = (*x).values.pop_back().unwrap();
        let xlv_right = B3Node::new_ptr(x_right_key, x_right_val);

        // case-0, root is null
        // case-1,
        //     8              (3,8)
        //   /    \   ==>    /  |  \
        // (2,3,5) 9        2   5   9
        //
        let paren =
            self.shift_onechild_with_twochildren(x, xlv_left, xlv_right);
        self.promote(paren);

        //   3,    8            3,5,8           5
        //  /   |   \   ==>    / | | \   ==>   / \
        // 2 (4,5,7) 9        2  4 7  9       3   8
        //                                   / \ / \
        //                                  2  4 7  9
        // case-2, 2 key 3 br => 3 key 4 br, split and continue to promote.
    }


    /// x contains mid key and mid val
    unsafe fn shift_onechild_with_twochildren(
        &mut self,
        x: *mut B3Node<K, V>,
        xlv_left: *mut B3Node<K, V>,
        xlv_right: *mut B3Node<K, V>,
    ) -> *mut B3Node<K, V> {
        let mut x_paren = (*x).paren;

        if x_paren.is_null() {
            self.root = B3Node::new_ptr((*x).keys[0], (*x).values[0]);
            x_paren = self.root;

            (*self.root).connect_child_append(xlv_left);
            (*self.root).connect_child_append(xlv_right);
        } else {
            let x_idx = (*x_paren).index_of_child(x);
            (*x_paren).children.remove(x_idx);

            (*x_paren).connect_child_insert(xlv_right, x_idx);
            (*x_paren).connect_child_insert(xlv_left, x_idx);

            (*x_paren).node_insert((*x).keys[0], (*x).values[0]);
        }

        B3Node::inherrit_children(x, xlv_left, xlv_right);

        x_paren
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
        todo!()
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
                // println!("{:?}", (*res).format_keys());

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

        dict.insert(9, ());
        assert!(dict.lookup(&9).is_some());
        assert!(dict.self_validate().is_ok());

        dict.insert(5, ());
        assert!(dict.lookup(&5).is_some());
        dict.self_validate().unwrap();

        dict.insert(8, ());
        assert!(dict.lookup(&8).is_some());
        dict.self_validate().unwrap();

        dict.insert(3, ());
        assert!(dict.lookup(&3).is_some());
        dict.self_validate().unwrap();

        dict.insert(2, ());
        assert!(dict.lookup(&2).is_some());
        dict.self_validate().unwrap();

        dict.insert(4, ());
        assert!(dict.lookup(&4).is_some());
        dict.self_validate().unwrap();

        dict.insert(7, ());
        assert!(dict.lookup(&7).is_some());
        dict.self_validate().unwrap();

        b3.just_echo_stdout();
    }


    #[test]
    fn test_b3_fixeddata_case_1() {
        let mut seq = (10..110).step_by(10).map(|n| (n, ()));

        let b3 = B3::<i32, ()>::bulk_load(&mut seq);


        b3.just_echo_stdout();
    }

    #[test]
    fn test_b3_fixeddata_case_2() {
        let mut b3 = B3::<i32, ()>::new();

        let dict = &mut b3 as &mut dyn Dictionary<i32, ()>;

        dict.insert(74, ());
        dict.insert(27, ());
        dict.insert(14, ());
        dict.insert(1, ());
        dict.insert(42, ());
        dict.insert(9, ());
        dict.insert(19, ());
        dict.insert(53, ());
        dict.insert(40, ());
        dict.insert(88, ());
        dict.insert(56, ());
        dict.insert(68, ());

        assert!(dict.lookup(&68).is_some());
        dict.self_validate().unwrap();


        b3.just_echo_stdout();
    }


    #[test]
    pub(crate) fn test_b3_randomdata() {
        let provider = InodeProvider {};

        (&provider as &dyn DictProvider<u32, Inode>)
            .test_dict(|| box B3::new());
    }

    // ///
    // /// Debug B3 entry
    // ///
    // #[ignore]
    // #[test]
    // fn hack_b3() {
    //     for _ in 0..20 {
    //     let batch_num = 55;
    //     let mut collected_elems = vec![];
    //     let mut keys = vec![];
    //     let provider = InodeProvider {};
    //     let mut dict = B3::new();

    //     // Create
    //     let mut i = 0;
    //     while i < batch_num {
    //         let e = provider.get_one();
    //         let k = e.get_key();
    //         if keys.contains(&k) {
    //             continue;
    //         }

    //         keys.push(k.clone());
    //         collected_elems.push(e.clone());

    //         assert!(dict.insert(k, e));
    //         assert!(dict.lookup(&keys.last().unwrap()).is_some());

    //         dict.self_validate().unwrap();

    //         i += 1;
    //     }

    //     let mut dict_debug = dict.clone();

    //     collected_elems.shuffle(&mut thread_rng());

    //     // Remove-> Verify
    //     for i in 0..batch_num {
    //         let e = &collected_elems[i];
    //         let k = &e.get_key();

    //         assert!(dict.remove(k).is_some());
    //         assert!(!dict.lookup(k).is_some());

    //         if let Ok(_res) = dict.self_validate() {
    //         } else {
    //             // restore the scene
    //             println!("{}", i);

    //             println!("DEBUG: {}", dict_debug.total());
    //             // dict_debug.echo_stdout();

    //             println!("ORIGIN: {}", dict.total());
    //             // dict.echo_stdout();

    //             for j in 0..i {
    //                 let e = &collected_elems[j];
    //                 let k = &e.get_key();

    //                 assert!(dict_debug.remove(k).is_some());
    //                 assert!(!dict_debug.lookup(k).is_some());
    //                 dict_debug.self_validate().unwrap();
    //             }

    //             unsafe {
    //                 let target = (*dict_debug.search_approximately(k)).try_as_bst_mut().unwrap();
    //                 let target_paren = (*target).paren_bst();

    //                 println!("Target: {:?}", k);
    //                 BSTNode::just_echo_stdout(&*target_paren);
    //             }

    //             dict_debug.remove(k).unwrap();
    //             dict_debug.self_validate().unwrap();
    //         }
    //     }}
    // }
}
