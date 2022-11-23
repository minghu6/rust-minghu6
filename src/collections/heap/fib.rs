//! Fibonacci Heap (decent impl)
//!

use std::{
    borrow::Borrow,
    cell::RefCell,
    cmp::Ordering::*,
    collections::{hash_map::Entry::*, HashMap},
    fmt::{Debug, Display},
    hash::Hash,
    ptr::null_mut,
    rc::{Rc, Weak},
};

use concat_idents::concat_idents;

use crate::{
    attr,
    collections::{AdvHeap, Coll, CollKey, Heap},
    hashmap, mattr,
};


////////////////////////////////////////////////////////////////////////////////
//// Macro


////////////////////////////////////////
//// Node wrapper

macro_rules! boxptr {
    ($v:expr) => {
        Box::into_raw(Box::new($v))
    };
}


macro_rules! unboxptr {
    ($ptr:expr) => {
        unsafe { *Box::from_raw($ptr) }
    };
}


macro_rules! node {
    ($i:expr, $k:expr) => {
        node!($i, $k, 0, false)
    };

    ($i:expr, $k:expr, $rank:expr, $marked:expr) => {
        Node(Some(Rc::new(RefCell::new(Node_ {
            idx: $i,
            key: boxptr!($k),
            rank: $rank,
            left: WeakNode::none(),
            right: Node::none(),
            paren: WeakNode::none(),
            child: Node::none(),
            marked: $marked,
        }))))
    };
}


macro_rules! unwrap_into {
    ($node:expr) => {
        std::rc::Rc::try_unwrap($node.0.unwrap())
            .unwrap()
            .into_inner()
    };
}


////////////////////////////////////////
//// Attr macros

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


macro_rules! def_attr_macro {
    ($($name:ident),+) => {
        $(
            macro_rules! $name {
                ($node:expr) => {
                    attr!($$node, $name)
                }
            }

            concat_idents! (mname = m, $name {
                #[allow(unused)]
                macro_rules! mname {
                    ($node:expr) => {
                        mattr!($$node, $name)
                    }
                }
            });
        )+
    };
}


macro_rules! key {
    ($node:expr) => {
        unsafe { &*attr!($node, key) }
    };
}


def_attr_macro!(left, right, child, paren, idx, rank, marked);



////////////////////////////////////////////////////////////////////////////////
//// Structure


/// [Fibonacci Heap](https://en.wikipedia.org/wiki/Fibonacci_heap)
/// : Indexed Min Heap based on linked list.
///
/// size(x) >= F(d+2)
///
/// I should be cheap to clone
pub struct FibHeap<I: CollKey, T: CollKey> {
    len: usize,
    /// roots count
    rcnt: usize,
    min: Node<I, T>,
    /// index of nodes
    nodes: HashMap<I, Node<I, T>>,
}


struct Node<I, T>(Option<Rc<RefCell<Node_<I, T>>>>);


/// Used for reverse reference to avoid circular-reference
///
/// So we can easy auto drop
struct WeakNode<I, T>(Option<Weak<RefCell<Node_<I, T>>>>);


#[derive(Clone)]
struct Node_<I, T> {
    idx: I,
    key: *mut T,
    rank: usize, // children number

    /// rev ref
    left: WeakNode<I, T>,
    right: Node<I, T>,
    /// rev ref
    paren: WeakNode<I, T>,
    child: Node<I, T>,
    /// Indicate that it has lost a child
    marked: bool,
}



////////////////////////////////////////////////////////////////////////////////
//// Implementation

impl<I: Debug, T: Debug> Debug for Node_<I, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?}[{:?}]{}",
            self.idx,
            unsafe { &*self.key },
            if self.marked { " X" } else { "" }
        )
    }
}


impl<I, T> Node<I, T> {
    fn downgrade(&self) -> WeakNode<I, T> {
        WeakNode(self.0.clone().map(|ref rc| Rc::downgrade(rc)))
    }

    #[allow(unused)]
    fn as_ptr(&self) -> *mut Node_<I, T> {
        match self.0 {
            Some(ref rc) => rc.as_ptr(),
            None => null_mut(),
        }
    }

    fn none() -> Self {
        Self(None)
    }

    fn is_some(&self) -> bool {
        self.0.is_some()
    }

    fn is_none(&self) -> bool {
        self.0.is_none()
    }

    fn replace(&mut self, node: Node<I, T>) -> Self {
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
        let mut child = child!(self);
        let mut res = vec![];

        while child.is_some() {
            res.push(child.clone());
            child = right!(child);
        }

        res
    }

    /// remove paren, left and right
    fn purge_as_root(&self) {
        mparen!(self) = WeakNode::none();
        mleft!(self) = WeakNode::none();
        mright!(self) = Node::none();
    }

    fn cut_child(&self, x: Node<I, T>) {
        if !left!(x).is_none() {
            mright!(left!(x).upgrade()) = right!(x);
        } else {
            debug_assert!(child!(self).rc_eq(&x));
            mchild!(self) = right!(x);
        }

        if !right!(x).is_none() {
            mleft!(right!(x)) = left!(x);
        }

        mrank!(self) = rank!(self) - 1;

        x.purge_as_root();
    }

    /// replace with new val, return old val
    fn replace_key(&self, key: T) -> T {
        let oldk = attr!(self, key);
        let newk = boxptr!(key);
        mattr!(self, key) = newk;

        unboxptr!(oldk)
    }

    #[cfg(test)]
    #[allow(unused)]
    fn validate_ref(&self)
    where
        I: Clone,
    {
        assert!(self.is_some());
        let _self_idx = idx!(self);

        /* validate right sibling */
        let rh = right!(self);

        if rh.is_some() {
            let _rh_idx = idx!(rh);

            let rhlf = left!(rh).upgrade();
            assert!(rhlf.rc_eq(self));
            assert!(rhlf.is_some());

            rh.validate_ref();
        }

        /* validate children */
        let child = child!(self);

        if child.is_some() {
            let _child_idx = idx!(child);

            let cpw = paren!(child);
            assert!(!cpw.is_none());

            let cp = cpw.upgrade();
            assert!(cp.rc_eq(self));
            assert!(cp.is_some());

            child.validate_ref();
        }
    }
}


impl<I, T> Default for Node<I, T> {
    fn default() -> Self {
        Self::none()
    }
}


impl<I, T> Clone for Node<I, T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}


impl<I: Debug, T: Debug> Debug for Node<I, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_none() {
            write!(f, "None")
        } else {
            write!(f, "{:?}", self.0.as_ref().unwrap().as_ref().borrow())
        }
    }
}


impl<I: Debug, T: Debug> Display for Node<I, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "R({:?}) ", self)?;

        let mut curq = vec![(self.clone(), self.children())];
        loop {
            let mut nxtq = vec![];
            for (p, children) in curq {
                if children.is_empty() {
                    break;
                }

                write!(f, "P({:?}) ", p)?;
                let childlen = children.len();
                for (i, child) in children.into_iter().enumerate() {
                    write!(f, "{:?}", child)?;
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
        }

        Ok(())
    }
}


impl<I, T> PartialEq for Node<I, T> {
    fn eq(&self, other: &Self) -> bool {
        self.rc_eq(other)
    }
}


impl<I, T> Eq for Node<I, T> {}


// impl<I, T> Hash for Node<I, T> {
//     fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
//         state.write_usize(self.as_ptr() as usize)
//     }
// }


impl<I, T> WeakNode<I, T> {
    fn upgrade(&self) -> Node<I, T> {
        Node(self.0.clone().map(|weak| weak.upgrade().unwrap()))
    }

    fn none() -> Self {
        Self(None)
    }

    fn is_none(&self) -> bool {
        self.0.is_none()
    }
}


impl<I, T> Clone for WeakNode<I, T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}


impl<I: CollKey + Hash + Clone, T: CollKey> FibHeap<I, T> {
    ////////////////////////////////////////////////////////////////////////////
    //// Public method

    pub fn new() -> Self {
        Self {
            len: 0,
            rcnt: 0,
            min: Node::none(),
            nodes: HashMap::new(),
        }
    }


    /// Same index node would be overidden
    pub fn push(&mut self, i: I, v: T) {
        let node = node!(i.clone(), v);
        self.nodes.insert(i, node.clone());

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
    pub fn pop_item(&mut self) -> Option<(I, T)> {
        if self.min.is_none() {
            return None;
        }

        self.len -= 1;

        /* push children of oldmin into roots */

        for child in self.min.children() {
            self.push_into_roots(child.clone());
        }

        /* update min */

        let newmin = self.roots()[1..]
            .into_iter()
            .min_by_key(|&sib| key!(sib))
            .cloned()
            .unwrap_or_default();

        /* just del old min */

        self.remove_from_roots(self.min.clone());
        let oldmin = self.min.replace(newmin);


        self.consolidate();

        Some((
            self.remove_from_index(&oldmin),
            unboxptr!(unwrap_into!(oldmin).key),
        ))
    }


    /// merge same rank trees recusively
    pub fn consolidate(&mut self) {

        let mut rank: HashMap<usize, Node<I, T>> = hashmap!();

        for mut sib in self.roots() {

            while let Some(x) = rank.remove(&rank!(sib)) {
                sib = self.merge_same_rank_root(x, sib);
            }

            rank.insert(rank!(sib), sib);
        }
    }


    /// Return oldval
    ///
    /// Exec push if the key doesn't exist.
    ///
    pub fn insert(&mut self, i: I, v: T) -> Option<T> {
        match self.nodes.entry(i.clone()) {
            Occupied(ent) => {
                let x = ent.get().clone();
                let oldv = x.replace_key(v);

                match key!(x).cmp(&oldv) {
                    Less => self.decrease_key_(x),
                    Equal => (),
                    Greater => self.increase_key_(x),
                }
                Some(oldv)
            }
            Vacant(_ent) => {
                self.push(i, v);
                None
            }
        }
    }


    pub fn union(&mut self, _other: Self) {
        unimplemented!("link roots, but not O(1) for link index reference")
    }


    pub fn delete<Q: AsRef<I>>(&mut self, _i: Q) -> Option<T> {
        unimplemented!("1. decrease-key to -infi, 2. pop");
    }


    ////////////////////////////////////////////////////////////////////////////
    //// Extra functional method

    /// Return oldval
    ///
    pub fn decrease_key(&mut self, i: I, v: T) -> T {
        let x;
        match self.nodes.entry(i.clone()) {
            Occupied(ent) => {
                x = ent.get().clone();
                let oldv = x.replace_key(v);

                debug_assert!(
                    key!(x) < &oldv,
                    "decrease violated! {:?} !(<) {:?}",
                    key!(x),
                    &oldv
                );

                self.decrease_key_(x);
                oldv
            }
            Vacant(_ent) => {
                unreachable!("Empty index {i:?}")
            }
        }
    }


    pub fn top_item(&self) -> Option<(I, &T)> {
        if self.min.is_some() {
            Some((idx!(self.min), key!(self.min)))
        } else {
            None
        }
    }


    pub fn top(&self) -> Option<&T> {
        self.top_item().map(|x| x.1)
    }


    pub fn pop(&mut self) -> Option<T> {
        self.pop_item().map(|x| x.1)
    }


    pub fn get<Q>(&self, i: &Q) -> Option<&T>
    where
        I: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.nodes.get(i).map(|node| key!(node))
    }


    pub fn indexes(&self) -> impl Iterator<Item=&I> {
        self.nodes.keys()
    }


    ////////////////////////////////////////////////////////////////////////////
    //// Assistant method

    fn decrease_key_(&mut self, x: Node<I, T>) {
        let ent;
        let p = paren!(x);

        if !p.is_none() && key!(x) < key!(p.upgrade()) {
            // 假装x节点本身也是一个符合条件的父节点
            mmarked!(x) = true;
            ent = x.downgrade();
        } else {
            ent = WeakNode::none();
        }

        self.cut_meld_unmark_to_roots(ent);

        if key!(x) < key!(self.min) {
            debug_assert!(paren!(x).is_none());
            self.min = x;
        }
    }


    /// WARNING: O(rank) = O(n)
    fn increase_key_(&mut self, x: Node<I, T>) {
        let ent;
        let mut children_lost = if marked!(x) { 1 } else { 0 };

        for child in x.children() {
            if key!(child) < key!(x) {
                x.cut_child(child.clone());
                self.push_into_roots(child.clone());
                mmarked!(child) = false;

                children_lost += 1;
            }
        }

        match children_lost.cmp(&1) {
            Less => {
                ent = WeakNode::none()
            }
            Equal => {
                mmarked!(x) = true;
                ent = paren!(x);
            },
            Greater => {
                mmarked!(x) = true;
                ent = x.downgrade();
            },
        }

        self.cut_meld_unmark_to_roots(ent);

        // WARNING: O(rank), update self.min
        if x.rc_eq(&self.min) {
            let min_node =
            self.roots().into_iter().min_by_key(|x| key!(x)).unwrap();

            self.min = min_node;
        }

    }


    fn cut_meld_unmark_to_roots(&mut self, ent: WeakNode<I, T>) {
        if ent.is_none() {
            return;
        }

        let mut x = ent.upgrade();
        let mut p = paren!(x);

        while marked!(x) && !p.is_none() {
            let strongp = p.upgrade();

            strongp.cut_child(x.clone());
            self.push_into_roots(x.clone());
            mmarked!(x) = false;

            x = strongp;
            p = paren!(x);
        }

        // 定义上不标记根，但这应该是无所谓的，标记对于可能的pop导致的树规整后的树情况更精确
        mmarked!(x) = true;
    }


    fn remove_from_index(&mut self, x: &Node<I, T>) -> I {
        let k = idx!(x);
        self.nodes.remove(&k);

        k
    }


    /// insert at sib of self.min, with purge
    fn push_into_roots(&mut self, x: Node<I, T>) {
        debug_assert!(!self.min.rc_eq(&x));

        self.rcnt += 1;
        x.purge_as_root();

        if self.min.is_none() {
            self.min = x;
            mleft!(self.min) = self.min.downgrade();
            mright!(self.min) = self.min.clone();
        } else {
            debug_assert!(right!(self.min).is_some());

            mright!(x) = right!(self.min);
            mleft!(x) = self.min.downgrade();

            mright!(self.min) = x.clone();

            mleft!(right!(x)) = x.downgrade();
        }
    }


    /// from self.min go through all roots
    fn roots(&self) -> Vec<Node<I, T>> {
        let mut sibs = vec![];

        if self.min.is_none() {
            return sibs;
        }
        else {
            sibs.push(self.min.clone());
        }

        let mut sib = right!(self.min);

        while !sib.rc_eq(&self.min) {
            sibs.push(sib.clone());
            sib = right!(sib);
        }

        sibs
    }


    fn remove_from_roots(&mut self, x: Node<I, T>) {
        self.rcnt -= 1;

        if self.rcnt > 0 {
            if left!(x).is_none() {
                unreachable!("{:?} left is none", key!(x));
            }
            if right!(x).is_none() {
                unreachable!("{:?} rh is none", key!(x));
            }

            mright!(left!(x).upgrade()) = right!(x);
            mleft!(right!(x)) = left!(x);
        }

        x.purge_as_root();
    }


    /// update self.rcnt
    fn merge_same_rank_root(
        &mut self,
        mut x: Node<I, T>,
        mut y: Node<I, T>,
    ) -> Node<I, T> {
        debug_assert_eq!(rank!(x), rank!(y));

        // let x be parent
        if key!(y) < key!(x) || key!(y) == key!(x) && y.rc_eq(&self.min) {
            (x, y) = (y, x);
        }

        // remove y from roots
        self.remove_from_roots(y.clone());

        // link y to x child
        mright!(y) = child!(x);
        if child!(x).is_some() {
            mleft!(child!(x)) = y.downgrade();
        }

        // link y to x
        mparen!(y) = x.downgrade();
        mchild!(x) = y.clone();
        mrank!(x) = rank!(x) + 1;

        x
    }


    ////////////////////////////////////////////////////////////////////////////
    //// Validation method

    /// Validate nodes are not None or Failed to upgrade to Rc
    #[cfg(test)]
    #[allow(unused)]
    pub(crate) fn validate_ref(&self) {
        if self.is_empty() {
            return;
        }

        /* validate roots */

        for root in self.roots() {
            assert!(root.is_some());

            let rh = right!(root);
            assert!(rh.is_some());

            let wlf = left!(root);
            assert!(!wlf.is_none());
            let left = wlf.upgrade();
            assert!(left.is_some());

            let child = child!(root);
            if child.is_some() {
                child.validate_ref();
            }
        }
    }
}


impl<I: CollKey + Hash + Clone, T: CollKey + Clone> FibHeap<I, T> {
    fn overall_clone(
        &self,
        nodes: &mut HashMap<I, Node<I, T>>,
        x: Node<I, T>,
    ) -> Node<I, T> {
        if x.is_none() {
            return Node::none();
        }

        // overall clone node body
        let newx = node!(idx!(x), key!(x).clone(), rank!(x), marked!(x));
        // update index reference
        nodes.insert(idx!(x), newx.clone());

        // recursive call it
        let mut childen_iter = x.children().into_iter();

        if let Some(child) = childen_iter.next() {
            let newchild = self.overall_clone(nodes, child);

            mchild!(newx) = newchild.clone();
            mparen!(newchild) = newx.downgrade();

            let mut cur = newchild;

            for child in childen_iter {
                let newchild = self.overall_clone(nodes, child);

                mright!(cur) = newchild.clone();
                mleft!(newchild) = cur.downgrade();

                cur = newchild;
            }
        }

        newx
    }
}


impl<I: CollKey, T: CollKey> Drop for FibHeap<I, T> {
    fn drop(&mut self) {
        if self.len > 0 {
            // break circle dependency to enable drop
            let tail = left!(self.min).upgrade();
            mright!(tail) = Node::none();

            self.nodes.clear();
        }
    }
}


impl<T: CollKey, K: CollKey> Display for FibHeap<T, K> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut sib = self.min.clone();

        for i in 1..=self.rcnt {
            writeln!(f, "{}  ({i:03})  {}", "-".repeat(28), "-".repeat(28))?;
            // writeln!(f)?;
            if sib.rc_eq(&self.min) {
                write!(f, "M=>")?;
            }
            writeln!(f, "{}", sib)?;

            debug_assert!(sib.is_some());
            sib = right!(sib);
        }
        writeln!(f, "{}>> end <<{}", "-".repeat(28), "-".repeat(28))?;

        Ok(())
    }
}


impl<I: CollKey + Hash + Clone, T: CollKey + Clone> Clone for FibHeap<I, T> {
    fn clone(&self) -> Self {
        let len = self.len;
        let rcnt = self.rcnt;
        let mut nodes = HashMap::new();
        let min;
        let mut roots_iter = self.roots().into_iter();

        if let Some(_min) = roots_iter.next() {
            min = self.overall_clone(&mut nodes, _min.clone());

            let mut cur = min.clone();

            for root in roots_iter {
                let newroot = self.overall_clone(&mut nodes, root);

                mright!(cur) = newroot.clone();
                mleft!(newroot) = cur.downgrade();

                cur = newroot;
            }

            mright!(cur) = min.clone();
            mleft!(min) = cur.downgrade();
        } else {
            min = Node::none();
        }

        Self {
            len,
            rcnt,
            min,
            nodes,
        }
    }
}


impl<I: CollKey + Hash + Clone, T: CollKey> Coll for FibHeap<I, T> {
    fn len(&self) -> usize {
        self.len
    }
}


impl<I: CollKey + Hash + Clone, T: CollKey> Heap<I, T> for FibHeap<I, T> {
    fn top(&self) -> Option<&T> {
        self.top()
    }

    fn pop(&mut self) -> Option<T> {
        self.pop()
    }

    fn push(&mut self, key: I, val: T) {
        self.push(key, val);
    }
}


impl<I: CollKey + Hash + Clone, T: CollKey> AdvHeap<I, T> for FibHeap<I, T> {
    fn update(&mut self, i: I, val: T) -> Option<T> {
        self.insert(i, val)
    }
}




#[cfg(test)]
mod tests {
    use super::FibHeap;
    use crate::{
        algs::random,
        test::{
            heap::{AdvHeapProvider, HeapProvider},
            normalize, UZProvider,
        },
    };

    #[ignore = "for debug"]
    #[test]
    fn debug_fib_heap() {}

    #[test]
    fn test_fibheap_fixeddata() {
        let mut heap = FibHeap::<usize, usize>::new();

        heap.push(0, 0);
        heap.push(0, 5);
        heap.push(0, 2);
        heap.push(0, 3);
        heap.push(0, 1);

        assert_eq!(heap.pop(), Some(0));
        assert_eq!(heap.pop(), Some(1));
        assert_eq!(heap.pop(), Some(2));
        assert_eq!(heap.pop(), Some(3));
        assert_eq!(heap.pop(), Some(5));

        let mut heap = FibHeap::<usize, usize>::new();
        heap.push(0, 3);
        heap.push(0, 41);
        heap.push(0, 44);
        heap.push(0, 2);

        assert_eq!(heap.pop(), Some(2));
        assert_eq!(heap.pop(), Some(3));
        assert_eq!(heap.pop(), Some(41));
        assert_eq!(heap.pop(), Some(44));


        let raw =
            vec![705, 265, 150, 265, 645, 497, 121, 173, 504, 671, 96, 761];
        let data = normalize(&raw);
        let mut heap = FibHeap::<usize, usize>::new();
        for e in data {
            heap.push(0, e);
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

        let mut heap = FibHeap::<usize, usize>::new();
        let gen = || {
            let mut _inner = 0;
            move || {
                let old = _inner;
                _inner += 1;
                old
            }
        };
        let mut auto = gen();

        let data = normalize(&vec![981, 498, 719, 684, 28, 187]);
        for e in data[1..].iter().cloned() {
            heap.push(auto(), e);
        }

        // println!("{heap}")
    }


    #[test]
    fn test_fibheap_randomdata() {
        let provider = UZProvider {};

        (&provider as &dyn HeapProvider<usize>)
            .test_heap(true, || box FibHeap::new());

        (&provider as &dyn AdvHeapProvider<usize>)
            .test_advheap(true, || box FibHeap::new());
    }

    #[test]
    fn test_fibheap_randomdata_extra() {
        // let batch_num = 10;
        let get_one = || random::<usize>() % 1000;
        let validate = |heap: &FibHeap<i32, usize>, non_dec: bool| {
            let mut heap = (*heap).clone();
            let mut storage = vec![];

            while let Some(e) = heap.pop() {
                storage.push(e);
            }

            if !non_dec {
                storage.reverse();
            }

            // println!("storage: {storage:?}");

            let mut iter = storage.into_iter().enumerate();
            let mut prev = iter.next().unwrap().1;

            for (_i, e) in iter {
                // println!("{i}: {:?}", e);
                assert!(prev <= e, "prev: {prev:?}, e: {e:?}");
                prev = e;
            }
        };

        let non_dec = true;

        for _ in 0..1 {
            // let batch_num = 400;

            let mut heap = FibHeap::<i32, usize>::new();

            // pad 50% of batch
            for i in 0..300 {
                let e = get_one();
                heap.push(i, e); // push
            }

            for _ in 0..100 {
                let newkey = get_one();
                let i = random::<usize>() % heap.len;
                heap.insert(i as i32, newkey.clone());

                validate(&heap, non_dec);
            }
        }
    }
}
