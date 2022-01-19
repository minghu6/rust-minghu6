#![allow(unused_imports)]
//! Clojure Vector
//!
//! Reference [It](https://hypirion.com/musings/understanding-persistent-vector-pt-1)
//!

use std::{
    cmp::{max, min},
    collections::VecDeque,
    fmt::Debug,
    fmt::Display,
    ops::{Index, IndexMut},
    ptr::null_mut, error::Error,
};

use itertools::Itertools;

use super::Vector;
use crate::{
    array,
    collections::{as_ptr, bare::array::Array, Collection},
    etc::BitLen, should,
};

const BIT_WIDTH: usize = 5;
const NODE_SIZE: usize = 2usize.pow(BIT_WIDTH as u32);
const MASK: usize = NODE_SIZE - 1;


////////////////////////////////////////////////////////////////////////////////
//// Structure

pub struct PVec<T> {
    cnt: usize,
    root: *mut Node<T>,
    tail: *mut Node<T>,
}


enum Node<T> {
    BR(Array<*mut Node<T>>),
    LEAF(Array<*mut T>),
}


pub struct TVec<T> {
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
            Node::BR(arr) => arr,
            Node::LEAF(_) => unreachable!(),
        }
    }

    fn as_leaf(&self) -> &Array<*mut T> {
        match self {
            Node::BR(_) => unreachable!(),
            Node::LEAF(arr) => arr,
        }
    }

    fn as_br_mut(&mut self) -> &mut Array<*mut Node<T>> {
        match self {
            Node::BR(arr) => arr,
            Node::LEAF(_) => unreachable!(),
        }
    }

    fn as_leaf_mut(&mut self) -> &mut Array<*mut T> {
        match self {
            Node::BR(_) => unreachable!(),
            Node::LEAF(arr) => arr,
        }
    }

    fn new_br(cap: usize) -> *mut Self {
        as_ptr(Node::BR(Array::new(cap)))
    }

    fn new_leaf(cap: usize) -> *mut Self {
        as_ptr(Node::LEAF(Array::new(cap)))
    }

    fn duplicate(&self) -> *mut Self {
        as_ptr(self.clone())
    }

    #[allow(unused)]
    fn is_empty(&self) -> bool {
        match self {
            Node::BR(arr) => arr.is_empty(),
            Node::LEAF(arr) => arr.is_empty(),
        }
    }

    fn len(&self) -> usize {
        match self {
            Node::BR(arr) => arr.len(),
            Node::LEAF(arr) => arr.len(),
        }
    }
}

impl<T> Clone for Node<T> {
    fn clone(&self) -> Self {
        match self {
            Node::BR(arr) => Node::BR(arr.clone()),
            Node::LEAF(arr) => Node::LEAF(arr.clone()),
        }
    }
}


impl<T> Index<usize> for Node<T> {
    type Output = *mut Node<T>;

    fn index(&self, index: usize) -> &Self::Output {
        match self {
            Node::BR(arr) => &arr[index],
            Node::LEAF(_) => todo!(),
        }
    }
}

impl<T: Display> Debug for Node<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Node::BR(arr) => {
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
            Node::LEAF(arr) => {
                write!(f, "leaf: ")?;

                unsafe {
                    match arr.iter().filter(|&&x| !x.is_null()).collect_vec().len()
                    {
                        0 => writeln!(f, "[]"),
                        1 => writeln!(f, "[{}]", *arr[0]),
                        2 => writeln!(f, "[{}, {}]", *arr[0], *arr[1]),
                        3 => writeln!(
                            f,
                            "[{}, {}, {}]",
                            *arr[0], *arr[1], *arr[2],
                        ),
                        upper => writeln!(
                            f,
                            "[{}, {}, ... {}]",
                            *arr[0],
                            *arr[1],
                            *arr[upper - 1],
                        ),
                    }
                }
            }
        }
    }
}



/// Impl Persistent Vec
impl<T: Debug> PVec<T> {
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


    pub fn iter<'a>(&'a self) -> Box<dyn Iterator<Item = *mut T> + 'a> {
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


    unsafe fn push_(&self, item: *mut T) -> Self {
        let cnt = self.cnt + 1;

        if self.tail.is_null() {
            let root = self.root;
            let tail = Node::new_leaf(1);
            (*tail).as_leaf_mut()[0] = item;

            return Self::new(cnt, root, tail);
        }

        // tail isn't full
        if self.cnt - self.tailoff() < NODE_SIZE {
            let root = self.root;

            let old_tail_arr = (*self.tail).as_leaf();
            let tail = Node::new_leaf(old_tail_arr.len() + 1);

            Array::copy(old_tail_arr, (*tail).as_leaf(), old_tail_arr.len());
            (*tail).as_leaf_mut()[old_tail_arr.len()] = item;

            return Self::new(cnt, root, tail);
        }

        let root;

        let tail = Node::new_leaf(1);
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
            root = Node::new_br(2);
            (*root).as_br_mut()[0] = self.root;
            (*root).as_br_mut()[1] = Self::new_path(shift, leaf);
        } else {
            root = self.push_tail_into_trie(shift, self.root, leaf)
        }

        Self::new(cnt, root, tail)
    }


    fn new_path(shift: i32, node: *mut Node<T>) -> *mut Node<T> {
        if shift == 0 as i32 {
            return node;
        }

        let ret = Node::new_br(1);

        unsafe {
            (*ret).as_br_mut()[0] =
                Self::new_path(shift - BIT_WIDTH as i32, node);
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
        let ret = Node::new_br(min(paren_arr.len() + 1, NODE_SIZE));
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
                Self::new_path(shift - BIT_WIDTH as i32, leaf)
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
            let tail = Node::new_leaf((*self.tail).len() - 1);
            Array::copy(
                (*self.tail).as_leaf(),
                (*tail).as_leaf(),
                (*tail).len()
            );

            let root = self.root;

            return Ok(Self::new(cnt, root, tail))
        }

        let tail = self.array_for(self.cnt - 2);

        let mut root;
        if self.cnt == NODE_SIZE + 1 {
            root = null_mut();
        }
        else {
            let shift = self.shift();

            root = self.pop_tail_from_trie(shift, self.root);

            if shift >= BIT_WIDTH as i32 && (*root).as_br()[1].is_null() {
                root = (*root).as_br()[0];  // remove empty root
            }
        }

        Ok(Self::new(cnt, root, tail))
    }


    unsafe fn pop_tail_from_trie(
        &self,
        shift: i32,
        node: *mut Node<T>
    ) -> *mut Node<T> {
        let sub_id = ((self.cnt - 2) >> shift) & MASK;

        if shift > BIT_WIDTH as i32 {

            debug_assert!((*node).len() > sub_id);

            let child = self.pop_tail_from_trie(
                shift - BIT_WIDTH as i32,
                (*node).as_br()[sub_id]
            );

            if child.is_null() && sub_id == 0 {
                child
            }
            else {
                let ret = (*node).duplicate();
                (*ret).as_br_mut()[sub_id] = child;

                ret
            }

        }

        else if sub_id == 0 {
            null_mut()
        }

        else {

            let ret = (*node).duplicate();

            if (*ret).len() > sub_id {
                (*ret).as_br_mut()[sub_id] = null_mut();
            }

            ret
        }

    }

    unsafe fn assoc_(&self, idx: usize, item: *mut T) -> Self {
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
        }
        else {
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
        item: *mut T
    ) -> *mut Node<T> {

        let ret = (*node).duplicate();

        if shift == 0 {
            (*ret).as_leaf_mut()[idx & MASK] = item;
        }
        else {
            let sub_idx = (idx >> shift) & MASK;
            let child = (*node).as_br()[sub_idx];

            (*ret).as_br_mut()[sub_idx]
            = Self::do_assoc(
                shift - BIT_WIDTH as i32,
                child,
                idx,
                item
            );

        }

        ret
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

                    if let Node::BR(arr) = &(*x) {
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



impl<'a, T: 'a + Debug> Vector<'a, T> for PVec<T> {
    fn nth(&self, idx: usize) -> *mut T {
        debug_assert!(idx < self.cnt);

        let arr = self.array_for(idx);

        unsafe { (*arr).as_leaf()[idx & MASK] }
    }

    fn peek(&self) -> Option<*mut T> {
        if self.cnt == 0 {
            return None;
        }

        Some(self.nth(self.cnt - 1))
    }

    fn push(&self, item: *mut T) -> Box<dyn Vector<'a, T> + 'a> {
        unsafe { box self.push_(item) }
    }

    fn pop(&self) -> Result<Box<dyn Vector<'a, T> + 'a>, Box<dyn Error>> {
        unsafe {
            match self.pop_() {
                Ok(it) => Ok(box it),
                Err(err) => Err(err),
            }
        }
    }

    fn assoc(
        &self,
        idx: usize,
        item: *mut T,
    ) -> Box<dyn Vector<'a, T> + 'a> {

        unsafe {
            box self.assoc_(idx, item)
        }
    }


    fn duplicate(&self) -> Box<dyn Vector<'a, T> + 'a> {
        box Self {
            cnt: self.cnt,
            root: self.root.clone(),
            tail: self.tail.clone(),
        }
    }

}


impl<T: Debug> Debug for PVec<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for t in self.iter() {
            unsafe { write!(f, "{:?} ", (*t))? }
        }

        Ok(())
    }
}


impl<T> Collection for PVec<T> {
    fn len(&self) -> usize {
        self.cnt
    }
}



/// Impl Transient Vec
impl<T: Debug> TVec<T> {
    pub fn empty() -> Self {
        TVec {
            cnt: 0,
            root: null_mut(),
            tail: null_mut(),
        }
    }

    // fn new(cnt: usize, root: *mut Node<T>, tail: *mut Node<T>) -> Self {
    //     Self { cnt, root, tail }
    // }


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


    pub fn iter<'a>(&'a self) -> Box<dyn Iterator<Item = *mut T> + 'a> {
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


    unsafe fn push_(&mut self, item: *mut T) -> Self {
        if self.tail.is_null() {
            self.tail = Node::new_leaf(NODE_SIZE);
            (*self.tail).as_leaf_mut()[0] = item;
            self.cnt += 1;

            return self.clone()
        }

        // tail isn't full
        if self.cnt - self.tailoff() < NODE_SIZE {
            let idx = self.cnt;

            (*self.tail).as_leaf_mut()[idx & MASK] = item;
            self.cnt += 1;

            return self.clone();
        }

        let root;

        let tail = Node::new_leaf(NODE_SIZE);
        (*tail).as_leaf_mut()[0] = item;

        // check overflow root?
        // that's: tailoff == Full Trie Nodes Number
        // So: tailoff == NODE_SIZE ^ H(trie)
        let tailoff = self.tailoff();
        if tailoff == 0 {
            self.root = self.tail;
            self.tail = tail;
            self.cnt += 1;

            return self.clone()
        }

        let leaf = self.tail;
        let shift = self.shift();

        if tailoff == NODE_SIZE.pow(self.height() as u32) {
            root = Node::new_br(NODE_SIZE);
            (*root).as_br_mut()[0] = self.root;
            (*root).as_br_mut()[1] = Self::new_path(shift, leaf);
        } else {
            root = self.push_tail_into_trie(shift, self.root, leaf)
        }

        self.root = root;
        self.tail = tail;
        self.cnt += 1;

        return self.clone();
    }


    // The Trie isn't full.
    unsafe fn push_tail_into_trie(
        &self,
        shift: i32,
        paren: *mut Node<T>,
        leaf: *mut Node<T>,
    ) -> *mut Node<T> {
        let sub_idx = ((self.cnt - 1) >> shift) & MASK;

        let ret = paren;

        let node_to_insert;
        if shift == BIT_WIDTH as i32 {
            node_to_insert = leaf;
        } else {
            let child = (*paren).as_br()[sub_idx];

            node_to_insert = if !child.is_null()
            {
                self.push_tail_into_trie(shift - BIT_WIDTH as i32, child, leaf)
            } else {
                Self::new_path(shift - BIT_WIDTH as i32, leaf)
            };
        }

        (*ret).as_br_mut()[sub_idx] = node_to_insert;

        ret
    }

    fn new_path(shift: i32, node: *mut Node<T>) -> *mut Node<T> {
        if shift == 0 as i32 {
            return node;
        }

        let ret = Node::new_br(NODE_SIZE);

        unsafe {
            (*ret).as_br_mut()[0] =
                Self::new_path(shift - BIT_WIDTH as i32, node);
        }

        ret
    }

    unsafe fn pop_(&mut self) -> Result<Self, Box<dyn Error>> {
        should!(self.cnt > 0, "Can't pop empty vector");

        if self.cnt == 1 || self.cnt - self.tailoff() > 1 {
            self.cnt -= 1;

            return Ok(self.clone())
        }

        let tail = self.array_for(self.cnt - 2);

        let mut root;
        if self.cnt == NODE_SIZE + 1 {
            root = null_mut();
        }
        else {
            let shift = self.shift();

            root = self.pop_tail_from_trie(shift, self.root);

            if shift >= BIT_WIDTH as i32 && (*root).as_br()[1].is_null() {
                root = (*root).as_br()[0];  // remove empty root
            }
        }

        self.root = root;
        self.tail = tail;
        self.cnt -= 1;

        Ok(self.clone())
    }


    unsafe fn pop_tail_from_trie(
        &self,
        shift: i32,
        node: *mut Node<T>
    ) -> *mut Node<T> {
        let sub_id = ((self.cnt - 2) >> shift) & MASK;

        if shift > BIT_WIDTH as i32 {

            debug_assert!((*node).len() > sub_id);

            let child = self.pop_tail_from_trie(
                shift - BIT_WIDTH as i32,
                (*node).as_br()[sub_id]
            );

            if child.is_null() && sub_id == 0 {
                child
            }
            else {
                let ret = node;
                (*ret).as_br_mut()[sub_id] = child;

                ret
            }

        }

        else if sub_id == 0 {
            null_mut()
        }

        else {

            let ret = node;

            if (*ret).len() > sub_id {
                (*ret).as_br_mut()[sub_id] = null_mut();
            }

            ret
        }

    }


    unsafe fn assoc_(&mut self, idx: usize, item: *mut T) -> Self {
        assert!(self.cnt >= idx);

        if idx == self.cnt {
            return self.push_(item);
        }

        debug_assert!(self.cnt > 0);

        if idx >= self.tailoff() {
            (*self.tail).as_leaf_mut()[idx & MASK] = item;
        }
        else {
            let shift = self.shift();
            self.root = Self::do_assoc(shift, self.root, idx, item);
        }

        self.clone()
    }

    unsafe fn do_assoc(
        shift: i32,
        node: *mut Node<T>,
        idx: usize,
        item: *mut T
    ) -> *mut Node<T> {

        let ret = node;

        if shift == 0 {
            (*ret).as_leaf_mut()[idx & MASK] = item;
        }
        else {
            let sub_idx = (idx >> shift) & MASK;
            let child = (*node).as_br()[sub_idx];

            (*ret).as_br_mut()[sub_idx]
            = Self::do_assoc(
                shift - BIT_WIDTH as i32,
                child,
                idx,
                item
            );

        }

        ret
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

                    if let Node::BR(arr) = &(*x) {
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



impl<'a, T: 'a + Debug> Vector<'a, T> for TVec<T> {
    fn nth(&self, idx: usize) -> *mut T {
        debug_assert!(idx < self.cnt);

        let arr = self.array_for(idx);

        unsafe { (*arr).as_leaf()[idx & MASK] }
    }

    fn peek(&self) -> Option<*mut T> {
        if self.cnt == 0 {
            return None;
        }

        Some(self.nth(self.cnt - 1))
    }

    fn push(&self, item: *mut T) -> Box<dyn Vector<'a, T> + 'a> {
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

    fn assoc(
        &self,
        idx: usize,
        item: *mut T,
    ) -> Box<dyn Vector<'a, T> + 'a> {

        unsafe {
            let mut_self = &mut *(self as *const Self as *mut Self);

            box mut_self.assoc_(idx, item)
        }
    }


    fn duplicate(&self) -> Box<dyn Vector<'a, T> + 'a> {
        box self.clone()
    }
}


impl<T: Debug> Debug for TVec<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for t in self.iter() {
            unsafe { write!(f, "{:?} ", (*t))? }
        }

        Ok(())
    }
}


impl<T> Collection for TVec<T> {
    fn len(&self) -> usize {
        self.cnt
    }
}


impl<T> Clone for TVec<T> {
    fn clone(&self) -> Self {
        Self {
            cnt: self.cnt,
            root: self.root,
            tail: self.tail
        }
    }
}




#[cfg(test)]
mod tests {
    use itertools::Itertools;

    use super::{ TVec, PVec };
    use crate::{
        collections::{as_ptr, persistent::Vector, Collection},
        test::{dict::InodeProvider, persistent::VectorProvider},
    };

    #[test]
    fn test_pvec_randomedata() {
        unsafe { InodeProvider {}.test_pvec(|| box PVec::empty()) }
    }

    #[test]
    fn test_tvec_randomedata() {
        unsafe { InodeProvider {}.test_tvec(|| box TVec::empty()) }
    }


    #[test]
    fn test_pvec_manually() {
        let pv = PVec::empty();

        // let mut bpv = (box pv) as Box<dyn Vector<usize>>;
        let mut bpv = pv;

        unsafe {

            bpv = bpv.push_(as_ptr(0));
            bpv = bpv.push_(as_ptr(1));

            bpv = bpv.push_(as_ptr(2));
            bpv = bpv.push_(as_ptr(3));

            // let upv = bpv.assoc_(3, as_ptr(30));

            // bpv = bpv.push_(as_ptr(4));
            // bpv = bpv.push_(as_ptr(5));
            // bpv = bpv.push_(as_ptr(6));
            // bpv = bpv.push_(as_ptr(7));

            // bpv = bpv.push_(as_ptr(8));
            // bpv = bpv.push_(as_ptr(9));
            // bpv = bpv.push_(as_ptr(10));
            // bpv = bpv.push_(as_ptr(11));


            // println!("0: {:?}", (*bpv.nth(0)));
            // println!("1: {:?}", (*bpv.nth(1)));
            // println!("2: {:?}", (*bpv.nth(2)));
            // println!("3: {:?}", (*bpv.nth(3)));
            // println!("4: {:?}", (*bpv.nth(4)));
            // println!("5: {:?}", (*bpv.nth(5)));
            // println!("6: {:?}", (*bpv.nth(6)));
            // println!("7: {:?}", (*bpv.nth(7)));
            // println!("8: {:?}", (*bpv.nth(8)));
            // println!("9: {:?}", (*bpv.nth(9)));
            // println!("10: {:?}", (*bpv.nth(10)));

            // bpv = bpv.push_(as_ptr(12));

            // bpv = bpv.pop_().unwrap();
            // let total = 1000;

            // for i in 12..total {
            //     bpv = bpv.push_(as_ptr(i));
            //     // println!("{}: {:?}", i, (*bpv.nth(i)));

            // }

            // let update_vec = (0..total)
            //     .into_iter()
            //     .map(|i| as_ptr(i * 100))
            //     .collect_vec();

            // for i in 0..total {
            //     bpv = bpv.assoc_(i, update_vec[i]);

            //     assert_eq!(*update_vec[i], *bpv.nth(i));
            // }


            // bpv = bpv.assoc_(200, as_ptr(99));

            // for _ in 0..20 {
            //     bpv = bpv.pop_().unwrap();
            // }

            // bpv = bpv.pop_().unwrap();


            bpv.bfs_display();
        }
    }


    #[test]
    fn test_tvec_manually() {
        let tv = TVec::empty();

        // let mut bpv = (box pv) as Box<dyn Vector<usize>>;
        let mut btv = tv;

        unsafe {

            btv = btv.push_(as_ptr(0));
            btv = btv.push_(as_ptr(1));
            btv = btv.push_(as_ptr(2));
            btv = btv.push_(as_ptr(3));
            btv = btv.push_(as_ptr(4));
            btv = btv.push_(as_ptr(5));
            btv = btv.push_(as_ptr(6));
            btv = btv.push_(as_ptr(7));
            btv = btv.push_(as_ptr(8));
            btv = btv.push_(as_ptr(9));
            btv = btv.push_(as_ptr(10));
            btv = btv.push_(as_ptr(11));

            let total = 500;

            for i in 12..total {
                btv = btv.push_(as_ptr(i));
            }

            for i in 0..total {
                btv = btv.assoc_(i, as_ptr(i * 100));
            }

            let mut uvec = btv.duplicate();
            let mut uelem_vec = vec![];
            for i in 0..total {
                let e = as_ptr(i * 100);
                uelem_vec.push(e);
            }
            for i in 0..total {
                uvec = uvec.assoc(i, uelem_vec[i]);

                assert_eq!(uvec.nth(i), uelem_vec[i])
            }


            for i in (0..total).rev() {
                btv = btv.pop_().unwrap();

                for j in 0..i {
                    assert_eq!(btv.nth(j), uelem_vec[j]);
                }
            }


            btv.bfs_display();

        }

    }

}
