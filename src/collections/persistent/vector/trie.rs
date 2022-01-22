#![allow(unused_imports)]
//! Clojure Vector
//!
//! Reference [It](https://hypirion.com/musings/understanding-persistent-vector-pt-1)
//!

use std::{
    cmp::{max, min},
    collections::VecDeque,
    error::Error,
    fmt::Debug,
    fmt::Display,
    ops::{Index, IndexMut},
    ptr::null_mut,
};

use itertools::Itertools;
use uuid::Uuid;

use super::Vector;
use crate::{
    array,
    collections::{as_ptr, bare::array::Array, Collection},
    etc::BitLen,
    should,
};

const BIT_WIDTH: usize = 5;
const NODE_SIZE: usize = 2usize.pow(BIT_WIDTH as u32);
const MASK: usize = NODE_SIZE - 1;


////////////////////////////////////////////////////////////////////////////////
//// Structure

pub struct PTrieVec<T> {
    cnt: usize,
    root: *mut Node<T>,
    tail: *mut Node<T>,
}


enum Node<T> {
    BR(Option<Uuid>, Array<*mut Node<T>>),
    LEAF(Option<Uuid>, Array<T>),
}


pub struct TTrieVec<T> {
    cnt: usize,
    root: *mut Node<T>, // C void*
    tail: *mut Node<T>,
}



////////////////////////////////////////////////////////////////////////////////
//// Implement

/// Impl Node
impl<T> Node<T> {
    fn as_br(&self) -> &Array<*mut Node<T>> {
        match self {
            Node::BR(_, arr) => arr,
            Node::LEAF(_, _) => unreachable!(),
        }
    }

    fn as_leaf(&self) -> &Array<T> {
        match self {
            Node::BR(_, _) => unreachable!(),
            Node::LEAF(_, arr) => arr,
        }
    }

    fn as_br_mut(&mut self) -> &mut Array<*mut Node<T>> {
        match self {
            Node::BR(_, arr) => arr,
            Node::LEAF(_, _) => unreachable!(),
        }
    }

    fn as_leaf_mut(&mut self) -> &mut Array<T> {
        match self {
            Node::BR(_, _) => unreachable!(),
            Node::LEAF(_, arr) => arr,
        }
    }

    fn new_br(id: Option<Uuid>, cap: usize) -> *mut Self {
        as_ptr(Node::BR(id, Array::new(cap)))
    }

    fn new_leaf(id: Option<Uuid>, cap: usize) -> *mut Self {
        as_ptr(Node::LEAF(id, Array::new(cap)))
    }

    fn duplicate(&self) -> *mut Self {
        as_ptr(self.clone())
    }

    fn duplicate_with(&self, id: Option<Uuid>, cap: usize) -> *mut Self {
        as_ptr(match self {
            Node::BR(_id, arr) => {
                debug_assert!(cap >= arr.len(), "cap: {}, arr.len: {}", cap, arr.len());

                let newarr = Array::new(cap);
                Array::copy(arr, &newarr, arr.len());

                Node::BR(id, newarr)
            }
            Node::LEAF(_id, arr) => {
                debug_assert!(cap >= arr.len());

                let newarr = Array::new(cap);
                Array::copy(arr, &newarr, arr.len());

                Node::LEAF(id, newarr)
            }
        })
    }


    #[allow(unused)]
    fn is_empty(&self) -> bool {
        match self {
            Node::BR(_, arr) => arr.is_empty(),
            Node::LEAF(_, arr) => arr.is_empty(),
        }
    }

    fn len(&self) -> usize {
        match self {
            Node::BR(_, arr) => arr.len(),
            Node::LEAF(_, arr) => arr.len(),
        }
    }

    fn id(&self) -> Option<Uuid> {
        match self {
            Node::BR(id, _) => id,
            Node::LEAF(id, _) => id,
        }
        .clone()
    }

    #[allow(unused)]
    fn clone_tree(&self) -> *mut Node<T> {
        unsafe {
            Self::clone_tree_(self as *const Node<T> as *mut Node<T>)
        }
    }

    unsafe fn clone_tree_(node: *mut Node<T>) -> *mut Node<T> {
        if node.is_null() {
            return node;
        }

        let cloned_node = as_ptr((*node).clone());

        if let Self::BR(_, ref mut arr) = &mut *cloned_node {
            for i in 0..arr.len() {
                arr[i] = Self::clone_tree_(arr[i]);
            }
        }

        cloned_node
    }

}


impl<T> Clone for Node<T> {
    fn clone(&self) -> Self {
        match self {
            Node::BR(uuid, arr) => Node::BR(uuid.clone(), arr.clone()),
            Node::LEAF(uuid, arr) => Node::LEAF(uuid.clone(), arr.clone()),
        }
    }
}


impl<T> Index<usize> for Node<T> {
    type Output = *mut Node<T>;

    fn index(&self, index: usize) -> &Self::Output {
        match self {
            Node::BR(_, arr) => &arr[index],
            Node::LEAF(_, _) => todo!(),
        }
    }
}


impl<T: Display> Debug for Node<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Node::BR(_, arr) => {
                write!(f, "br: ")?;

                match arr.iter().filter(|&&x| !x.is_null()).collect_vec().len()
                {
                    0 => writeln!(f, "[]"),
                    1 => writeln!(f, "[0]"),
                    2 => writeln!(f, "[0, 1]"),
                    3 => writeln!(f, "[0, 1, 2]"),
                    upper => writeln!(f, "[0, 1, ... {}]", upper - 1),
                }
            }
            Node::LEAF(_, arr) => {
                write!(f, "leaf: ")?;

                match arr.len()
                {
                    0 => writeln!(f, "[]"),
                    1 => writeln!(f, "[{}]", arr[0]),
                    2 => writeln!(f, "[{}, {}]", arr[0], arr[1]),
                    3 => writeln!(
                        f,
                        "[{}, {}, {}]",
                        arr[0], arr[1], arr[2],
                    ),
                    upper => writeln!(
                        f,
                        "[{}, {}, ... {}]",
                        arr[0],
                        arr[1],
                        arr[upper - 1],
                    ),
                }
            }
        }
    }
}



/// Impl Persistent Vec
impl<T: Debug> PTrieVec<T> {
    pub fn empty() -> Self {
        Self {
            cnt: 0,
            root: null_mut(),
            tail: null_mut(),
        }
    }


    fn new(cnt: usize, root: *mut Node<T>, tail: *mut Node<T>) -> Self {
        Self { cnt, root, tail }
    }


    /// Tail Offset (elements number before tail)
    fn tailoff(&self) -> usize {
        if self.cnt == 0 {
            return 0;
        }

        ((self.cnt - 1) >> BIT_WIDTH) << BIT_WIDTH
    }


    /// Indicate the trie structure.
    fn shift(&self) -> i32 {
        (self.height() as i32 - 1) * BIT_WIDTH as i32
    }


    /// Leaf should be same level
    fn height(&self) -> usize {
        Self::height_(self.tailoff())
    }
    fn height_(trie_size: usize) -> usize {
        if trie_size == 0 {
            return 0;
        }

        let mut h = 1;
        let mut shift = (trie_size - 1) >> BIT_WIDTH;
        while shift > 0 {
            shift >>= BIT_WIDTH;
            h += 1;
        }

        h
    }


    pub fn id(&self) -> Option<Uuid> {
        if self.root.is_null() {
            None
        } else {
            unsafe { (*self.root).id() }
        }
    }


    pub fn iter<'a>(&'a self) -> Box<dyn Iterator<Item = &T> + 'a> {
        let mut i = 0;

        box std::iter::from_fn(move || {
            if i == self.cnt {
                return None;
            }

            let nxt = self.nth(i);
            i += 1;

            Some(nxt)
        })
    }


    /// Get the Bucket
    fn array_for(&self, idx: usize) -> *mut Node<T> {
        debug_assert!(idx < self.cnt);

        unsafe {
            if idx >= self.tailoff() {
                return self.tail;
            }

            let mut shift = self.shift();
            let mut cur = self.root;

            while shift > 0 {
                cur = (*cur).as_br()[(idx >> shift) & MASK];

                shift -= BIT_WIDTH as i32;
            }

            cur
        }
    }


    pub fn push_(&self, item: T) -> Self {
        unsafe {
            let cnt = self.cnt + 1;

            if self.cnt == 0 {
                let root = self.root;
                let tail = Node::new_leaf(self.id(), 1);
                (*tail).as_leaf_mut()[0] = item;

                return Self::new(cnt, root, tail);
            }

            // tail isn't full
            if self.cnt - self.tailoff() < NODE_SIZE {
                let root = self.root;

                let old_tail_arr = (*self.tail).as_leaf();
                let tail = Node::new_leaf(self.id(), old_tail_arr.len() + 1);

                Array::copy(old_tail_arr, (*tail).as_leaf(), old_tail_arr.len());
                (*tail).as_leaf_mut()[old_tail_arr.len()] = item;

                return Self::new(cnt, root, tail);
            }

            let root;

            let tail = Node::new_leaf(self.id(), 1);
            (*tail).as_leaf_mut()[0] = item;

            // check overflow root?
            // that's: tailoff == Full Trie Nodes Number
            // So: tailoff == NODE_SIZE ^ H(trie)
            let tailoff = self.tailoff();
            if tailoff == 0 {
                root = self.tail;

                return Self::new(cnt, root, tail);
            }

            let leaf = self.tail;
            let shift = self.shift();

            if tailoff == NODE_SIZE.pow(self.height() as u32) {
                root = Node::new_br(self.id(), 2);
                (*root).as_br_mut()[0] = self.root;
                (*root).as_br_mut()[1] = Self::new_path(self.id(), shift, leaf);
            } else {
                root = self.push_tail_into_trie(shift, self.root, leaf)
            }

            Self::new(cnt, root, tail)
        }
    }


    fn new_path(id: Option<Uuid>, shift: i32, node: *mut Node<T>) -> *mut Node<T> {
        if shift == 0 as i32 {
            return node;
        }

        let ret = Node::new_br(id, 1);

        unsafe {
            (*ret).as_br_mut()[0] =
                Self::new_path(id, shift - BIT_WIDTH as i32, node);
        }

        ret
    }


    // The Trie isn't full.
    unsafe fn push_tail_into_trie(
        &self,
        shift: i32,
        paren: *mut Node<T>,
        leaf: *mut Node<T>,
    ) -> *mut Node<T> {
        let sub_idx = ((self.cnt - 1) >> shift) & MASK;

        let paren_arr = (*paren).as_br();
        let ret = Node::new_br(self.id(), min(paren_arr.len() + 1, NODE_SIZE));
        Array::copy(paren_arr, (*ret).as_br(), paren_arr.len());

        let node_to_insert;
        if shift == BIT_WIDTH as i32 {
            node_to_insert = leaf;
        } else {
            node_to_insert = if (*paren).len() > sub_idx
                && !(*paren).as_br()[sub_idx].is_null()
            {
                let child = (*paren).as_br()[sub_idx];
                self.push_tail_into_trie(shift - BIT_WIDTH as i32, child, leaf)
            } else {
                Self::new_path(self.id(), shift - BIT_WIDTH as i32, leaf)
            };
        }

        (*ret).as_br_mut()[sub_idx] = node_to_insert;

        ret
    }


    unsafe fn pop_(&self) -> Result<Self, Box<dyn Error>> {
        should!(self.cnt > 0, "Can't pop empty vector");

        if self.cnt == 1 {
            return Ok(Self::empty());
        }

        let cnt = self.cnt - 1;

        if self.cnt - self.tailoff() > 1 {
            let tail = Node::new_leaf(self.id(), (*self.tail).len() - 1);
            Array::copy(
                (*self.tail).as_leaf(),
                (*tail).as_leaf(),
                (*tail).len(),
            );

            let root = self.root;

            return Ok(Self::new(cnt, root, tail));
        }

        let tail = self.array_for(self.cnt - 2);

        let mut root;
        if self.cnt == NODE_SIZE + 1 {
            root = null_mut();
        } else {
            let shift = self.shift();

            root = self.pop_tail_from_trie(shift, self.root);

            if shift >= BIT_WIDTH as i32 && (*root).as_br()[1].is_null() {
                root = (*root).as_br()[0]; // remove empty root
            }
        }

        Ok(Self::new(cnt, root, tail))
    }


    unsafe fn pop_tail_from_trie(
        &self,
        shift: i32,
        node: *mut Node<T>,
    ) -> *mut Node<T> {
        let sub_id = ((self.cnt - 2) >> shift) & MASK;

        if shift > BIT_WIDTH as i32 {
            debug_assert!((*node).len() > sub_id);

            let child = self.pop_tail_from_trie(
                shift - BIT_WIDTH as i32,
                (*node).as_br()[sub_id],
            );

            if child.is_null() && sub_id == 0 {
                child
            } else {
                let ret = (*node).duplicate();
                (*ret).as_br_mut()[sub_id] = child;

                ret
            }
        } else if sub_id == 0 {
            null_mut()
        } else {
            let ret = (*node).duplicate();

            if (*ret).len() > sub_id {
                (*ret).as_br_mut()[sub_id] = null_mut();
            }

            ret
        }
    }

    unsafe fn assoc_(&self, idx: usize, item: T) -> Self {
        assert!(self.cnt >= idx);

        if idx == self.cnt {
            return self.push_(item);
        }

        debug_assert!(self.cnt > 0);

        let cnt = self.cnt;
        let root;
        let tail;
        if idx >= self.tailoff() {
            root = self.root;
            tail = (*self.tail).duplicate();

            (*tail).as_leaf_mut()[idx & MASK] = item;
        } else {
            let shift = self.shift();
            root = Self::do_assoc(shift, self.root, idx, item);
            tail = self.tail;
        }

        Self::new(cnt, root, tail)
    }

    unsafe fn do_assoc(
        shift: i32,
        node: *mut Node<T>,
        idx: usize,
        item: T,
    ) -> *mut Node<T> {
        let ret = (*node).duplicate();

        if shift == 0 {
            (*ret).as_leaf_mut()[idx & MASK] = item;
        } else {
            let sub_idx = (idx >> shift) & MASK;
            let child = (*node).as_br()[sub_idx];

            (*ret).as_br_mut()[sub_idx] =
                Self::do_assoc(shift - BIT_WIDTH as i32, child, idx, item);
        }

        ret
    }

    fn transient_(&self) -> TTrieVec<T>{
        TTrieVec {
            cnt: self.cnt,
            root: TTrieVec::editable(self.root),
            tail: TTrieVec::editable(self.tail)
        }
    }

    pub fn bfs_display(&self)
    where
        T: Display,
    {
        unsafe {
            let mut lv = 1usize;
            let mut cur_q = VecDeque::new();

            println!();
            println!("MAIN TRIE:");
            println!();

            if !self.root.is_null() {
                cur_q.push_back(self.root);
            } else {
                println!("null\n");
            }

            while !cur_q.is_empty() {
                println!("############ Level: {} #############", lv);

                let mut nxt_q = VecDeque::new();

                while !cur_q.is_empty() {
                    let x = cur_q.pop_front().unwrap();

                    // print all x chidren
                    println!();
                    println!("{:?}", (*x));

                    if let Node::BR(_idopt, arr) = &(*x) {
                        nxt_q.extend(arr.iter().filter(|&&x| !x.is_null()))
                    }
                }

                cur_q = nxt_q;
                lv += 1;
            }

            // print tail
            println!("###################################\n");
            println!("TAIL: \n");

            if !self.tail.is_null() {
                println!("{:?}", (*self.tail));
            } else {
                println!("null");
            }

            println!("------------- end --------------");
        }
    }

}



impl<'a, T: 'a + Debug> Vector<'a, T> for PTrieVec<T> {
    fn nth(&self, idx: usize) -> &T {
        debug_assert!(idx < self.cnt);

        let arr = self.array_for(idx);

        unsafe { &(*arr).as_leaf()[idx & MASK] }
    }

    fn peek(&self) -> Option<&T> {
        if self.cnt == 0 {
            return None;
        }

        Some(self.nth(self.cnt - 1))
    }

    fn push(&self, item: T) -> Box<dyn Vector<'a, T> + 'a> {
        box self.push_(item)
    }

    fn pop(&self) -> Result<Box<dyn Vector<'a, T> + 'a>, Box<dyn Error>> {
        unsafe {
            match self.pop_() {
                Ok(it) => Ok(box it),
                Err(err) => Err(err),
            }
        }
    }

    fn assoc(&self, idx: usize, item: T) -> Box<dyn Vector<'a, T> + 'a> {
        unsafe { box self.assoc_(idx, item) }
    }

    fn duplicate(&self) -> Box<dyn Vector<'a, T> + 'a> {
        box Self {
            cnt: self.cnt,
            root: self.root.clone(),
            tail: self.tail.clone(),
        }
    }

    fn transient(&self) -> Result<Box<dyn Vector<'a, T> + 'a>, ()> {
        Ok(box self.transient_())
    }

    fn persistent(&self) -> Result<Box<dyn Vector<'a, T> + 'a>, ()> {
        Ok(box self.clone())
    }
}


impl<T: Debug> Debug for PTrieVec<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for t in self.iter() {
            write!(f, "{:?} ", (*t))?
        }

        Ok(())
    }
}


impl<T> Collection for PTrieVec<T> {
    fn len(&self) -> usize {
        self.cnt
    }
}


impl<T> Clone for PTrieVec<T> {
    fn clone(&self) -> Self {
        Self {
            cnt: self.cnt,
            root: self.root,
            tail: self.tail,
        }
    }
}



/// Impl Transient Vec
impl<T: Debug> TTrieVec<T> {
    pub fn empty() -> Self {
        TTrieVec {
            cnt: 0,
            root: null_mut(),
            tail: null_mut(),
        }
    }


    unsafe fn ensure_editable(&self, node: *mut Node<T>) -> *mut Node<T> {
        if self.id() == (*node).id() {
            node
        }
        else {
            Self::editable(node)
        }
    }

    fn editable( node: *mut Node<T>) -> *mut Node<T> {
        unsafe {
            if node.is_null() {
                return node;
            }

            (*node).duplicate_with(
                Some(Uuid::new_v4()),
                NODE_SIZE
            )
        }
    }


    /// Tail Offset (elements number before tail)
    fn tailoff(&self) -> usize {
        if self.cnt == 0 {
            return 0;
        }

        ((self.cnt - 1) >> BIT_WIDTH) << BIT_WIDTH
    }


    /// Indicate the trie structure.
    fn shift(&self) -> i32 {
        (self.height() as i32 - 1) * BIT_WIDTH as i32
    }


    /// Leaf should be same level
    fn height(&self) -> usize {
        Self::height_(self.tailoff())
    }
    fn height_(trie_size: usize) -> usize {
        if trie_size == 0 {
            return 0;
        }

        let mut h = 1;
        let mut shift = (trie_size - 1) >> BIT_WIDTH;
        while shift > 0 {
            shift >>= BIT_WIDTH;
            h += 1;
        }

        h
    }


    /// Get the Bucket
    fn array_for(&self, idx: usize) -> *mut Node<T> {
        debug_assert!(idx < self.cnt);

        unsafe {
            if idx >= self.tailoff() {
                return self.tail;
            }

            let mut shift = self.shift();
            let mut cur = self.root;

            while shift > 0 {
                cur = (*cur).as_br()[(idx >> shift) & MASK];

                shift -= BIT_WIDTH as i32;
            }

            cur
        }
    }


    pub fn iter<'a>(&'a self) -> Box<dyn Iterator<Item = &T> + 'a> {
        let mut i = 0;

        box std::iter::from_fn(move || {
            if i == self.cnt {
                return None;
            }

            let nxt = self.nth(i);
            i += 1;

            Some(nxt)
        })
    }


    unsafe fn push_(&mut self, item: T) -> Self {
        if self.tail.is_null() {
            self.tail = Node::new_leaf(self.id(), NODE_SIZE);
            (*self.tail).as_leaf_mut()[0] = item;
            self.cnt += 1;

            return self.clone_head();
        }

        // tail isn't full
        if self.cnt - self.tailoff() < NODE_SIZE {
            let idx = self.cnt;

            if (*self.tail).len() <= idx & MASK {
                self.tail = (*self.tail).duplicate_with(self.id(), NODE_SIZE);
            }

            (*self.tail).as_leaf_mut()[idx & MASK] = item;
            self.cnt += 1;

            return self.clone_head();
        }

        let root;

        let tail = Node::new_leaf(self.id(), NODE_SIZE);
        (*tail).as_leaf_mut()[0] = item;

        // check overflow root?
        // that's: tailoff == Full Trie Nodes Number
        // So: tailoff == NODE_SIZE ^ H(trie)
        let tailoff = self.tailoff();
        if tailoff == 0 {
            self.root = self.tail;
            self.tail = tail;
            self.cnt += 1;

            return self.clone_head();
        }

        let leaf = self.tail;
        let shift = self.shift();

        if tailoff == NODE_SIZE.pow(self.height() as u32) {
            root = Node::new_br(self.id(), NODE_SIZE);
            (*root).as_br_mut()[0] = self.root;
            (*root).as_br_mut()[1] = Self::new_path(self.id(), shift, leaf);
        } else {
            root = self.push_tail_into_trie(shift, self.root, leaf)
        }

        self.root = root;
        self.tail = tail;
        self.cnt += 1;

        return self.clone_head();
    }


    // The Trie isn't full.
    unsafe fn push_tail_into_trie(
        &self,
        shift: i32,
        paren: *mut Node<T>,
        leaf: *mut Node<T>,
    ) -> *mut Node<T> {
        let sub_idx = ((self.cnt - 1) >> shift) & MASK;

        let ret = self.ensure_editable(paren);

        let node_to_insert;
        if shift == BIT_WIDTH as i32 {
            node_to_insert = leaf;
        } else {
            let child = (*paren).as_br()[sub_idx];

            node_to_insert = if !child.is_null() {
                self.push_tail_into_trie(shift - BIT_WIDTH as i32, child, leaf)
            } else {
                Self::new_path(self.id(), shift - BIT_WIDTH as i32, leaf)
            };
        }

        (*ret).as_br_mut()[sub_idx] = node_to_insert;

        ret
    }


    fn new_path(id: Option<Uuid>, shift: i32, node: *mut Node<T>) -> *mut Node<T> {
        if shift == 0 as i32 {
            return node;
        }

        let ret = Node::new_br(id, NODE_SIZE);

        unsafe {
            (*ret).as_br_mut()[0] =
                Self::new_path(id, shift - BIT_WIDTH as i32, node);
        }

        ret
    }


    fn editable_array_for(&self, idx: usize) -> *mut Node<T> {
        debug_assert!(idx < self.cnt);

        unsafe {
            if idx >= self.tailoff() {
                return self.tail;
            }

            let mut shift = self.shift();
            let mut cur = self.root;

            while shift > 0 {
                cur = (*self.ensure_editable(cur))
                    .as_br()[(idx >> shift) & MASK];

                shift -= BIT_WIDTH as i32;
            }

            cur
        }
    }


    unsafe fn pop_(&mut self) -> Result<Self, Box<dyn Error>> {
        should!(self.cnt > 0, "Can't pop empty vector");

        if self.cnt == 1 || self.cnt - self.tailoff() > 1 {
            self.cnt -= 1;

            return Ok(self.clone_head());
        }

        let tail = self.editable_array_for(self.cnt - 2);

        let mut root;
        if self.cnt == NODE_SIZE + 1 {
            root = null_mut();
        } else {
            let shift = self.shift();

            root = self.pop_tail_from_trie(shift, self.root);

            if shift >= BIT_WIDTH as i32 && (*root).as_br()[1].is_null() {
                // remove empty root
                root = self.ensure_editable((*root).as_br()[0]);
            }
        }

        self.root = root;
        self.tail = tail;
        self.cnt -= 1;

        Ok(self.clone_head())
    }


    unsafe fn pop_tail_from_trie(
        &self,
        shift: i32,
        node: *mut Node<T>,
    ) -> *mut Node<T> {
        let node = self.ensure_editable(node);
        let sub_id = ((self.cnt - 2) >> shift) & MASK;

        if shift > BIT_WIDTH as i32 {
            debug_assert!((*node).len() > sub_id);

            let child = self.pop_tail_from_trie(
                shift - BIT_WIDTH as i32,
                (*node).as_br()[sub_id],
            );

            if child.is_null() && sub_id == 0 {
                child
            } else {
                let ret = node;
                (*ret).as_br_mut()[sub_id] = child;

                ret
            }
        } else if sub_id == 0 {
            null_mut()
        } else {
            let ret = node;

            if (*ret).len() > sub_id {
                (*ret).as_br_mut()[sub_id] = null_mut();
            }

            ret
        }
    }


    unsafe fn assoc_(&mut self, idx: usize, item: T) -> Self {
        assert!(self.cnt >= idx);

        if idx == self.cnt {
            return self.push_(item);
        }

        debug_assert!(self.cnt > 0);

        if idx >= self.tailoff() {
            (*self.tail).as_leaf_mut()[idx & MASK] = item;
        } else {
            let shift = self.shift();
            self.root = self.do_assoc(shift, self.root, idx, item);
        }

        self.clone_head()
    }


    unsafe fn do_assoc(
        &self,
        shift: i32,
        node: *mut Node<T>,
        idx: usize,
        item: T,
    ) -> *mut Node<T> {
        let ret = self.ensure_editable(node);

        if shift == 0 {
            (*ret).as_leaf_mut()[idx & MASK] = item;
        } else {
            let sub_idx = (idx >> shift) & MASK;
            let child = (*node).as_br()[sub_idx];

            (*ret).as_br_mut()[sub_idx] =
                self.do_assoc(shift - BIT_WIDTH as i32, child, idx, item);
        }

        ret
    }


    pub fn id(&self) -> Option<Uuid> {
        if self.root.is_null() {
            None
        } else {
            unsafe { (*self.root).id() }
        }
    }


    fn persistent_(&self) -> PTrieVec<T> {
        unsafe {

            let root = if self.root.is_null() {
                self.root
            } else {
                (*self.root).duplicate_with(None, (*self.root).len())
            };

            let root_id = if root.is_null() {
                None
            } else {
                (*self.root).id()
            };

            let tail = if self.tail.is_null() {
                self.tail
            } else {
                let tail = Node::new_leaf(root_id, self.cnt - self.tailoff());
                Array::copy(
                    (*self.tail).as_leaf(),
                    (*tail).as_leaf(),
                    (*tail).len()
                );

                tail
            };

            PTrieVec { cnt: self.cnt, root, tail }

        }

    }


    pub fn clone_tree(&self) -> Self {
        unsafe {
            Self {
                cnt: self.cnt,
                root: Node::clone_tree_(self.root),
                tail: Node::clone_tree_(self.tail),
            }
        }
    }

    pub fn clone_head(&self) -> Self {
        Self {
            cnt: self.cnt,
            root: self.root,
            tail: self.tail,
        }
    }


    pub fn bfs_display(&self)
    where
        T: Display,
    {
        unsafe {
            let mut lv = 1usize;
            let mut cur_q = VecDeque::new();

            println!();
            println!("MAIN TRIE: {}", self.cnt);
            println!();

            if !self.root.is_null() {
                cur_q.push_back(self.root);
            } else {
                println!("null\n");
            }

            while !cur_q.is_empty() {
                println!("############ Level: {} #############", lv);

                let mut nxt_q = VecDeque::new();

                while !cur_q.is_empty() {
                    let x = cur_q.pop_front().unwrap();

                    // print all x chidren
                    println!();
                    println!("{:?}", (*x));

                    if let Node::BR(_idopt, arr) = &(*x) {
                        nxt_q.extend(arr.iter().filter(|&&x| !x.is_null()))
                    }
                }

                cur_q = nxt_q;
                lv += 1;
            }

            // print tail
            println!("###################################\n");
            println!("TAIL: \n");

            if !self.tail.is_null() {
                println!("{:?}", (*self.tail));
            } else {
                println!("null");
            }

            println!("------------- end --------------");
        }
    }



}



impl<'a, T: 'a + Debug> Vector<'a, T> for TTrieVec<T> {
    fn nth(&self, idx: usize) -> &T {
        debug_assert!(idx < self.cnt);

        let arr = self.array_for(idx);

        unsafe { &(*arr).as_leaf()[idx & MASK] }
    }

    fn peek(&self) -> Option<&T> {
        if self.cnt == 0 {
            return None;
        }

        Some(self.nth(self.cnt - 1))
    }

    fn push(&self, item: T) -> Box<dyn Vector<'a, T> + 'a> {
        unsafe {
            let mut_self = &mut *(self as *const Self as *mut Self);

            box mut_self.push_(item)
        }
    }

    fn pop(&self) -> Result<Box<dyn Vector<'a, T> + 'a>, Box<dyn Error>> {
        unsafe {
            let mut_self = &mut *(self as *const Self as *mut Self);

            match mut_self.pop_() {
                Ok(it) => Ok(box it),
                Err(err) => Err(err),
            }
        }
    }

    fn assoc(&self, idx: usize, item: T) -> Box<dyn Vector<'a, T> + 'a> {
        unsafe {
            let mut_self = &mut *(self as *const Self as *mut Self);

            box mut_self.assoc_(idx, item)
        }
    }

    fn duplicate(&self) -> Box<dyn Vector<'a, T> + 'a> {
        box self.clone_tree()
    }

    fn transient(&self) -> Result<Box<dyn Vector<'a, T> + 'a>, ()> {
        Ok(box self.clone_head())
    }

    fn persistent(&self) -> Result<Box<dyn Vector<'a, T> + 'a>, ()> {
        Ok(box self.persistent_())
    }
}


impl<T: Debug> Debug for TTrieVec<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for t in self.iter() {
            write!(f, "{:?} ", (*t))?
        }

        Ok(())
    }
}


impl<T> Collection for TTrieVec<T> {
    fn len(&self) -> usize {
        self.cnt
    }
}


// impl<T> Clone for TTrieVec<T> {
//     fn clone(&self) -> Self {
//         Self {
//             cnt: self.cnt,
//             root: self.root,
//             tail: self.tail,
//         }
//     }
// }



#[cfg(test)]
mod tests {
    use itertools::Itertools;

    use super::{PTrieVec, TTrieVec};
    use crate::{
        collections::{as_ptr, persistent::vector::Vector, Collection},
        test::{ persistent::VectorProvider, * },
    };

    #[test]
    fn test_ptrie_vec_randomedata() {
        unsafe { InodeProvider {}.test_pvec(|| box PTrieVec::empty()) }
    }

    #[test]
    fn test_ttrie_vec_randomedata() {
        unsafe { InodeProvider {}.test_tvec(|| box TTrieVec::empty()) }
    }

    #[test]
    fn test_pttrie_tran_randomdata() {
        unsafe { InodeProvider {}.test_pttran(|| box PTrieVec::empty()) }
    }

    #[test]
    fn test_ptrie_vec_manually() {
        // let pv = PTrieVec::empty();

        // let mut bpv = (box pv) as Box<dyn Vector<Inode>>;
        // // let mut bpv = pv;

        // // let batch = provider.prepare_batch(BATCH_NUM);
        // let provider = InodeProvider {};
        // let batch_num = 1000;
        // let batch = (0..batch_num).into_iter().map(|_| provider.get_one()).collect_vec();
        // let batch = batch.clone();

        // let mut i = 0;
        // for e in batch.into_iter() {
        //     bpv = bpv.push(e);

        //     // println!("{}", i);
        //     i += 1;
        // }

        let bench = || {
            let provider = &InodeProvider{};
            let batch = provider.prepare_batch(500);

            for _ in 0..10 {
                let mut vec = (box PTrieVec::empty()) as Box<dyn Vector<Inode>>;
                let batch = batch.clone();

                let mut _i = 0;
                for e in batch.into_iter() {
                    vec = vec.push(e);

                    // println!("{}", _i);
                    _i += 1;
                }
            }
        };


        bench();
        bench();

        // bpv.bfs_display();
    }


    #[test]
    fn test_ttrievec_manually() {
        let tv = TTrieVec::empty();

        // let mut bpv = (box pv) as Box<dyn Vector<usize>>;
        let mut btv = tv;

        unsafe {
            btv = btv.push_(0);
            btv = btv.push_(1);
            btv = btv.push_(2);
            btv = btv.push_(3);
            btv = btv.push_(4);
            btv = btv.push_(5);
            btv = btv.push_(6);
            btv = btv.push_(7);
            btv = btv.push_(8);
            btv = btv.push_(9);
            btv = btv.push_(10);
            btv = btv.push_(11);

            let total = 500;

            for i in 12..total {
                btv = btv.push_(i);
            }

            for i in 0..total {
                btv = btv.assoc_(i, i * 100);
            }

            let mut uvec = btv.duplicate();
            let mut uelem_vec = vec![];
            for i in 0..total {
                let e = i * 100;
                uelem_vec.push(e);
            }
            for i in 0..total {
                uvec = uvec.assoc(i, uelem_vec[i]);

                assert_eq!(uvec.nth(i), &uelem_vec[i])
            }


            for i in (0..total).rev() {
                btv = btv.pop_().unwrap();

                for j in 0..i {
                    assert_eq!(btv.nth(j), &uelem_vec[j]);
                }
            }

            // btv = btv.pop_().unwrap();


            btv.bfs_display();
        }
    }


    #[test]
    fn test_pttrietran_manually() {
        unsafe {

            // let mut vec = (box PVec::empty()) as Box<dyn Vector<usize>>;
            let mut pvec = PTrieVec::empty();
            let pbatchnum = 300;
            for i in 0..pbatchnum {
                pvec = pvec.push_(i);
            }

            // println!("Before Transistent");
            // pvec.bfs_display();


            let mut tvec = pvec.transient_();

            // tvec = tvec.push_(10);
            let tbatchnum = 300;
            for i in pbatchnum..pbatchnum + tbatchnum {
                tvec = tvec.push_(i * 10);

                for j in pbatchnum..i {
                    // println!("j: {}", j);
                    assert_eq!(*tvec.nth(j), j * 10);
                }

            }

            // println!("Transistrent");
            // tvec.bfs_display();

        }
    }

}
