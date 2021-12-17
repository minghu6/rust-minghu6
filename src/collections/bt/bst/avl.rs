//!
//! By two Soviet inventors, Georgy Adelson-Velsky and Evgenii Landis(1962)
//!
//! ref 1: https://en.wikipedia.org/wiki/AVL_tree
//!
//! ref 2: https://en.wikipedia.org/wiki/Binary_search_tree
//!


use std::cmp::max;
use std::fmt;
use std::fmt::Write;
use std::ptr::{null, null_mut};

use either::Either;
use serde::de::DeserializeSeed;
use serde::{ Serialize, Deserialize, Deserializer };

use super::{BSTNode, BST};
use crate::collections::bt::BTNode;
use crate::collections::{DictKey, Dictionary};
use crate::error_code::ValidateFailedError;
use crate::etc::Reverse;


////////////////////////////////////////////////////////////////////////////////
//// Struct
////

pub struct AVL<K, V> {
    root: *mut AVLNode<K, V>,
}


struct AVLNode<K, V> {
    left: *mut AVLNode<K, V>,
    right: *mut AVLNode<K, V>,
    paren: *mut AVLNode<K, V>,
    height: i32, // using C style int, as it's default for Rust
    key: *const K,
    value: *mut V,
}



////////////////////////////////////////////////////////////////////////////////
//// Implement


impl<'a, K: DictKey + 'a, V: 'a> AVLNode<K, V> {
    pub fn new(key: K, value: V) -> *mut Self {
        Box::into_raw(box Self {
            left: null_mut(),
            right: null_mut(),
            paren: null_mut(),
            height: 0,
            key: Box::into_raw(box key) as *const K,
            value: Box::into_raw(box value),
        })
    }

    pub fn into_value(self) -> V {
        unsafe { *Box::from_raw(self.value) }
    }

    fn bf(&self) -> i32 {
        self.right_height() - self.left_height()
    }

    fn calc_bf(&self) -> i32 {
        self.calc_right_height() - self.calc_left_height()
    }

    pub fn self_validate(&self) -> Result<(), Box<dyn std::error::Error>> {
        let bf = self.calc_bf();
        if bf.abs() >= 2 {
            return Err(ValidateFailedError::new_box_err(&format!(
                "BF: {}",
                bf
            )));
        }

        Ok(())
    }

    pub fn echo_in_mm(&self, cache: &mut String) -> fmt::Result {
        unsafe {
            (*self.itself()).echo_in_mm(cache, |x, cache| {
                let x_self = x as *mut AVLNode<K, V>;


                let check_res = if (*x_self).calc_bf().abs() >= 2 {
                    "failed"
                } else {
                    "pass"
                };

                writeln!(
                    cache,
                    "BF: {:?}, H(LF): {}, H(RH): {},  {}",
                    (*x_self).calc_bf(),
                    (*x).calc_left_height(),
                    (*x).calc_right_height(),
                    check_res
                )
            })
        }
    }

    pub fn echo_stdout(&self) {
        let mut cache = String::new();

        self.echo_in_mm(&mut cache).unwrap();

        println!("{}", cache);
    }
}


impl<'a, K: DictKey + Clone + 'a, V: Clone + 'a> Clone for AVLNode<K, V> {
    /// Expensive Implements.
    ///
    /// **WARNING: The Field `paren` isn't set!, it should be set manually!**
    fn clone(&self) -> Self {
        unsafe {
            let key = Box::into_raw(box (*self.key).clone());
            let value = Box::into_raw(box (*self.value).clone());
            let height = self.height;

            // let paren = self.paren;
            let paren = null_mut();
            let left = if self.left.is_null() {
                null_mut()
            } else {
                Box::into_raw(box (*self.left).clone())
            };
            let right = if self.right.is_null() {
                null_mut()
            } else {
                Box::into_raw(box (*self.right).clone())
            };


            Self { left, right, paren, height, key, value }
        }

    }
}



impl<'a, K: DictKey + 'a, V: 'a> BTNode<'a, K, V> for AVLNode<K, V> {
    fn order(&self) -> usize {
        2
    }

    fn child(&self, idx: usize) -> *mut (dyn BTNode<'a, K, V> + 'a) {
        if idx == 0 {
            self.left
        } else {
            self.right
        }
    }

    fn paren(&self) -> *mut (dyn BTNode<'a, K, V> + 'a) {
        self.paren as *mut (dyn BTNode<K, V> + 'a)
    }

    fn key(&self, _idx: usize) -> &K {
        unsafe { &*self.key }
    }

    fn value(&self, _idx: usize) -> &V {
        unsafe { &*self.value }
    }

    fn value_mut(&self, _idx: usize) -> &mut V {
        unsafe { &mut *self.value }
    }

    fn height(&self) -> i32 {
        self.height
    }

    fn try_as_bst(&self) -> Result<*const (dyn BSTNode<'a, K, V> + 'a), ()> {
        Ok(self as *const Self)
    }
}



impl<'a, K: DictKey + 'a, V: 'a> BSTNode<'a, K, V> for AVLNode<K, V> {
    fn key(&self) -> &K {
        unsafe { &*self.key }
    }

    fn value(&self) -> &V {
        unsafe { &*self.value }
    }

    fn itself(&self) -> *const (dyn BSTNode<'a, K, V> + 'a) {
        self as *const Self
    }

    fn null(&self) -> *const (dyn BSTNode<'a, K, V> + 'a) {
        null::<Self>()
    }

    fn assign_left(&mut self, left: *mut (dyn BSTNode<'a, K, V> + 'a)) {
        self.left = left as *mut Self;
    }

    fn assign_right(&mut self, right: *mut (dyn BSTNode<'a, K, V> + 'a)) {
        self.right = right as *mut Self;
    }

    fn assign_paren(&mut self, paren: *mut (dyn BSTNode<'a, K, V> + 'a)) {
        self.paren = paren as *mut Self;
    }

    fn assign_value(&mut self, value: V) {
        self.value = Box::into_raw(box value);
    }
}



impl<'a, K: DictKey + 'a, V: 'a> AVL<K, V> {
    pub fn new() -> Self {
        Self { root: null_mut() }
    }

    pub fn echo_stdout(&self) {
        if !self.root.is_null() {
            unsafe { (*self.root).echo_stdout() }
        }
    }

    /// Simple Rotation
    /// ```no_run
    ///             rotate left
    ///    x        =========>          z
    ///  /  \                          / \
    /// t1   z                        x   t4
    /// |   / \                      / \   |
    ///   t23 t4                    t1 t23 |
    ///     |  |                     |   |
    ///        |
    /// ```
    ///
    unsafe fn rotate(
        &mut self,
        x: *mut AVLNode<K, V>,
        rotation: Either<(), ()>, // rotate to left = from right rotation
    ) -> *mut AVLNode<K, V> {
        let z = if rotation.is_left() {
            (*x).right
        } else {
            (*x).left
        };

        let t23 = if rotation.is_left() {
            (*z).left
        } else {
            (*z).right
        };

        if !t23.is_null() {
            (*t23).assign_paren(x);
        }

        if rotation.is_left() {
            (*x).assign_right(t23);
            (*z).assign_left(x);
        } else {
            (*x).assign_left(t23);
            (*z).assign_right(x);
        }

        self.subtree_shift(x, z);
        (*x).assign_paren(z);

        (*x).height = 1 + max((*x).left_height(), (*x).right_height());
        (*z).height = 1 + max((*z).left_height(), (*z).right_height());

        z
    }


    /// Double Rotation
    /// ```no_run
    ///             rotate [right]-left         rotate right-[left]
    ///    x        =========>         x        =========>       y
    ///  /   \                        /  \                      / \
    /// t1    z                      t1   y                    x   z
    /// |   /  \                     |   / \                  / \ / \
    ///    y   t4                      t2   z                t1 t2t3t4
    ///   / \   |                       |  / \                |  | | |
    ///  t2 t3                            t3 t4
    ///   |  |                            |   |
    /// ```
    unsafe fn double_rotate(
        &mut self,
        x: *mut AVLNode<K, V>,
        snd_rotation: Either<(), ()>,
    ) -> *mut AVLNode<K, V> {
        let z = if snd_rotation.is_left() {
            (*x).right
        } else {
            (*x).left
        };

        self.rotate(z, snd_rotation.reverse());
        self.rotate(x, snd_rotation)

        // // Manualy Implements
        // /* FIRST ROTATION */
        // // z is by 2 higher than its sibing(t1)
        // // y is by 1 higher than its sibling(t4) (thereis shouldn't be empty)
        // let z = if snd_rotation.is_left() {
        //     (*x).right
        // } else {
        //     (*x).left
        // };

        // let y = if snd_rotation.is_left() {
        //     (*z).left
        // } else {
        //     (*z).right
        // };

        // let (t2, t3) = if snd_rotation.is_left() {
        //     ((*y).left, (*y).right)
        // } else {
        //     ((*y).right, (*y).left)
        // };

        // if !t3.is_null() {
        //     (*t3).assign_paren(z);
        // }
        // (*z).assign_paren(y);

        // if snd_rotation.is_left() {
        //     (*z).assign_left(t3);
        //     (*y).assign_right(z);
        // } else {
        //     (*z).assign_right(t3);
        //     (*y).assign_left(z);
        // }

        // // skip x-R->z => x-R->y for it would be overrided by second rotation

        // /* SECOND ROTATION */
        // if snd_rotation.is_left() {
        //     (*x).assign_right(t2);
        //     (*y).assign_left(x);
        // } else {
        //     (*x).assign_left(t2);
        //     (*y).assign_right(x);
        // }
        // if !t2.is_null() {
        //     (*t2).assign_paren(x);
        // }

        // self.subtree_shift(x, y);
        // (*x).assign_paren(y);

        // y
    }


    unsafe fn insert_retracing(&mut self, new_node: *mut AVLNode<K, V>) {
        let mut y = new_node;
        let mut z = (*y).paren;

        while !z.is_null() {
            (*z).height = 1 + max((*z).left_height(), (*z).right_height());

            let x = (*z).paren;

            if !x.is_null() && (*x).bf().abs() > 1 {
                let direction = if z == (*x).right {
                    Either::Left(())
                } else {
                    Either::Right(())
                };

                if y == BSTNode::child(
                    &*BSTNode::child(&*x, direction.reverse()),
                    direction.reverse(),
                ) as *mut AVLNode<K, V>
                {
                    self.rotate(x, direction);
                } else {
                    self.double_rotate(x, direction);
                }
            }

            y = z;
            z = x;
        }
    }

    unsafe fn remove_retracing(
        &mut self,
        unbalanced_root: *mut AVLNode<K, V>,
    ) {
        let mut p = unbalanced_root;

        while !p.is_null() {
            (*p).height = 1 + max((*p).left_height(), (*p).right_height());

            if (*p).bf().abs() > 1 {
                let x = p;

                let direction = if (*x).right_height() > (*x).left_height() {
                    Either::Left(())
                } else {
                    Either::Right(())
                };

                let same_direction_height =
                    (*BSTNode::child(&*x, direction.reverse()))
                        .child_height(direction.reverse());

                let reverse_direction_height =
                    (*BSTNode::child(&*x, direction.reverse()))
                        .child_height(direction);

                p = if same_direction_height >= reverse_direction_height {
                    self.rotate(x, direction)
                } else {
                    self.double_rotate(x, direction)
                }
            }

            p = (*p).paren;
        }
    }
}


impl<'a, K: DictKey + Clone + 'a, V: Clone + 'a> Clone for AVL<K, V> {
    fn clone(&self) -> Self {
        if self.root.is_null() {
            return Self { root: null_mut() }
        }

        unsafe {
            let root = Box::into_raw(box (*self.root).clone());

            (*root).bfs_do(|x| {
                if !(*x).left().is_null() {
                    (*(*x).left()).assign_paren((*x).itself_mut());
                }

                if !(*x).right().is_null() {
                    (*(*x).right()).assign_paren((*x).itself_mut());
                }

            });

            Self { root }
        }
    }
}


impl<'a, K: DictKey + 'a, V: 'a> Dictionary<K, V> for AVL<K, V> {
    fn insert(&mut self, key: K, value: V) -> bool {
        let new_node = AVLNode::new(key, value);

        if !self.basic_insert(new_node) {
            return false;
        }

        unsafe {
            self.insert_retracing(new_node);
        }

        true
    }

    ///
    /// case-3
    ///       z
    ///      / \
    ///         y
    ///        / \
    ///     null  x
    ///          / \
    ///
    fn remove(&mut self, key: &K) -> Option<V> {
        let z = self.search_approximately(&key) as *mut AVLNode<K, V>;
        if z.is_null() {
            return None;
        }

        unsafe {
            if BSTNode::key(&*z) != key {
                return None;
            }

            let retracing_entry;
            if (*z).left().is_null() {
                retracing_entry = (*z).paren;
                self.subtree_shift(z, (*z).right());
            } else if (*z).right().is_null() {
                retracing_entry = (*z).paren;
                self.subtree_shift(z, (*z).left());
            } else {
                let y = (*z).successor();
                retracing_entry =
                    if (*y).paren() != z { (*y).paren_bst() } else { y }
                        as *mut AVLNode<K, V>;

                if (*y).paren() != z {
                    self.subtree_shift(y, (*y).right());

                    (*y).assign_right((*z).right());
                    (*(*y).right()).assign_paren(y);
                }

                self.subtree_shift(z, y);
                (*y).assign_left((*z).left());
                (*(*y).left()).assign_paren(y);

            }
            self.remove_retracing(retracing_entry);


            let origin_node = Box::from_raw(z as *mut AVLNode<K, V>);

            Some(origin_node.into_value())
        }
    }

    fn modify(&mut self, key: &K, value: V) -> bool {
        self.basic_modify(key, value)
    }

    fn lookup(&self, income_key: &K) -> Option<&V> {
        if let Some(e) = self.basic_lookup(income_key) {
            unsafe { Some(BSTNode::value(&*(e as *mut AVLNode<K, V>))) }
        } else {
            None
        }
    }

    fn lookup_mut(&mut self, income_key: &K) -> Option<&mut V> {
        if let Some(e) = self.basic_lookup(income_key) {
            unsafe { Some(BSTNode::value_mut(&*(e as *mut AVLNode<K, V>))) }
        } else {
            None
        }
    }

    fn self_validate(&self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.root.is_null() {
            unsafe { (*self.root).self_validate()? }
        }

        Ok(())
    }
}

impl<'a, K: DictKey + 'a, V: 'a> BST<'a, K, V> for AVL<K, V> {
    fn itself(&self) -> *const (dyn BST<'a, K, V> + 'a) {
        self as *const Self
    }

    fn root(&self) -> *mut (dyn BSTNode<'a, K, V> + 'a) {
        self.root
    }

    fn assign_root(&mut self, root: *mut (dyn BSTNode<'a, K, V> + 'a)) {
        self.root = root as *mut AVLNode<K, V>;
    }
}


#[cfg(test)]
mod tests {

    use itertools::Itertools;
    use rand::{prelude::SliceRandom, thread_rng};
    use serde_json;

    use super::AVL;
    use crate::{
        collections::{Dictionary, bt::bst::BST},
        test::{
            dict::{DictProvider, GetKey, Inode, InodeProvider},
            Provider,
        },
    };


    #[test]
    fn test_avl_randomdata() {
        let mut dict = AVL::new();

        let provider = InodeProvider {};

        (&provider as &dyn DictProvider<u32, Inode>).test_dict(&mut dict);
    }

    ///
    /// Debug AVL entry
    ///
    #[test]
    fn hack_avl() {
        for _ in 0..20 {
        let batch_num = 55;
        let mut collected_elems = vec![];
        let mut keys = vec![];
        let provider = InodeProvider {};
        let mut dict = AVL::new();

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
            assert!(dict.lookup(&keys.last().unwrap()).is_some());

            dict.self_validate().unwrap();

            i += 1;
        }

        let mut dict_debug = dict.clone();

        collected_elems.shuffle(&mut thread_rng());

        // Remove-> Verify
        for i in 0..batch_num {
            let e = &collected_elems[i];
            let k = &e.get_key();

            assert!(dict.remove(k).is_some());
            assert!(!dict.lookup(k).is_some());

            if let Ok(_res) = dict.self_validate() {
            } else {
                // restore the scene
                println!("{}", i);

                println!("DEBUG: {}", dict_debug.total());
                // dict_debug.echo_stdout();

                println!("ORIGIN: {}", dict.total());
                // dict.echo_stdout();


                for j in 0..i {
                    let e = &collected_elems[j];
                    let k = &e.get_key();

                    assert!(dict_debug.remove(k).is_some());
                    assert!(!dict_debug.lookup(k).is_some());
                    dict_debug.self_validate().unwrap();
                }

                unsafe {
                    let target = dict_debug.search_approximately(k);
                    let target_paren = (*target).paren_bst();

                    println!("Target: {:?}", k);
                    (*target_paren).just_echo_stdout();
                }

                dict_debug.remove(k).unwrap();
                dict_debug.self_validate().unwrap();
            }
        }}
    }

    #[test]
    fn test_avl_fixeddata_case_0() {
        let mut avl = AVL::<i32, ()>::new();

        let dict = &mut avl as &mut dyn Dictionary<i32, ()>;

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

        avl.self_validate().unwrap();
        avl.echo_stdout();
    }

    #[test]
    fn test_avl_fixeddata_case_1() {
        let mut avl = AVL::<u16, ()>::new();

        let dict = &mut avl as &mut dyn Dictionary<u16, ()>;

        dict.insert(52, ());
        assert!(dict.lookup(&52).is_some());

        dict.insert(47, ());
        assert!(dict.lookup(&47).is_some());

        dict.insert(3, ());
        assert!(dict.lookup(&3).is_some());

        dict.insert(35, ());
        assert!(dict.lookup(&35).is_some());

        dict.insert(24, ());
        assert!(dict.lookup(&24).is_some());

        // avl.echo_stdout();
    }

    #[test]
    fn test_avl_fixeddata_case_2() {
        let mut avl = AVL::<u16, ()>::new();

        let dict = &mut avl as &mut dyn Dictionary<u16, ()>;

        dict.insert(6, ());
        dict.insert(29, ());
        dict.insert(26, ());
        dict.insert(10, ());
        dict.insert(17, ());
        dict.insert(18, ());
        dict.insert(12, ());

        // avl.echo_stdout();
    }
}