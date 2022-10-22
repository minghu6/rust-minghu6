//! https://oi-wiki.org/ds/treap/
//!
//! https://en.wikipedia.org/wiki/Treap
//!



use std::cmp::{max, min};
use std::fmt;
use std::fmt::Write;
use std::ptr::{null, null_mut};

use either::Either;
use itertools::Itertools;
use rand::random;

use super::{BSTNode, BST};
use crate::collections::bt::{BTNode, BT};
use crate::collections::{CollKey, Dictionary, Heap};
use crate::etc::Reverse;
use crate::*;


////////////////////////////////////////////////////////////////////////////////
//// Struct



pub struct Treap<K, V, W = usize> {
    root: *mut TreapNode<K, V, W>,
}


struct TreapNode<K, V, W = usize> {
    left: *mut Self,
    right: *mut Self,
    paren: *mut Self,
    weight: W,
    key: *mut K,
    value: *mut V,
}



////////////////////////////////////////////////////////////////////////////////
//// Implement


impl<'a, K: CollKey + 'a, V: 'a, W: CollKey> TreapNode<K, V, W> {
    pub fn new(key: K, value: V, weight: W) -> *mut Self {
        Box::into_raw(box Self {
            left: null_mut(),
            right: null_mut(),
            paren: null_mut(),
            weight,
            key: Box::into_raw(box key),
            value: Box::into_raw(box value),
        })
    }

    fn into_value(self) -> V {
        unsafe { *Box::from_raw(self.value) }
    }

    /// validate red/black
    pub fn self_validate(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.basic_self_validate()?;

        unsafe {
            // Validate Max-Heap properties
            self.bfs_do(|x| {
                let x_self = &*(x as *mut TreapNode<K, V>);

                if !x_self.left.is_null() {
                    assert!(x_self.weight >= (*x_self.left).weight);
                }

                if !x_self.right.is_null() {
                    assert!(x_self.weight >= (*x_self.right).weight);
                }
            });

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
                let x_self = x as *mut TreapNode<K, V, W>;

                writeln!(cache, "W: {:?}", (*x_self).weight)
            })
        }
    }

    pub fn echo_stdout(&self) {
        let mut cache = String::new();

        self.echo_in_mm(&mut cache).unwrap();

        println!("{}", cache);
    }
}


impl<'a, K: CollKey + 'a, V: 'a, W: 'a> BTNode<'a, K, V>
    for TreapNode<K, V, W>
{
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

impl<'a, K: CollKey + 'a, V: 'a, W: 'a> BSTNode<'a, K, V>
    for TreapNode<K, V, W>
{
}


impl<'a, K: CollKey + 'a, V: 'a, W: CollKey> Treap<K, V, W> {
    pub fn new() -> Self {
        Self { root: null_mut() }
    }

    pub fn is_empty(&self) -> bool {
        self.root.is_null()
    }

    pub fn bulk_load(seq: &mut dyn Iterator<Item = (K, V, W)>) -> Self {
        let mut treap = Treap::new();
        // let mut rhlink = Vec::new();

        if let Some((key, value, weight)) = seq.next() {
            treap.insert_(key, value, weight);
            // rhlink.push(treap.root);
        }

        for (key, value, weight) in seq {
            unsafe {
                let mut x = treap.root;
                let mut prev = x;

                while !x.is_null() && (*x).weight > weight {
                    prev = x;
                    x = (*x).right;
                }

                let new_node = TreapNode::new(key, value, weight);

                if !x.is_null() {
                    let x_paren = (*x).paren;
                    (*new_node).connect_left(x);

                    if x_paren.is_null() {
                        treap.root = new_node;
                    } else {
                        (*x_paren).connect_right(new_node);
                    }
                } else {
                    (*prev).connect_right(new_node);
                }
            }
        }

        treap
    }

    pub fn echo_stdout(&self) {
        if !self.root.is_null() {
            unsafe { (*self.root).echo_stdout() }
        }
    }

    fn reset_root(&mut self, root: *mut TreapNode<K, V, W>) {
        self.root = root;

        unsafe {
            if !self.root.is_null() {
                (*self.root).paren = null_mut();
            }
        }
    }

    fn insert_(&mut self, key: K, value: V, weight: W) -> bool {
        if self.root.is_null() {
            self.root = TreapNode::new(key, value, weight);
            return true;
        }

        unsafe {
            if !Treap::find(self.root, &key).is_null() {
                return false;
            }

            let (mut lf, rh) = Treap::split(self.root, &key);

            lf = Treap::join(lf, TreapNode::new(key, value, weight));

            self.reset_root(Treap::join(lf, rh));
        }

        true
    }

    fn remove_(&mut self, key: &K) -> Option<Box<TreapNode<K, V, W>>> {
        if self.root.is_null() {
            return None;
        }

        unsafe {
            let x = Treap::find(self.root, key);

            if x.is_null() {
                return None;
            }

            let pred = (*x).precessor_bst() as *mut TreapNode<K, V, W>;

            if pred.is_null() {
                // key is the minimum
                let (_, rh) = Treap::split(self.root, key);
                self.reset_root(rh);
            } else {
                let pred_key = (*pred).key_bst();

                let (pred_lf, pred_rh) = Treap::split(self.root, pred_key);
                let (_, rh) = Treap::split(pred_rh, key);

                self.reset_root(Treap::join(pred_lf, rh));
            }

            Some(Box::from_raw(x))
        }
    }

    unsafe fn find(
        x: *mut TreapNode<K, V, W>,
        key: &K,
    ) -> *mut TreapNode<K, V, W> {
        if x.is_null() {
            return null_mut();
        }

        if key == (*x).key_bst() {
            return x;
        }

        if key < (*x).key_bst() {
            Treap::find((*x).left, key)
        } else {
            Treap::find((*x).right, key)
        }
    }


    /// Split into two by key.
    ///
    /// The left <= key
    ///
    /// The right > key
    ///
    unsafe fn split(
        t: *mut TreapNode<K, V, W>,
        key: &K,
    ) -> (*mut TreapNode<K, V, W>, *mut TreapNode<K, V, W>) {
        if t.is_null() {
            return (null_mut(), null_mut());
        }

        if key < (*t).key_bst() {
            let (lf_treap, part_rh_treap) = Treap::split((*t).left, key);
            (*t).connect_left(part_rh_treap);

            (lf_treap, t)
        } else {
            let (part_lf_treap, rh_treap) = Treap::split((*t).right, key);
            (*t).connect_right(part_lf_treap);

            (t, rh_treap)
        }
    }

    /// Join:
    ///
    /// merge left and right tree based on weight.
    ///
    /// **MUST:** all keys of u <= keys of v
    ///
    unsafe fn join(
        u: *mut TreapNode<K, V, W>,
        v: *mut TreapNode<K, V, W>,
    ) -> *mut TreapNode<K, V, W> {
        if u.is_null() {
            return v;
        }

        if v.is_null() {
            return u;
        }

        if (*u).weight > (*v).weight {
            (*u).connect_right(Treap::join((*u).right, v));

            u
        } else {
            (*v).connect_left(Treap::join(u, (*v).left));

            v
        }
    }
}


impl<'a, K: CollKey + 'a, V: 'a> BT<'a, K, V> for Treap<K, V> {
    fn order(&self) -> usize {
        2
    }

    fn root(&self) -> *mut (dyn BTNode<'a, K, V> + 'a) {
        self.root
    }

    fn assign_root(&mut self, root: *mut (dyn BTNode<'a, K, V> + 'a)) {
        self.root = root as *mut TreapNode<K, V>;
    }
}


impl<'a, K: CollKey + 'a, V: 'a> BST<'a, K, V> for Treap<K, V> {
    unsafe fn rotate_cleanup(
        &mut self,
        _x: *mut (dyn BSTNode<'a, K, V> + 'a),
        _z: *mut (dyn BSTNode<'a, K, V> + 'a),
    ) {
    }
}




impl<'a, K: CollKey + 'a, V: 'a> Dictionary<K, V> for Treap<K, V> {
    fn insert(&mut self, key: K, value: V) -> bool {
        self.insert_(key, value, random::<usize>())
    }

    fn remove(&mut self, key: &K) -> Option<V> {
        self.remove_(key).map(|node| node.into_value())
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
            }
        }

        Ok(())
    }
}


impl<W: CollKey> Heap<W> for Treap<usize, (), W> {
    fn top(&self) -> Option<&W> {
        if self.root.is_null() {
            None
        } else {
            unsafe { Some(&(*self.root).weight) }
        }
    }

    fn pop(&mut self) -> Option<W> {
        if self.root.is_null() {
            return None;
        }

        unsafe {
            let key = &*(*self.root).key;

            self.remove_(key).map(|node| node.weight)
        }
    }

    fn push(&mut self, val: W) {
        if self.root.is_null() {
            self.insert_(0, (), val);
            return;
        }

        unsafe {
            let max = (*(*self.root).maximum()).try_as_bst().unwrap();
            let max_key = (*max).key_bst();
            let nxy_key = max_key + 1;
            self.insert_(nxy_key, (), val);
        }
    }
}



#[cfg(test)]
mod test {
    use itertools::Itertools;
    use rand::{prelude::SliceRandom, random, thread_rng};

    use super::Treap;
    use crate::{
        collections::{
            bt::{
                bst::{BSTNode, BST, ROTATE_NUM},
                BTNode, BT,
            },
            Dictionary, Heap,
        },
        test::{
            dict::{DictProvider, GetKey},
            heap::HeapProvider,
            Provider, *,
        },
    };


    #[test]
    pub(crate) fn test_treap_randomdata() {
        let provider = InodeProvider {};

        (&provider as &dyn DictProvider<u32, Inode>)
            .test_dict(|| box Treap::new());
    }


    #[test]
    fn test_treap_fixeddata_case_0() {
        let mut treap = Treap::<i32, ()>::new();

        let dict = &mut treap as &mut dyn Dictionary<i32, ()>;

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

        treap.echo_stdout();
    }


    #[test]
    fn test_treap_fixeddata_case_1() {
        let mut treap = Treap::<i32, ()>::new();

        treap.insert(6, ());
        treap.insert(52, ());
        treap.insert(40, ());
        treap.insert(18, ());

        println!("BEFORE REMOVE");
        treap.echo_stdout();

        assert!(treap.remove(&40).is_some());
        assert!(treap.lookup(&40).is_none());
        treap.self_validate().unwrap();

        assert!(treap.remove(&6).is_some());
        assert!(treap.lookup(&6).is_none());
        treap.self_validate().unwrap();

        treap.echo_stdout();

        assert!(treap.remove(&18).is_some());
        assert!(treap.lookup(&18).is_none());
        treap.self_validate().unwrap();

        treap.echo_stdout();
    }


    #[test]
    fn test_treap_fixeddata_case_2() {
        let mut treap = Treap::<i32, ()>::new();

        treap.insert_(6, (), 14);
        treap.insert_(52, (), 21);
        treap.insert_(40, (), 82);
        treap.insert_(18, (), 22);

        assert!(treap.remove(&40).is_some());
        assert!(treap.remove(&6).is_some());
        assert!(treap.remove(&18).is_some());
        assert!(treap.lookup(&18).is_none());

        treap.self_validate().unwrap();

        treap.echo_stdout();
    }


    /// Debug Treap
    ///
    #[ignore = "Only used for debug"]
    #[test]
    fn hack_treap() {
        for _ in 0..20 {
            let batch_num = 14;
            let mut collected_elems = vec![];
            let mut keys = vec![];
            let provider = InodeProvider {};
            let dict = &mut Treap::new() as &mut dyn Dictionary<u32, Inode>;
            let m = 100;

            // Create
            let mut i = 0;
            while i < batch_num {
                let e = provider.get_one();
                let k = e.get_key() % m;
                if keys.contains(&k) {
                    continue;
                }

                keys.push(k.clone());
                collected_elems.push(e.clone());

                println!("insert {}: {:02?}", i, k);
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
                let k = &(e.get_key() % m);

                println!("remove: {}", k);

                assert!(dict.remove(k).is_some());
                assert!(!dict.lookup(k).is_some());

                match dict.self_validate() {
                    Ok(_) => (),
                    Err(err) => {
                        panic!("{}", err)
                    }
                }
            }
        }
    }


    #[test]
    fn test_treap_heap() {
        let provider = InodeProvider {};

        (&provider as &dyn HeapProvider<Inode>).test_heap(false, || box Treap::new());
    }


    #[test]
    fn test_treap_bulk_load() {
        let mut seq = (0..1000).map(|i| (i, (), random()));

        let treap = Treap::bulk_load(&mut seq);
        treap.self_validate().unwrap();
    }
}
