//! Relaxed Radix Balanced Tree AKA RRB, That's Extended Digit Trie Vector
//!
//! Reference [It](https://hypirion.com/musings/thesis)
//!
//! Origin Author: hypirion


use std::{
    cmp::{max, min},
    collections::VecDeque,
    error::Error,
    fmt::{Debug, Display},
    ops::{Index, IndexMut, Add},
    ptr::{copy_nonoverlapping, null, null_mut},
};

use derive_where::*;
use coll::Itertools;
use uuid::Uuid;

use super::Vector;
use crate::{
    array,
    {as_ptr, aux::RoadMap, bare::array::Array, Collection},
    etc::strshift,
    roadmap, should,
};

const BIT_WIDTH: usize = 1;
const NODE_SIZE: usize = 1 << BIT_WIDTH;
const MASK: usize = NODE_SIZE - 1;
const MAX_H: usize = 14;

// const INVARIANT: usize = 1;
// const EXTRAS: usize = 2;


////////////////////////////////////////////////////////////////////////////////
//// Structures

#[derive(DeriveWhere)]
#[derive_where(Clone, Copy)]
enum Node<T> {
    BR(*mut Br<T>),
    LF(*mut Leaf<T>),
}

#[derive(DeriveWhere)]
#[derive_where(Clone)]
struct Br<T> {
    id: *const Uuid,
    szt: *mut SZT,
    children: Array<Node<T>>,
}

/// Size Table
struct SZT {
    id: *const Uuid,
    tbl: Array<usize>,
}


struct Leaf<T> {
    id: *const Uuid,
    items: Array<T>,
}


#[derive(Copy)]
pub struct RRBVec<T> {
    cnt: usize,
    shift: usize,
    taillen: usize,
    tail: Node<T>,
    root: *mut Node<T>,
}



////////////////////////////////////////////////////////////////////////////////
//// Impls


impl<T> Node<T> {
    fn as_br(&self) -> &Br<T> {
        match self {
            Self::BR(it) => unsafe {
                debug_assert!(!it.is_null());

                &(**it)
            },
            _ => unreachable!(),
        }
    }

    fn as_br_mut(&self) -> &mut Br<T> {
        match self {
            Self::BR(it) => unsafe {
                debug_assert!(!it.is_null());

                &mut (**it)
            },
            _ => unreachable!(),
        }
    }

    fn as_leaf(&self) -> &Leaf<T> {
        match self {
            Self::LF(it) => unsafe {
                debug_assert!(!it.is_null());

                &(**it)
            },
            _ => unreachable!(),
        }
    }

    fn as_leaf_mut(&self) -> &mut Leaf<T> {
        match self {
            Self::LF(it) => unsafe {
                debug_assert!(!it.is_null());

                &mut (**it)
            },
            _ => unreachable!(),
        }
    }

    fn is_br(&self) -> bool {
        match self {
            Node::BR(_) => true,
            Node::LF(_) => false,
        }
    }

    fn is_leaf(&self) -> bool {
        !self.is_br()
    }


    fn cnt(&self, shift: usize) -> usize {
        if shift > 0 {
            debug_assert!(self.is_br());

            let br = self.as_br();
            if br.szt.is_null() {
                let the_last_child_cnt = br[br.len() - 1].cnt(shift.decshf());

                // all but the last child are filled
                ((br.len() - 1) << shift) + the_last_child_cnt
            } else {
                br.szt()[br.len() - 1]
            }
        } else {
            debug_assert!(self.is_leaf());

            self.len()
        }
    }
}


impl<T> Collection for Node<T> {
    fn len(&self) -> usize {
        match self {
            Node::BR(arr) => unsafe {
                if arr.is_null() {
                    return 0;
                }

                (**arr).len()
            },
            Node::LF(arr) => unsafe {
                if arr.is_null() {
                    return 0;
                }

                (**arr).len()
            },
        }
    }
}

impl Display for Node<usize> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BR(br) => {
                writeln!(f, "{}", unsafe { &**br })
            }
            Self::LF(leaf) => {
                writeln!(f, "{}", unsafe { &**leaf })
            }
        }
    }
}

fn fuck_node<T: Debug>(node: Node<T>) {
    unsafe {
        match node {
            Node::BR(br) => {
                let br = &*br;

                println!("BR: {}", br.len());
            }
            Node::LF(leaf) => {
                let leaf = &*leaf;

                println!("LEAF: {:?}", leaf);
            }
        }
    }
}


impl<T> Index<&RoadMap> for Node<T> {
    type Output = Node<T>;

    fn index(&self, index: &RoadMap) -> &Self::Output {
        assert!(!index.is_empty());

        let mut cur = self;
        for i in 0..index.len() {
            let pos = index[i] as usize;

            if cur.is_br() {
                cur = &cur.as_br()[pos];
            } else {
                unreachable!("Found Leaf too early!");
            }
        }

        cur
    }
}




///////////////////////////////////////
//// Impl Branch Node (Internal Node)

impl<T> Br<T> {
    fn new_(
        id: *const Uuid,
        szt: *mut SZT,
        children: Array<Node<T>>,
    ) -> Node<T> {
        Node::BR(as_ptr(Self { id, szt, children }))
    }

    fn new(cap: usize) -> Node<T> {
        Self::new_(null(), null_mut(), Array::new(cap))
    }

    fn empty() -> Node<T> {
        Self::new(0)
    }

    /// with just one Branch
    fn new_with_1(child: Node<T>) -> Node<T> {
        debug_assert!(child.is_br());

        let newbr = Self::new(1);
        newbr.as_br_mut()[0] = child;

        newbr
    }

    /// with two Branch
    fn new_with_2(child_0: Node<T>, child_1: Node<T>) -> Node<T> {
        debug_assert!(child_0.is_br());
        debug_assert!(child_1.is_br());

        let newbr = Self::new(1);
        newbr.as_br_mut()[0] = child_0;
        newbr.as_br_mut()[1] = child_1;

        newbr
    }

    /// (0..left-1) + centre + (1..right)
    fn merge_with_3(
        child_0: Node<T>,
        child_1: Node<T>,
        child_2: Node<T>,
    ) -> Node<T> {
        let left_len = if child_0.is_empty() {
            0
        } else {
            child_0.len() - 1
        };

        let centre_len = if child_1.is_empty() { 0 } else { child_1.len() };

        let right_len = if child_2.is_empty() {
            0
        } else {
            child_2.len() - 1
        };

        let merged = Br::new(left_len + centre_len + right_len);

        let ptr: *mut Node<T> = merged.as_br().children.as_ptr();

        if left_len != 0 {
            unsafe {
                copy_nonoverlapping(
                    child_0.as_br().children.as_ptr(),
                    ptr,
                    left_len,
                )
            }
        }
        if centre_len != 0 {
            unsafe {
                copy_nonoverlapping(
                    child_1.as_br().children.as_ptr(),
                    ptr.add(left_len),
                    centre_len,
                )
            }
        }
        if right_len != 0 {
            unsafe {
                copy_nonoverlapping(
                    child_2.as_br().children.as_ptr().add(1),
                    ptr.add(left_len + centre_len),
                    right_len,
                )
            }
        }

        merged
    }


    fn append_empty(
        to_set: *mut Node<T>,
        empty_height: usize,
    ) -> *mut Node<T> {
        if empty_height > 0 {
            let leaf = Br::new(1);
            let mut empty = leaf;

            for _ in 1..empty_height {
                let new_empty = Br::new(1);
                new_empty.as_br_mut()[0] = empty;
                empty = new_empty;
            }

            unsafe {
                (*to_set) = empty;
            }

            leaf.as_br().children.as_ptr()
        } else {
            to_set
        }
    }


    fn just_clone(&self) -> Node<T> {
        let children = self.children.clone();
        let szt = self.szt;
        let id = self.id;

        Self::new_(id, szt, children)
    }

    fn clone_extract(&self, start: usize, len: usize) -> Node<T> {
        debug_assert!(self.len() >= start + len);

        debug_assert!(self.id.is_null());
        let newit = Self::new(len);

        let newptr = newit.as_br().children.as_ptr();
        let selfptr = self.children.as_ptr();

        unsafe { copy_nonoverlapping(selfptr.add(start), newptr, len) }

        newit
    }

    fn clone_with_inc(&self) -> Node<T> {
        let children = self.children.clone_with(self.len(), self.len() + 1);

        let szt = if self.szt.is_null() {
            self.szt
        } else {
            self.szt().clone_with_inc()
        };

        Self::new_(null(), szt, children)
    }

    fn clone_with_dec(&self) -> Node<T> {
        debug_assert!(!self.is_empty());

        let children =
            self.children.clone_with(self.len() - 1, self.len() - 1);

        let szt = if self.szt.is_null() {
            self.szt
        } else {
            self.szt().clone_with_dec()
        };

        debug_assert!(self.id.is_null());
        Self::new_(null(), szt, children)
    }

    fn set_szt(&mut self, shift: usize)
    where
        T: Clone,
    {
        let mut szt = SZT::new(self.len());

        let mut acc = 0;
        for i in 0..self.len() {
            acc += self.children[i].cnt(shift.decshf());
            szt[i] = acc;
        }

        self.szt = as_ptr(szt);
    }

    fn szt(&self) -> &SZT {
        debug_assert!(!self.szt.is_null());

        unsafe { &*self.szt }
    }

    fn szt_mut(&self) -> &mut SZT {
        debug_assert!(!self.szt.is_null());

        unsafe { &mut *self.szt }
    }

    fn sized_pos(&self, idx: &mut usize, shift: usize) -> usize {
        let mut is = *idx >> shift;
        while self.szt()[is] <= *idx {
            is += 1
        }

        if is != 0 {
            *idx -= self.szt()[is - 1]
        }

        is
    }
}


impl<T> Collection for Br<T> {
    fn len(&self) -> usize {
        self.children.len()
    }
}

impl<T> Index<usize> for Br<T> {
    type Output = Node<T>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.children[index]
    }
}

impl<T> IndexMut<usize> for Br<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.children[index]
    }
}

impl<T> Display for Br<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "sz: ")?;
        if self.szt.is_null() {
            writeln!(f, "null")?;
        } else {
            writeln!(f, "{:?}", unsafe { &*self.szt })?;
        }

        write!(f, "[]: ")?;

        let mut acc = 0;
        for item in self.children.iter() {
            if item.is_empty() {
                break;
            }

            acc += 1;
        }

        if acc == 0 {
            writeln!(f, "0")?;
        } else {
            writeln!(f, "0..{}", acc)?;
        }

        Ok(())
    }
}


///////////////////////////////////////
//// Impl Size Table

impl SZT {
    fn new(cap: usize) -> Self {
        Self {
            id: null(),
            tbl: Array::new(cap),
        }
    }

    fn empty() -> Self {
        Self::new(0)
    }

    fn clone_with(&self, len: usize) -> *mut Self {
        debug_assert!(self.tbl.len() >= len);

        as_ptr(Self {
            id: self.id,
            tbl: self.tbl.clone_with(len, len),
        })
    }

    /// Clone with capacity be greater one than self.len
    fn clone_with_inc(&self) -> *mut Self {
        as_ptr(Self {
            id: self.id,
            tbl: self.tbl.clone_with(self.len(), self.len() + 1),
        })
    }

    fn clone_with_dec(&self) -> *mut Self {
        as_ptr(Self {
            id: self.id,
            tbl: self.tbl.clone_with(self.len() - 1, self.len() - 1),
        })
    }
}

impl Collection for SZT {
    fn len(&self) -> usize {
        self.tbl.len()
    }
}

impl Index<usize> for SZT {
    type Output = usize;

    fn index(&self, index: usize) -> &Self::Output {
        &self.tbl[index]
    }
}

impl IndexMut<usize> for SZT {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.tbl[index]
    }
}

impl Debug for SZT {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.len() > 10 {
            writeln!(f)?;
            for chunk in &self.tbl.iter().chunks(10) {
                for item in chunk {
                    write!(f, "\t{:>4}, ", item)?;
                }
                writeln!(f)?;
            }
        } else {
            write!(f, "[ ")?;
            for i in 0..self.len() {
                if i < self.len() - 1 {
                    write!(f, "{}, ", self[i])?;
                }
                else {
                    write!(f, "{}", self[i])?;
                }
            }
            write!(f, " ]")?;
        }

        Ok(())
    }
}


///////////////////////////////////////
//// Impl Leaf Node

impl<T: Clone> Leaf<T> {
    fn new_(id: *const Uuid, items: Array<T>) -> Node<T> {
        Node::LF(as_ptr(Self { id, items }))
    }

    fn new(cap: usize) -> Node<T> {
        Self::new_(null(), Array::new(cap))
    }

    fn empty() -> Node<T> {
        Self::new(0)
    }

    fn merge(lf: Node<T>, rh: Node<T>) -> Node<T> {
        Self::new_(
            null(),
            Array::merge(&lf.as_leaf().items, &rh.as_leaf().items),
        )
    }

    fn just_clone(&self) -> Node<T>
    where
        T: Clone,
    {
        self.clone_with(self.len())
    }

    fn clone_with(&self, len: usize) -> Node<T>
    where
        T: Clone,
    {
        Self::new_(self.id, self.items.clone_with(len, len))
    }

    fn clone_with_inc(&self) -> Node<T>
    where
        T: Clone,
    {
        Self::new_(self.id, self.items.clone_with(self.len(), self.len() + 1))
    }

    fn clone_with_dec(&self) -> Node<T>
    where
        T: Clone,
    {
        debug_assert!(!self.is_empty());

        Self::new_(
            self.id,
            self.items.clone_with(self.len() - 1, self.len() - 1),
        )
    }
}

impl<T> Collection for Leaf<T> {
    fn len(&self) -> usize {
        self.items.len()
    }
}

impl<T> Index<usize> for Leaf<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.items[index]
    }
}

impl<T> IndexMut<usize> for Leaf<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.items[index]
    }
}

impl<T: Display> Display for Leaf<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.len() > 10 {
            writeln!(f)?;
            for chunk in &self.items.iter().chunks(10) {
                for item in chunk {
                    write!(f, "{:>4}, ", item)?;
                }
                writeln!(f)?;
            }
        } else {
            write!(f, "[ ")?;
            for i in 0..self.len() {
                write!(f, "{}", self.items[i])?;
                if i < self.len() - 1 {
                    write!(f, ", ")?;
                }
            }
            write!(f, " ]")?;
        }


        Ok(())
    }
}

impl<T: Debug> Debug for Leaf<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}


///////////////////////////////////////
//// Impl NextShift (Show some tricks)

trait NxtShf<T> {
    fn incshf(&self) -> T;
    fn decshf(&self) -> T;
}

impl NxtShf<usize> for usize {
    fn incshf(&self) -> usize {
        *self + BIT_WIDTH
    }

    fn decshf(&self) -> usize {
        debug_assert!(
            *self >= BIT_WIDTH,
            "invalid shift dec {} - {}",
            self,
            BIT_WIDTH
        );
        *self - BIT_WIDTH
    }
}

impl PartialOrd for dyn NxtShf<usize> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.incshf().cmp(&other.incshf()))
    }
}

impl PartialEq for dyn NxtShf<usize> {
    fn eq(&self, other: &Self) -> bool {
        self.incshf() == other.incshf()
    }
}

impl dyn NxtShf<usize> {
    fn itself(&self) -> usize {
        self.incshf().decshf()
    }

    fn iter<'a>(&'a self) -> Box<dyn Iterator<Item = usize> + 'a> {
        let mut cur = self.itself();

        Box::new(std::iter::from_fn(move || {
            if cur > 0 {
                let prev = cur;
                cur = cur.decshf();
                Some(prev)
            } else {
                None
            }
        }))
    }
}


///////////////////////////////////////
//// Impl RBBVec

impl<T: Clone + Debug> RRBVec<T> {
    fn empty() -> Self {
        Self {
            cnt: 0,
            shift: 0,
            taillen: 0,
            tail: Leaf::empty(),
            root: as_ptr(Br::empty()),
        }
    }

    fn root(&self) -> &Node<T> {
        debug_assert!(!self.root.is_null());

        unsafe { &*self.root }
    }

    pub fn head_clone(&self) -> Self {
        // Clone is implemented as head_clone
        self.clone()
    }


    fn assoc_(&self, idx: usize, item: T) -> Self {
        debug_assert!(idx <= self.cnt);

        if idx == self.cnt {
            return self.push_(item);
        }

        let mut rrb = self.head_clone();
        let mut idx = idx;
        let tailoff = self.cnt - self.taillen;

        if tailoff <= idx {
            let tail = self.tail.as_leaf().just_clone();
            tail.as_leaf_mut()[idx - tailoff] = item;
            rrb.tail = tail;
            rrb
        } else {
            debug_assert!(!rrb.root.is_null());
            debug_assert!(rrb.root().is_br());

            // *mut Node<T>
            let mut prev = rrb.root;
            let mut cur = unsafe { *self.root };

            for shift in (&self.shift as &dyn NxtShf<usize>).iter() {
                cur = cur.as_br().just_clone();
                unsafe { *prev = cur };
                let cur_br = cur.as_br_mut();

                let subidx;
                if cur_br.szt.is_null() {
                    subidx = (idx >> shift) & MASK;
                } else {
                    subidx = cur_br.sized_pos(&mut idx, shift);
                }

                prev = unsafe { cur_br.children.as_ptr().add(subidx) };
                cur = cur_br[subidx];
            }

            let leaf_node = cur.as_leaf().just_clone();
            leaf_node.as_leaf_mut()[idx & MASK] = item;
            unsafe { *prev = leaf_node }

            rrb
        }
    }


    fn push_(&self, item: T) -> Self {
        let mut rrb = self.head_clone();
        rrb.cnt += 1;

        if self.taillen < NODE_SIZE {
            let tail = rrb.tail.as_leaf().clone_with_inc();
            tail.as_leaf_mut()[rrb.taillen] = item;
            rrb.tail = tail;
            rrb.taillen += 1;

            return rrb;
        }

        let tail = Leaf::new(1);
        tail.as_leaf_mut()[0] = item;
        rrb.taillen = 1;

        self.push_into_trie(rrb, tail)
    }

    fn push_into_trie(&self, mut rrb: Self, tail: Node<T>) -> Self {
        debug_assert!(tail.is_leaf());

        let old_tail = rrb.tail;
        rrb.tail = tail;

        if self.cnt <= NODE_SIZE {
            rrb.shift = 0;
            rrb.root = as_ptr(old_tail);
            return rrb;
        }

        // TODO: Can find last rightmost jump in constant time for pvec subvecs:
        // use the fact that (index & large_mask) == 1 << (RRB_BITS * H) - 1 -> 0 etc.

        let mut idx = self.cnt - 1;
        let mut nodes_to_copy = 0;
        let mut nodes_visited = 0;

        let mut cur = unsafe { *self.root };
        let mut shift = self.shift;
        let mut goto_copyable_count_end = false;

        // checking all non-leaf nodes (or if tail, all but the lowest two levels)
        while shift > 0usize.incshf() {
            let cur_br = cur.as_br_mut();

            let subidx;
            if cur_br.szt.is_null() {
                // some check here to ensure we're not overflowing the pvec subvec.
                // important to realise that this only needs to be done once
                // in a better impl,
                // the same way the size_table check only has to be done
                // until it's false.

                if idx >> shift.incshf() > 0 {
                    nodes_visited += 1;
                    goto_copyable_count_end = true;

                    break;
                }

                subidx = (idx >> shift) & MASK;

                // index filtering is not necessary
                // when the check above is performed at most once.
                idx &= !(MASK << shift);
            } else {
                subidx = cur_br.len() - 1;

                if subidx != 0 {
                    idx -= cur_br.szt()[subidx - 1];
                }
            }

            nodes_visited += 1;
            if subidx < MASK {
                nodes_to_copy = nodes_visited;
            }

            if subidx >= cur_br.len() {
                cur = Leaf::empty();
            } else {
                cur = cur_br[subidx];
            }

            if cur.is_empty() {
                nodes_to_copy = nodes_visited;

                goto_copyable_count_end = true;
                break;
            }

            shift = shift.decshf();
        }

        if !goto_copyable_count_end {
            if shift != 0 {
                nodes_visited += 1;
                if cur.len() < NODE_SIZE {
                    nodes_to_copy = nodes_visited;
                }
            }
        }

        // nodes_visited is not yet handled nicely. for loop down to get
        // nodes_visited set straight.

        while shift > 0usize.incshf() {
            nodes_visited += 1;

            shift = shift.decshf();
        }

        // inc tree height
        if nodes_to_copy == 0 {
            let root = Br::new(2);
            root.as_br_mut()[0] = unsafe { *self.root };
            rrb.root = as_ptr(root);
            rrb.shift = self.shift.incshf();

            // create size table if the original rrb root has a size table.
            if self.root().is_br() && !self.root().as_br().szt.is_null() {
                let mut szt = SZT::new(2);
                szt[0] = self.cnt - old_tail.len();
                // If we insert the tail, the old size minus the old tail size
                // will be the amount of elements in the left branch.
                // If there is no tail, the size is just the old rrb-tree.

                szt[1] = self.cnt;
                // If we insert the tail, the old size would include the tail.
                // Consequently, it has to be the old size.
                // If we have no tail, we append a single element to the old vector,
                // therefore it has to be one more than the original.

                root.as_br_mut().szt = as_ptr(szt);
            }

            let to_set =
                unsafe { rrb.root().as_br().children.as_ptr().add(1) };

            let to_set = Br::append_empty(to_set, nodes_visited);

            unsafe { (*to_set) = old_tail }
        } else {
            let node = self.copy_first_k(&rrb, nodes_to_copy, old_tail.len());

            let to_set = Br::append_empty(node, nodes_visited - nodes_to_copy);

            unsafe { (*to_set) = old_tail }
        }

        rrb
    }


    fn copy_first_k(
        &self,
        rrb: &RRBVec<T>,
        k: usize,
        tailsize: usize,
    ) -> *mut Node<T> {
        let mut cur = self.root();
        let mut to_set = rrb.root;
        let mut idx = self.cnt - 1;
        let mut shift = self.shift;

        // Copy all non-leaf nodes first. Happens when shift > NODE_SIZE

        let mut i = 1;
        while i <= k && shift != 0 {
            let cur_br = cur.as_br();
            let new_cur;

            if i != k {
                new_cur = cur_br.just_clone();
                if !cur_br.szt.is_null() {
                    let new_cur_br = new_cur.as_br_mut();
                    let new_szt =
                        new_cur_br.szt().clone_with(new_cur_br.len());
                    unsafe {
                        (*new_szt)[new_cur_br.len() - 1] += tailsize;
                    }
                    new_cur_br.szt = new_szt;
                }
            } else {
                new_cur = cur_br.clone_with_inc();
                if !cur_br.szt.is_null() {
                    let new_cur_len = new_cur.len();
                    new_cur.as_br_mut().szt_mut()[new_cur_len - 1] =
                        new_cur.as_br().szt()[new_cur_len - 2] + tailsize;
                }
            }

            unsafe { *to_set = new_cur }

            let subidx;
            if cur_br.szt.is_null() {
                subidx = (idx >> shift) & MASK;
            } else {
                subidx = new_cur.len() - 1;

                if subidx != 0 {
                    idx -= cur_br.szt()[subidx - 1];
                }
            }

            to_set = unsafe { new_cur.as_br().children.as_ptr().add(subidx) };

            i += 1;
            shift = shift.decshf();

            if i <= k && shift != 0 {
                cur = &cur_br[subidx];
            }
        }

        to_set
    }


    fn pop_(&self) -> Result<Self, Box<dyn Error>> {
        should!(self.cnt > 0, "Can't pop empty vector");

        if self.cnt == 1 {
            return Ok(Self::empty());
        }

        let mut rrb = self.head_clone();
        rrb.cnt -= 1;

        if self.taillen == 1 {
            rrb.pop_from_trie();
        } else {
            let tail = self.tail.as_leaf().clone_with_dec();
            rrb.taillen -= 1;
            rrb.tail = tail;
        }

        Ok(rrb)
    }


    fn pop_from_trie(&mut self) {
        let mut path = array![Leaf::empty(); MAX_H + 1];

        path[0] = *self.root();

        let mut i = 0;
        let mut shift = 0;

        while shift < self.shift {
            path[i + 1] = path[i].as_br()[path[i].len() - 1];

            i += 1;
            shift = shift.incshf();
        }

        let height = i;

        // Set the leaf as tail
        self.tail = path[height];
        self.taillen = path[height].len();
        let taillen = self.taillen;

        path[height] = Leaf::empty();

        while i > 0 {
            i -= 1;

            if path[i + 1].is_empty() {
                if path[i].len() == 1 {
                    path[i] = Leaf::empty();
                } else if i == 0 && path[i].len() == 2 {
                    path[i] = path[i].as_br()[0];
                    self.shift = self.shift.decshf();
                } else {
                    path[i] = path[i].as_br().clone_with_dec();
                }
            } else {
                path[i] = path[i].as_br().just_clone();
                let path_i_len = path[i].len() - 1;
                path[i].as_br_mut()[path_i_len] = path[i + 1];

                if !path[i].as_br().szt.is_null() {
                    let path_i_len = path[i].len();
                    let szt = path[i].as_br().szt().clone_with(path_i_len);
                    unsafe {
                        (*szt)[path_i_len - 1] -= taillen;
                    }

                    path[i].as_br_mut().szt = szt;
                }
            }
        }

        self.root = as_ptr(path[0]);
    }


    pub fn iter<'a>(&'a self) -> Box<dyn Iterator<Item = &T> + 'a> {
        let mut i = 0;

        Box::new(std::iter::from_fn(move || {
            if i == self.cnt {
                return None;
            }

            let nxt = self.nth(i);
            i += 1;

            Some(nxt)
        }))
    }

    pub fn slice(&self, start: usize, end: usize) -> Self {
        self.slice_right(start).slice_left(end)
    }

    /// ... x)
    fn slice_right(&self, end: usize) -> Self {
        let rrb = self.head_clone();

        if end == 0 {
            return Self::empty();
        }

        if end < rrb.cnt {
            let tailoff = rrb.cnt - rrb.taillen;

            if tailoff < end {

                let mut new_rrb = rrb.head_clone();
                let new_taillen = end - tailoff;
                let new_tail = Leaf::new(new_taillen);

                unsafe {
                    copy_nonoverlapping(
                        rrb.tail.as_leaf().items.as_ptr(),
                        new_tail.as_leaf().items.as_ptr(),
                        new_taillen
                    );
                }

                new_rrb.cnt = end;
                new_rrb.tail = new_tail;
                new_rrb.taillen = new_taillen;

                return new_rrb;
            }

            let mut new_rrb = RRBVec::empty();
            let root = Self::slice_right_rec(
                &mut new_rrb.shift,
                unsafe{ *rrb.root },
                end - 1,
                rrb.shift,
                false
            );

            new_rrb.cnt = end;
            new_rrb.root = as_ptr(root);

            // Not sure if this is necessary in this part of the program, due to issues wrt.
            // slice_left and roots without size tables.

            new_rrb.pop_from_trie();
            new_rrb.taillen = new_rrb.tail.len();

            new_rrb
        }

        else {
            rrb
        }

    }

    fn slice_right_rec(
        total_shift: &mut usize,
        root: Node<T>,
        end: usize,
        shift: usize,
        has_left: bool
    ) -> Node<T>
    {

        let mut subidx = end >> shift;

        if shift > 0 {
            let subshift = shift.decshf();

            let root_br = root.as_br_mut();

            if root_br.szt.is_null() {
                let rh_node = Self::slice_right_rec(
                    total_shift,
                    root_br[subidx],
                    end - (subidx << shift),
                    subshift,
                    subidx != 0 || has_left
                );

                if subidx == 0 {

                    if has_left {
                        let rh_paren = Br::new(1);
                        rh_paren.as_br_mut()[0] = rh_node;
                        *total_shift = shift;

                        return rh_paren;
                    }
                    else {
                        return rh_node;
                    }
                }
                else {
                    let sliced_root = Br::new(subidx + 1);

                    unsafe {
                        copy_nonoverlapping(
                            root_br.children.as_ptr(),
                            sliced_root.as_br().children.as_ptr(),
                            subidx
                        );
                    }

                    sliced_root.as_br_mut()[subidx] = rh_node;
                    *total_shift = shift;

                    return sliced_root;
                }
            }
            else {

                let mut idx = end;

                while root_br.szt()[subidx] <= idx {
                    subidx += 1;
                }

                if subidx != 0 {
                    idx -= root_br.szt()[subidx - 1];
                }

                let rh_node = Self::slice_right_rec(
                        total_shift,
                        root_br[subidx],
                        idx,
                        subshift,
                        subidx != 0 || has_left
                    );

                if subidx == 0 {
                    if has_left {

                        let rh_paren = Br::new(1);
                        let mut rh_szt = SZT::new(1);

                        rh_szt[0] = end + 1;
                        rh_paren.as_br_mut().szt = as_ptr(rh_szt);
                        rh_paren.as_br_mut()[0] = rh_node;

                        *total_shift = shift;

                        return rh_paren;
                    }
                    else {
                        return rh_node;
                    }
                }
                else {

                    let sliced_root = Br::new(subidx + 1);
                    let mut sliced_szt = SZT::new(subidx + 1);

                    unsafe {
                        copy_nonoverlapping(
                            root_br.szt().tbl.as_ptr(),
                            sliced_szt.tbl.as_ptr(),
                            subidx
                        );

                        sliced_szt[subidx] = end + 1;

                        copy_nonoverlapping(
                            root_br.children.as_ptr(),
                            sliced_root.as_br().children.as_ptr(),
                            subidx
                        );
                    }

                    sliced_root.as_br_mut().szt = as_ptr(sliced_szt);
                    sliced_root.as_br_mut()[subidx] = rh_node;

                    *total_shift = shift;
                    return sliced_root;
                }
            }
        }
        else {

            // Just pure copying into a new node
            let left_vals = Leaf::new(subidx + 1);

            unsafe {
                copy_nonoverlapping(
                    root.as_leaf().items.as_ptr(),
                    left_vals.as_leaf().items.as_ptr(),
                    subidx + 1
                );
            }

            *total_shift = shift;
            left_vals
        }

    }


    /// [x, ...
    fn slice_left(&self, start: usize) -> Self {
        let mut rrb = self.head_clone();

        if start >= rrb.cnt {
            return Self::empty();
        }
        else if start > 0 {
            // slice cnt
            let slice_cnt = rrb.cnt - start;

            let mut new_rrb = Self::empty();
            rrb.cnt = slice_cnt;

            if slice_cnt <= rrb.taillen {
                let new_tail = Leaf::new(slice_cnt);

                unsafe {
                    copy_nonoverlapping(
                        rrb.tail
                        .as_leaf()
                        .items
                        .as_ptr()
                        .add(rrb.taillen - slice_cnt),

                        new_tail.as_leaf().items.as_ptr(),
                        slice_cnt
                    );
                }

                rrb.taillen = slice_cnt;
                rrb.tail = new_tail;

                return rrb;
            }

            let root = Self::slice_left_rec(
                &mut new_rrb.shift,
                *rrb.root(),
                start,
                rrb.shift,
                false
            );

            new_rrb.cnt = slice_cnt;
            new_rrb.root = as_ptr(root);

            // Ensure last element in size table is correct size,
            // if the root is an internal node.

            if new_rrb.shift != 0 && !new_rrb.root().as_br().szt.is_null() {
                new_rrb.root().as_br().szt_mut()[new_rrb.root().len() - 1]
                    = new_rrb.cnt - rrb.taillen;
            }

            new_rrb.tail = rrb.tail;
            new_rrb.taillen = rrb.taillen;
            rrb = new_rrb;
        }


        if rrb.shift == 0 && !rrb.root.is_null() {

            if rrb.cnt <= NODE_SIZE {
                let new_tail = Leaf::new(rrb.cnt);

                unsafe {
                    // let root_leaf = rrb.root().as_leaf();
                    // let leaf_len = root_leaf.len();
                    // let new_tail_leaf = new_tail.len();

                    copy_nonoverlapping(
                        rrb.root().as_leaf().items.as_ptr(),
                        new_tail.as_leaf().items.as_ptr(),
                        rrb.root().len()
                    );

                    copy_nonoverlapping(
                        rrb.tail.as_leaf().items.as_ptr(),
                        new_tail.as_leaf().items.as_ptr().add(rrb.root().len()),
                        rrb.root().len()
                    );
                }

                rrb.taillen = rrb.cnt;
                rrb.root = null_mut();
                rrb.tail = new_tail;
            }

            else if rrb.cnt - rrb.taillen < NODE_SIZE {

                let tailcut = NODE_SIZE - rrb.root().len();
                let new_root = Leaf::new(NODE_SIZE);
                let new_tail = Leaf::new(rrb.taillen - tailcut);

                unsafe {

                    copy_nonoverlapping(
                        rrb.root().as_leaf().items.as_ptr(),
                        new_root.as_leaf().items.as_ptr(),
                        rrb.root().len()
                    );

                    copy_nonoverlapping(
                        rrb.tail.as_leaf().items.as_ptr(),
                        new_root.as_leaf().items.as_ptr().add(rrb.root().len()),
                        tailcut
                    );

                    copy_nonoverlapping(
                        rrb.tail.as_leaf().items.as_ptr().add(tailcut),
                        new_tail.as_leaf().items.as_ptr(),
                        rrb.taillen - tailcut
                    );

                }

                rrb.taillen -= tailcut;
                rrb.tail = new_tail;
                rrb.root = as_ptr(new_root);
            }
        }

        rrb
    }

    fn slice_left_rec(
        total_shift: &mut usize,
        root: Node<T>,
        start: usize,
        shift: usize,
        has_right: bool
    ) -> Node<T>
    {

        let mut subidx = start >> shift;

        if shift > 0 {
            let subshift = shift.decshf();

            let root_br = root.as_br_mut();
            let mut idx = start;
            if root_br.szt.is_null() {
                idx -= subidx << shift;
            }
            else {
                while root_br.szt()[subidx] <= idx {
                    subidx += 1;
                }
                if subidx != 0 {
                    idx -= root_br.szt()[subidx - 1];
                }
            }

            let last_slot = root_br.len() - 1;
            let child = root_br[subidx];
            let left_hand_node = Self::slice_left_rec(
                    total_shift,
                    child,
                    idx,
                    subshift,
                    subidx != last_slot || has_right
                );

            if subidx == last_slot {
                if has_right {
                    let left_hand_paren = Br::new(1);
                    let left_hand_br = left_hand_node.as_br();
                    left_hand_paren.as_br_mut()[0] = left_hand_node;

                    if subshift != 0 && !left_hand_br.szt.is_null() {
                        let mut sliced_tbl = SZT::new(1);
                        sliced_tbl[0] = left_hand_br.szt()[left_hand_br.len() - 1];
                        left_hand_paren.as_br_mut().szt = as_ptr(sliced_tbl);
                    }

                    *total_shift = shift;
                    return left_hand_paren;
                }
                else {
                    return left_hand_node;
                }
            }
            else {
                let sliced_len = root_br.len() - subidx;
                let sliced_root = Br::new(sliced_len);

                // TODO: Can shrink size here if sliced_len == 2,
                // using the ambidextrous vector technique w. offset.
                // Takes constant time.

                unsafe {
                    copy_nonoverlapping(
                        sliced_root.as_br().children.as_ptr().add(1),
                         root_br.children.as_ptr().add(subidx + 1),
                        sliced_len - 1
                    );
                }

                // TODO: Can check if left is a power of the tree size. If so, all nodes
                // will be completely populated, and we can ignore the size table. Most
                // importantly, this will remove the need to alloc a size table, which
                // increases perf.
                let mut sliced_tbl = SZT::new(sliced_len);

                if root_br.szt.is_null() {
                    for i in 0..sliced_len {
                        // left is total amount sliced off. By adding in subidx, we get faster
                        // computation later on.
                        sliced_tbl[i] = (subidx + 1 + i) << shift;
                        // NOTE: This doesn't really work properly for top root, as last node
                        // may have a higher count than it *actually* has. To remedy for this,
                        // the top function performs a check afterwards, which may insert the
                        // correct value if there's a size table in the root.
                    }
                }
                else {
                    unsafe {
                        copy_nonoverlapping(
                            root_br.szt().tbl.as_ptr().add(subidx),
                            sliced_tbl.tbl.as_ptr(),
                            sliced_len
                        );
                    }
                }

                for i in 0..sliced_len {
                    sliced_tbl[i] -= shift;
                }

                sliced_root.as_br_mut().szt = as_ptr(sliced_tbl);
                sliced_root.as_br_mut()[0] = left_hand_node;
                *total_shift = shift;

                sliced_root
            }

        }
        else {
            let right_vals_len = root.len() - subidx;
            let right_vals = Leaf::new(right_vals_len);

            unsafe {
                copy_nonoverlapping(
                    root.as_leaf().items.as_ptr().add(subidx),
                    right_vals.as_leaf().items.as_ptr(),
                    right_vals_len
                );
            }

            *total_shift = shift;

            right_vals
        }
    }



}


#[allow(unused)]
impl RRBVec<usize> {
    pub fn bfs_display(&self) {
        unsafe {
            let mut lv = 1usize;
            let mut cur_q = VecDeque::new();

            println!();
            println!("MAIN TRIE: {}", self.cnt);
            println!();

            if !self.root.is_null() {
                cur_q.push_back((roadmap![], *self.root()));
            } else {
                println!("null\n");
            }

            while !cur_q.is_empty() {
                println!("############ Level: {} #############", lv);

                let mut nxt_q = VecDeque::new();

                while !cur_q.is_empty() {
                    let (roadmap, x) = cur_q.pop_front().unwrap();

                    // print all x chidren
                    println!();

                    match x {
                        Node::BR(br) => {
                            println!("Br: ({})", roadmap);
                            println!("{}", strshift(&(*br), "  "));
                        }
                        Node::LF(leaf) => {
                            println!("Leaf: ({})", roadmap);
                            println!("{}", strshift(&(*leaf), "  "));
                            println!();
                        }
                    }

                    if let Node::BR(arr) = x {
                        for (i, child) in (*arr).children.iter().enumerate() {
                            if child.is_empty() {
                                break;
                            }

                            nxt_q.push_back((roadmap.ppush(i as i32), *child));
                        }
                    }
                }

                cur_q = nxt_q;
                lv += 1;
            }

            // print tail
            println!("###################################\n");
            println!("TAIL: \n");

            if !self.tail.is_empty() {
                println!("{}", self.tail);
            } else {
                println!("null");
            }

            println!("------------- end --------------");
        }
    }
}


impl<'a, T: Debug + Clone + 'a> Vector<'a, T> for RRBVec<T> {
    fn nth(&self, mut idx: usize) -> &T {
        debug_assert!(self.cnt > idx);

        let tailoff = self.cnt - self.taillen;
        if tailoff <= idx {
            &self.tail.as_leaf()[idx - tailoff]
        } else {
            let mut cur = *self.root();
            for shift in (&self.shift as &dyn NxtShf<usize>).iter() {
                let cur_br = cur.as_br();

                if cur_br.szt.is_null() {
                    cur = cur_br[(idx >> shift) & MASK];
                } else {
                    let is = cur_br.sized_pos(&mut idx, shift);
                    cur = cur_br[is];
                }
            }

            match cur {
                Node::BR(_) => unreachable!(),
                Node::LF(leaf) => unsafe { &(*leaf)[idx & MASK] },
            }
        }
    }

    fn peek(&self) -> Option<&T> {
        if self.is_empty() {
            None
        } else {
            if let Node::LF(leaf) = self.tail {
                unsafe { Some(&(*leaf)[self.taillen - 1]) }
            } else {
                unreachable!()
            }
        }
    }

    fn push(&self, item: T) -> Box<dyn Vector<'a, T> + 'a> {
        Box::new(self.push_(item))
    }

    fn pop(&self) -> Result<Box<dyn Vector<'a, T> + 'a>, Box<dyn Error>> {
        match self.pop_() {
            Ok(it) => Ok(Box::new(it)),
            Err(err) => Err(err),
        }
    }

    fn assoc(&self, idx: usize, item: T) -> Box<dyn Vector<'a, T> + 'a> {
        Box::new(self.assoc_(idx, item))
    }

    fn duplicate(&self) -> Box<dyn Vector<'a, T> + 'a> {
        Box::new(self.clone())
    }

    fn transient(&self) -> Result<Box<dyn Vector<'a, T> + 'a>, ()> {
        todo!()
    }

    fn persistent(&self) -> Result<Box<dyn Vector<'a, T> + 'a>, ()> {
        Err(())
    }
}


impl<T> Collection for RRBVec<T> {
    fn len(&self) -> usize {
        self.cnt
    }
}


impl<T: Debug + Clone> Debug for RRBVec<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for t in self.iter() {
            write!(f, "{:?} ", (*t))?
        }

        Ok(())
    }
}

impl<T> Clone for RRBVec<T> {
    fn clone(&self) -> Self {
        Self {
            cnt: self.cnt.clone(),
            shift: self.shift.clone(),
            taillen: self.taillen.clone(),
            tail: self.tail.clone(),
            root: unsafe{ as_ptr(*self.root) },
        }
    }
}




#[cfg(test)]
mod tests {
    use coll::Itertools;

    use super::RRBVec;
    use crate::{
        {as_ptr, persistent::vector::Vector, Collection},
        roadmap,
        test::{persistent::VectorProvider, *},
    };

    #[test]
    fn test_prrb_vec_randomedata() {
        unsafe { UZProvider {}.test_pvec(|| Box::new(RRBVec::empty())) }
    }

    #[test]
    fn test_prrb_vec_manually() {
        let rrb = RRBVec::empty();

        // let mut bpv = (Box::new(pv) as Box<dyn Vector<usize>>;)
        let mut brrb = rrb;
        brrb = brrb.push_(0usize);
        brrb = brrb.push_(1);

        brrb = brrb.push_(2);
        brrb = brrb.push_(3);

        brrb = brrb.push_(4);
        brrb = brrb.push_(5);

        // let stub = brrb;

        // brrb = brrb.assoc_(1, 100);
        // brrb = brrb.assoc_(2, 200);
        // brrb = brrb.assoc_(3, 300);

        // println!("stub: {:?}", stub);
        // println!("brrb: {:?}", brrb);
        brrb = brrb.push_(6);
        brrb = brrb.push_(7);

        brrb = brrb.push_(8);

        brrb = brrb.push_(9);
        brrb = brrb.push_(10);
        brrb = brrb.push_(11);

        brrb = brrb.push_(12);
        brrb = brrb.push_(13);

        brrb = brrb.push_(14);
        brrb = brrb.push_(15);

        let total = 400;

        for i in 16..total {
            brrb = brrb.push_(i);
        }

        // let mut uvec = brrb.duplicate();
        // let stub = brrb;
        let mut uvec = brrb;

        let mut uelem_vec = vec![];
        for i in 0..total {
            let e = i * 100;
            uelem_vec.push(e);
        }
        for i in 0..total {
            uvec = uvec.assoc_(i, uelem_vec[i]);

            assert_eq!(uvec.nth(i), &uelem_vec[i])
        }

        for i in (0..total).rev() {
            brrb = brrb.pop_().unwrap();

            for j in 0..i {
                assert_eq!(brrb.nth(j), &j);
            }
        }

        // brrb = brrb.pop_().unwrap();


        // brrb.bfs_display();
    }


    #[test]
    fn test_extended_pvec_rrb_randomedata() {

        let rrb = RRBVec::empty();
        let batch_num = 1000;

        // Init data
        let mut rrb = rrb;
        for i in 0..batch_num {
            rrb = rrb.push_(i);
        }

        let slice_step = batch_num / 10;

        for i in (0..batch_num + slice_step).step_by(slice_step) {
        for j in (i..batch_num + slice_step).step_by(slice_step).rev() {
            if j <= i { break; }

            let subrrb = rrb.slice(i, j);

            assert_eq!(subrrb.len(), j - i);

            for k in i..j + 1 {
                assert_eq!(&k, subrrb.nth(k));
            }

        }}

    }

    #[test]
    fn test_extended_pvec_rrb_manually() {

        let mut rrb = RRBVec::empty();

        rrb = rrb.push_(0usize);
        rrb = rrb.push_(1);

        rrb = rrb.push_(2);
        rrb = rrb.push_(3);

        rrb = rrb.push_(4);
        rrb = rrb.push_(5);

        // let stub = brrb;
        // println!("stub: {:?}", stub);
        // println!("brrb: {:?}", brrb);

        // rrb = rrb.push_(6);
        // rrb = rrb.push_(7);

        // rrb = rrb.push_(8);

        // rrb = rrb.push_(9);
        // rrb = rrb.push_(10);
        // rrb = rrb.push_(11);

        // rrb = rrb.push_(12);
        // rrb = rrb.push_(13);

        // rrb = rrb.push_(14);
        // rrb = rrb.push_(15);

        // let total = 400;

        // for i in 16..total {
        //     rrb = rrb.push_(i);
        // }

        // for i in 0..6 {
        //     // let sr_rrb = rrb.slice_right(i);
        //     // assert_eq!(sr_rrb.len(), i);

        //     // for j in 0..i {
        //     //     let s_rrb = sr_rrb.slice_left(j);

        //     //     assert_eq!(s_rrb.len(), i - j);
        //     // }

        //         println!("{}", i);
        //     let sr_rrb = rrb.slice_left(i);
        //     assert_eq!(sr_rrb.len(), 6 - i);

        // }

        // let subrrb = rrb.slice_left(0);
        let subrrb = rrb.slice_left(1);
        let subrrb = rrb.slice_left(2);
        // let subrrb = rrb.slice_left(3);

        println!("subrrb");
        subrrb.bfs_display();

    }

    // #[test]
    // fn test_ttrie_vec_randomedata() {
    //     unsafe { InodeProvider {}.test_tvec(|| Box::new(TTrieVec::empty()) })
    // }
}
