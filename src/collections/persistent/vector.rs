//! Clojure Vector
//!
//! Reference [It](https://hypirion.com/musings/understanding-persistent-vector-pt-1)
//!

use std::{
    fmt::Debug,
    ops::{Index, IndexMut},
    ptr::null_mut,
};

use super::Vector;
use crate::{collections::as_ptr, etc::BitLen};

const BIT_WIDTH: usize = 1;
const NODE_SIZE: usize = 2usize.pow(BIT_WIDTH as u32);
const MASK: usize = NODE_SIZE - 1;

////////////////////////////////////////////////////////////////////////////////
//// Structure

pub struct PVec<T> {
    cnt: usize,
    root: *mut (), // C void*
    tail: *mut DigitTrieLeaf<T>,
}


// #[derive(Copy, Debug)]
// enum DigitTrieNode<T> {
//     Interal(*mut DigitTrieInternal<T>),
//     Leaf(*mut DigitTrieLeaf<T>)
// }


#[derive(Clone, Copy)]
struct DigitTrieInternal {
    br: [*mut (); NODE_SIZE],
}


struct DigitTrieLeaf<T> {
    val: [*mut T; NODE_SIZE],
}



////////////////////////////////////////////////////////////////////////////////
//// Implement

// Impl DigitTrieInternal
impl DigitTrieInternal {
    fn new(br: *mut ()) -> Self {
        let mut new_br = Self::empty();
        new_br[0] = br;

        new_br
    }

    fn empty() -> Self {
        Self {
            br: [null_mut(); NODE_SIZE],
        }
    }

    /// Copy to new node
    fn duplicate(&self) -> *mut Self {
        as_ptr(self.clone())
    }
}

impl Index<usize> for DigitTrieInternal {
    type Output = *mut ();

    fn index(&self, index: usize) -> &Self::Output {
        &self.br[index]
    }
}

impl IndexMut<usize> for DigitTrieInternal {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.br[index]
    }
}


// Impl DigitTrieLeaf
impl<T> DigitTrieLeaf<T> {
    fn empty() -> Self {
        Self {
            val: [null_mut(); NODE_SIZE],
        }
    }

    fn new(item: *mut T) -> Self {
        let mut t = Self::empty();
        t[0] = item;

        t
    }

    /// 0 <= id < nxt_pos
    fn nxt_pos(&self) -> usize {
        for i in (0..NODE_SIZE).rev() {
            if !self.val[i].is_null() {
                return i + 1;
            }
        }

        NODE_SIZE
    }

    fn is_full(&self) -> bool {
        !self.val[NODE_SIZE - 1].is_null()
    }

    fn duplicate(&self) -> *mut DigitTrieLeaf<T> {
        let val = self.val.clone();

        as_ptr(Self { val })
    }

    fn push(&mut self, item: *mut T) -> Result<(), ()> {
        let nxt_pos = self.nxt_pos();
        if nxt_pos == NODE_SIZE {
            Err(())
        } else {
            self.val[nxt_pos] = item;
            Ok(())
        }
    }

    fn array(&self) -> *mut *mut T {
        self.val.as_ptr() as *mut *mut T
    }
}

impl<T> Index<usize> for DigitTrieLeaf<T> {
    type Output = *mut T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.val[index]
    }
}

impl<T> IndexMut<usize> for DigitTrieLeaf<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.val[index]
    }
}



impl<T: Debug> PVec<T> {
    pub fn empty() -> Self {
        PVec {
            cnt: 0,
            root: null_mut(),
            tail: null_mut(),
        }
    }

    fn new(cnt: usize, root: *mut (), tail: *mut DigitTrieLeaf<T>) -> Self {
        Self { cnt, root, tail }
    }

    /// Indicate the trie structure.
    fn shift(&self) -> i32 {
        (self.tail_off() - 1).bit_len() as i32 - BIT_WIDTH as i32
    }

    /// Tail Offset (elements number before tail)
    fn tail_off(&self) -> usize {
        if self.cnt < NODE_SIZE {
            return 0;
        }

        ((self.cnt - 1) >> BIT_WIDTH) << BIT_WIDTH
    }


    /// Ret: (Root, Prev Leaf)
    unsafe fn copy_approximately(
        &self,
        idx: usize,
    ) -> (*mut (), *mut (), usize) {
        debug_assert!(idx < self.cnt);
        debug_assert!(!self.root.is_null());

        // if idx > self.tail_off() {
        //     let root = self.root.clone();
        //     let tail = (*self.tail).duplicate();

        //     return (root, tail, 0);
        // }

        let mut shift = self.shift();

        let mut cur_self = self.root as *mut DigitTrieInternal;

        let new_root = (*cur_self).duplicate();
        let mut cur_new = new_root;
        let mut prev_new = cur_new;  // fake assign
        let mut i = 0;  // fake assign

        while shift > BIT_WIDTH as i32 {
            i = (self.cnt >> shift) & MASK;
            cur_self = (*cur_self)[i] as *mut DigitTrieInternal;

            prev_new = cur_new;
            cur_new = (*cur_self).duplicate();

            (*prev_new)[i] = cur_new as *mut ();

            shift -= BIT_WIDTH as i32;
        }

        (new_root as *mut (), prev_new as *mut (), i)
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
    fn array_for(&self, idx: usize) -> *mut *mut T {
        debug_assert!(idx < self.cnt);

        unsafe {
            if idx >= self.tail_off() {
                return (*self.tail).array();
            }

            let mut shift = self.shift();
            let mut cur = self.root as *mut DigitTrieInternal;

            while shift > 0 {
                let i = (idx >> shift) & MASK;
                cur = (*cur)[i] as *mut DigitTrieInternal;

                shift -= BIT_WIDTH as i32;
            }

            (*(cur as *mut DigitTrieLeaf<T>)).array()
        }
    }
}



impl<'a, T: 'a + Debug> Vector<'a, T> for PVec<T> {
    fn nth(&self, idx: usize) -> *mut T {
        debug_assert!(idx < self.cnt);

        unsafe {
            let arr = self.array_for(idx);

            *arr.add(idx & MASK)
        }
    }

    fn peek(&self) -> *mut T {
        todo!()
    }

    fn push(&self, item: *mut T) -> Box<dyn Vector<'a, T> + 'a> {
        unsafe {
            let cnt = self.cnt + 1;

            if self.tail.is_null() {
                debug_assert!(self.root.is_null());

                let root = null_mut();
                let tail = as_ptr(DigitTrieLeaf::new(item));

                return box PVec::new(cnt, root, tail);
            }

            debug_assert!(!self.tail.is_null());

            if !(*self.tail).is_full() {
                let root = self.root.clone();
                let tail = (*self.tail).duplicate();
                (*tail).push(item).unwrap();

                return box PVec::new(cnt, root, tail);
            }

            let tail = as_ptr(DigitTrieLeaf::new(item));

            if self.root.is_null() {
                let root = self.tail as *mut ();

                return box PVec::new(cnt, root, tail);
            }
            debug_assert!(!self.root.is_null());

            let (root, prev, idx) = self.copy_approximately(self.cnt - 1);

            if root == prev {
                // Root Overflow
                let root2 = as_ptr(DigitTrieInternal::new(root));
                (*root2)[1] = self.tail as *mut ();

                let root = root2 as *mut ();

                return box PVec::new(cnt, root, tail);
            }

            // promote

            let prev = prev as *mut DigitTrieInternal;
            let new_mid_br = as_ptr(DigitTrieInternal::new((*prev)[idx]));
            (*new_mid_br)[1] = self.tail as *mut ();
            (*prev)[idx] = new_mid_br as *mut ();


            box PVec::new(cnt, root, tail)
        }
    }

    fn pop(&self, _item: *mut T) -> Box<dyn Vector<'a, T> + 'a> {
        todo!()
    }

    fn assoc(&self, _idx: usize, _item: *mut T) -> Box<dyn Vector<'a, T> + 'a> {
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
        test::dict::InodeProvider,
    };

    // #[test]
    // fn test_plist_randomedata() {
    //     unsafe {
    //         InodeProvider{}.test_list(|| box PList::nil())
    //     }

    // }


    #[test]
    fn test_pvec_manually() {
        let pv = PVec::empty();

        let mut bpv = (box pv) as Box<dyn Vector<usize>>;
        // for i in 0..3 {
        //     bpv = bpv.push(as_ptr(i));
        // }
        bpv = bpv.push(as_ptr(0));
        bpv = bpv.push(as_ptr(1));
        bpv = bpv.push(as_ptr(2));
        bpv = bpv.push(as_ptr(3));
        bpv = bpv.push(as_ptr(4));
        bpv = bpv.push(as_ptr(5));
        // bpv = bpv.push(as_ptr(6));

        // bpv = bpv.push(as_ptr(3));

        unsafe {
            println!("0: {:?}", (*bpv.nth(0)));
            println!("1: {:?}", (*bpv.nth(1)));
            println!("2: {:?}", (*bpv.nth(2)));
            println!("3: {:?}", (*bpv.nth(3)));
            println!("4: {:?}", (*bpv.nth(4)));
            println!("5: {:?}", (*bpv.nth(5)));
            // println!("6: {:?}", (*bpv.nth(6)));


        }

        // println!("{:?}", bpv);
    }
}
