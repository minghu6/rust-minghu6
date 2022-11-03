//! Fibonacci Heap (decent impl)
//!

use std::{
    cell::RefCell,
    cmp::Ordering::*,
    collections::{hash_map::Entry::*, HashMap},
    fmt::{Debug, Display},
    hash::Hash,
    ptr::null_mut,
    rc::{Rc, Weak},
};

use crate::{
    attr,
    collections::{AdvHeap, Coll, CollKey, Heap},
    hashmap, justinto, mattr,
};


////////////////////////////////////////////////////////////////////////////////
//// Macro

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
    ($k:expr) => {
        node!($k, 0, false)
    };

    ($k:expr, $rank:expr, $marked:expr) => {
        Node(Some(Rc::new(RefCell::new(Node_ {
            key: boxptr!($k),
            rank: $rank,
            lf: WeakNode::none(),
            rh: Node::none(),
            paren: WeakNode::none(),
            child: Node::none(),
            marked: $marked,
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


/// Linked List [Fibonacci Heap](https://en.wikipedia.org/wiki/Fibonacci_heap)
///
/// size(x) >= F(d+2)
///
/// K should be cheap to clone
pub struct FibHeap<I: CollKey, T: CollKey> {
    len: usize,
    /// roots count
    rcnt: usize,
    min: Node<T>,
    /// index of nodes
    nodes: HashMap<I, Node<T>>,
    /// rev index of nodes
    rev: HashMap<Node<T>, I>,
}


struct Node<T>(Option<Rc<RefCell<Node_<T>>>>);


/// Used for reverse reference to avoid circular-reference
///
/// So we can easy auto drop
struct WeakNode<T>(Option<Weak<RefCell<Node_<T>>>>);


#[derive(Clone)]
struct Node_<T> {
    key: *mut T,
    rank: usize, // children number

    /// rev ref
    lf: WeakNode<T>,
    rh: Node<T>,
    /// rev ref
    paren: WeakNode<T>,
    child: Node<T>,
    /// Indicate that it has lost a child
    marked: bool,
}



////////////////////////////////////////////////////////////////////////////////
//// Implementation


impl<T: Debug> Debug for Node_<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", unsafe { &*self.key })
    }
}


impl<T> Node<T> {
    fn downgrade(&self) -> WeakNode<T> {
        WeakNode(self.0.clone().map(|ref rc| Rc::downgrade(rc)))
    }

    fn as_ptr(&self) -> *mut Node_<T> {
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

    /// remove paren, lf and rh
    fn purge_as_root(&self) {
        mattr!(self, paren) = WeakNode::none();
        mattr!(self, lf) = WeakNode::none();
        mattr!(self, rh) = Node::none();
    }

    fn cut_child(&self, x: Node<T>) {
        if attr!(x, lf).is_none() {
            debug_assert!(attr!(self, child).rc_eq(&x));
            mattr!(self, child) = attr!(x, rh);
        } else {
            let x_lf = attr!(x, lf).upgrade();
            mattr!(x_lf, rh) = attr!(x, rh);

            if !attr!(x, rh).is_none() {
                let x_rh = attr!(x, rh);
                mattr!(x_rh, lf) = x_lf.downgrade();
            }
        }

        mattr!(self, rank) = attr!(self, rank) - 1;

        x.purge_as_root();
    }

    /// replace with new val, return old val
    fn replace_key(&self, key: T) -> T {
        let oldk = attr!(self, key);
        let newk = boxptr!(key);
        mattr!(self, key) = newk;

        unboxptr!(oldk)
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
        writeln!(
            f,
            "R({:?}) {}",
            key!(self),
            if attr!(self, marked) { "X" } else { "" }
        )?;

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

impl<T> PartialEq for Node<T> {
    fn eq(&self, other: &Self) -> bool {
        self.rc_eq(other)
    }
}

impl<T> Eq for Node<T> {}

impl<T> Hash for Node<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_usize(self.as_ptr() as usize)
    }
}


impl<T> WeakNode<T> {
    fn upgrade(&self) -> Node<T> {
        Node(self.0.clone().map(|weak| weak.upgrade().unwrap()))
    }

    fn none() -> Self {
        Self(None)
    }

    fn is_none(&self) -> bool {
        self.0.is_none()
    }
}


impl<T> Clone for WeakNode<T> {
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
            rev: HashMap::new(),
        }
    }


    pub fn push(&mut self, i: I, v: T) {
        let node = node!(v);

        self.push_into_roots(node.clone());

        self.nodes.insert(i.clone(), node.clone());
        self.rev.insert(node.clone(), i);


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
            self.remove_from_index(&oldmin);

            return Some(unboxptr!(justinto!(oldmin).key));
        }

        /* push children of oldmin into roots */

        for child in self.min.children() {
            self.push_into_roots(child.clone());
        }

        /* update min */

        let mut newmin = attr!(self.min, rh);

        for sib in &self.roots()[1..] {
            if key!(sib) < key!(newmin) {
                newmin = sib.clone();
            }
        }

        /* just del old min */
        self.remove_from_roots(self.min.clone());

        let oldmin = self.min.replace(newmin);

        self.consolidate();

        self.remove_from_index(&oldmin);
        Some(unboxptr!(justinto!(oldmin).key))
    }


    pub fn consolidate(&mut self) {
        /* merge same rank trees recusively */

        let mut rank: HashMap<usize, Node<T>> = hashmap!();

        for mut sib in self.roots() {
            // println!("scan {:?}", key!(sib));
            // try merge backward
            while let Some(x) = rank.remove(&attr!(sib, rank)) {
                // println!("merge {:?} and {:?}", key!(x), key!(sib));
                sib = self.merge_same_rank_root(x, sib);
                // println!("{}", self);
            }

            rank.insert(attr!(sib, rank), sib);
        }
    }


    /// Return oldval
    ///
    /// Exec push if the key doesn't exist.
    ///
    pub fn update(&mut self, i: I, v: T) -> Option<T> {
        let x;
        match self.nodes.entry(i.clone()) {
            Occupied(ent) => {
                x = ent.get().clone();
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
    //// Assistant method

    fn decrease_key_(&mut self, x: Node<T>) {
        let unmeld_ent;
        let p = attr!(x, paren);

        if !p.is_none() && key!(x) < key!(p.upgrade()) {
            // 假装x节点本身也是一个符合条件的父节点
            mattr!(x, marked) = true;
            unmeld_ent = x.downgrade();
        } else {
            unmeld_ent = WeakNode::none();
        }

        self.unmeld_to_roots(unmeld_ent);

        if key!(x) < key!(self.min) {
            debug_assert!(attr!(x, paren).is_none());
            self.min = x;
        }
    }


    /// WARNING: O(rank)
    fn increase_key_(&mut self, x: Node<T>) {
        let unmeld_ent;
        let mut children_lost = if attr!(x, marked) { 1 } else { 0 };

        for child in x.children() {
            if key!(child) < key!(x) {
                x.cut_child(child.clone());
                self.push_into_roots(child.clone());
                mattr!(child, marked) = false;

                children_lost += 1;
            }
        }

        if children_lost < 1 {
            // unviolated increase
            unmeld_ent = WeakNode::none();
        } else if children_lost == 1 {
            let p = attr!(x, paren);

            mattr!(x, marked) = true;
            unmeld_ent = p;
        } else {
            mattr!(x, marked) = true;
            unmeld_ent = x.downgrade();
        }

        if !unmeld_ent.is_none() {
            self.unmeld_to_roots(unmeld_ent);
        }

        // WARNING: O(rank), update self.min
        let min_node =
            self.roots().into_iter().min_by_key(|x| key!(x)).unwrap();

        self.min = min_node;
    }


    fn unmeld_to_roots(&mut self, ent: WeakNode<T>) {
        if ent.is_none() {
            return;
        }

        let mut x = ent.upgrade();
        let mut p = attr!(x, paren);

        while attr!(x, marked) && !p.is_none() {
            p.upgrade().cut_child(x.clone());
            self.push_into_roots(x.clone());
            mattr!(x, marked) = false;

            x = attr!(x, paren).upgrade();
            p = attr!(x, paren);
        }

        // 定义上不标记根，但这应该是无所谓的，标记对于可能的pop导致的树规整后的树情况更精确
        mattr!(x, marked) = true;
    }


    fn remove_from_index(&mut self, node: &Node<T>) {
        let k = self.rev.remove(node).unwrap();
        self.nodes.remove(&k);
    }


    /// insert at sib of self.min, with purge
    fn push_into_roots(&mut self, node: Node<T>) {
        self.rcnt += 1;
        node.purge_as_root();

        if self.min.is_none() {
            self.min = node;
            mattr!(self.min, lf) = self.min.downgrade();
            mattr!(self.min, rh) = self.min.clone();
        } else {
            mattr!(node, rh) = attr!(self.min, rh);
            mattr!(node, lf) = self.min.downgrade();

            mattr!(self.min, rh) = node.clone();

            mattr!(attr!(node, rh), lf) = node.downgrade();
        }
    }


    /// from self.min go through all roots
    fn roots(&self) -> Vec<Node<T>> {
        let mut sib = self.min.clone();
        let mut sibs = vec![sib.clone()];

        for _ in 1..self.rcnt {
            sib = attr!(sib, rh);
            sibs.push(sib.clone());
        }

        sibs
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

            mattr!(attr!(node, lf).upgrade(), rh) = attr!(node, rh);
            mattr!(attr!(node, rh), lf) = attr!(node, lf);
        }

        mattr!(node, lf) = WeakNode::none();
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
            mattr!(attr!(x, child), lf) = y.downgrade();
        }

        // link y to x
        mattr!(y, paren) = x.downgrade();
        mattr!(x, child) = y.clone();
        mattr!(x, rank) = attr!(x, rank) + 1;

        // println!(
        //     "merge-roots> |tail| x.child {:?} y.child {:?} y.rh {:?} ",
        //     attr!(x, child),
        //     attr!(y, child),
        //     attr!(y, rh)
        // );

        x
    }

}


impl<I: CollKey + Hash + Clone, T: CollKey + Clone> FibHeap<I, T> {
    fn overall_clone(
        &self,
        nodes: &mut HashMap<I, Node<T>>,
        rev: &mut HashMap<Node<T>, I>,
        x: Node<T>,
    ) -> Node<T> {
        if x.is_none() {
            return Node::none();
        }

        // overall clone node body
        let newx = node!(key!(x).clone(), attr!(x, rank), attr!(x, marked));
        // update index reference
        let i = self.rev.get(&x).unwrap();
        nodes.insert(i.clone(), newx.clone());
        rev.insert(newx.clone(), i.clone());

        // recursive call it
        let mut childen_iter = x.children().into_iter();

        if let Some(child) = childen_iter.next() {
            let newchild = self.overall_clone(nodes, rev, child);

            mattr!(newx, child) = newchild.clone();
            mattr!(newchild, paren) = newx.downgrade();

            let mut cur = newchild;

            for child in childen_iter {
                let newchild = self.overall_clone(nodes, rev, child);

                mattr!(cur, rh) = newchild.clone();
                mattr!(newchild, lf) = cur.downgrade();

                cur = newchild;
            }
        }

        newx
    }
}


impl<T: CollKey, K: CollKey> Drop for FibHeap<T, K> {
    fn drop(&mut self) {
        if self.len > 0 {
            // break circle dependency to enable drop
            let tail = attr!(self.min, lf).upgrade();
            mattr!(tail, rh) = Node::none();

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
            sib = attr!(sib, rh);
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
        let mut rev = HashMap::new();
        let min;
        let mut roots_iter = self.roots().into_iter();

        if let Some(_min) = roots_iter.next() {
            min = self.overall_clone(&mut nodes, &mut rev, _min.clone());

            let mut cur = min.clone();

            for root in roots_iter {
                let newroot = self.overall_clone(&mut nodes, &mut rev, root);

                mattr!(cur, rh) = newroot.clone();
                mattr!(newroot, lf) = cur.downgrade();

                cur = newroot;
            }

            mattr!(cur, rh) = min.clone();
            mattr!(min, lf) = cur.downgrade();
        } else {
            min = Node::none();
        }

        Self {
            len,
            rcnt,
            min,
            nodes,
            rev,
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
        if self.min.is_some() {
            Some(unsafe { &*attr!(self.min, key) })
        } else {
            None
        }
    }

    fn pop(&mut self) -> Option<T> {
        self.pop()
    }

    fn push(&mut self, key: I, val: T) {
        self.push(key, val)
    }
}


impl<I: CollKey + Hash + Clone, T: CollKey> AdvHeap<I, T> for FibHeap<I, T> {
    fn update(&mut self, i: I, val: T) -> Option<T> {
        self.update(i, val)
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

        // println!("{}", heap);

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

        // let heap2 = heap.clone();
        // println!("heap2: {}", heap2);

        // println!("update: 1, {}", data[0]);
        heap.update(1, data[0]);

        // println!("heap: {}", heap);
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
        let get_one = || random() % 1000;
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
                // println!("{i}: {e:?}");
                heap.push(i, e); // push
            }

            for _ in 0..100 {
                let newkey = get_one();
                let i = random() % heap.len;
                // println!("update {i} {newkey}");
                heap.update(i as i32, newkey.clone());

                validate(&heap, non_dec);
            }
        }
    }
}
