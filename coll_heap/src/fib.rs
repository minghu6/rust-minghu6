//! Fibonacci Heap (decent impl)
//!

use std::{
    borrow::Borrow,
    cmp::Ordering::*,
    collections::{hash_map::Entry::*, HashMap},
    fmt::{Debug, Display},
    hash::Hash, mem::replace,
};

use common::hashmap;

use coll::*;


////////////////////////////////////////////////////////////////////////////////
//// Macros

def_attr_macro!(clone|
    left, right, child, paren, rank, marked, idx
);

def_attr_macro!(ref|
    (val, T)
);

////////////////////////////////////////
//// Node wrapper

macro_rules! node {
    ($i:expr, $k:expr) => {
        node!($i, $k, 0, false)
    };

    ($i:expr, $k:expr, $rank:expr, $marked:expr) => {{
        aux_node!({
            idx: $i,
            val: $k,
            rank: $rank,
            left: WeakNode::none(),
            right: Node::none(),
            paren: WeakNode::none(),
            child: Node::none(),
            marked: $marked
        })
    }};
}

////////////////////////////////////////////////////////////////////////////////
//// Structures


/// [Fibonacci Heap](https://en.wikipedia.org/wiki/Fibonacci_heap)
/// : Indexed Min Heap based on linked list.
///
/// size(x) >= F(d+2)
///
/// I should be cheap to clone
pub struct FibHeap<I, T> {
    len: usize,
    /// roots count
    rcnt: usize,
    min: Node<I, T>,
    /// index of nodes
    nodes: HashMap<I, Node<I, T>>,
}


#[derive(Clone)]
struct Node_<I, T> {
    idx: I,
    val: T,
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
//// Implementations

impl<I: Debug, T: Debug> Debug for Node_<I, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?}[{:?}]{}",
            self.idx,
            self.val,
            if self.marked { " X" } else { "" }
        )
    }
}


impl_node!();


impl<I, T> Node<I, T> {
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
        paren!(self, WeakNode::none());
        left!(self, WeakNode::none());
        right!(self, Node::none());
    }

    fn cut_child(&self, x: Node<I, T>) {
        if !left!(x).is_none() {
            right!(left!(x).upgrade(), right!(x));
        } else {
            debug_assert!(child!(self).rc_eq(&x));
            child!(self, right!(x));
        }

        if !right!(x).is_none() {
            left!(right!(x), left!(x));
        }

        rank!(self, rank!(self) - 1);

        x.purge_as_root();
    }

    /// replace with new val, return old val
    fn replace_key(&self, val: T) -> T
    where
        I: Debug,
        T: Debug
    {
        replace(val_mut!(self), val)
    }

    fn replace(&mut self, x: Self) -> Self {
        let old = Self(self.0.clone());
        self.0 = x.0;
        old
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



impl<I, T> FibHeap<I, T>
where
    I: Eq + Hash + Clone + Debug,
    T: Ord + Debug
{
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

    pub fn len(&self) -> usize {
        self.len
    }

    /// Same index node would be overidden
    pub fn push(&mut self, i: I, v: T)

    {
        let node = node!(i.clone(), v);
        self.nodes.insert(i, node.clone());

        self.push_into_roots(node.clone());

        if val!(node) < val!(self.min) {
            self.min = node;
        }

        self.len += 1;
    }


    /// Amortized cost O(rank(H))
    ///
    /// trees(H') <= rank(H) + 1 # since no two trees have same rank.
    ///
    /// delete-min
    pub fn pop_item(&mut self) -> Option<(I, T)>
    {
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
            .min_by_key(|&sib| val!(sib))
            .cloned()
            .unwrap_or_default();

        /* just del old min */

        self.remove_from_roots(self.min.clone());
        let oldmin = self.min.replace(newmin);


        self.consolidate();

        Some((
            self.remove_from_index(&oldmin),
            unwrap_into!(oldmin).val
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


    /// Return oldval, alias of ReplaceOrPush
    ///
    /// Exec push if the val doesn't exist.
    ///
    pub fn insert(&mut self, i: I, v: T) -> Option<T>
    where
        I: Eq + Hash + Clone,
        T: Ord + Debug
    {
        match self.nodes.entry(i.clone()) {
            Occupied(ent) => {
                let x = ent.get().clone();
                let oldv = x.replace_key(v);

                match val!(x).cmp(&oldv) {
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
        unimplemented!("1. decrease-val to -infi, 2. pop");
    }


    ////////////////////////////////////////////////////////////////////////////
    //// Extra functional method

    /// Return oldval
    ///
    pub fn decrease_key(&mut self, i: I, v: T) -> Option<T>
    where
        I: Eq + Hash + Clone,
        T: Debug
    {
        let x;
        match self.nodes.entry(i.clone()) {
            Occupied(ent) => {
                x = ent.get().clone();
                let oldv = x.replace_key(v);

                self.decrease_key_(x);
                Some(oldv)
            }
            Vacant(_ent) => None,
        }
    }


    pub fn top_item(&self) -> Option<(I, &T)>
    where
        I: Eq + Clone
    {
        if self.min.is_some() {
            Some((idx!(self.min), val!(self.min)))
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
        Q: Ord + Hash + ?Sized,
    {
        self.nodes.get(i).map(|node| val!(node))
    }


    pub fn indexes(&self) -> impl Iterator<Item = &I> {
        self.nodes.keys()
    }


    ////////////////////////////////////////////////////////////////////////////
    //// Assistant method

    fn decrease_key_(&mut self, x: Node<I, T>) {
        let ent;
        let p = paren!(x);

        if !p.is_none() && val!(x) < val!(p.upgrade()) {
            // 假装x节点本身也是一个符合条件的父节点
            marked!(x, true);
            ent = x.downgrade();
        } else {
            ent = WeakNode::none();
        }

        self.cut_meld_unmark_to_roots(ent);

        if val!(x) < val!(self.min) {
            debug_assert!(paren!(x).is_none());
            self.min = x;
        }
    }


    /// WARNING: O(rank) = O(n)
    fn increase_key_(&mut self, x: Node<I, T>) {
        let ent;
        let mut children_lost = if marked!(x) { 1 } else { 0 };

        for child in x.children() {
            if val!(child) < val!(x) {
                x.cut_child(child.clone());
                self.push_into_roots(child.clone());
                marked!(child, false);

                children_lost += 1;
            }
        }

        match children_lost.cmp(&1) {
            Less => ent = WeakNode::none(),
            Equal => {
                marked!(x, true);
                ent = paren!(x);
            }
            Greater => {
                marked!(x, true);
                ent = x.downgrade();
            }
        }

        self.cut_meld_unmark_to_roots(ent);

        // WARNING: O(rank), update self.min
        if x.rc_eq(&self.min) {
            let min_node =
                self.roots().into_iter().min_by_key(|x| val!(x)).unwrap();

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
            marked!(x, false);

            x = strongp;
            p = paren!(x);
        }

        // 定义上不标记根，但这应该是无所谓的，标记对于可能的pop导致的树规整后的树情况更精确
        marked!(x, true);
    }


    fn remove_from_index(&mut self, x: &Node<I, T>) -> I
    where
        I: Eq + Hash + Clone
    {
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
            left!(self.min, self.min.downgrade());
            right!(self.min, self.min.clone());
        } else {
            debug_assert!(right!(self.min).is_some());

            right!(x, right!(self.min));
            left!(x, self.min.downgrade());

            right!(self.min, x.clone());

            left!(right!(x), x.downgrade());
        }
    }


    /// from self.min go through all roots
    fn roots(&self) -> Vec<Node<I, T>> {
        let mut sibs = vec![];

        if self.min.is_none() {
            return sibs;
        } else {
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
            right!(left!(x).upgrade(), right!(x));
            left!(right!(x), left!(x));
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
        if val!(y) < val!(x) || val!(y) == val!(x) && y.rc_eq(&self.min) {
            (x, y) = (y, x);
        }

        // remove y from roots
        self.remove_from_roots(y.clone());

        // link y to x child
        right!(y, child!(x));
        if child!(x).is_some() {
            left!(child!(x), y.downgrade());
        }

        // link y to x
        paren!(y, x.downgrade());
        child!(x, y.clone());
        rank!(x, rank!(x) + 1);

        x
    }


    ////////////////////////////////////////////////////////////////////////////
    //// Validation method

    /// Validate nodes are not None or Failed to upgrade to Rc
    #[cfg(test)]
    #[allow(unused)]
    pub(crate) fn validate_ref(&self) {
        if self.len() == 0 {
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


impl<I: Eq + Hash + Clone, T: Clone> FibHeap<I, T> {
    fn overall_clone(
        &self,
        nodes: &mut HashMap<I, Node<I, T>>,
        x: Node<I, T>,
    ) -> Node<I, T> {
        if x.is_none() {
            return Node::none();
        }

        // overall clone node body
        let newx = node!(idx!(x), val!(x).clone(), rank!(x), marked!(x));
        // update index reference
        nodes.insert(idx!(x), newx.clone());

        // recursive call it
        let mut childen_iter = x.children().into_iter();

        if let Some(child) = childen_iter.next() {
            let newchild = self.overall_clone(nodes, child);

            child!(newx, newchild.clone());
            paren!(newchild, newx.downgrade());

            let mut cur = newchild;

            for child in childen_iter {
                let newchild = self.overall_clone(nodes, child);

                right!(cur, newchild.clone());
                left!(newchild, cur.downgrade());

                cur = newchild;
            }
        }

        newx
    }
}


impl<I, T> Drop for FibHeap<I, T> {
    fn drop(&mut self) {
        if self.len > 0 {
            // break circle dependency to enable drop
            let tail = left!(self.min).upgrade();
            right!(tail, Node::none());

            self.nodes.clear();
        }
    }
}


impl<T: Debug, K: Debug> Display for FibHeap<T, K> {
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


impl<I: Ord + Hash + Clone + Debug, T: Ord + Clone + Debug> Clone for FibHeap<I, T> {
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

                right!(cur, newroot.clone());
                left!(newroot, cur.downgrade());

                cur = newroot;
            }

            right!(cur, min.clone());
            left!(min, cur.downgrade());
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





#[cfg(test)]
mod tests {
    use super::{ FibHeap, super::* };
    use common::random;


    #[ignore = "for debug"]
    #[test]
    fn debug_fib_heap() {}

    #[test]
    fn test_fibheap_fixeddata() {
        let mut heap = FibHeap::<usize, usize>::new();
        let mut auto = common::generate();

        heap.insert(auto(), 2);
        heap.insert(auto(), 4);
        heap.insert(auto(), 1);

        assert_eq!(heap.pop().unwrap(), 1);
        assert_eq!(heap.pop().unwrap(), 2);
        assert_eq!(heap.pop().unwrap(), 4);
        assert_eq!(heap.pop(), None);
    }


    #[test]
    fn test_fibheap_randomdata() {
        test_heap!(FibHeap::new(), MIN);
        test_heap_update!(FibHeap::new(), MIN);
    }

    #[test]
    fn test_fibheap_randomdata_extra() {
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

            let mut iter = storage.into_iter().enumerate();
            let mut prev = iter.next().unwrap().1;

            for (_i, e) in iter {
                assert!(prev <= e, "prev: {prev:?}, e: {e:?}");
                prev = e;
            }
        };

        let non_dec = true;

        for _ in 0..1 {
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
