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
//!
//! There are two variant version:
//!
//! 1. complete 2-4 (which break black balance),
//!
//! 1. complete black balance (which isn't complete 2-4)
//!
//!
//!
//!




use std::cmp::max;
use std::fmt;
use std::fmt::Write;
use std::ptr::{null, null_mut};

use either::Either;
use itertools::Itertools;

use super::rb::Color;
use super::{BSTNode, BST, ROTATE_NUM};
use crate::collections::bt::{BTNode, BT};
use crate::collections::{DictKey, Dictionary};
use crate::etc::Reverse;
use crate::*;


////////////////////////////////////////////////////////////////////////////////
//// Struct


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


#[derive(Copy)]
struct PhB4Node<K, V> {
    centre: *mut LLRBNode<K, V>,
}


struct PhB3Node<K, V> {
    centre: *mut LLRBNode<K, V>,
}



////////////////////////////////////////////////////////////////////////////////
//// Implement

fn is_black<K, V>(node: *mut LLRBNode<K, V>) -> bool {
    unsafe { node.is_null() || (*node).color == Color::BLACK }
}

fn is_nonnil_black<K, V>(node: *mut LLRBNode<K, V>) -> bool {
    unsafe {
        if node.is_null() {
            false
        } else {
            (*node).color == Color::BLACK
        }
    }
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

impl<K, V> Clone for PhB4Node<K, V> {
    fn clone(&self) -> Self {
        Self { centre: self.centre }
    }
}


#[allow(unused)]
impl<'a, K: DictKey + 'a, V: 'a> PhB4Node<K, V> {
    fn new(centre: *mut LLRBNode<K, V>) -> Self {
        Self { centre }
    }

    fn node_size(&self) -> usize {
        unsafe {
            if is_red((*self.centre).right) {
                if is_red((*(*self.centre).right).left)
                    || is_red((*(*self.centre).right).right)
                    || is_red((*(*self.centre).left).left)
                    || is_red((*(*self.centre).left).right) {
                        4
                    } else {
                        3
                    }

            } else if is_red((*self.centre).left) {
                2
            } else {
                1
            }
        }
    }

    fn child(&self, idx: usize) -> *mut LLRBNode<K, V> {
        let node_size = self.node_size();

        if idx > node_size {
            return null_mut();
        }

        unsafe {
            match (node_size, idx) {
                // 2-node
                (1, 0) => (*self.centre).left,
                (1, 1) => (*self.centre).right,

                // 3-node
                (2, 0) => (*(*self.centre).left).left,
                (2, 1) => (*(*self.centre).left).right,
                (2, 2) => (*self.centre).right,

                // 4-node
                (3, 0) => (*(*self.centre).left).left,
                (3, 1) => (*(*self.centre).left).right,
                (3, 2) => (*(*self.centre).right).left,
                (3, 3) => (*(*self.centre).right).right,

                // 5 or more
                _ => unimplemented!("{}-node[{}]", node_size, idx),
            }
        }
    }

    fn item(&self, idx: usize) -> *mut LLRBNode<K, V> {
        let node_size = self.node_size();

        if idx >= node_size {
            return null_mut();
        }

        unsafe {
            match (node_size, idx) {
                // 2-node
                (1, 0) => self.centre,

                // 3-node
                (2, 0) => (*self.centre).left,
                (2, 1) => self.centre,

                // 4-node
                (3, 0) => (*self.centre).left,
                (3, 1) => self.centre,
                (3, 2) => (*self.centre).right,

                // 5 or more
                _ => unimplemented!("{}-node[{}]", node_size, idx),
            }
        }
    }

    fn item_iter(
        &'a self,
    ) -> Box<dyn Iterator<Item = *mut LLRBNode<K, V>> + 'a> {
        let mut i = -1i32;

        box std::iter::from_fn(move || -> Option<*mut LLRBNode<K, V>> {
            i += 1;
            let item = self.item(i as usize);

            if item.is_null() {
                None
            } else {
                Some(item)
            }
        })
    }

    fn children_iter(
        &'a self,
    ) -> Box<dyn Iterator<Item = *mut LLRBNode<K, V>> + 'a> {
        let mut i = -1i32;

        box std::iter::from_fn(move || -> Option<*mut LLRBNode<K, V>> {
            i += 1;
            let item = self.child(i as usize);

            if item.is_null() {
                None
            } else {
                Some(item)
            }
        })
    }

    /// How many key-value pairs does B-node contains
    ///
    /// EXP: 2-node, contains 1 item, 3 node contains 2 item, ...
    ///
    /// (node_size, item_idx)
    #[allow(unused)]
    fn index_of_item(&self, income: *mut LLRBNode<K, V>) -> (usize, usize) {
        for (i, item) in self.item_iter().enumerate() {
            if item == income {
                return (self.node_size(), i);
            }
        }

        unreachable!()
    }

    #[allow(unused)]
    fn index_of_child(&self, income: *mut LLRBNode<K, V>) -> (usize, usize) {
        for (i, child) in self.children_iter().enumerate() {
            if child == income {
                return (self.node_size(), i);
            }
        }

        unreachable!()
    }

    unsafe fn pop_item(
        &mut self,
        t: &mut LLRB<K, V>,
        dir: Either<(), ()>,
    ) -> *mut LLRBNode<K, V> {
        match (self.node_size(), dir) {
            (3, Either::Left(())) => {
                let child = (*self.centre).left;

                (*self.centre).assign_left(null_mut::<LLRBNode<K, V>>());
                self.centre =
                t.rotate(self.centre, Either::Left(()))
                as *mut LLRBNode<K, V>;

                set_black(child);

                child
            }
            (3, Either::Right(())) => {
                let child = (*self.centre).right;

                (*self.centre).assign_right(null_mut::<LLRBNode<K, V>>());
                set_black(child);

                child
            }
            (2, Either::Left(())) => {
                let child = (*self.centre).left;

                (*self.centre).assign_left(null_mut::<LLRBNode<K, V>>());
                set_black(child);

                child
            }
            (2, Either::Right(())) => {
                // Refactor LLRB
                t.subtree_shift(self.centre, (*self.centre).left);
                set_black((*self.centre).left);

                // Update Popped Node
                let child = self.centre;
                (*child).left = null_mut();
                set_black(child);

                // Update B4Node
                self.centre = (*self.centre).left;

                child
            }

            _ => unreachable!(),
        }
    }


    // unsafe fn push_item(
    //     &mut self,
    //     income: *mut LLRBNode<K, V>,
    //     dir: Either<(), ()>,
    // ) {
    //     set_red(income);

    //     match (self.node_size(), dir) {
    //         (1, Either::Left(())) => {

    //         }
    //         (1, Either::Right(())) => {

    //         }


    //         // (2, Either::Left(())) => {

    //         // }
    //         // (2, Either::Right(())) => {

    //         // }

    //         _ => unimplemented!("{}: {:?}", self.node_size(), dir),
    //     }


    // }

    fn connect_child(&mut self, child: *mut LLRBNode<K, V>, idx: usize) {
        let node_size = self.node_size();

        debug_assert!(idx <= node_size);

        set_black(child);

        unsafe {
            match (node_size, idx) {
                // 2-node
                (1, 0) => (*self.centre).connect_left(child),
                (1, 1) => (*self.centre).connect_right(child),

                // 3-node
                (2, 0) => (*(*self.centre).left).connect_left(child),
                (2, 1) => (*(*self.centre).left).connect_right(child),
                (2, 2) => (*self.centre).connect_right(child),

                // 4-node
                (3, 0) => (*(*self.centre).left).connect_left(child),
                (3, 1) => (*(*self.centre).left).connect_right(child),
                (3, 2) => (*(*self.centre).right).connect_left(child),
                (3, 3) => (*(*self.centre).right).connect_right(child),

                // 5 or more
                _ => unimplemented!("{}-node[{}]", node_size, idx),
            }
        }
    }

    fn swap_item(&mut self, income: *mut LLRBNode<K, V>, idx: usize) {
        let item = self.item(idx);

        unsafe {
            (*item).swap_with(income);
        }
    }

    // is b4 leaf
    fn is_leaf(&self) -> bool {
        // self.children_iter()
        // .all(|child| child.is_null())

        todo!()
    }
}


impl<'a, K: DictKey + 'a, V: 'a> PhB3Node<K, V> {
    fn new(centre: *mut LLRBNode<K, V>) -> Self {
        Self { centre }
    }

    fn node_size(&self) -> usize {
        unsafe {
            debug_assert!(is_black(self.centre));

            if is_red((*self.centre).left) {
                if is_red((*self.centre).right) {
                    3
                } else {
                    2
                }
            } else {
                1
            }

        }
    }

    fn child(&self, idx: usize) -> *mut LLRBNode<K, V> {
        let node_size = self.node_size();

        if idx > node_size {
            return null_mut();
        }

        unsafe {
            match (node_size, idx) {
                // 2-node
                (1, 0) => (*self.centre).left,
                (1, 1) => (*self.centre).right,

                // 3-node
                (2, 0) => (*(*self.centre).left).left,
                (2, 1) => (*(*self.centre).left).right,
                (2, 2) => (*self.centre).right,

                // 4 or more
                _ => unimplemented!("{}-node[{}]", node_size, idx),
            }
        }
    }

    fn item(&self, idx: usize) -> *mut LLRBNode<K, V> {
        let node_size = self.node_size();

        if idx >= node_size {
            return null_mut();
        }

        unsafe {
            match (node_size, idx) {
                // 2-node
                (1, 0) => self.centre,

                // 3-node
                (2, 0) => (*self.centre).left,
                (2, 1) => self.centre,

                // 4 or more
                _ => unimplemented!("{}-node[{}]", node_size, idx),
            }
        }
    }

    fn item_iter(
        &'a self,
    ) -> Box<dyn Iterator<Item = *mut LLRBNode<K, V>> + 'a> {
        let mut i = -1i32;

        box std::iter::from_fn(move || -> Option<*mut LLRBNode<K, V>> {
            i += 1;
            let item = self.item(i as usize);

            if item.is_null() {
                None
            } else {
                Some(item)
            }
        })
    }

    fn children_iter(
        &'a self,
    ) -> Box<dyn Iterator<Item = *mut LLRBNode<K, V>> + 'a> {
        let mut i = -1i32;

        box std::iter::from_fn(move || -> Option<*mut LLRBNode<K, V>> {
            i += 1;
            let item = self.child(i as usize);

            if item.is_null() {
                None
            } else {
                Some(item)
            }
        })
    }

    /// How many key-value pairs does B-node contains
    ///
    /// EXP: 2-node, contains 1 item, 3 node contains 2 item, ...
    ///
    /// (node_size, item_idx)
    #[allow(unused)]
    fn index_of_item(&self, income: *mut LLRBNode<K, V>) -> (usize, usize) {
        for (i, item) in self.item_iter().enumerate() {
            if item == income {
                return (self.node_size(), i);
            }
        }

        unreachable!()
    }

    #[allow(unused)]
    fn index_of_child(&self, income: *mut LLRBNode<K, V>) -> (usize, usize) {
        for (i, child) in self.children_iter().enumerate() {
            if child == income {
                return (self.node_size(), i);
            }
        }

        unreachable!()
    }

    unsafe fn pop_item(
        &mut self,
        t: &mut LLRB<K, V>,
        dir: Either<(), ()>,
    ) -> *mut LLRBNode<K, V> {
        match (self.node_size(), dir) {
            (2, Either::Left(())) => {
                let child = (*self.centre).left;

                (*self.centre).connect_left((*child).right);

                (*child).right = null_mut();
                set_black(child);

                child
            }
            (2, Either::Right(())) => {
                let root
                = t.rotate(self.centre, Either::Right(())) as *mut LLRBNode<K, V>;

                let child = (*root).right;
                (*root).connect_right((*child).left);

                // Update B4Node
                self.centre = root;

                (*child).left = null_mut();
                set_black(child);

                child
            }

            _ => unreachable!(),
        }
    }

    fn connect_child(&mut self, child: *mut LLRBNode<K, V>, idx: usize) {
        let node_size = self.node_size();

        debug_assert!(idx <= node_size);

        set_black(child);

        unsafe {
            match (node_size, idx) {
                // 2-node
                (1, 0) => (*self.centre).connect_left(child),
                (1, 1) => (*self.centre).connect_right(child),

                // 3-node
                (2, 0) => (*(*self.centre).left).connect_left(child),
                (2, 1) => (*(*self.centre).left).connect_right(child),
                (2, 2) => (*self.centre).connect_right(child),

                // 4 or more
                _ => unimplemented!("{}-node[{}]", node_size, idx),
            }
        }
    }

    fn swap_item(&mut self, income: *mut LLRBNode<K, V>, idx: usize) {
        let item = self.item(idx);

        unsafe {
            (*item).swap_with(income);
        }
    }

    // // is b3 leaf
    // fn is_leaf(&self) -> bool {
    //     todo!()

    // }

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

    /// validate red/black
    pub fn self_validate(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.basic_self_validate()?;

        unsafe {
            if self.color == Color::RED {
                // right-learning red violation
                assert!(!(is_black(self.left) && is_red(self.right)));

                // // at most 4-node
                // if is_red(self.left) && is_red(self.right) {
                //     assert!(!is_red(self.paren));

                //     assert!(!is_red((*self.left).left));
                //     assert!(!is_red((*self.left).right));

                //     assert!(!is_red((*self.right).left));
                //     assert!(!is_red((*self.right).right));
                // }

                // at most 3-node
                if is_red(self.left) {
                    assert!(is_black(self.right));
                }

            } else {
                // Complete 2-4 Tree would break black balance
                // if !self.left.is_null()
                // && !self.right.is_null()
                // && is_black(self.left)
                // && is_black(self.right)
                // {

                //     assert!(
                //         (*self.left).b4_node_size() > 1
                //         || (*self.right).b4_node_size() > 1
                //     );
                // }
            }

            // unbalanced
            assert!(!
                (is_red(self.left) && is_red((*self.left).left))
            );

            // Validate 2-4 tree property
            // let phb4node = PhantomB4Node::new(self.b4_centre());

            // if phb4node.children_iter().any(|child| child.is_null())
            // {
            //     assert!(
            //         phb4node.children_iter().all(|child| child.is_null())
            //     )
            // }

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
            unsafe {
                (*self.left).color = (*self.left).color.reverse();
            }
        }

        if !self.right.is_null() {
            unsafe {
                (*self.right).color = (*self.right).color.reverse();
            }
        }
    }


    /// WARNING b-node shouldn't be overfilled
    /// b3 or b4 centre
    #[allow(unused)]
    unsafe fn centre(&self) -> *mut Self {
        let self_ptr = self as *const Self as *mut Self;

        if is_red(self.left) {
            self_ptr
        } else if self.color == Color::RED {
            self.paren
        } else {
            self_ptr
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
        #[cfg(test)] {
            unsafe { ROTATE_NUM = 0; }
        }

        Self { root: null_mut() }
    }

    #[allow(unused)]
    unsafe fn promote(&mut self, x: *mut LLRBNode<K, V>) {
        debug_assert!(!x.is_null());
        let x_paren = (*x).paren;

        if x_paren.is_null() {
            set_black(x);

            // if !(*x).left.is_null() {
            //     (*(*x).left).try_merge_spec_2_node();
            // }

            // if !(*x).right.is_null() {
            //     (*(*x).right).try_merge_spec_2_node();
            // }

            return;
        }

        let mut x_dir = (*x).dir();
        let x_sibling =
            (*x_paren).child_bst(x_dir.reverse()) as *mut LLRBNode<K, V>;

        let mut u = x_paren;

        if is_red(x_sibling) {
            // origin 3-node is just ok!

            return;
        }

        if x_dir.is_right() {
            // Both of 3-node and 4-node
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
        let u_sibling =
            (*u_paren).child_bst(u_dir.reverse()) as *mut LLRBNode<K, V>;

        if is_black(u_sibling) {
            if u_dir == x_dir {
                self.rotate(u_paren, x_dir.reverse());
            } else {
                debug_assert!(!(*u_paren).child_bst(u_dir).is_null());
                debug_assert!(!(*(*u_paren).child_bst(u_dir))
                    .child_bst(u_dir.reverse())
                    .is_null());

                self.double_rotate(u_paren, x_dir);
            }
        } else {
            // u_paren is overfilled
            (*u_paren).color_flip();

            // (*u_sibling).try_merge_spec_2_node();

            self.promote(u_paren);
        }
    }

    /// B4 Version
    /// Params: b4centre, removed child index of this centre
    // #[allow(unused)]
    // unsafe fn unpromote(
    //     &mut self,
    //     mut paren_b4: PhB4Node<K, V>,
    //     leaf_idx: usize,
    // ) {
    //     if paren_b4.is_leaf() {
    //         return;
    //     }

    //     let rh_sibl = paren_b4.child(leaf_idx + 1);

    //     if !rh_sibl.is_null() && (*rh_sibl).b3_node_size() > 1 {
    //         let mut rh_sibl_b4 = PhB4Node::new(rh_sibl);

    //         // split right_sibling
    //         let split_sibl = rh_sibl_b4.pop_item(self, Either::Left(()));

    //         // connect split sibling
    //         paren_b4.connect_child(split_sibl, leaf_idx);

    //         // redistribute
    //         paren_b4.swap_item(split_sibl, leaf_idx);
    //         // swap branch
    //         (*split_sibl).connect_right((*split_sibl).left);
    //         (*split_sibl).connect_left(null_mut::<LLRBNode<K, V>>());

    //         // handle 1-key-val, 1 br with child of split sibling
    //         let nxt_b4 = PhB4Node::new(split_sibl);
    //         let nxt_step_idx = 0;

    //         self.unpromote(nxt_b4, nxt_step_idx);

    //         return;
    //     }

    //     if leaf_idx > 0
    //         && !paren_b4.child(leaf_idx - 1).is_null()
    //         && (*paren_b4.child(leaf_idx - 1)).b3_node_size() > 1
    //     {
    //         let lf_sibl = paren_b4.child(leaf_idx - 1);
    //         let mut lf_sibl_b4 = PhB4Node::new(lf_sibl);

    //         // split left sibling
    //         let split_sibl = lf_sibl_b4.pop_item(self, Either::Right(()));

    //         // connect split sibling
    //         paren_b4.connect_child(split_sibl, leaf_idx);

    //         // redistribute
    //         paren_b4.swap_item(split_sibl, leaf_idx - 1);
    //         // swap branch
    //         (*split_sibl).connect_left((*split_sibl).right);
    //         (*split_sibl).connect_right(null_mut::<LLRBNode<K, V>>());

    //         // recur
    //         let nxt_b4 = PhB4Node::new(split_sibl);
    //         let nxt_step_idx = nxt_b4.node_size();
    //         self.unpromote(nxt_b4, nxt_step_idx);

    //         return;
    //     }


    //     // For 1-key-val node
    //     // Move down && Merge (Recursive)
    //     let sibl_dir;
    //     let sibl_idx;
    //     let sibl;

    //     match (paren_b4.node_size(), leaf_idx) {
    //         (3, 1) | (3, 3) | (2, 1) | (2, 2) | (1, 1) => {
    //             sibl_idx = leaf_idx - 1;
    //             sibl_dir = Either::Left(());
    //             sibl = paren_b4.child(leaf_idx - 1);
    //         }

    //         _ => {
    //             sibl_idx = leaf_idx + 1;
    //             sibl_dir = Either::Right(());
    //             sibl = rh_sibl;
    //         }
    //     }

    //     debug_assert!((*sibl).left.is_null() && (*sibl).right.is_null());


    //     // move down && merge
    //     match (paren_b4.node_size(), sibl_dir) {
    //         // 1
    //         (1, Either::Left(())) => {
    //             set_red(sibl);
    //             self.fix_spec_2_node_up(paren_b4.centre);
    //         }
    //         (1, Either::Right(())) => {
    //             set_red(sibl);
    //             self.rotate(paren_b4.centre, Either::Left(()));
    //             self.fix_spec_2_node_up(sibl);
    //         }


    //         // 2
    //         (2, Either::Left(())) => {
    //             let paren_lf = paren_b4.item(0);

    //             if sibl_idx == 1 {
    //                 let paren = paren_b4.centre;

    //                 self.subtree_shift(paren, paren_lf);
    //                 set_black(paren_lf);

    //                 (*sibl).connect_right(paren);
    //                 set_red(paren);
    //                 (*paren).connect_left(null_mut::<LLRBNode<K, V>>());
    //                 debug_assert!((*paren).right.is_null());

    //                 self.rotate(sibl, Either::Left(()));

    //             } else {  // sibl_idx = 0
    //                 (*paren_lf).color_flip();

    //             };

    //         }
    //         (2, Either::Right(())) => {
    //             debug_assert!(sibl_idx == 1);

    //             let paren_lf = paren_b4.item(0);

    //             (*paren_lf).color_flip();

    //             self.rotate(paren_lf, Either::Left(()));
    //         }


    //         // 3
    //         (3, Either::Left(())) => {
    //             if sibl_idx == 0 {
    //                 let paren_lf = paren_b4.item(0);

    //                 (*paren_lf).color_flip();
    //                 self.rotate(paren_b4.centre, Either::Left(()));

    //             } else {
    //                 debug_assert!(sibl_idx == 2);

    //                 let paren_rh = paren_b4.item(2);

    //                 (*paren_rh).color_flip();
    //             }

    //         }
    //         (3, Either::Right(())) => {
    //             if sibl_idx == 3 {
    //                 let paren_rh = paren_b4.item(2);

    //                 (*paren_rh).color_flip();
    //                 self.rotate(paren_rh, Either::Left(()));

    //             } else {
    //                 debug_assert!(sibl_idx == 1);

    //                 let paren_lf = paren_b4.item(0);

    //                 (*paren_lf).color_flip();

    //                 self.rotate(paren_b4.centre, Either::Left(()));
    //                 self.rotate(paren_lf, Either::Left(()));
    //             }
    //         }

    //         _ => unreachable!()
    //     };


    // }


    #[allow(unused)]
    unsafe fn unpromote_b3(
        &mut self,
        mut paren_b3: PhB3Node<K, V>,
        leaf_idx: usize,
    ) {
        if (*paren_b3.centre).is_leaf() {
            return;
        }

        let rh_sibl = paren_b3.child(leaf_idx + 1);
        let mut rh_sibl_b3 = PhB3Node::new(rh_sibl);

        if !rh_sibl.is_null() && rh_sibl_b3.node_size() > 1 {
            // split right_sibling
            let split_sibl = rh_sibl_b3.pop_item(self, Either::Left(()));

            // connect split sibling
            paren_b3.connect_child(split_sibl, leaf_idx);

            // redistribute
            paren_b3.swap_item(split_sibl, leaf_idx);
            // swap branch
            (*split_sibl).connect_right((*split_sibl).left);
            (*split_sibl).connect_left(null_mut::<LLRBNode<K, V>>());

            // handle 1-key-val, 1 br with child of split sibling
            let nxt_b3 = PhB3Node::new(split_sibl);
            let nxt_step_idx = 0;

            self.unpromote_b3(nxt_b3, nxt_step_idx);

            return;
        }

        if leaf_idx > 0 && !paren_b3.child(leaf_idx - 1).is_null() {
            let lf_sibl = paren_b3.child(leaf_idx - 1);
            let mut lf_sibl_b3 = PhB3Node::new(lf_sibl);

            if lf_sibl_b3.node_size() > 1 {
                // split left sibling
                let split_sibl = lf_sibl_b3.pop_item(self, Either::Right(()));

                // connect split sibling
                paren_b3.connect_child(split_sibl, leaf_idx);

                // redistribute
                paren_b3.swap_item(split_sibl, leaf_idx - 1);
                // swap branch
                (*split_sibl).connect_left((*split_sibl).right);
                (*split_sibl).connect_right(null_mut::<LLRBNode<K, V>>());

                // recur
                let nxt_b3 = PhB3Node::new(split_sibl);
                let nxt_step_idx = nxt_b3.node_size();

                self.unpromote_b3(nxt_b3, nxt_step_idx);

                return;
            }

        }


        // For 1-key-val node
        // Move down && Merge (Recursive)
        let sibl_dir;
        let sibl_idx;
        let sibl;

        match (paren_b3.node_size(), leaf_idx) {
            (2, 1) | (2, 2) | (1, 1) => {
                sibl_idx = leaf_idx - 1;
                sibl_dir = Either::Left(());
                sibl = paren_b3.child(leaf_idx - 1);
            }

            _ => {
                sibl_idx = leaf_idx + 1;
                sibl_dir = Either::Right(());
                sibl = rh_sibl;
            }
        }

        debug_assert!((*sibl).left.is_null() && (*sibl).right.is_null());


        // move down && merge
        match (paren_b3.node_size(), sibl_dir) {
            // 1
            (1, Either::Left(())) => {
                set_red(sibl);
            }
            (1, Either::Right(())) => {
                set_red(sibl);
                self.rotate(paren_b3.centre, Either::Left(()));
            }


            // 2
            (2, Either::Left(())) => {
                let paren_lf = paren_b3.item(0);

                if sibl_idx == 1 {
                    let paren = paren_b3.centre;

                    self.subtree_shift(paren, paren_lf);
                    set_black(paren_lf);

                    (*sibl).connect_right(paren);
                    set_red(paren);
                    (*paren).connect_left(null_mut::<LLRBNode<K, V>>());
                    debug_assert!((*paren).right.is_null());

                    self.rotate(sibl, Either::Left(()));

                } else {  // sibl_idx = 0
                    (*paren_lf).color_flip();

                };

            }
            (2, Either::Right(())) => {
                debug_assert!(sibl_idx == 1);

                let paren_lf = paren_b3.item(0);

                (*paren_lf).color_flip();

                self.rotate(paren_lf, Either::Left(()));
            }

            _ => unreachable!()
        };


    }

    /// Fix black balance
    #[allow(unused)]
    unsafe fn fix_spec_2_node_up(&mut self, x: *mut LLRBNode<K, V>) {
        let x_paren = (*x).paren;

        if x_paren.is_null() {
            return;
        }

        let sibling = (*x).sibling() as *mut LLRBNode<K, V>;

        if is_nonnil_black((*sibling).left)
        && is_nonnil_black((*sibling).right) {
            set_red((*sibling).left);
            set_red((*sibling).right);
        }

        self.fix_spec_2_node_up(x_paren)
    }

    unsafe fn fixup(&mut self, mut x: *mut LLRBNode<K, V>)
    -> *mut LLRBNode<K, V>
    {
        debug_assert!(!x.is_null());

        if is_red((*x).right) {
            x = self.rotate(x, Either::Left(())) as *mut LLRBNode<K, V>;
        }

        if is_red((*x).left) && is_red((*(*x).left).left) {
            x = self.rotate(x, Either::Right(())) as *mut LLRBNode<K, V>;
        }

        if is_red((*x).left) && is_red((*x).right) {
            (*x).color_flip();
        }

        x
    }


    unsafe fn remove_min_(&mut self, mut x: *mut LLRBNode<K, V>)
    -> *mut LLRBNode<K, V>
    {
        if (*x).left.is_null() {
            return null_mut();
        }

        if !is_red((*x).left) && !is_red((*(*x).left).left) {
            x = self.move_red_left(x);
        }

        (*x).connect_left(self.remove_min_((*x).left));

        self.fixup(x)
    }

    ///```no_run
    ///    | r                     | r
    ///   (a)      <-- x -->      (b)
    /// b / \ r       ===>      b / \ b
    ///     (c)                 (a) (c)
    ///   r / \               r / b / \ b
    ///   (b)
    ///```
    unsafe fn move_red_left(&mut self, mut x: *mut LLRBNode<K, V>)
    -> *mut LLRBNode<K, V>
    {
        (*x).color_flip();

        if is_red((*(*x).right).left) {
            x = self.double_rotate(x, Either::Left(())) as *mut LLRBNode<K, V>;

            (*x).color_flip();
        }

        x
    }


    unsafe fn move_red_right(&mut self, mut x: *mut LLRBNode<K, V>)
    -> *mut LLRBNode<K, V>
    {
        (*x).color_flip();

        if is_red((*(*x).left).left) {
            x = self.rotate(x, Either::Right(())) as *mut LLRBNode<K, V>;
            (*x).color_flip();
        }

        x
    }

    #[allow(unused)]
    unsafe fn remove_(&mut self, mut x: *mut LLRBNode<K, V>, key: &K, res: &mut Vec<V>)
    -> *mut LLRBNode<K, V>
    {
        if key < (*x).key_bst() {
            let x_lf = (*x).left;

            if !x_lf.is_null() {
                if (!is_red(x_lf) && !is_red((*x_lf).left)) {
                    x = self.move_red_left(x);
                }

                (*x).connect_left(
                    self.remove_((*x).left, key, res)
                );
            }

        } else {

            if is_red((*x).left) {
                x = self.rotate(x, Either::Right(())) as *mut LLRBNode<K, V>;
            }

            if key == (*x).key_bst() && (*x).right.is_null() {
                res.push(*Box::from_raw((*x).value));

                return null_mut();
            }

            if !(*x).right.is_null() {
                if !is_red((*x).right) && !is_red((*(*x).right).left) {
                    x = self.move_red_right(x);
                }

                if key == (*x).key_bst() {
                    let nxt = (*x).successor_bst();

                    res.push(*Box::from_raw((*x).value));

                    (*x).key = (*nxt).key_ptr(0);
                    (*x).value = (*nxt).val_ptr(0);

                    (*x).connect_right(
                        self.remove_min_((*x).right)
                    )
                }
                else {
                    (*x).connect_right(
                        self.remove_((*x).right, key, res)
                    )
                }
            }

        }

        self.fixup(x)

    }

    // insert at x
    #[allow(unused)]
    unsafe fn insert_at(
        &mut self,
        mut x: *mut LLRBNode<K, V>,
        key: K,
        value: V,
    ) -> Result<*mut LLRBNode<K, V>, ()> {
        if x.is_null() {
            return Ok(LLRBNode::new(key, value));
        }

        // // split 4-node (protect from possible 5-node) on the way down
        // if is_red((*x).left) && is_red((*x).right) {
        //     (*x).color_flip();
        // }

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

        // split 4-node (protect from possible 5-node) on the way down
        if is_red((*x).left) && is_red((*x).right) {
            (*x).color_flip();
        }

        Ok(x)
    }

    pub fn echo_stdout(&self) {
        if !self.root.is_null() {
            unsafe { (*self.root).echo_stdout() }
        } else {
            println!("EMPTY.")
        }
    }

}



impl<'a, K: DictKey + 'a, V: 'a> Dictionary<K, V> for LLRB<K, V> {
    fn insert(&mut self, key: K, value: V) -> bool {
        unsafe{
            let res;
            if self.root.is_null() {
                self.root = LLRBNode::new(key, value);

                res = true;
            } else {
                res = self.insert_at(self.root, key, value).is_ok();
            }

            set_black(self.root);
            res
        }
    }

    // Recur Version
    fn remove(&mut self, key: &K) -> Option<V> {
        unsafe {
            if self.root.is_null() {
                return None;
            }

            let mut res = Vec::new();
            let t = self.remove_(self.root, key, &mut res);
            self.reset_root(t);
            set_black(self.root);

            res.pop()

        }

    }

    // /// clone from 2-4 version
    // fn insert(&mut self, key: K, value: V) -> bool {
    //     unsafe {
    //         let new_node = LLRBNode::new(key, value);
    //         let key = BSTNode::key_bst(&*new_node);

    //         let approxi_node =
    //             (self.search_approximately(&key)) as *mut LLRBNode<K, V>;

    //         if approxi_node.is_null() {
    //             self.root = new_node;
    //             set_black(new_node);

    //             return true;
    //         }

    //         if key == BSTNode::key_bst(&*approxi_node) {
    //             return false;
    //         }

    //         // let insert_entry = (*approxi_node).get_b4_centre();

    //         // default: red
    //         if *key < *(*approxi_node).key {
    //             (*approxi_node).connect_left(new_node);
    //         } else {
    //             (*approxi_node).connect_right(new_node);
    //         }

    //         self.promote(new_node); // or alias as fixup

    //         true
    //     }
    // }



    // fn remove(&mut self, key: &K) -> Option<V> {
    //     unsafe {
    //         let approxi_node =
    //             (*self.search_approximately(&key)).try_as_bst_mut().unwrap();

    //         if approxi_node.is_null() {
    //             return None;
    //         }

    //         if BSTNode::key_bst(&*approxi_node) != key {
    //             return None;
    //         }

    //         let mut x = approxi_node as *mut LLRBNode<K, V>;

    //         /* Prepare Deleting */
    //         if !(*x).right.is_null() {
    //             let successor =
    //                 (*(*x).right).minimum() as *mut LLRBNode<K, V>;
    //             (*x).swap_with(successor);

    //             x = successor; // x.left is null
    //         } // else x.right is null

    //         let x_b4 = PhantomB4Node::new((*x).b4_centre());

    //         match x_b4.index_of_item(x) {
    //             (1, _) => {
    //                 debug_assert!(x == x_b4.centre);
    //                 debug_assert!((*x).left.is_null() && (*x).right.is_null());

    //                 let x_paren = (*x).paren;
    //                 if x_paren.is_null() {
    //                     self.root = null_mut();
    //                 } else {
    //                     let x_paren_b4 = PhantomB4Node::new((*x_paren).b4_centre());

    //                     let x_b4_idx = x_paren_b4.index_of_child(x).1;

    //                     self.subtree_shift(x, null_mut::<LLRBNode<K, V>>());
    //                     self.unpromote(
    //                         x_paren_b4,
    //                         x_b4_idx,
    //                     );
    //                 }

    //             }
    //             (2, idx) => {
    //                 if idx == 0 {
    //                     self.subtree_shift(x, null_mut::<LLRBNode<K, V>>());
    //                 } else {
    //                     self.subtree_shift(x, (*x).left);
    //                     set_black((*x).left);
    //                 }
    //             }
    //             (3, idx) => {
    //                 debug_assert!(idx == 0 || idx == 2);
    //                 self.subtree_shift(x, null_mut::<LLRBNode<K, V>>());

    //                 if idx == 0 {
    //                     self.rotate(x_b4.centre, Either::Left(()));
    //                 }
    //             }
    //             _ => unreachable!(),
    //         }

    //         Some(LLRBNode::node_into_value(x))
    //     }
    // }


    // // B3 Version
    // fn remove(&mut self, key: &K) -> Option<V> {
    //     unsafe {
    //         let approxi_node =
    //             (*self.search_approximately(&key)).try_as_bst_mut().unwrap();

    //         if approxi_node.is_null() {
    //             return None;
    //         }

    //         if BSTNode::key_bst(&*approxi_node) != key {
    //             return None;
    //         }

    //         let mut x = approxi_node as *mut LLRBNode<K, V>;

    //         /* Prepare Deleting */
    //         if !(*x).right.is_null() {
    //             let successor =
    //                 (*(*x).right).minimum() as *mut LLRBNode<K, V>;
    //             (*x).swap_with(successor);

    //             x = successor; // x.left is null
    //         } // else x.right is null

    //         let x_b3 = PhB3Node::new((*x).centre());

    //         match x_b3.index_of_item(x) {
    //             (1, _) => {
    //                 debug_assert!(x == x_b3.centre);
    //                 debug_assert!((*x).left.is_null() && (*x).right.is_null());

    //                 let x_paren = (*x).paren;
    //                 if x_paren.is_null() {
    //                     self.root = null_mut();
    //                 } else {
    //                     let x_paren_b3 = PhB3Node::new((*x_paren).centre());

    //                     let x_b4_idx = x_paren_b3.index_of_child(x).1;

    //                     self.subtree_shift(x, null_mut::<LLRBNode<K, V>>());
    //                     self.unpromote_b3(
    //                         x_paren_b3,
    //                         x_b4_idx,
    //                     );
    //                 }

    //             }
    //             (2, idx) => {
    //                 if idx == 0 {
    //                     self.subtree_shift(x, null_mut::<LLRBNode<K, V>>());
    //                 } else {
    //                     self.subtree_shift(x, (*x).left);
    //                     set_black((*x).left);
    //                 }
    //             }
    //             _ => unreachable!(),
    //         }

    //         Some(LLRBNode::node_into_value(x))
    //     }
    // }

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

                let is_black_balance = (*self.root)
                    .leafs()
                    .into_iter()
                    .map(|leaf| (*leaf).black_depth())
                    .tuple_windows()
                    .all(|(a, b)| a == b);

                assert!(is_black_balance);

                // // Relax black balance of restriction a little
                // // to satisfy the 2-4 tree definition
                // let black_depths = (*self.root)
                //     .leafs()
                //     .into_iter()
                //     .map(|leaf| (*leaf).black_depth())
                //     .collect_vec();

                // let max_depth = black_depths.iter().max().unwrap().clone();
                // let min_depth = black_depths.iter().min().unwrap().clone();

                // if max_depth > min_depth + 1  {
                //     panic!("max: {}, min: {}", max_depth, min_depth)
                // }
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
        z: *mut (dyn BSTNode<'a, K, V> + 'a), // new root
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
            dict::{DictProvider, GetKey},
            *,
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
    #[ignore = "Only used for debug"]
    #[test]
    fn hack_llrb() {
        for _ in 0..20 {
            let batch_num = 7;
            let mut collected_elems = vec![];
            let mut keys = vec![];
            let provider = InodeProvider {};
            let mut dict = LLRB::new();
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
    fn test_llrb_fixeddata_case_0() {
        let mut llrb = LLRB::<i32, ()>::new();

        let dict = &mut llrb as &mut dyn Dictionary<i32, ()>;

        dict.insert(18, ());
        dict.insert(24, ());
        dict.insert(13, ());
        dict.insert(75, ());
        dict.insert(95, ());
        dict.insert(79, ());
        dict.insert(96, ());

        assert!(dict.remove(&13).is_some());

        dict.self_validate().unwrap();

        llrb.echo_stdout();
    }



    #[test]
    fn test_llrb_fixeddata_case_1() {
        let mut llrb = LLRB::<i32, ()>::new();

        let dict = &mut llrb as &mut dyn Dictionary<i32, ()>;

        dict.insert(2, ());
        dict.insert(22, ());
        dict.insert(14, ());
        dict.insert(92, ());
        dict.insert(3, ());
        dict.insert(28, ());
        dict.insert(14, ());
        dict.insert(35, ());
        dict.insert(14, ());
        dict.insert(58, ());

        dict.remove(&22);
        dict.remove(&14);
        dict.remove(&92);
        dict.remove(&58);
        dict.remove(&14);
        dict.remove(&35);
        dict.remove(&14);


        llrb.echo_stdout();
    }

    #[test]
    fn test_llrb_bench() {
        let mut llrb = LLRB::<u32, ()>::new();
        let dict = &mut llrb as &mut dyn Dictionary<u32, ()>;
        let provider = InodeProvider {};
        let batch_num = 10;

        let mut rng = thread_rng();
        let batch = provider.prepare_batch(batch_num);
        let mut keys = batch.iter().map(|(k, _)| k.clone()).collect_vec();

        for (k, _v) in batch.into_iter() {
            let k = k % 100;

            println!("insert: {}", k);

            dict.insert(k, ());
        }
        keys.shuffle(&mut rng);

        for k in keys.iter() {
            let k = k % 100;

            println!("remove: {}", k);
            dict.remove(&k);
        }

        // provider.bench_dict_remove(dict, &keys);

    }

}
