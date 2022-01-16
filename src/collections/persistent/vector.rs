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
    ptr::null_mut,
};

use itertools::Itertools;

use super::Vector;
use crate::{
    array,
    collections::{as_ptr, bare::array::Array},
    etc::BitLen,
};

const BIT_WIDTH: usize = 1;
const NODE_SIZE: usize = 2usize.pow(BIT_WIDTH as u32);
const MASK: usize = NODE_SIZE - 1;

////////////////////////////////////////////////////////////////////////////////
//// Structure

pub struct PVec<T> {
    cnt: usize,
    root: *mut Node<T>, // C void*
    tail: *mut Node<T>,
}


enum Node<T> {
    BR(Array<*mut Node<T>>),
    LEAF(Array<*mut T>),
}


////////////////////////////////////////////////////////////////////////////////
//// Implement

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

    // fn duplicate(&self) -> *mut Self {
    //     as_ptr(self.clone())
    // }

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
                    match arr.len() {
                        0 => writeln!(f, "[]"),
                        1 => writeln!(f, "[{}]", *arr[0]),
                        2 => writeln!(f, "[{}, {}]", *arr[0], *arr[1]),
                        3 => writeln!(
                            f,
                            "[{}, {}, {}]",
                            *arr[0], *arr[1], *arr[2]
                        ),
                        upper => writeln!(
                            f,
                            "[{}, {}, ... {}]",
                            *arr[0],
                            *arr[1],
                            *arr[upper - 1]
                        ),
                    }
                }
            }
        }
    }
}



impl<T: Debug> PVec<T> {
    pub fn empty() -> Self {
        PVec {
            cnt: 0,
            root: Node::new_br(0),
            tail: Node::new_leaf(0),
        }
    }

    fn new(cnt: usize, root: *mut Node<T>, tail: *mut Node<T>) -> Self {
        Self { cnt, root, tail }
    }

    /// Tail Offset (elements number before tail)
    fn tail_off(&self) -> usize {
        if self.cnt == 0 {
            return 0;
        }

        ((self.cnt - 1) >> BIT_WIDTH) << BIT_WIDTH
    }

    /// Indicate the trie structure.
    ///
    /// It's compute based on tailoff.
    fn shift(&self) -> i32 {
        let tailoff = self.tail_off();

        if tailoff == 0 {
            return -(BIT_WIDTH as i32);
        }

        (tailoff - 1).bit_len() as i32 - BIT_WIDTH as i32
    }

    /// Leaf should be same level
    #[allow(unused)]
    fn height(&self) -> usize {
        (self.tail_off() - 1).bit_len() / BIT_WIDTH
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
            if idx >= self.tail_off() {
                return self.tail;
            }

            let mut shift = self.shift();
            let mut cur = self.root;

            while shift > 0 {
                let i = (idx >> shift) & MASK;
                cur = (*cur).as_br()[i];

                shift -= BIT_WIDTH as i32;
            }

            cur
        }
    }

    unsafe fn push_(&self, item: *mut T) -> Self {
        let cnt = self.cnt + 1;

        // tail isn't full
        if self.cnt - self.tail_off() < NODE_SIZE {
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
        //
        // Full Trie Nodes Number = 1 << ((H(trie) * BIT_WIDTH) - 1)
        // H(trie) = len_b(tailoff(cnt)) / BIT_WIDTH
        // shift = len_b(tailoff(cnt)) - BIT_WIDTH
        //
        // Full Trie Nodes Number = 1 << (shift + BIT_WIDTH)
        //
        // So: tailoff == 1 << ((shift + BIT_WIDTH) - 1)
        //              == 1 << (len_b(tailoff(cnt)) - 1)
        let tailoff = self.tail_off();
        if tailoff == 0 {
            root = self.tail;

            return Self::new(cnt, root, tail);
        }

        let leaf = self.tail;

        if tailoff == 1 << (tailoff.bit_len() - 1) {
            root = Node::new_br(2);
            (*root).as_br_mut()[0] = self.root;
            (*root).as_br_mut()[1] = Self::new_path(self.shift(), leaf);
        } else {
            root = self.push_leaf(self.shift(), self.root, leaf)
        }

        Self::new(cnt, root, tail)
    }

    fn new_path(shift: i32, node: *mut Node<T>) -> *mut Node<T> {
        if shift == 0 {
            return node;
        }

        let ret = Node::new_br(1);

        unsafe {
            (*ret).as_br_mut()[0] =
                Self::new_path(shift - BIT_WIDTH as i32, node);
        }

        ret
    }

    // push the leaf(tail) into the Trie
    unsafe fn push_leaf(
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
                self.push_leaf(shift - BIT_WIDTH as i32, child, leaf)
            } else {
                Self::new_path(shift - BIT_WIDTH as i32, leaf)
            };
        }

        // match *node_to_insert {
        //     Node::BR(_) => {
        //         2+4;
        //     },
        //     Node::LEAF(_) => {
        //         1+1;
        //     },
        // }

        (*ret).as_br_mut()[sub_idx] = node_to_insert;

        ret
    }

    pub fn bfs_display(&self)
    where
        T: Display,
    {
        unsafe {
            let mut lv = 1usize;
            let mut cur_q = VecDeque::new();
            cur_q.push_back(self.root);

            println!();
            println!("MAIN TRIE:");
            println!();
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
            println!("{:?}", (*self.tail));

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

    fn peek(&self) -> *mut T {
        todo!()
    }

    fn push(&self, item: *mut T) -> Box<dyn Vector<'a, T> + 'a> {
        unsafe { box self.push_(item) }
    }

    fn pop(&self, _item: *mut T) -> Box<dyn Vector<'a, T> + 'a> {
        todo!()
    }

    fn assoc(
        &self,
        _idx: usize,
        _item: *mut T,
    ) -> Box<dyn Vector<'a, T> + 'a> {
        // search_approximately

        todo!()
    }

    fn duplicate(&self) -> Box<dyn Vector<'a, T> + 'a> {
        todo!()
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



#[cfg(test)]
mod tests {
    use super::PVec;
    use crate::{
        collections::{as_ptr, persistent::Vector},
        test::{dict::InodeProvider, persistent::VectorProvider},
    };

    #[test]
    fn test_pvec_randomedata() {
        unsafe { InodeProvider {}.test_vec(|| box PVec::empty()) }
    }


    #[test]
    fn test_pvec_manually() {
        let pv = PVec::empty();

        let mut bpv = (box pv) as Box<dyn Vector<usize>>;
        // let mut bpv = pv;

        unsafe {
            // fst
            bpv = bpv.push(as_ptr(0));
            bpv = bpv.push(as_ptr(1));

            // tail full
            bpv = bpv.push(as_ptr(2));
            bpv = bpv.push(as_ptr(3));

            // root overflow
            bpv = bpv.push(as_ptr(4));
            bpv = bpv.push(as_ptr(5));
            bpv = bpv.push(as_ptr(6));

            // root re-overflow
            bpv = bpv.push(as_ptr(7));
            bpv = bpv.push(as_ptr(8));
            bpv = bpv.push(as_ptr(9));
            bpv = bpv.push(as_ptr(10));

            bpv = bpv.push(as_ptr(11));


            println!("0: {:?}", (*bpv.nth(0)));
            println!("1: {:?}", (*bpv.nth(1)));
            println!("2: {:?}", (*bpv.nth(2)));
            println!("3: {:?}", (*bpv.nth(3)));
            println!("4: {:?}", (*bpv.nth(4)));
            println!("5: {:?}", (*bpv.nth(5)));
            println!("6: {:?}", (*bpv.nth(6)));
            println!("7: {:?}", (*bpv.nth(7)));
            println!("8: {:?}", (*bpv.nth(8)));
            println!("9: {:?}", (*bpv.nth(9)));
            println!("10: {:?}", (*bpv.nth(10)));


            for i in 11..35 {
                bpv = bpv.push(as_ptr(i));
            }

            // for i in 0..15 {
            //     bpv.nth(i);
            // }

            // bpv = bpv.push_(as_ptr(12));

            // println!("{:?}", bpv);

            // bpv.bfs_display();
        }
    }


    // #[test]
    // fn hack_pvec() {

    //     let batch_num = 1000;
    //     let mut vec= (box PVec::empty()) as Box<dyn Vector<usize>>;

    //     let mut plain_elem_vec = vec![];
    //     for i in 0..batch_num {
    //         let e = as_ptr(i);
    //         plain_elem_vec.push(e);
    //     }

    //     for i in 0..batch_num {
    //         vec = vec.push(plain_elem_vec[i]);

    //         for j in 0..i+1 {
    //             assert_eq!(vec.nth(j), plain_elem_vec[j]);
    //         }
    //     }

    // }

}
