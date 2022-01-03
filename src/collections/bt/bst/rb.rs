//!
//! Reference: https://en.wikipedia.org/wiki/Red%E2%80%93black_tree#loopInvariantI
//! Red is central item of 2-4 tree
//!



use std::cmp::max;
use std::fmt;
use std::fmt::Write;
use std::ptr::{null, null_mut};

use either::Either;
use itertools::Itertools;
use serde::de::DeserializeSeed;
use serde::{Deserialize, Deserializer, Serialize};

use super::{BSTNode, BST};
use crate::collections::bt::{BTNode, BT};
use crate::collections::{DictKey, Dictionary};
use crate::etc::Reverse;
use crate::*;


////////////////////////////////////////////////////////////////////////////////
//// Struct
////

pub struct RB<K, V> {
    root: *mut RBNode<K, V>,
}


struct RBNode<K, V> {
    left: *mut Self,
    right: *mut Self,
    paren: *mut Self,
    color: Color,
    key: *mut K,
    value: *mut V,
}


#[derive(Debug, Clone, PartialEq)]
#[repr(u8)]
pub(crate) enum Color {
    RED,
    BLACK,
}


////////////////////////////////////////////////////////////////////////////////
//// Implement

impl Reverse for Color {
    fn reverse(&self) -> Self {
        match &self {
            Color::RED => Color::BLACK,
            Color::BLACK => Color::RED,
        }
    }
}

fn is_black<K, V>(node: *mut RBNode<K, V>) -> bool {
    unsafe { node.is_null() || (*node).color == Color::BLACK }
}

fn is_red<K, V>(node: *mut RBNode<K, V>) -> bool {
    !is_black(node)
}

fn set_black<K, V>(node: *mut RBNode<K, V>) {
    unsafe {
        if !node.is_null() {
            (*node).color = Color::BLACK
        }
    }
}

fn set_red<K, V>(node: *mut RBNode<K, V>) {
    unsafe {
        if !node.is_null() {
            (*node).color = Color::RED
        }
    }
}



impl<'a, K: DictKey + 'a, V: 'a> RBNode<K, V> {
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

    fn node_into_value(node: *mut RBNode<K, V>) -> V {
        unsafe {
            let origin_node = Box::from_raw(node);
            *Box::from_raw(origin_node.value)
        }
    }

    /// validate red/black
    pub fn self_validate(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.basic_self_validate()?;

        unsafe {
            // Single Red Color Rule
            if self.color == Color::RED {
                // Ignore this for impl convenience
                // if !self.paren.is_null() {
                //     assert!(self.color.is_black());
                // }

                assert!(is_black(self.left));
                assert!(is_black(self.right));
            }

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
                    leafs.push(RBNode::leaf(p));
                } else {
                    queue.push_back((*p).left);
                }

                if (*p).right.is_null() {
                    leafs.push(RBNode::leaf(p));
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
                let x_self = x as *mut RBNode<K, V>;

                writeln!(cache, "{:?}", (*x_self).color)
            })
        }
    }

    pub fn echo_stdout(&self) {
        let mut cache = String::new();

        self.echo_in_mm(&mut cache).unwrap();

        println!("{}", cache);
    }
}


impl<'a, K: DictKey + 'a, V: 'a> BTNode<'a, K, V> for RBNode<K, V> {
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

impl<'a, K: DictKey + 'a, V: 'a> BSTNode<'a, K, V> for RBNode<K, V> {}


impl<'a, K: DictKey + 'a, V: 'a> RB<K, V> {
    pub fn new() -> Self {
        Self { root: null_mut() }
    }

    // ref: https://www.geeksforgeeks.org/red-black-tree-set-3-delete-2/?ref=lbp
    unsafe fn remove_retracing(
        &mut self,
        mut n: *mut RBNode<K, V>,
    ) -> *mut RBNode<K, V> {

        /* Prepare Deleting */
        if !(*n).right.is_null() {
            let successor = (*(*n).right).minimum() as *mut RBNode<K, V>;
            (*n).swap_with(successor);

            n = successor;
        }
        // Either n.left or n.right is null.
        let u = n;
        let v = if (*u).left.is_null() {
            (*u).right
        } else {
            (*u).left
        };

        /* Handle SPECIAL v is null case (for it need retracing before remove) */
        if v.is_null() {
            if (*u).paren.is_null() {
                self.root = v;
            } else {
                if is_black(u) {
                    self.remove_retracing_black_non_root_leaf(u);
                } else {
                    set_red((*u).sibling() as *mut RBNode<K, V>);
                }

                self.subtree_shift(u, v);
            }

            return u;
        }

        /* Simple Case: either u and v is red */
        /* No change in black height */

        self.subtree_shift(u, v);

        if is_red(u) || is_red(v) {
            set_black(v);

            return u;
        }

        /* Both u, v are black case */

        // u is root
        if (*u).paren.is_null() {
            return u;
        }

        // u isn't root
        self.remove_retracing_black_non_root_leaf(v);

        u
    }

    unsafe fn remove_retracing_black_non_root_leaf(&mut self, n: *mut RBNode<K, V>) {
        let p = (*n).paren;
        if p.is_null() {
            return;
        }

        let dir = if n == (*p).left {
            Either::Left(())
        } else {
            Either::Right(())
        };

        let s
        = BSTNode::child(&*p, dir.reverse())  // Sibling
        as *mut RBNode<K, V>;

        if s.is_null() {
            return self.remove_retracing_black_non_root_leaf(p);
        }

        let c
        = BSTNode::child(&*s, dir)  // Close Nephew
        as *mut RBNode<K, V>;

        let d
        = BSTNode::child(&*s, dir.reverse())  // Distant Nephew
        as *mut RBNode<K, V>;


        if is_red(s) {  // indicates that p c d are black
            self.rotate(p, dir);
            (*p).color = Color::RED;
            (*s).color = Color::BLACK;

            return self.remove_retracing_black_non_root_leaf(n);
        }

        /* s is black */

        if is_black(c) && is_black(d) {
            (*s).color = Color::RED;

            if is_black(p) {
                self.remove_retracing_black_non_root_leaf(p);
            } else {
                set_black(p);
            }
        } else if is_red(c) {
            self.double_rotate(p, dir);
            (*c).color = (*p).color.clone();
            (*p).color = Color::BLACK;

        } else {  // d is red
            self.rotate(p, dir);
            (*s).color = (*p).color.clone();
            (*d).color = Color::BLACK;
            (*p).color = Color::BLACK;

        }

    }


    unsafe fn insert_retracing(&mut self, x: *mut RBNode<K, V>) {
        let p = (*x).paren;
        if p.is_null() {
            return;
        }

        if (*p).color == Color::BLACK {
            return;
        }

        // P is RED, G is BLACK
        let g = (*p).paren;
        if g.is_null() {
            (*p).color = Color::BLACK;

            return;
        }

        let pdir = if x == (*p).left {
            Either::Left(())
        } else {
            Either::Right(())
        };
        let u
        = BSTNode::child(&*g, pdir.reverse()) as *mut RBNode<K, V>;

        if is_red(u) {
            // Repaint
            (*p).color = Color::BLACK;
            (*u).color = Color::BLACK;
            (*g).color = Color::RED;

            self.insert_retracing(g)
        } else {
            let new_root;
            let the_other_child;

            if p as *const ()
                == BSTNode::child(&*g, pdir.reverse()) as *const ()
            {
                new_root = self.double_rotate(g, pdir) as *mut RBNode<K, V>;
                the_other_child =
                    BSTNode::child(&*new_root, pdir) as *mut RBNode<K, V>;
            } else {
                new_root = self.rotate(g, pdir.reverse()) as *mut RBNode<K, V>;
                the_other_child = BSTNode::child(&*new_root, pdir.reverse())
                    as *mut RBNode<K, V>;
            }

            (*new_root).color = Color::BLACK;
            (*the_other_child).color = Color::RED;
        }
    }

    pub fn echo_stdout(&self) {
        if !self.root.is_null() {
            unsafe { (*self.root).echo_stdout() }
        }
    }
}



impl<'a, K: DictKey + 'a, V: 'a> Dictionary<K, V> for RB<K, V> {
    fn insert(&mut self, key: K, value: V) -> bool {
        let new_node = RBNode::new(key, value);

        if !self.basic_insert(new_node) {
            return false;
        }

        unsafe {
            self.insert_retracing(new_node);
        }

        true
    }


    fn remove(&mut self, key: &K) -> Option<V> {
        unsafe {
            let approxi_node =
                (*self.search_approximately(&key)).try_as_bst_mut().unwrap();

            if approxi_node.is_null() {
                return None;
            }

            if BSTNode::key_bst(&*approxi_node) != key {
                return None;
            }

            let removed_node =
                self.remove_retracing(approxi_node as *mut RBNode<K, V>);

            Some(RBNode::node_into_value(removed_node))
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

impl<'a, K: DictKey + 'a, V: 'a> BT<'a, K, V> for RB<K, V> {
    fn order(&self) -> usize {
        2
    }

    fn root(&self) -> *mut (dyn BTNode<'a, K, V> + 'a) {
        self.root
    }

    fn assign_root(&mut self, root: *mut (dyn BTNode<'a, K, V> + 'a)) {
        self.root = root as *mut RBNode<K, V>;
    }
}


impl<'a, K: DictKey + 'a, V: 'a> BST<'a, K, V> for RB<K, V> {
    unsafe fn rotate_cleanup(
        &mut self,
        _x: *mut (dyn BSTNode<'a, K, V> + 'a),
        _z: *mut (dyn BSTNode<'a, K, V> + 'a),
    ) {
    }
}




#[cfg(test)]
mod test {


    use itertools::Itertools;
    use rand::{prelude::SliceRandom, thread_rng};

    use super::RB;
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
    pub(crate) fn test_rb_randomdata() {
        let provider = InodeProvider {};

        (&provider as &dyn DictProvider<u32, Inode>)
            .test_dict(|| box RB::new());

        println!("rotate numer: {}", unsafe { ROTATE_NUM })
    }

    ///
    /// Debug RB entry
    ///
    #[ignore = "Only used for debug"]
    #[test]
    fn hack_rb() {
        for _ in 0..20 {
            let batch_num = 10;
            let mut collected_elems = vec![];
            let mut keys = vec![];
            let provider = InodeProvider {};
            let mut dict = RB::new();

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

                // println!("insert {}: {:?}", i, k);
                assert!(dict.insert(k, e));
                assert!(dict.lookup(&keys.last().unwrap()).is_some());

                dict.self_validate().unwrap();

                i += 1;
            }

            // let mut dict_debug = dict.clone();

            collected_elems.shuffle(&mut thread_rng());

            // Remove-> Verify
            for i in 0..batch_num {
                let e = &collected_elems[i];
                let k = &e.get_key();

                assert!(dict.remove(k).is_some());
                assert!(!dict.lookup(k).is_some());

                println!("{}", i);
                if let Ok(_res) = dict.self_validate() {
                } else {
                }
            }
        }
    }

    #[test]
    fn test_rb_fixeddata_case_0() {
        let mut rb = RB::<i32, ()>::new();

        let dict = &mut rb as &mut dyn Dictionary<i32, ()>;

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


        assert!(dict.remove(&10).is_some());
        assert!(dict.lookup(&10).is_none());
        dict.self_validate().unwrap();

        assert!(dict.remove(&5).is_some());
        assert!(dict.lookup(&5).is_none());
        dict.self_validate().unwrap();

        assert!(dict.remove(&12).is_some());
        dict.self_validate().unwrap();

        assert!(dict.remove(&13).is_some());
        dict.self_validate().unwrap();

        assert!(dict.remove(&14).is_some());
        dict.self_validate().unwrap();

        assert!(dict.remove(&18).is_some());
        dict.self_validate().unwrap();

        assert!(dict.remove(&7).is_some());
        dict.self_validate().unwrap();

        assert!(dict.remove(&9).is_some());
        dict.self_validate().unwrap();

        assert!(dict.remove(&11).is_some());
        dict.self_validate().unwrap();

        assert!(dict.remove(&22).is_some());

        rb.self_validate().unwrap();
        rb.echo_stdout();
    }



    #[test]
    fn test_rb_fixeddata_case_1() {
        let mut rb = RB::<i32, ()>::new();

        let dict = &mut rb as &mut dyn Dictionary<i32, ()>;

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

        rb.echo_stdout();
    }
}
