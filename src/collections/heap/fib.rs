//! Fibonacci Heap
//!

use std::{
    cell::RefCell,
    collections::HashMap,
    fmt::{Debug, Display},
    rc::Rc,
};

use crate::{
    attr,
    collections::{CollKey, Heap},
    hashmap, justinto, mattr,
};


////////////////////////////////////////////////////////////////////////////////
//// Macro

macro_rules! node {
    ($k:expr) => {
        Node(Some(Rc::new(RefCell::new(Node_ {
            key: Box::into_raw(Box::new($k)),
            rank: 0,
            lf: Node::none(),
            rh: Node::none(),
            paren: Node::none(),
            child: Node::none(),
            marked: false,
        }))))
    };
}

macro_rules! key {
    ($node:expr) => {
        unsafe { &*attr!($node, key) }
    };
}


/// Clone attr
macro_rules! attr {
    ($node:expr, $attr:ident) => {{
        let _unr = $node.clone().0.unwrap();
        let _bor = _unr.as_ref().borrow();
        let _attr = _bor.$attr.clone();
        drop(_bor);
        _attr
    }}; // $node.clone().unwrap().as_ref().borrow().$attr};
}

macro_rules! mattr {
    ($node:expr, $attr:ident) => {
        $node.clone().0.unwrap().as_ref().borrow_mut().$attr
    };
}

macro_rules! justinto {
    ($node:expr) => {
        std::rc::Rc::try_unwrap($node.0.unwrap())
            .unwrap()
            .into_inner()
    };
}

////////////////////////////////////////////////////////////////////////////////
//// Structure


/// Linked List Fibonacci Heap
///
/// T shouldb e cheap to clone
pub struct FibHeap<T: CollKey> {
    len: usize,
    rcnt: usize,  // roots count
    min: Node<T>, // also used for head
}

// type Node<T> = Option<Rc<RefCell<Node_<T>>>>;

#[derive(PartialEq)]
struct Node<T>(Option<Rc<RefCell<Node_<T>>>>);


#[derive(PartialEq, Debug, Clone)]
struct Node_<T> {
    key: *mut T,
    rank: usize, // height (alias degree)

    lf: Node<T>,
    rh: Node<T>,
    paren: Node<T>,
    child: Node<T>, // left most node

    marked: bool,
}




////////////////////////////////////////////////////////////////////////////////
//// Implementation

impl<T> Node_<T>
where
    T: CollKey,
{
    fn into_key(self) -> T {
        unsafe { *Box::from_raw(self.key) }
    }
}



impl<T> Node<T> {
    fn none() -> Self {
        Self(None)
    }

    fn is_some(&self) -> bool {
        self.0.is_some()
    }

    fn is_none(&self) -> bool {
        self.0.is_none()
    }

    fn replace(&mut self, node: Node<T>) -> Self {
        let old = Node(self.0.clone());
        self.0 = node.0;
        old
    }

    fn rc_eq(&self, other: &Self) -> bool {
        match self.0 {
            Some(ref rc1) => {
                if let Some(ref rc2) = other.0 {
                    Rc::ptr_eq(rc1, rc2)
                } else {
                    false
                }
            }
            None => other.is_none(),
        }
    }

    fn children(&self) -> Vec<Self> {
        let mut child = attr!(self, child);
        let mut res = vec![];

        while child.is_some() {
            res.push(child.clone());
            child = attr!(child, rh);
        }

        res
    }
}



impl<T> Clone for Node<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}


impl<T: Debug> Debug for Node<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_none() {
            write!(f, "None")
        } else {
            write!(f, "{:?}", key!(self))
        }
    }
}


impl<T: Debug> Display for Node<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "R({:?})", key!(self))?;

        let mut curq = vec![(self.clone(), self.children())];
        loop {
            let mut nxtq = vec![];
            for (p, children) in curq {
                if children.is_empty() {
                    break;
                }

                write!(f, "P({:?}) ", key!(p))?;
                let childlen = children.len();
                for (i, child) in children.into_iter().enumerate() {
                    write!(f, "{:?}", key!(child))?;
                    if i < childlen - 1 {
                        write!(f, ", ")?;
                    }
                    nxtq.push((child.clone(), child.children()));
                }
                write!(f, "; ")?;
            }
            if !nxtq.is_empty() {
                writeln!(f)?;
                curq = nxtq;
            } else {
                break;
            }
            // writeln!(f, "{}", "-".repeat(40))?;
        }

        Ok(())
    }
}


impl<T> FibHeap<T>
where
    T: CollKey + Debug,
{
    pub fn new() -> Self {
        Self {
            len: 0,
            rcnt: 0,
            min: Node::none(),
        }
    }


    pub fn push(&mut self, key: T) {
        let node = node!(key);

        self.push_into_roots(node.clone());

        if key!(node) < key!(self.min) {
            self.min = node;
        }

        self.len += 1;
    }


    /// Amortized cost O(rank(H))
    ///
    /// trees(H') <= rank(H) + 1 # since no two trees have same rank.
    ///
    /// delete-min
    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }

        self.len -= 1;

        if self.len == 0 {
            let oldmin = self.min.clone();
            self.min = Node::none();

            self.remove_from_roots(oldmin.clone());

            return Some(justinto!(oldmin).into_key());
        }

        /* push children of oldmin into roots */

        let mut child = attr!(self.min, child);
        while child.is_some() {
            let nxtchild = attr!(child, rh);
            self.push_into_roots(child.clone());
            child = nxtchild;
        }

        /* just del min */

        let mut sib = attr!(self.min, rh);
        self.remove_from_roots(self.min.clone());

        /* update min */

        let mut newmin = sib.clone();

        for _ in 1..self.rcnt {
            sib = attr!(sib, rh);

            if key!(sib) < key!(newmin) {
                newmin = sib.clone();
            }
        }

        let oldmin = self.min.replace(newmin);

        /* merge same rank trees recusively */

        let mut rank: HashMap<usize, Node<T>> = hashmap!();
        let mut sib = self.min.clone();

        for _ in 0..self.rcnt {
            let nxtsib = attr!(sib, rh);
            // println!("scan {:?}", key!(sib));
            // try merge backward
            while let Some(x) = rank.remove(&attr!(sib, rank)) {
                // println!("merge {:?} and {:?}", key!(x), key!(sib));
                sib = self.merge_same_rank_root(x, sib);
                // println!("{}", self);
            }

            rank.insert(attr!(sib, rank), sib);
            sib = nxtsib;
        }

        Some(justinto!(oldmin).into_key())
    }


    /// push at sib of self.min
    fn push_into_roots(&mut self, node: Node<T>) {
        self.rcnt += 1;
        mattr!(node, paren) = Node::none();

        if self.min.is_none() {
            self.min = node;
            mattr!(self.min, lf) = self.min.clone();
            mattr!(self.min, rh) = self.min.clone();
        } else {
            mattr!(node, rh) = attr!(self.min, rh);
            mattr!(node, lf) = self.min.clone();

            mattr!(self.min, rh) = node.clone();

            mattr!(attr!(node, rh), lf) = node.clone();
        }
    }


    fn remove_from_roots(&mut self, node: Node<T>) {
        self.rcnt -= 1;

        if self.rcnt > 0 {
            if attr!(node, lf).is_none() {
                unreachable!("{:?} lf is none", key!(node));
            }
            if attr!(node, rh).is_none() {
                unreachable!("{:?} rh is none", key!(node));
            }

            mattr!(attr!(node, lf), rh) = attr!(node, rh);
            mattr!(attr!(node, rh), lf) = attr!(node, lf);
        }

        mattr!(node, lf) = Node::none();
        mattr!(node, rh) = Node::none();
    }


    /// update self.rcnt
    fn merge_same_rank_root(
        &mut self,
        mut x: Node<T>,
        mut y: Node<T>,
    ) -> Node<T> {
        debug_assert_eq!(attr!(x, rank), attr!(y, rank));

        // let x be parent
        if key!(y) < key!(x) || key!(y) == key!(x) && y.rc_eq(&self.min) {
            (x, y) = (y, x);
        }

        // remove y from roots
        self.remove_from_roots(y.clone());

        // link y to x child
        mattr!(y, rh) = attr!(x, child);
        if attr!(x, child).is_some() {
            mattr!(attr!(x, child), lf) = y.clone();
            // mattr!(attr!(x, child), paren) = Node::none();
        }

        // link y to x
        mattr!(y, paren) = x.clone();
        mattr!(x, child) = y.clone();
        mattr!(x, rank) = attr!(x, rank) + 1; // same rank

        // println!(
        //     "merge-roots> |tail| x.child {:?} y.child {:?} y.rh {:?} ",
        //     attr!(x, child),
        //     attr!(y, child),
        //     attr!(y, rh)
        // );

        x
    }


    #[allow(unused)]
    #[cfg(test)]
    fn check_roots_connective(&self) {}
}


impl<T: CollKey> Heap<T> for FibHeap<T> {
    fn top(&self) -> Option<&T> {
        if self.min.is_some() {
            Some(unsafe { &*attr!(self.min, key) })
        } else {
            None
        }
    }

    fn pop(&mut self) -> Option<T> {
        self.pop()
    }

    fn push(&mut self, val: T) {
        self.push(val)
    }
}


impl<T: CollKey> Drop for FibHeap<T> {
    fn drop(&mut self) {
        // for _ in 0..self.len {
        //     self.pop();
        // }
    }
}


impl<T: CollKey + Debug> Display for FibHeap<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut sib = self.min.clone();

        for i in 1..=self.rcnt {
            writeln!(f, "{}  ({i:03})  {}", "-".repeat(28), "-".repeat(28))?;
            // writeln!(f)?;
            if sib.rc_eq(&self.min) {
                write!(f, "M=>")?;
            }
            writeln!(f, "{}", sib)?;
            sib = attr!(sib, rh);
        }
        writeln!(f, "{}>> end <<{}", "-".repeat(28), "-".repeat(28))?;

        Ok(())
    }
}



#[cfg(test)]
mod tests {
    use super::FibHeap;
    use crate::{
        algs::random,
        test::{
            heap::{HeapProvider, UnionBinHeap},
            normalize, UZProvider,
        },
        collections::Heap
    };

    #[ignore = "for debug"]
    #[test]
    fn debug_fib_heap() {
        let batch_num = 10;
        let get_one = || random() % 1000;

        let mut seq = vec![];
        let mut rems = 0;

        // pad 25% of batch
        for _ in 0..batch_num / 4 {
            seq.push(true); // push
            rems += 1;
        }
        // random push or pop until rem been ran out of
        for _ in 0..(3 * batch_num) / 4 {
            if random() % 2 == 0 {
                seq.push(true);
                rems += 1;
            } else {
                seq.push(false);
                rems -= 1;
            }

            if rems == 0 {
                break;
            }
        }

        let mut refheap = UnionBinHeap::new(true);
        let mut testheap = FibHeap::new();

        for flag in seq {
            if flag {
                let e = get_one();
                refheap.push(e.clone());
                testheap.push(e);
            } else {
                let target = refheap.pop();
                assert_eq!(testheap.pop(), target);
            }
        }
    }

    #[test]
    fn test_fibheap_fixeddata() {
        let mut heap = FibHeap::new();

        heap.push(0);
        heap.push(5);
        heap.push(2);
        heap.push(3);
        heap.push(1);

        assert_eq!(heap.pop(), Some(0));
        assert_eq!(heap.pop(), Some(1));
        assert_eq!(heap.pop(), Some(2));
        assert_eq!(heap.pop(), Some(3));
        assert_eq!(heap.pop(), Some(5));

        let mut heap = FibHeap::new();
        heap.push(3);
        heap.push(41);
        heap.push(44);
        heap.push(2);

        assert_eq!(heap.pop(), Some(2));
        assert_eq!(heap.pop(), Some(3));
        assert_eq!(heap.pop(), Some(41));
        assert_eq!(heap.pop(), Some(44));

        // println!("{}", heap);

        let raw =
            vec![705, 265, 150, 265, 645, 497, 121, 173, 504, 671, 96, 761];
        let data = normalize(&raw);
        let mut heap = FibHeap::new();
        for e in data {
            heap.push(e);
        }

        assert_eq!(heap.pop(), Some(1));
        assert_eq!(heap.pop(), Some(2));
        assert_eq!(heap.pop(), Some(3));
        assert_eq!(heap.pop(), Some(4));
        assert_eq!(heap.pop(), Some(5));
        assert_eq!(heap.pop(), Some(5));
        assert_eq!(heap.pop(), Some(6));
        assert_eq!(heap.pop(), Some(7));
        assert_eq!(heap.pop(), Some(8));
        assert_eq!(heap.pop(), Some(9));
        assert_eq!(heap.pop(), Some(10));
        assert_eq!(heap.pop(), Some(11));

        println!("{}", heap);
    }

    #[test]
    fn test_fibheap_randomdata() {
        let provider = UZProvider {};

        (&provider as &dyn HeapProvider<usize>)
            .test_heap(true, || box FibHeap::new());
    }
}
