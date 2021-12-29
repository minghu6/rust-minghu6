//! Left Learning Red Black Tree
//!
//! ref: https://oi-wiki.org/ds/llllrbt/
//!
//! Painting edge instead of node (node's color indicates it's edge to parent)
//! Restrict black children are either black-black or red-black (red is only left)
//!
//! red link same node's item of 2-4 tree, black link different node respectly.


use std::cmp::max;
use std::fmt;
use std::fmt::Write;
use std::ptr::{null, null_mut};

use either::Either;
use itertools::Itertools;
use serde::de::DeserializeSeed;
use serde::{Deserialize, Deserializer, Serialize};

use super::rb::Color;
use super::{BSTNode, BST};
use crate::collections::bt::{BTNode, BT};
use crate::collections::{DictKey, Dictionary};
use crate::etc::Reverse;
use crate::*;


////////////////////////////////////////////////////////////////////////////////
//// Struct
////


///
pub struct LLRB<K, V> {
    root: *mut LLRBNode<K, V>,
}


struct LLRBNode<K, V> {
    left: *mut Self,
    right: *mut Self,
    paren: *mut Self,
    color: Color,
    key: *mut K,
    value: *mut V,
}


////////////////////////////////////////////////////////////////////////////////
//// Implement

fn is_black<K, V>(node: *mut LLRBNode<K, V>) -> bool {
    unsafe { node.is_null() || (*node).color == Color::BLACK }
}

fn is_red<K, V>(node: *mut LLRBNode<K, V>) -> bool {
    !is_black(node)
}

fn set_black<K, V>(node: *mut LLRBNode<K, V>) {
    unsafe {
        if !node.is_null() {
            (*node).color = Color::BLACK
        }
    }
}

fn set_red<K, V>(node: *mut LLRBNode<K, V>) {
    unsafe {
        if !node.is_null() {
            (*node).color = Color::RED
        }
    }
}



impl<'a, K: DictKey + 'a, V: 'a> LLRBNode<K, V> {
    pub fn new(key: K, value: V) -> *mut Self {
        Box::into_raw(box Self {
            left: null_mut(),
            right: null_mut(),
            paren: null_mut(),
            color: Color::RED,
            key: Box::into_raw(box key),
            value: Box::into_raw(box value),
        })
    }

    fn leaf(paren: *mut Self) -> *mut Self {
        Box::into_raw(box Self {
            left: null_mut(),
            right: null_mut(),
            paren,
            color: Color::BLACK,
            key: null_mut(),
            value: null_mut(),
        })
    }

    fn node_into_value(node: *mut LLRBNode<K, V>) -> V {
        unsafe {
            let origin_node = Box::from_raw(node);
            *Box::from_raw(origin_node.value)
        }
    }

    /// validate red/black
    pub fn self_validate(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.basic_self_validate()?;

        unsafe {
            if self.color == Color::RED {
                // right-learning red violation
                assert!(!
                    (is_black(self.left) && is_red(self.right))
                );

                // at most 4-node
                if is_red(self.left) && is_red(self.right) {
                    assert!(!is_red(self.paren));

                    assert!(!is_red((*self.left).left));
                    assert!(!is_red((*self.left).right));

                    assert!(!is_red((*self.right).left));
                    assert!(!is_red((*self.right).right));

                }
            }

            // unbalanced 4-node
            assert!(!
                (is_red(self.left) && is_red((*self.left).left))
            );

            // All descendant leaf's black depth
            // validate it from root

            // Validate recursively
            if !self.left.is_null() {
                (*self.left).self_validate()?;
            }

            if !self.right.is_null() {
                (*self.right).self_validate()?;
            }
        }

        Ok(())
    }

    /// Knuth's leaf, carry no information.
    pub fn leafs(&self) -> Vec<*mut Self> {
        let mut queue = vecdeq![self as *const Self as *mut Self];
        let mut leafs = vec![];

        while !queue.is_empty() {
            let p = queue.pop_front().unwrap();

            unsafe {
                if (*p).left.is_null() {
                    leafs.push(LLRBNode::leaf(p));
                } else {
                    queue.push_back((*p).left);
                }

                if (*p).right.is_null() {
                    leafs.push(LLRBNode::leaf(p));
                } else {
                    queue.push_back((*p).right);
                }
            }
        }


        leafs
    }

    /// Black nodes number from root to this.
    ///
    /// Alias as the number of black ancestors.
    pub fn black_depth(&self) -> usize {
        unsafe {
            let mut p = self.paren;
            let mut acc = 0;

            while !p.is_null() {
                if is_black(p) {
                    acc += 1;
                }

                p = (*p).paren;
            }

            acc
        }
    }

    pub fn echo_in_mm(&self, cache: &mut String) -> fmt::Result {
        unsafe {
            BSTNode::echo_in_mm(self, cache, |x, cache| {
                let x_self = x as *mut LLRBNode<K, V>;

                writeln!(cache, "{:?}", (*x_self).color)
            })
        }
    }

    pub fn echo_stdout(&self) {
        let mut cache = String::new();

        self.echo_in_mm(&mut cache).unwrap();

        println!("{}", cache);
    }

    /// Split or Merge of 4-node
    fn color_flip(&mut self) {
        self.color = self.color.reverse();

        if !self.left.is_null() {
            unsafe { (*self.left).color = (*self.left).color.reverse(); }
        }

        if !self.right.is_null() {
            unsafe { (*self.right).color = (*self.right).color.reverse(); }
        }
    }
}


impl<'a, K: DictKey + 'a, V: 'a> BTNode<'a, K, V> for LLRBNode<K, V> {
    fn itself(&self) -> *const (dyn BTNode<'a, K, V> + 'a) {
        self as *const Self
    }

    fn null(&self) -> *const (dyn BTNode<'a, K, V> + 'a) {
        null::<Self>()
    }

    fn try_as_bst(&self) -> Result<*const (dyn BSTNode<'a, K, V> + 'a), ()> {
        Ok(self as *const Self)
    }

    fn order(&self) -> usize {
        2
    }

    fn child(&self, idx: usize) -> *mut (dyn BTNode<'a, K, V> + 'a) {
        if idx == 0 {
            self.left
        } else if idx == 1 {
            self.right
        } else {
            self.null_mut()
        }
    }

    fn assign_child(
        &mut self,
        child: *mut (dyn BTNode<'a, K, V> + 'a),
        idx: usize,
    ) {
        if idx == 0 {
            self.left = child as *mut Self;
        } else if idx == 1 {
            self.right = child as *mut Self;
        } else {
            unreachable!()
        }
    }

    fn assign_value(&mut self, value: V, _idx: usize) {
        self.value = Box::into_raw(box value);
    }

    fn assign_paren(&mut self, paren: *mut (dyn BTNode<'a, K, V> + 'a)) {
        self.paren = paren as *mut Self;
    }

    fn paren(&self) -> *mut (dyn BTNode<'a, K, V> + 'a) {
        self.paren as *mut (dyn BTNode<K, V> + 'a)
    }

    fn height(&self) -> i32 {
        self.calc_height()
    }

    fn key_ptr(&self, idx: usize) -> *mut K {
        if idx == 0 {
            self.key
        } else {
            null_mut::<K>()
        }
    }

    fn assign_key_ptr(&mut self, idx: usize, key_ptr: *mut K) {
        if idx == 0 {
            self.key = key_ptr;
        }
    }

    fn val_ptr(&self, idx: usize) -> *mut V {
        if idx == 0 {
            self.value
        } else {
            null_mut::<V>()
        }
    }

    fn assign_val_ptr(&mut self, idx: usize, val_ptr: *mut V) {
        if idx == 0 {
            self.value = val_ptr;
        }
    }
}

impl<'a, K: DictKey + 'a, V: 'a> BSTNode<'a, K, V> for LLRBNode<K, V> {}


impl<'a, K: DictKey + 'a, V: 'a> LLRB<K, V> {
    pub fn new() -> Self {
        Self { root: null_mut() }
    }

    unsafe fn remove_retracing(
        &mut self,
        mut n: *mut LLRBNode<K, V>,
    ) -> *mut LLRBNode<K, V> {
        todo!()
    }

    unsafe fn insert_retracing(&mut self, x: *mut LLRBNode<K, V>) {
        let p = (*x).paren;
        if p.is_null() {
            return;

        }
    }

    // insert at x
    unsafe fn insert_at(&mut self, mut x:  *mut LLRBNode<K, V>, key: K, value: V) -> Result<*mut LLRBNode<K, V>, ()> {
        if x.is_null() {
            return Ok(LLRBNode::new(key, value));
        }

        // split 4-node (protect from possible 5-node) on the way down
        if is_red((*x).left) && is_red((*x).right) {
            (*x).color_flip();
        }

        if key == (*(*x).key) {
            return Err(());
        } else if key < (*(*x).key) {
            (*x).connect_left(self.insert_at((*x).left, key, value)?)
        } else {
            (*x).connect_right(self.insert_at((*x).right, key, value)?)
        }

        // fix right-learning reds on the way up (enforce left-learning)
        if is_red((*x).right) {
            x = self.rotate(x, Either::Left(())) as *mut LLRBNode<K, V>;
        }

        // fix two reds in a row on the way up (balance a 4-node)
        if is_red((*x).left) && is_red((*(*x).left).left) {
            x = self.rotate(x, Either::Right(())) as *mut LLRBNode<K, V>;
        }

        Ok(x)
    }

    pub fn echo_stdout(&self) {
        if !self.root.is_null() {
            unsafe { (*self.root).echo_stdout() }
        }
    }
}



impl<'a, K: DictKey + 'a, V: 'a> Dictionary<K, V> for LLRB<K, V> {
    fn insert(&mut self, key: K, value: V) -> bool {
        unsafe{
            if self.root.is_null() {
                self.root = LLRBNode::new(key, value);

                return true;
            }

            self.insert_at(self.root, key, value).is_ok()
        }
    }


    fn remove(&mut self, key: &K) -> Option<V> {
        unsafe {
            let approxi_node =
                (*self.search_approximately(&key)).try_as_bst_mut().unwrap();

            if approxi_node.is_null() {
                return None;
            }

            if BSTNode::key(&*approxi_node) != key {
                return None;
            }

            let removed_node =
                self.remove_retracing(approxi_node as *mut LLRBNode<K, V>);

            Some(LLRBNode::node_into_value(removed_node))
        }
    }



    fn modify(&mut self, key: &K, value: V) -> bool {
        self.basic_modify(key, value)
    }

    fn lookup(&self, income_key: &K) -> Option<&V> {
        self.basic_lookup(income_key)
    }

    fn lookup_mut(&mut self, income_key: &K) -> Option<&mut V> {
        self.basic_lookup_mut(income_key)
    }

    fn self_validate(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.basic_self_validate()?;

        if !self.root.is_null() {
            unsafe {
                (*self.root).self_validate()?;

                assert!((*self.root)
                    .leafs()
                    .into_iter()
                    .map(|leaf| (*leaf).black_depth())
                    .tuple_windows()
                    .all(|(a, b)| a == b))
            }
        }

        Ok(())
    }
}

impl<'a, K: DictKey + 'a, V: 'a> BT<'a, K, V> for LLRB<K, V> {
    fn order(&self) -> usize {
        2
    }

    fn root(&self) -> *mut (dyn BTNode<'a, K, V> + 'a) {
        self.root
    }

    fn assign_root(&mut self, root: *mut (dyn BTNode<'a, K, V> + 'a)) {
        self.root = root as *mut LLRBNode<K, V>;
    }
}


impl<'a, K: DictKey + 'a, V: 'a> BST<'a, K, V> for LLRB<K, V> {
    unsafe fn rotate_cleanup(
        &mut self,
        x: *mut (dyn BSTNode<'a, K, V> + 'a),
        z: *mut (dyn BSTNode<'a, K, V> + 'a),
    ) {
        debug_assert!(!x.is_null());
        debug_assert!(!z.is_null());

        let x_self = x as *mut LLRBNode<K, V>;
        let z_self = z as *mut LLRBNode<K, V>;

        (*z_self).color = (*x_self).color.clone();
        (*x_self).color = Color::RED;
    }
}


#[cfg(test)]
mod test {


    use itertools::Itertools;
    use rand::{prelude::SliceRandom, thread_rng};

    use super::LLRB;
    use crate::{
        collections::{
            bt::{
                bst::{BSTNode, BST, ROTATE_NUM},
                BTNode, BT,
            },
            Dictionary,
        },
        test::{
            dict::{DictProvider, GetKey, Inode, InodeProvider},
            Provider,
        },
    };


    #[test]
    pub(crate) fn test_llrb_randomdata() {
        let provider = InodeProvider {};

        (&provider as &dyn DictProvider<u32, Inode>)
            .test_dict(|| box LLRB::new());

        println!("LLRB rotate numer: {}", unsafe { ROTATE_NUM })
    }

    ///
    /// Debug RB entry
    ///
    // #[ignore = "Only used for debug"]
    #[test]
    fn hack_llrb() {
        for _ in 0..20 {
            let batch_num = 10;
            let mut collected_elems = vec![];
            let mut keys = vec![];
            let provider = InodeProvider {};
            let mut dict = LLRB::new();

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

                println!("insert {}: {:?}", i, k);
                assert!(dict.insert(k, e));
                assert!(dict.lookup(&keys.last().unwrap()).is_some());

                dict.self_validate().unwrap();

                i += 1;
            }

            // let mut dict_debug = dict.clone();

            // collected_elems.shuffle(&mut thread_rng());

            // // Remove-> Verify
            // for i in 0..batch_num {
            //     let e = &collected_elems[i];
            //     let k = &e.get_key();

            //     assert!(dict.remove(k).is_some());
            //     assert!(!dict.lookup(k).is_some());

            //     println!("{}", i);
            //     if let Ok(_res) = dict.self_validate() {
            //     } else {
            //     }
            // }
        }
    }

    #[test]
    fn test_llrb_fixeddata_case_0() {
        let mut llrb = LLRB::<i32, ()>::new();

        let dict = &mut llrb as &mut dyn Dictionary<i32, ()>;

        dict.insert(10, ());
        assert!(dict.self_validate().is_ok());

        dict.insert(5, ());
        dict.self_validate().unwrap();

        dict.insert(12, ());
        dict.self_validate().unwrap();

        dict.insert(13, ());
        dict.self_validate().unwrap();

        dict.insert(14, ());
        dict.self_validate().unwrap();

        dict.insert(18, ());
        dict.self_validate().unwrap();

        dict.insert(7, ());
        dict.self_validate().unwrap();

        dict.insert(9, ());
        dict.self_validate().unwrap();

        dict.insert(11, ());
        dict.self_validate().unwrap();

        dict.insert(22, ());
        dict.self_validate().unwrap();

        assert!(dict.lookup(&10).is_some());
        assert!(dict.lookup(&5).is_some());
        assert!(dict.lookup(&12).is_some());
        assert!(dict.lookup(&13).is_some());
        assert!(dict.lookup(&14).is_some());
        assert!(dict.lookup(&18).is_some());
        assert!(dict.lookup(&7).is_some());
        assert!(dict.lookup(&9).is_some());
        assert!(dict.lookup(&11).is_some());
        assert!(dict.lookup(&22).is_some());


        // assert!(dict.remove(&10).is_some());
        // assert!(dict.lookup(&10).is_none());
        // dict.self_validate().unwrap();

        // assert!(dict.remove(&5).is_some());
        // assert!(dict.lookup(&5).is_none());
        // dict.self_validate().unwrap();

        // assert!(dict.remove(&12).is_some());
        // dict.self_validate().unwrap();

        // assert!(dict.remove(&13).is_some());
        // dict.self_validate().unwrap();

        // assert!(dict.remove(&14).is_some());
        // dict.self_validate().unwrap();

        // assert!(dict.remove(&18).is_some());
        // dict.self_validate().unwrap();

        // assert!(dict.remove(&7).is_some());
        // dict.self_validate().unwrap();

        // assert!(dict.remove(&9).is_some());
        // dict.self_validate().unwrap();

        // assert!(dict.remove(&11).is_some());
        // dict.self_validate().unwrap();

        // assert!(dict.remove(&22).is_some());

        llrb.self_validate().unwrap();
        llrb.echo_stdout();
    }



    #[test]
    fn test_llrb_fixeddata_case_1() {
        let mut llrb = LLRB::<i32, ()>::new();

        let dict = &mut llrb as &mut dyn Dictionary<i32, ()>;

        dict.insert(87, ());
        assert!(dict.self_validate().is_ok());

        dict.insert(40, ());
        dict.self_validate().unwrap();

        dict.insert(89, ());
        dict.self_validate().unwrap();

        dict.insert(39, ());
        dict.self_validate().unwrap();

        dict.insert(24, ());
        dict.self_validate().unwrap();

        dict.insert(70, ());
        dict.self_validate().unwrap();

        dict.insert(9, ());
        dict.self_validate().unwrap();

        dict.insert(2, ());
        dict.self_validate().unwrap();

        dict.insert(67, ());
        dict.self_validate().unwrap();

        llrb.echo_stdout();
    }
}
