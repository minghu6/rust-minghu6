//! Clojure Vector
//!
//! Reference [It](https://hypirion.com/musings/understanding-persistent-vector-pt-1)
//!

use std::{
    cmp::min, collections::VecDeque, fmt::Debug, fmt::Display,
    ops::Index, ptr::null_mut,
};

use coll::{boxptr, uuid::Uuid, Array};
use common::Itertools;

const BIT_WIDTH: u32 = 2;
const NODE_SIZE: usize = 1 << BIT_WIDTH as usize;
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
        boxptr!(Node::BR(id, Array::new(cap)))
    }

    fn new_leaf(id: Option<Uuid>, cap: usize) -> *mut Self {
        boxptr!(Node::LEAF(id, Array::new(cap)))
    }

    fn duplicate(&self) -> *mut Self
    where
        T: Clone,
    {
        boxptr!(self.clone())
    }

    fn duplicate_with(&self, id: Option<Uuid>, cap: usize) -> *mut Self
    where
        T: Clone,
    {
        boxptr!(match self {
            Node::BR(_id, arr) => {
                debug_assert!(
                    cap >= arr.len(),
                    "cap: {}, arr.len: {}",
                    cap,
                    arr.len()
                );

                let mut newarr = Array::new(cap);
                newarr[..arr.len()].copy_from_slice(&arr[..]);

                Node::BR(id, newarr)
            }
            Node::LEAF(_id, arr) => {
                debug_assert!(cap >= arr.len());

                let mut newarr = Array::new(cap);
                newarr[..arr.len()].clone_from_slice(&arr[..]);

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

    fn clone_tree(&self) -> *mut Node<T>
    where
        T: Clone,
    {
        unsafe { Self::clone_tree_(self as *const Node<T> as *mut Node<T>) }
    }

    unsafe fn clone_tree_(node: *mut Node<T>) -> *mut Node<T>
    where
        T: Clone,
    {
        if node.is_null() {
            return node;
        }

        let cloned_node = boxptr!((*node).clone());

        if let Self::BR(_, ref mut arr) = &mut *cloned_node {
            for i in 0..arr.len() {
                arr[i] = Self::clone_tree_(arr[i]);
            }
        }

        cloned_node
    }
}


impl<T: Clone> Clone for Node<T> {
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

                match arr.len() {
                    0 => writeln!(f, "[]"),
                    1 => writeln!(f, "[{}]", arr[0]),
                    2 => writeln!(f, "[{}, {}]", arr[0], arr[1]),
                    3 => writeln!(f, "[{}, {}, {}]", arr[0], arr[1], arr[2],),
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


impl<T> PTrieVec<T> {
    /// Tail Offset (elements number before tail)
    fn tailoff(&self) -> usize {
        if self.cnt == 0 {
            return 0;
        }

        ((self.cnt - 1) >> BIT_WIDTH) << BIT_WIDTH
    }

    /// Indicate the trie structure.
    fn shift(&self) -> u32 {
        (self.height() - 1) * BIT_WIDTH
    }

    /// Leaf should be same level
    fn height(&self) -> u32 {
        trie_height(self.tailoff())
    }
}


/// Impl Persistent Vec
impl<T: Debug + Clone> PTrieVec<T> {
    ////////////////////////////////////////////////////////////////////////////
    //// Public API

    pub fn empty() -> Self {
        Self {
            cnt: 0,
            root: null_mut(),
            tail: null_mut(),
        }
    }

    pub fn len(&self) -> usize {
        self.cnt
    }

    fn new(cnt: usize, root: *mut Node<T>, tail: *mut Node<T>) -> Self {
        Self { cnt, root, tail }
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

    pub fn push(&self, item: T) -> Self {
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

                (*tail).as_leaf_mut()[..old_tail_arr.len()]
                    .clone_from_slice(&old_tail_arr[..]);
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

            // including height == 1
            if tailoff == NODE_SIZE.pow(self.height() as u32) {
                root = Node::new_br(self.id(), 2);
                (*root).as_br_mut()[0] = self.root;
                (*root).as_br_mut()[1] =
                    Self::new_path(self.id(), shift, leaf);
            } else {
                debug_assert!(self.height() >= 2);

                root = self.push_tail_into_trie(shift, self.root, leaf)
            }

            Self::new(cnt, root, tail)
        }
    }

    pub fn pop(&self) -> Self {
        unsafe {
            assert!(self.cnt > 0, "Can't pop empty vector");

            if self.cnt == 1 {
                return Self::empty();
            }

            let cnt = self.cnt - 1;

            if self.cnt - self.tailoff() > 1 {
                let tail = Node::new_leaf(self.id(), (*self.tail).len() - 1);
                (*tail).as_leaf_mut()[..].clone_from_slice(
                    &(*self.tail).as_leaf()[..(*tail).len()],
                );

                let root = self.root;

                return Self::new(cnt, root, tail);
            }


            let tail = self.array_for(self.cnt - 2);

            let mut root;
            if self.cnt == NODE_SIZE + 1 {
                debug_assert_eq!(self.height(), 1);

                root = null_mut();
            } else {
                debug_assert!(self.height() > 1);

                let shift = self.shift();

                root = self.pop_tail_from_trie(shift, self.root);

                if shift >= BIT_WIDTH && (*root).as_br()[1].is_null() {
                    root = (*root).as_br()[0]; // remove empty root
                }
            }

            Self::new(cnt, root, tail)
        }
    }

    pub fn assoc(&self, idx: usize, item: T) -> Self {
        unsafe {
            assert!(self.cnt >= idx);

            if idx == self.cnt {
                return self.push(item);
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
    }

    pub fn transient(&self) -> TTrieVec<T> {
        TTrieVec {
            cnt: self.cnt,
            root: TTrieVec::editable(self.root),
            tail: TTrieVec::editable(self.tail),
        }
    }

    pub fn persistent(&self) -> Self {
        self.clone()
    }

    pub fn nth(&self, idx: usize) -> &T {
        debug_assert!(idx < self.cnt);

        let arr = self.array_for(idx);

        unsafe { &(*arr).as_leaf()[idx & MASK] }
    }

    pub fn peek(&self) -> Option<&T> {
        if self.cnt == 0 {
            return None;
        }

        Some(self.nth(self.cnt - 1))
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


    ////////////////////////////////////////////////////////////////////////////
    //// Inner Method


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

                shift -= BIT_WIDTH;
            }

            cur
        }
    }

    fn new_path(
        id: Option<Uuid>,
        shift: u32,
        node: *mut Node<T>,
    ) -> *mut Node<T> {
        if shift == 0 {
            return node;
        }

        let ret = Node::new_br(id, 1);

        unsafe {
            (*ret).as_br_mut()[0] =
                Self::new_path(id, shift - BIT_WIDTH, node);
        }

        ret
    }

    // The Trie isn't full.
    unsafe fn push_tail_into_trie(
        &self,
        shift: u32,
        paren: *mut Node<T>,
        leaf: *mut Node<T>,
    ) -> *mut Node<T> {
        let sub_idx = ((self.cnt - 1) >> shift) & MASK;

        let paren_arr = (*paren).as_br();
        let ret = Node::new_br(
            self.id(),
            min(paren_arr.len() + 1, NODE_SIZE)
        );

        (*ret).as_br_mut()[..paren_arr.len()].copy_from_slice(&paren_arr[..]);

        let node_to_insert;
        if shift == BIT_WIDTH {
            node_to_insert = leaf;
        } else {
            node_to_insert =
            if (*paren).len() > sub_idx
                && !(*paren).as_br()[sub_idx].is_null()
            {
                let child = (*paren).as_br()[sub_idx];
                self.push_tail_into_trie(shift - BIT_WIDTH, child, leaf)
            } else {
                Self::new_path(self.id(), shift - BIT_WIDTH, leaf)
            };
        }

        (*ret).as_br_mut()[sub_idx] = node_to_insert;

        ret
    }

    unsafe fn pop_tail_from_trie(
        &self,
        shift: u32,
        node: *mut Node<T>,
    ) -> *mut Node<T> {
        let sub_id = ((self.cnt - 2) >> shift) & MASK;

        if shift > BIT_WIDTH {
            debug_assert!((*node).len() > sub_id);

            let child = self.pop_tail_from_trie(
                shift - BIT_WIDTH,
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

    unsafe fn do_assoc(
        shift: u32,
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
                Self::do_assoc(shift - BIT_WIDTH, child, idx, item);
        }

        ret
    }
}




impl<T: Debug + Clone> Debug for PTrieVec<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for t in self.iter() {
            write!(f, "{:?} ", (*t))?
        }

        Ok(())
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
impl<T: Debug + Clone> TTrieVec<T> {
    ////////////////////////////////////////////////////////////////////////////
    //// Public API

    pub fn empty() -> Self {
        TTrieVec {
            cnt: 0,
            root: null_mut(),
            tail: null_mut(),
        }
    }

    pub fn len(&self) -> usize {
        self.cnt
    }

    pub fn id(&self) -> Option<Uuid> {
        if self.root.is_null() {
            None
        } else {
            unsafe { (*self.root).id() }
        }
    }

    pub fn clone_tree(&self) -> Self {
        unsafe {
            Self {
                cnt: self.cnt,
                root: (*self.root).clone_tree(),
                tail: (*self.tail).clone_tree(),
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

    pub fn nth(&self, idx: usize) -> &T {
        debug_assert!(idx < self.cnt);

        let arr = self.array_for(idx);

        unsafe { &(*arr).as_leaf()[idx & MASK] }
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

    pub fn peek(&self) -> Option<&T> {
        if self.cnt == 0 {
            return None;
        }

        Some(self.nth(self.cnt - 1))
    }

    pub fn push(&mut self, item: T) -> Self {
        unsafe {
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
                    self.tail =
                        (*self.tail).duplicate_with(self.id(), NODE_SIZE);
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
                (*root).as_br_mut()[1] =
                    new_path(self.id(), shift, leaf, NODE_SIZE);
            } else {
                root = self.push_tail_into_trie(shift, self.root, leaf)
            }

            self.root = root;
            self.tail = tail;
            self.cnt += 1;

            return self.clone_head();
        }
    }

    pub fn pop(&mut self) -> Self {
        unsafe {
            assert!(self.cnt > 0, "Can't pop empty vector");

            if self.cnt == 1 || self.cnt - self.tailoff() > 1 {
                self.cnt -= 1;

                return self.clone_head();
            }

            let tail = self.editable_array_for(self.cnt - 2);

            let mut root;
            if self.cnt == NODE_SIZE + 1 {
                root = null_mut();
            } else {
                let shift = self.shift();

                root = self.pop_tail_from_trie(shift, self.root);

                if shift >= BIT_WIDTH && (*root).as_br()[1].is_null() {
                    // remove empty root
                    root = self.ensure_editable((*root).as_br()[0]);
                }
            }

            self.root = root;
            self.tail = tail;
            self.cnt -= 1;

            self.clone_head()
        }
    }

    pub fn assoc(&mut self, idx: usize, item: T) -> Self {
        unsafe {
            assert!(self.cnt >= idx);

            if idx == self.cnt {
                return self.push(item);
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
    }

    pub fn duplicate(&self) -> Self {
        self.clone_tree()
    }

    pub fn transient(&self) -> Self {
        self.clone_head()
    }

    pub fn persistent(&self) -> PTrieVec<T> {
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

                (*tail).as_leaf_mut()[..].clone_from_slice(
                    &(*self.tail).as_leaf()[..(*tail).len()],
                );

                tail
            };

            PTrieVec {
                cnt: self.cnt,
                root,
                tail,
            }
        }
    }


    ////////////////////////////////////////////////////////////////////////////
    //// Inner Method

    unsafe fn ensure_editable(&self, node: *mut Node<T>) -> *mut Node<T> {
        if self.id() == (*node).id() {
            node
        } else {
            Self::editable(node)
        }
    }

    fn editable(node: *mut Node<T>) -> *mut Node<T> {
        unsafe {
            if node.is_null() {
                return node;
            }

            (*node).duplicate_with(Some(Uuid::new_v4()), NODE_SIZE)
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
    fn shift(&self) -> u32 {
        (self.height() - 1) * BIT_WIDTH
    }

    /// Leaf should be same level
    fn height(&self) -> u32 {
        trie_height(self.tailoff())
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

                shift -= BIT_WIDTH;
            }

            cur
        }
    }

    // The Trie isn't full.
    unsafe fn push_tail_into_trie(
        &self,
        shift: u32,
        paren: *mut Node<T>,  // from root
        leaf: *mut Node<T>,
    ) -> *mut Node<T> {
        let sub_idx = ((self.cnt - 1) >> shift) & MASK;

        let ret = self.ensure_editable(paren);

        let node_to_insert;
        if shift == BIT_WIDTH {
            node_to_insert = leaf;
        } else {
            let child = (*paren).as_br()[sub_idx];

            node_to_insert = if !child.is_null() {
                self.push_tail_into_trie(shift - BIT_WIDTH, child, leaf)
            } else {
                new_path(self.id(), shift - BIT_WIDTH, leaf, NODE_SIZE)
            };
        }

        (*ret).as_br_mut()[sub_idx] = node_to_insert;

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
                cur = (*self.ensure_editable(cur)).as_br()
                    [(idx >> shift) & MASK];

                shift -= BIT_WIDTH;
            }

            cur
        }
    }

    unsafe fn pop_tail_from_trie(
        &self,
        shift: u32,
        node: *mut Node<T>,
    ) -> *mut Node<T> {
        let node = self.ensure_editable(node);
        let sub_id = ((self.cnt - 2) >> shift) & MASK;

        if shift > BIT_WIDTH {
            debug_assert!((*node).len() > sub_id);

            let child = self.pop_tail_from_trie(
                shift - BIT_WIDTH,
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

    unsafe fn do_assoc(
        &self,
        shift: u32,
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
                self.do_assoc(shift - BIT_WIDTH, child, idx, item);
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


impl<T: Clone + Debug> Clone for TTrieVec<T> {
    fn clone(&self) -> Self {
        self.duplicate()
    }
}


impl<T: Debug + Clone> Debug for TTrieVec<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for t in self.iter() {
            write!(f, "{:?} ", (*t))?
        }

        Ok(())
    }
}


const fn trie_height(trie_size: usize) -> u32 {
    match trie_size {
        0 => 0,
        1 => 1,
        x => {
            let mut h = (x - 1).ilog2() / BIT_WIDTH;

            if x > NODE_SIZE.pow(h as u32) {
                h += 1;
            }

            h
        }
    }
}


fn new_path<T>(
    id: Option<Uuid>,
    shift: u32,
    node: *mut Node<T>,
    cap: usize,
) -> *mut Node<T> {
    if shift == 0 {
        return node;
    }

    let ret = Node::new_br(id, cap);

    unsafe {
        (*ret).as_br_mut()[0] =
            new_path(id, shift - BIT_WIDTH, node, cap);
    }

    ret
}


#[cfg(test)]
mod tests {
    use super::{super::vec::*, *};

    #[test]
    fn test_ptrie0_vec_randomedata() {
        test_pvec!(PTrieVec::empty());
    }

    #[test]
    fn test_ttrie0_vec_randomedata() {
        test_tvec!(TTrieVec::empty());
    }

    #[test]
    fn test_pttrie0_vec_tran_randomdata() {
        test_pttran!(PTrieVec::empty());
    }

    #[test]
    fn test_ttrie0_vec_manually() {
        let tv = TTrieVec::empty();

        // let mut bpv = (box pv) as Box<dyn Vector<usize>>;
        let mut btv = tv;

        btv = btv.push(0);
        btv = btv.push(1);
        btv = btv.push(2);
        btv = btv.push(3);
        btv = btv.push(4);
        btv = btv.push(5);
        btv = btv.push(6);
        btv = btv.push(7);
        btv = btv.push(8);
        btv = btv.push(9);
        btv = btv.push(10);
        btv = btv.push(11);

        let total = 500;

        for i in 12..total {
            btv = btv.push(i);
        }

        for i in 0..total {
            btv = btv.assoc(i, i * 100);
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
            btv = btv.pop();

            for j in 0..i {
                assert_eq!(btv.nth(j), &uelem_vec[j]);
            }
        }

        // btv = btv.pop_().unwrap();

        // btv.bfs_display();
    }


    #[test]
    fn test_pttrie0_vec_tran_manually() {
        // let mut vec = (box PVec::empty()) as Box<dyn Vector<usize>>;
        let mut pvec = PTrieVec::empty();
        let pbatchnum = 300;
        for i in 0..pbatchnum {
            pvec = pvec.push(i);
        }

        // println!("Before Transistent");
        // pvec.bfs_display();


        let mut tvec = pvec.transient();

        // tvec = tvec.push(10);
        let tbatchnum = 300;
        for i in pbatchnum..pbatchnum + tbatchnum {
            tvec = tvec.push(i * 10);

            for j in pbatchnum..i {
                // println!("j: {}", j);
                assert_eq!(*tvec.nth(j), j * 10);
            }
        }
    }
}
