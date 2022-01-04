//! 1. https://en.wikipedia.org/wiki/AA_tree
//!
//! 1. https://iq.opengenus.org/aa-trees/
//!
//! AA Tree Invariants:
//!
//! 1. The level of every leaf node is one.
//!
//! 1. The level of every left child is exactly one less than that of its parent.
//!
//! 1. The level of every right child is equal to or one less than that of its parent.
//!
//! 1. The level of every right grandchild is strictly less than that of its grandparent.
//!
//! 1. Every node of level greater than one has two children.



use std::cmp::{max, min};
use std::fmt;
use std::fmt::Write;
use std::ptr::{null, null_mut};

use either::Either;
use itertools::Itertools;

use super::{BSTNode, BST};
use crate::collections::bt::{BTNode, BT};
use crate::collections::{DictKey, Dictionary};
use crate::etc::Reverse;
use crate::*;


////////////////////////////////////////////////////////////////////////////////
//// Struct



pub struct AA<K, V> {
    root: *mut AANode<K, V>,
}


struct AANode<K, V> {
    left: *mut Self,
    right: *mut Self,
    paren: *mut Self,
    level: usize,
    key: *mut K,
    value: *mut V,
}



////////////////////////////////////////////////////////////////////////////////
//// Implement

fn level<K, V>(node: *mut AANode<K, V>) -> usize {
    unsafe {
        if node.is_null() {
            0
        } else {
            (*node).level
        }
    }
}



impl<'a, K: DictKey + 'a, V: 'a> AANode<K, V> {
    pub fn new(key: K, value: V) -> *mut Self {
        Box::into_raw(box Self {
            left: null_mut(),
            right: null_mut(),
            paren: null_mut(),
            level: 1,  // Invariants 1
            key: Box::into_raw(box key),
            value: Box::into_raw(box value),
        })
    }


    fn leafs(&self) -> Vec<*mut Self> {
        let mut queue = vecdeq![self as *const Self as *mut Self];
        let mut leafs = vec![];

        while !queue.is_empty() {
            let p = queue.pop_front().unwrap();

            unsafe {
                if (*p).left.is_null() && (*p).right.is_null() {
                    leafs.push(p);
                } else if (*p).left.is_null() {
                    leafs.push(p);
                    queue.push_back((*p).right);
                } else if (*p).right.is_null() {
                    leafs.push(p);
                    queue.push_back((*p).left);
                } else {
                    queue.push_back((*p).left);
                    queue.push_back((*p).right);
                }
            }
        }

        leafs
    }


    /// validate red/black
    pub fn self_validate(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.basic_self_validate()?;

        unsafe {
            // Invariant 2: level(it) - 1 = level(left)
            if !self.left.is_null() {
                assert_eq!((*self.left).level + 1, self.level);
            }

            // Invariant 3: 0 <= level(it) - level(right) <= 1
            if !self.right.is_null() {
                assert!(self.level - (*self.right).level <= 1);
            }

            // Invariant 4:
            // level(it) > level(right.left)
            // level(it) > level(right.right)
            if !self.right.is_null() {
                if !(*self.right).left.is_null() {
                    assert!(self.level > (*(*self.right).left).level);
                }

                if !(*self.right).right.is_null() {
                    assert!(self.level > (*(*self.right).right).level);
                }
            }


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


    pub fn echo_in_mm(&self, cache: &mut String) -> fmt::Result {
        unsafe {
            BSTNode::echo_in_mm(self, cache, |x, cache| {
                let x_self = x as *mut AANode<K, V>;

                writeln!(cache, "LV: {:?}", (*x_self).level)
            })
        }
    }

    pub fn echo_stdout(&self) {
        let mut cache = String::new();

        self.echo_in_mm(&mut cache).unwrap();

        println!("{}", cache);
    }


    unsafe fn level_dec(&mut self) {
        let lv = min(level(self.left), level(self.right)) + 1;

        if lv < self.level {
            self.level = lv;

            if lv < level(self.right) {
                (*self.right).level = lv;
            }
        }

    }

}


impl<'a, K: DictKey + 'a, V: 'a> BTNode<'a, K, V> for AANode<K, V> {
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

impl<'a, K: DictKey + 'a, V: 'a> BSTNode<'a, K, V> for AANode<K, V> {}


impl<'a, K: DictKey + 'a, V: 'a> AA<K, V> {
    pub fn new() -> Self {
        Self { root: null_mut() }
    }


    pub fn echo_stdout(&self) {
        if !self.root.is_null() {
            unsafe { (*self.root).echo_stdout() }
        }
    }

    unsafe fn insert_at(&mut self, mut t: *mut AANode<K, V>, key: K, value: V) -> Result<*mut AANode<K, V>, ()> {
        if t.is_null() {
            t = AANode::new(key, value);
        } else if key < *(*t).key {
            (*t).connect_left(
                self.insert_at((*t).left, key, value)?
            )
        } else if key > *(*t).key {
            (*t).connect_right(
                self.insert_at((*t).right, key, value)?
            )
        } else {
            return Err(())
        }

        t = self.skew(t);
        t = self.split(t);

        Ok(t)
    }

    unsafe fn skew(&mut self, t: *mut AANode<K, V>) -> *mut AANode<K, V> {
        if t.is_null() || (*t).left.is_null() {
            return t
        }

        if (*(*t).left).level == (*t).level {
            return self.rotate(t, Either::Right(())) as *mut AANode<K, V>
        }

        t
    }

    unsafe fn split(&mut self, t: *mut AANode<K, V>) -> *mut AANode<K, V> {

        if t.is_null() || (*t).right.is_null() || (*(*t).right).right.is_null() {
            return t;
        }

        if (*(*(*t).right).right).level == (*t).level {
            let new_t
            = self.rotate(t, Either::Left(())) as *mut AANode<K, V>;

            (*new_t).level += 1;

            return new_t;
        }

        t
    }

    unsafe fn remove_at(
        &mut self,
        mut t: *mut AANode<K, V>,
        key: &K,
        res: &mut Vec<*mut V>
    ) -> *mut AANode<K, V> {

        if t.is_null() {
            return t;
        }

        if key < (*t).key_bst() {
            (*t).connect_left(
                self.remove_at((*t).left, key, res)
            );
        }
        else if key > (*t).key_bst() {
            (*t).connect_right(
                self.remove_at((*t).right, key, res)
            );
        }
        else {
            if (*t).is_leaf() {
                res.push((*t).value);
                return (*t).left  // null
            }
            else if (*t).left.is_null() {
                let succ = (*t).successor_bst() as *mut AANode<K, V>;

                (*t).connect_right(
                    self.remove_at((*t).right, (*succ).key_bst(), res)
                );

                res.push((*t).value);
                (*t).value = (*succ).value;
                (*t).key = (*succ).key;

            }
            else {
                let prec = (*t).precessor_bst() as *mut AANode<K, V>;

                (*t).connect_left(
                    self.remove_at((*t).left, (*prec).key_bst(), res)
                );

                res.push((*t).value);
                (*t).value = (*prec).value;
                (*t).key = (*prec).key;

            }

        }

        (*t).level_dec();

        t = self.skew(t);
        (*t).connect_right(
            self.skew((*t).right)
        );

        if !(*t).right.is_null() {
            let t_right = (*t).right;

            (*t_right).connect_right(
                self.skew((*t_right).right)
            );
        }

        t = self.split(t);
        (*t).connect_right(
            self.split((*t).right)
        );

        t

    }

}



impl<'a, K: DictKey + 'a, V: 'a> Dictionary<K, V> for AA<K, V> {
    fn insert(&mut self, key: K, value: V) -> bool {
        unsafe {
            if self.root.is_null() {
                self.root = AANode::new(key, value);
            } else {
                if let Ok(t) = self.insert_at(self.root, key, value) {
                    self.root = t;
                } else {
                    return false;
                }
            }

            true
        }
    }


    fn remove(&mut self, key: &K) -> Option<V> {
        unsafe {
            if self.root.is_null() {
                return None;
            }

            let mut res = Vec::new();
            self.root = self.remove_at(self.root, key, &mut res);

            if let Some(vp) = res.pop() {
                Some(*Box::from_raw(vp))

            } else {
                None
            }

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

                // Invariants: 1, all leaf level == 1
                assert!(
                    (*self.root).leafs()
                    .into_iter()
                    .all(|leaf| (*leaf).level == 1)
                );

                // Invariants: 5, if x.level > 1, then x.left != null && x.right != null
                for node in self.nodes_iter() {
                    let x = node as *mut AANode<K, V>;

                    if (*x).level > 1 {
                        assert!(
                            !(*x).left.is_null() && !(*x).right.is_null()
                        )
                    }
                }
            }
        }

        Ok(())
    }
}

impl<'a, K: DictKey + 'a, V: 'a> BT<'a, K, V> for AA<K, V> {
    fn order(&self) -> usize {
        2
    }

    fn root(&self) -> *mut (dyn BTNode<'a, K, V> + 'a) {
        self.root
    }

    fn assign_root(&mut self, root: *mut (dyn BTNode<'a, K, V> + 'a)) {
        self.root = root as *mut AANode<K, V>;
    }
}


impl<'a, K: DictKey + 'a, V: 'a> BST<'a, K, V> for AA<K, V> {
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

    use super::AA;
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
    pub(crate) fn test_aa_randomdata() {
        let provider = InodeProvider {};

        (&provider as &dyn DictProvider<u32, Inode>)
            .test_dict(|| box AA::new());

        println!("AA rotate numer: {}", unsafe { ROTATE_NUM })
    }


    #[test]
    fn test_aa_fixeddata_case_0() {
        let mut aa = AA::<i32, ()>::new();

        let dict = &mut aa as &mut dyn Dictionary<i32, ()>;

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

        assert!(!dict.insert(67, ()));
        dict.self_validate().unwrap();

        assert!(!dict.insert(24, ()));
        dict.self_validate().unwrap();

        assert!(!dict.insert(9, ()));
        dict.self_validate().unwrap();

        aa.echo_stdout();

    }


    #[test]
    fn test_aa_fixeddata_case_1() {
        let mut aa = AA::<i32, ()>::new();

        let dict = &mut aa as &mut dyn Dictionary<i32, ()>;

        dict.insert(255, ());
        dict.insert(242, ());
        dict.insert(393, ());

        assert!(dict.remove(&255).is_some());
        dict.self_validate().unwrap();


        aa.echo_stdout();

    }


    ///
    /// Debug RB entry
    ///
    // #[ignore = "Only used for debug"]
    #[test]
    fn hack_aa() {
        for _ in 0..20 {
            let batch_num = 13;
            let mut collected_elems = vec![];
            let mut keys = vec![];
            let provider = InodeProvider {};
            let mut dict = AA::new();

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

            collected_elems.shuffle(&mut thread_rng());

            // Remove-> Verify
            for i in 0..batch_num {
                let e = &collected_elems[i];
                let k = &e.get_key();

                println!("remove: {}", k);

                assert!(dict.remove(k).is_some());
                assert!(!dict.lookup(k).is_some());

                if let Ok(_res) = dict.self_validate() {
                } else {
                }
            }
        }
    }


}