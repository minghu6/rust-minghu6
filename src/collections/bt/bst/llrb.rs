//! Left Learning Red Black Tree
//!
//! ref: https://oi-wiki.org/ds/llrbt/
//!
//! Painting edge instead of node (node's color indicates that it's edge to parent)
//! Extra Restriction: red should be left with black (that's why left-learning)
//!
//! red link is same with node's item of 2-4 tree, black link as different node respectly.
//!
//!      4-node equivalence                  4-node
//!
//!            | B
//!           (x)                            x--x--x
//!       R /     \ R         ===>         /  \  /  \
//!       (x)     (x)                     c0  c1 c2  c3
//!     B / \ B B / \ B
//!      c0 c1   c2  c3
//!
//!
//!     3-node equivalence                  3-node
//!
//!          | B
//!         (x)                              x--x
//!      R /   \ B           ===>           /  |  \
//!      (x)    c2                         c0   c1  c2
//!    B/  \B
//!    c0  c1
//!
//!
//!    2-node equivalence                 2-node
//!
//!         | B
//!         x                                x
//!      B / \ B             ===>           / \
//!       c0 c1                            c0  c1
//!





use std::cmp::max;
use std::fmt;
use std::fmt::Write;
use std::ptr::{null, null_mut};

use either::Either;
use itertools::Itertools;

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

    pub fn new_ptr(key: *mut K, value: *mut V) -> *mut Self {
        Box::into_raw(box Self {
            left: null_mut(),
            right: null_mut(),
            paren: null_mut(),
            color: Color::RED,
            key,
            value,
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
            } else {
                // if is_black(self.left) && is_black(self.right) {
                //     assert!(self.left.is_null());
                //     assert!(self.right.is_null());
                // }
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


    /// Equivalence of 2-4 tree node_size
    #[inline]
    unsafe fn b4_node_size(&self) -> usize {
        if is_red(self.right) {
            if is_red((*self.left).left)
                || is_red((*self.left).right)
                || is_red((*self.right).left)
                || is_red((*self.right).right) {
                    4
                } else {
                    3
                }
        } else if is_red(self.left) {
            2
        } else {
            1
        }
    }

    unsafe fn b4_node_is_fullfilled(&self) -> bool {
        self.b4_node_size() >= 3
    }

    unsafe fn b4_node_is_overfilled(&self) -> bool {
        self.b4_node_size() > 3
    }


    /// WARNING b4node shouldn't be overfilled
    unsafe fn get_b4_centre(&self) -> *mut Self {
        let self_ptr = self as *const Self as *mut Self;

        if is_red(self.left) || is_red(self.right) {
            self_ptr
        } else if self.color == Color::RED {
            self.paren
        } else {
            self_ptr
        }

    }

    /// self should be b4node centre
    unsafe fn is_b4_leaf(&self) -> bool {
        match self.b4_node_size() {
            1 => {
                self.left.is_null() && self.right.is_null()
            }
            2 => {
                (*self.left).left.is_null()
                && (*self.left).right.is_null()
                && self.right.is_null()
            }
            3 => {
                (*self.left).left.is_null()
                && (*self.left).right.is_null()
                && (*self.right).left.is_null()
                && (*self.right).right.is_null()
            }
            _ => unreachable!()
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


    unsafe fn promote(&mut self, x: *mut LLRBNode<K, V>) {
        debug_assert!(!x.is_null());
        let x_paren = (*x).paren;

        if x_paren.is_null() {
            set_black(x);
            return;
        }

        let mut x_dir = (*x).dir();
        let x_sibling
        = (*x_paren).child_bst(x_dir.reverse()) as *mut LLRBNode<K, V>;

        let mut u = x_paren;

        if is_red(x_sibling) {
            // origin 3-node is just ok!

            return;
        }

        if x_dir.is_right() {  // Both of 3-node and 4-node
            u = self.rotate(x_paren, Either::Left(())) as *mut LLRBNode<K, V>;
            x_dir = x_dir.reverse();
        }

        if is_black(u) {
            // origin 2-node is just ok!

            return;
        }


        // x_sibling is black && u is red
        let u_paren = (*u).paren;
        let u_dir = (*u).dir();
        let u_sibling
        = (*u_paren).child_bst(u_dir.reverse()) as *mut LLRBNode<K, V>;

        if is_black(u_sibling) {
            if u_dir == x_dir {
                self.rotate(u_paren, x_dir.reverse());
            } else {
                debug_assert!( !(*u_paren).child_bst(u_dir).is_null() );
                debug_assert!( !
                    (*(*u_paren).child_bst(u_dir))
                    .child_bst(u_dir.reverse()).is_null()
                );

                self.double_rotate(u_paren, x_dir);
            }
        } else {  // u_paren is overfilled
            (*u_paren).color_flip();

            self.promote(u_paren);
        }


    }

    /// dir: direction of removed child of paren
    unsafe fn unpromote(&mut self, paren: *mut LLRBNode<K, V>, dir: Either<(), ()>) {
        debug_assert!(!paren.is_null());

        let removed
        = BSTNode::child(&*paren, dir) as *mut LLRBNode<K, V>;

        let sibling
        = BSTNode::child(&*paren, dir.reverse()) as *mut LLRBNode<K, V>;

        debug_assert!(!removed.is_null());
        debug_assert!(!sibling.is_null());

        if (*sibling).b4_node_size() > 1 {  // Check >= 3-node sibling
            // split
            let popped
                = self.b4_node_pop(sibling, dir);
            self.subtree_shift(removed, popped);

            // redistribute
            (*paren).swap_with(popped);

            if !(*popped).is_leaf() {  // recursively go dwon
                self.unpromote(popped, dir.reverse());
            }
        } else {
            let paren_b4_centre = (*paren).get_b4_centre();

            // move down
            let popped
            = self.b4_node_pop(paren_b4_centre, dir);

            // merge
            self.b4_single_node_merge(sibling, popped);

        }


    }

    unsafe fn remove_retracing(
        &mut self,
        mut n: *mut LLRBNode<K, V>,
    ) -> *mut LLRBNode<K, V> {
        /* Prepare Deleting */
        if !(*n).right.is_null() {
            let successor = (*(*n).right).minimum() as *mut LLRBNode<K, V>;
            (*n).swap_with(successor);

            n = successor;  // left is null
        }

        // Either n.left or n.right is null
        if is_red(n) {
            // sibling of n is red or not, self.root is black
            let dir_n = (*n).dir();
            debug_assert!(BSTNode::child(&*n, dir_n).is_null());

            return self.b4_node_pop((*n).paren, dir_n)
        }


        let u = n;
        let (v, dir_v) = if (*u).left.is_null() {
            ((*u).right, Either::Right(()))
        } else {
            ((*u).left, Either::Left(()))
        };

        if is_red(v) {  // u must be top node of 3-node
            debug_assert!(dir_v.is_left());
            return self.b4_node_pop(u, dir_v.reverse())
        }

        // u is root
        if (*u).paren.is_null() {
            debug_assert!(v.is_null());
            self.root = null_mut();

            return u;
        }

        // u is non-root and both u, v are black
        self.unpromote((*u).paren, (*u).dir());

        u
    }

    // unsafe fn insert_retracing(&mut self, x: *mut LLRBNode<K, V>) {
    //     let p = (*x).paren;
    //     if p.is_null() {
    //         return;

    //     }
    // }

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


    /// Equivalence of 2-4 tree remove_node && shift with child if it's not leaf (non-knuth-leaf)
    unsafe fn b4_node_pop(&mut self, x: *mut LLRBNode<K, V>, pop_dir: Either<(), ()>) -> *mut LLRBNode<K, V> {

        let popped_node;
        if (*x).b4_node_size() == 3 {
            if pop_dir.is_right() {  // pop right、pop back
                popped_node = (*x).right;
            } else {
                popped_node = (*x).left;
            }


        } else if (*x).b4_node_size() == 2 {
            if pop_dir.is_right() {
                popped_node = x;
            } else {
                popped_node = (*x).left;
            }

        } else {  // nonsense
            unimplemented!()

        }

        if pop_dir.is_right() {
            self.subtree_shift(popped_node, (*popped_node).left);
            (*popped_node).left = null_mut();

        } else {
            self.subtree_shift(popped_node, (*popped_node).right);
            (*popped_node).right = null_mut();

        }

        set_black(popped_node);

        popped_node
    }

    // unsafe fn b4_node_merge(
    //     &mut self,
    //     fixed_x: *mut LLRBNode<K, V>,
    //     y: *mut LLRBNode<K, V>  // y is single node
    // ) {
    //     // fix_x should be b4node formal leaf (non-nil)

    //     let node_size = (*fixed_x).node_size();

    //     if node_size == 3 {
    //         unimplemented!()
    //     } else if node_size == 2 {
    //         if *(*y).key < *(*(*fixed_x).left).key {
    //             (*fixed_x).swap_with((*fixed_x).left);
    //             (*(*fixed_x).left).swap_with(y);
    //         } else if *(*y).key < *(*fixed_x).key {
    //             (*fixed_x).swap_with(y);
    //         }

    //         (*fixed_x).connect_right(y);

    //     } else if node_size == 1 {
    //         if *(*y).key > *(*fixed_x).key {
    //             (*fixed_x).swap_with(y);
    //         }

    //         (*fixed_x).connect_left(y);

    //     } else {
    //         unreachable!()
    //     }

    //     set_red(y);

    // }


    /// Reture new centre node
    unsafe fn b4_single_node_merge(
        &mut self,
        fixed_x: *mut LLRBNode<K, V>,
        y: *mut LLRBNode<K, V>
    ) -> *mut LLRBNode<K, V> {
        debug_assert!((*fixed_x).node_size() == 1 && is_black(fixed_x));
        debug_assert!((*y).node_size() == 1 && is_black(y));

        if *(*fixed_x).key > *(*y).key {
            (*fixed_x).connect_left(y);
            set_red(y);
        } else {
            self.subtree_shift(fixed_x, y);
            (*y).connect_left(fixed_x);
        }

        todo!()
    }

}



impl<'a, K: DictKey + 'a, V: 'a> Dictionary<K, V> for LLRB<K, V> {
    // fn insert(&mut self, key: K, value: V) -> bool {
    //     unsafe{
    //         let res;
    //         if self.root.is_null() {
    //             self.root = LLRBNode::new(key, value);

    //             res = true;
    //         } else {
    //             res = self.insert_at(self.root, key, value).is_ok();
    //         }

    //         set_black(self.root);
    //         res
    //     }
    // }


    fn insert(&mut self, key: K, value: V) -> bool {
        unsafe {
            let new_node = LLRBNode::new(key, value);
            let key = BSTNode::key(&*new_node);

            let approxi_node =
                (self.search_approximately(&key)) as *mut LLRBNode<K, V>;

            if approxi_node.is_null() {
                self.root = new_node;
                set_black(new_node);

                return true;
            }

            if key == BSTNode::key(&*approxi_node) {
                return false;
            }

            // let insert_entry = (*approxi_node).get_b4_centre();

            // default: red
            if *key < *(*approxi_node).key {
                (*approxi_node).connect_left(new_node);
            } else {
                (*approxi_node).connect_right(new_node);
            }

            self.promote(new_node);  // or alias as fixup

            true
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

        assert!(is_black(self.root));

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
        z: *mut (dyn BSTNode<'a, K, V> + 'a),  // new root
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
