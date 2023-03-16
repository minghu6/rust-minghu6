//! B+ Tree
//!


use std::{
    borrow::Borrow,
    fmt::*,
    ops::{Bound::*, RangeBounds},
};

use coll::{def_coll_init, node as aux_node, KVEntry, *};

use crate::{
    bst::{Dir, Left, Right},
    bt::*,
};


impl_node!();
impl_tree!(
    /// B+ Trees
    ///
    BPT {
        cnt: usize,
        min_node: WeakNode<K, V>,
        max_node: WeakNode<K, V>
    }
);


////////////////////////////////////////////////////////////////////////////////
//// Macro

macro_rules! node {
    (kv| $k:expr, $v:expr) => {{
        // overflow is M
        let mut entries = Vec::with_capacity(M);
        entries.push(KVEntry($k, $v));

        node!(basic-leaf| entries, WeakNode::none(), WeakNode::none())
    }};
    (basic-leaf| $entries:expr, $succ:expr, $paren:expr) => {{
        aux_node!(FREE-ENUM Leaf {
            entries: $entries,
            succ: $succ,
            paren: $paren
        })
    }};
    (basic-internal| $keys:expr, $children:expr, $paren:expr) => {{
        aux_node!(FREE-ENUM Internal {
            keys: $keys,
            children: $children,
            paren: $paren
        })
    }};
}


macro_rules! key {
    ($x:expr, $idx:expr) => {{
        let x = $x;
        let idx = $idx;

        if $x.is_leaf() {
            &entries!(x)[idx].0
        } else {
            &keys!(x)[idx]
        }
    }};
}


/// WARNING: Search by Key O(logM)
macro_rules! index_of_child {
    ($p: expr, $child: expr) => {{
        let p = &$p;
        let child = $child;

        debug_assert!(child.is_some());

        match keys!(p).binary_search(child.min_key().unwrap()) {
            Ok(idx) => idx + 1,
            Err(idx) => idx,
        }
    }};
}


macro_rules! children_revref {
    ($x:expr) => {{
        children_revref!($x, ..)
    }};
    ($x:expr, $range:expr) => {{
        let x = $x;
        let range = $range;

        for child in &mut children_mut!(x)[range] {
            paren!(child, x.clone());
        }
    }}
}


/// Node_ heap data field access
macro_rules! def_node__heap_access {
    (internal, $name:ident, $ret:ty) => {
        fn $name(&self) -> &$ret {
            match self {
                Internal { $name, .. } => $name,
                Leaf {..} => panic!(
                    "Get `{}` on leaf",
                    stringify!($name)
                )
            }
        }
        coll::paste!(
            fn [<$name _mut>](&mut self) -> &mut $ret {
                match self {
                    Internal { $name, .. } => $name,
                    Leaf {..} => panic!(
                        "Get `{}` on leaf",
                        stringify!($name)
                    )
                }
            }
        );
    };
    (leaf, $name:ident, $ret:ty) => {
        fn $name(&self) -> &$ret {
            match self {
                Internal {..} => panic!(
                    "Get `{}` on internal node",
                    stringify!($name)
                ),
                Leaf { $name, ..} => $name
            }
        }
        coll::paste!(
            fn [<$name _mut>](&mut self) -> &mut $ret {
                match self {
                    Internal {..} => panic!(
                        "Get `{}` on internal node",
                        stringify!($name)
                    ),
                    Leaf { $name, ..} => $name
                }
            }
        );
    };
}


/// Node_ WeakNode field access
macro_rules! def_node__wn_access {
    (both, $name:ident) => {
        fn $name(&self) -> Node<K, V> {
            match self {
                Internal { $name, .. } => $name,
                Leaf { $name, .. } => $name,
            }
            .upgrade()
        }
        coll::paste!(
            fn [<set_ $name>](&mut self, x: Node<K, V>) {
                match self {
                    Internal { $name, .. } => $name,
                    Leaf { $name, .. } => $name,
                }
                .replace(x.downgrade());
            }
        );
    };
    (leaf, $name:ident) => {
        fn $name(&self) -> Node<K, V> {
            match self {
                Internal {..} => panic!(
                    "Get `{}` on internal node",
                    stringify!($name)
                ),
                Leaf { $name, .. } => $name,
            }
            .upgrade()
        }
        coll::paste!(
            fn [<set_ $name>](&mut self, x: Node<K, V>) {
                match self {
                    Internal {..} => panic!(
                        "Get `{}` on internal node",
                        stringify!($name)
                    ),
                    Leaf { $name, .. } => $name,
                }
                .replace(x.downgrade());
            }
        );
    };
}


macro_rules! def_attr_macro_bpt {
    ($($name:ident),+) => {
        $(
            coll::paste!(
                macro_rules! $name {
                    ($node:expr) => {
                        attr!(self | $$node).$name()
                    };
                    ($node:expr, $val:expr) => {
                        attr!(self_mut | $$node).[<set_ $name>]($$val)
                    };
                }
            );

            coll::paste!(
                #[allow(unused)]
                macro_rules! [<$name _mut>] {
                    ($node:expr) => {
                        attr!(self_mut | $$node).[<$name _mut>]()
                    };
                }
            );
        )+
    };
}


pub(super) use def_attr_macro_bpt;
pub(super) use def_node__heap_access;
pub(super) use def_node__wn_access;

def_attr_macro_bpt!(paren, succ, keys, entries, children);
def_coll_init!(map ^ 1 | bpt, crate::bt::bpt::BPT::new());


////////////////////////////////////////////////////////////////////////////////
//// Structure

enum Node_<K, V> {
    Internal {
        keys: Vec<K>,
        children: Vec<Node<K, V>>,
        paren: WeakNode<K, V>,
    },
    Leaf {
        entries: Vec<KVEntry<K, V>>,
        /// Successor (Leaf)
        succ: WeakNode<K, V>,
        paren: WeakNode<K, V>,
    },
}
use Node_::*;



////////////////////////////////////////////////////////////////////////////////
//// Implementation

impl<K: Ord, V, const M: usize> BPT<K, V, M> {
    ////////////////////////////////////////////////////////////////////////////
    //// Public API

    pub fn new() -> Self {
        assert!(M > 2, "M should be greater than 2");

        Self {
            root: Node::none(),
            cnt: 0,
            min_node: WeakNode::none(),
            max_node: WeakNode::none(),
        }
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.cnt
    }

    pub fn get<Q>(&self, k: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        mut_self!(self).get_mut(k).map(|v| &*v)
    }

    pub fn get_mut<Q>(&mut self, k: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        let x = Self::search_to_leaf(&self.root, k);

        // Nil
        if x.is_none() {
            None
        }
        // Leaf
        else {
            debug_assert!(x.is_leaf());

            match entries!(x).binary_search_by_key(&k, |ent| ent.0.borrow()) {
                Ok(idx) => Some(&mut entries_mut!(x)[idx].1),
                Err(_idx) => None,
            }
        }
    }

    pub fn select<'a, Q, R: RangeBounds<&'a Q>>(
        &self,
        range: R,
    ) -> impl Iterator<Item = &V>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized + 'a,
    {
        mut_self!(self).select_mut(range).map(|v| &*v)
    }

    pub fn select_mut<'a, Q, R: RangeBounds<&'a Q>>(
        &mut self,
        range: R,
    ) -> impl Iterator<Item = &mut V>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized + 'a,
    {
        /* find start_node */

        let mut cur;
        let mut idx = 0;

        match range.start_bound() {
            Included(&k) => {
                let x = Self::search_to_leaf(&self.root, k.borrow());

                // Nil
                if x.is_none() {
                    cur = Node::none();
                }
                // Leaf
                else {
                    match entries!(x)
                        .binary_search_by_key(&k, |ent| ent.0.borrow())
                    {
                        Ok(idx_) => idx = idx_,
                        Err(idx_) => idx = idx_,
                    };
                    cur = x;
                }
            }
            Excluded(_) => unimplemented!(),
            Unbounded => {
                cur = self.min_node.upgrade();
            }
        }

        if cur.is_some() && idx == entries!(cur).len() {
            cur = succ!(cur);
            idx = 0;
        }

        std::iter::from_generator(move || {
            if cur.is_none() {
                return;
            }

            let mut entries = &mut entries_mut!(cur)[idx..];

            loop {
                for ent in entries {
                    if range.contains(&ent.0.borrow()) {
                        yield &mut ent.1
                    } else {
                        return;
                    }
                }

                cur = succ!(cur);

                if cur.is_none() {
                    break;
                }

                entries = &mut entries_mut!(cur)[..];
            }
        })
    }

    pub fn entries(&self) -> impl Iterator<Item = (&K, &V)> {
        std::iter::from_generator(move || {
            for x in self.nodes() {
                for ent in entries!(x) {
                    yield (&ent.0, &ent.1)
                }
            }
        })
    }

    pub fn keys(&self) -> impl Iterator<Item = &K> {
        self.entries().map(|ent| ent.0)
    }

    pub fn values(&self) -> impl Iterator<Item = &V> {
        self.entries().map(|ent| ent.1)
    }

    pub fn insert(&mut self, k: K, v: V) -> Option<V>
    where
        K: Clone,
    {
        let x = Self::search_to_leaf(&self.root, &k);

        self.insert_into_leaf(x, k, v)
    }

    pub fn remove<Q>(&mut self, k: &Q) -> Option<V>
    where
        K: Borrow<Q> + Clone,
        Q: Ord + ?Sized,
        V: Debug,
    {
        let mut x = &self.root;
        let mut internal_and_idx = None;

        while x.is_internal() {
            match keys!(x).binary_search_by_key(&k, |k_| k_.borrow()) {
                Ok(idx) => {
                    debug_assert!(internal_and_idx.is_none());
                    internal_and_idx = Some((x.clone(), idx));

                    x = &children!(x)[idx + 1];
                }
                Err(idx) => {
                    x = &children!(x)[idx];
                }
            }
        }

        if x.is_none() {
            return None;
        }

        let idx;
        match entries!(x).binary_search_by_key(&k, |ent| ent.0.borrow()) {
            Ok(idx_) => idx = idx_,
            Err(_idx) => return None,
        }

        self.remove_on_leaf(internal_and_idx, x.clone(), idx)
            .map(|(_, v)| v)
    }


    ////////////////////////////////////////////////////////////////////////////
    //// Extend Method

    /// Approximate Search return node.v where node.k >= key
    pub fn low_bound_search<Q>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        let x = Self::search_to_leaf(&self.root, key);

        if x.is_none() {
            return None;
        }

        Some(
            match entries!(x).binary_search_by_key(&key, |ent| ent.0.borrow())
            {
                Ok(idx) => &entries!(x)[idx].1,
                Err(idx) => {
                    if idx == 0 {
                        return None;
                    } else {
                        &entries!(x)[idx - 1].1
                    }
                }
            },
        )
    }

    /// Start from 0 O(n/M)
    pub fn rank<Q>(&self, key: &Q) -> std::result::Result<usize, usize>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        if self.root.is_none() {
            return Err(0);
        }

        let x = Self::search_to_leaf(&self.root, key);

        debug_assert!(x.is_some());

        let idx;
        let mut is_err = false;

        match entries!(x).binary_search_by_key(&key, |ent| ent.0.borrow()) {
            Ok(idx_) => idx = idx_,
            Err(idx_) => {
                idx = idx_;
                is_err = true;
            }
        }

        let mut rem = entries!(x).len() - (idx + 1);
        let mut cur = succ!(x);

        while cur.is_some() {
            rem += entries!(cur).len();
            cur = succ!(cur);
        }

        let rk = self.cnt - rem - 1;

        if !is_err {
            Ok(rk)
        }
        else {
            Err(rk)
        }
    }

    /// return Nth child (start from 0), O(n/M)
    pub fn nth(&self, mut idx: usize) -> Option<(&K, &V)> {
        let mut cur = self.min_node.upgrade();

        while cur.is_some() {
            if idx < entries!(cur).len() {
                let ent = &entries!(cur)[idx];
                return Some((&ent.0, &ent.1));
            } else {
                idx -= entries!(cur).len();
                cur = succ!(cur);
            }
        }

        None
    }

    #[inline]
    pub fn min_key(&self) -> Option<&K> {
        self.min_node
            .upgrade()
            .min_key()
            .map(|k| unsafe { &*(k as *const K) })
    }

    #[inline]
    pub fn max_key(&self) -> Option<&K> {
        self.max_node
            .upgrade()
            .max_key()
            .map(|k| unsafe { &*(k as *const K) })
    }

    /// return [at, ...)
    pub fn split_off(&mut self, at: usize) -> Self
    where
        K: Clone,
        V: Debug,
    {
        let mut oth = Self::new();

        if self.cnt == 0 || at >= self.cnt {
            return oth;
        }

        oth.bulk_push_front(self.bulk_pop(self.cnt - at));

        oth
    }

    pub fn bulk_pop<'a>(
        &'a mut self,
        mut n: usize,
    ) -> impl Iterator<Item = (K, V)> + 'a
    where
        K: Clone,
        V: Debug,
    {
        std::iter::from_generator(move || {
            while n > 0 && let Some((k, v)) = self.pop_last() {
                yield (k, v);
                n -= 1;
            }
        })
    }

    /// push into max
    pub fn bulk_push_back(&mut self, iter: impl Iterator<Item = (K, V)>)
    where
        K: Clone,
        V: Debug,
    {
        for (k, v) in iter {
            self.push_back(k, v);
        }
    }

    /// push into min from max to min
    pub fn bulk_push_front(&mut self, iter: impl Iterator<Item = (K, V)>)
    where
        K: Clone,
        V: Debug,
    {
        for (k, v) in iter {
            let x = self.min_node.upgrade();
            self.insert_into_leaf(x, k, v);
        }
    }

    /// push into max
    pub fn push_back(&mut self, k: K, v: V)
    where
        K: Clone,
        V: Debug,
    {
        let x = self.max_node.upgrade();
        self.insert_into_leaf(x, k, v);
    }

    /// push into min
    pub fn push_front(&mut self, k: K, v: V)
    where
        K: Clone,
        V: Debug,
    {
        let x = self.min_node.upgrade();
        // self.insert_into_leaf(x, k, v);

        self.cnt += 1;

        /* Nil */
        if x.is_none() {
            self.root = node!(kv | k, v);
            self.min_node = self.root.downgrade();
            self.max_node = self.root.downgrade();
        }
        /* Leaf */
        else {
            /* insert into leaf */

            entries_mut!(x).insert(0, KVEntry(k, v));

            self.insert_retracing(x);
        }
    }

    pub fn pop_last(&mut self) -> Option<(K, V)>
    where
        K: Clone,
    {
        let x = self.max_node.upgrade();

        if x.is_none() {
            return None;
        }

        let p = paren!(x);

        let internal_and_idx;

        if p.is_some() && entries!(x).len() == 1 {
            internal_and_idx = Some((p.clone(), keys!(p).len() - 1));
        } else {
            internal_and_idx = None;
        }

        self.remove_on_leaf(internal_and_idx, x.clone(), entries!(x).len() - 1)
    }

    pub fn pop_first(&mut self) -> Option<(K, V)>
    where
        K: Clone,
    {
        let x = self.min_node.upgrade();

        if x.is_none() {
            return None;
        }

        /* min-key has no internal index */

        self.remove_on_leaf(None, x.clone(), 0)
    }

    pub fn into_iter(self) -> impl Iterator<Item = (K, V)> {
        let mut cur = self.min_node.upgrade();

        std::iter::from_generator(move || {
            while cur.is_some() {
                let nxt = succ!(cur);

                for ent in entries_mut!(cur).drain(..) {
                    yield ent.drain();
                }

                cur = nxt;
            }

            drop(self);
        })
    }

    /// collect all item without drop itself used for BPT2::remove
    pub(crate) fn drain_all(&mut self) -> impl Iterator<Item = (K, V)> {
        let mut cur = self.min_node.upgrade();

        std::iter::from_generator(move || {
            while cur.is_some() {
                let nxt = succ!(cur);

                for ent in entries_mut!(cur).drain(..) {
                    yield ent.drain();
                }

                cur = nxt;
            }
        })
    }

    ////////////////////////////////////////////////////////////////////////////
    //// Assistant Method

    #[inline(always)]
    fn search_to_leaf<Q>(mut x: &Node<K, V>, k: &Q) -> Node<K, V>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        while x.is_internal() {
            match keys!(x).binary_search_by_key(&k, |k_| k_.borrow()) {
                Ok(idx) => x = &children!(x)[idx + 1],
                Err(idx) => {
                    x = &children!(x)[idx];
                }
            }
        }

        x.clone()
    }

    fn nodes(&self) -> impl Iterator<Item = Node<K, V>> {
        let mut cur = self.min_node.upgrade();

        std::iter::from_generator(move || {
            while cur.is_some() {
                yield cur.clone();

                cur = succ!(cur);
            }
        })
    }

    fn remove_on_leaf(
        &mut self,
        internal_and_idx: Option<(Node<K, V>, usize)>,
        x: Node<K, V>,
        idx: usize,
    ) -> Option<(K, V)>
    where
        K: Clone,
    {
        /* Update internal key with its succsessor key */

        if let Some((internal, i_idx)) = internal_and_idx {
            self.update_internal_key(&internal, i_idx, &x, idx);
        }

        let popped = entries_mut!(x).remove(idx);
        self.remove_retracing(x.clone());

        self.cnt -= 1;

        Some((popped.0, popped.1))
    }

    fn insert_into_leaf(&mut self, x: Node<K, V>, k: K, v: V) -> Option<V>
    where
        K: Clone,
    {
        /* NonInternal Node */
        debug_assert!(!x.is_internal());

        /* Nil */
        if x.is_none() {
            self.root = node!(kv | k, v);
            self.min_node = self.root.downgrade();
            self.max_node = self.root.downgrade();
        }
        /* Leaf */
        else {
            let idx = match entries!(x).binary_search_by_key(&&k, |ent| &ent.0)
            {
                Ok(idx) => {
                    return Some(replace(&mut entries_mut!(x)[idx].1, v))
                }
                Err(idx) => idx,
            };

            /* insert into leaf */

            entries_mut!(x).insert(idx, KVEntry(k, v));

            self.insert_retracing(x);
        }

        self.cnt += 1;

        None
    }

    fn update_internal_key(
        &self,
        internal: &Node<K, V>,
        i_idx: usize,
        x: &Node<K, V>,
        idx: usize,
    ) where
        K: Clone,
    {
        debug_assert!(x.is_leaf());
        let p = paren!(x);
        debug_assert!(p.is_some());

        let new_key;

        if entries!(x).len() > 1 {
            // left first
            if idx > 0 {
                new_key = entries!(x)[idx - 1].0.clone();
            } else {
                new_key = entries!(x)[idx + 1].0.clone();
            }
        } else {
            let x_idx = index_of_child!(p, x);

            /* check remain node */

            // left first
            if x_idx > 0 && entries!(children!(p)[x_idx - 1]).len() > 1 {
                new_key = children!(p)[x_idx - 1].max_key().unwrap().clone();
            }
            // right sib
            else if x_idx < children!(p).len() - 1
                && entries!(children!(p)[x_idx + 1]).len() > 1
            {
                new_key = entries!(succ!(x))[0].0.clone();
            }
            /* use default (left first)*/
            else if x_idx > 0 {
                new_key = children!(p)[x_idx - 1].max_key().unwrap().clone();
            } else {
                new_key = entries!(succ!(x))[0].0.clone();
            }
        }

        keys_mut!(internal)[i_idx] = new_key;
    }

    fn insert_retracing(&mut self, x: Node<K, V>)
    where
        K: Clone,
    {
        if entries!(x).len() == M {
            self.promote(x);
        }
    }

    fn remove_retracing(&mut self, x: Node<K, V>)
    where
        K: Clone,
    {
        if entries!(x).is_empty() {
            if self.root.rc_eq(&x) {
                self.root = Node::none();
                self.min_node = WeakNode::none();
                self.max_node = WeakNode::none();
            } else {
                self.unpromote(x);
            }
        }
    }

    fn promote(&mut self, x: Node<K, V>)
    where
        K: Clone,
    {
        debug_assert!(x.is_some());

        let p = paren!(x);

        if p.is_some() {
            let idx = index_of_child_by_rc!(p, x);

            if Self::try_rebalancing(&p, idx) {
                return;
            }
        }

        /* split node */

        let lpos = M.div_floor(2);
        let hpos = M.div_ceil(2);

        let head_key;
        let x2;

        if x.is_leaf() {
            debug_assert_eq!(entries!(x).len(), Self::entries_high_bound());

            // keep key for leaf
            let entries_x2 = entries_mut!(x).split_off(lpos);

            head_key = entries_x2[0].0.clone();

            x2 = node!(
                basic - leaf | entries_x2,
                succ!(x).downgrade(),
                WeakNode::none()
            );

            // if succ!(x2).is_some() {
            //     pred!(succ!(x2), x2.clone());
            // }
            succ!(x, x2.clone());

            if x.rc_eq(&self.max_node.upgrade()) {
                self.max_node = x2.downgrade();
            }
        } else {
            debug_assert_eq!(keys!(x).len(), Self::entries_high_bound());

            let keys_x2 = keys_mut!(x).split_off(hpos);

            head_key = keys_mut!(x).pop().unwrap();

            let children_x2 = children_mut!(x).split_off(hpos);

            x2 = node!(
                basic - internal | keys_x2,
                children_x2,
                WeakNode::none()
            );

            children_revref!(&x2);
        }

        /* push new level */
        if p.is_none() {
            let keys = vec![head_key];
            let children = vec![x, x2];

            self.root =
                node!(basic - internal | keys, children, WeakNode::none());

            children_revref!(&self.root);
        }
        /* insert into paren node */
        else {
            let x_idx = index_of_child!(p, &x);

            keys_mut!(p).insert(x_idx, head_key);
            children_mut!(p).insert(x_idx + 1, x2.clone());

            paren!(x2, p.clone());

            if keys!(p).len() == Self::entries_high_bound() {
                self.promote(p);
            }
        }
    }

    fn unpromote(&mut self, x: Node<K, V>)
    where
        K: Clone,
    {
        debug_assert!(
            x.is_leaf() || keys!(x).len() == Self::entries_low_bound() - 1
        );

        let p = paren!(x);

        debug_assert!(p.is_some());

        let idx = index_of_child_by_rc!(p, x);

        if Self::try_rebalancing(&p, idx) {
            return;
        }

        /* merge node */

        if idx > 0 {
            self.merge_node(&p, idx - 1);
        } else {
            self.merge_node(&p, idx);
        }

        if paren!(p).is_none() {
            debug_assert!(p.is_internal());

            if keys!(p).is_empty() {
                // pop new level
                self.root = children_mut!(p).pop().unwrap();
                paren!(self.root, Node::none());
            }
        } else {
            if keys!(p).len() < Self::entries_low_bound() {
                self.unpromote(p);
            }
        }
    }

    /// (parent, left-idx)
    fn merge_node(&mut self, p: &Node<K, V>, idx: usize) {
        let children = children!(p);

        let left = children[idx].clone();
        let right = children[idx + 1].clone();

        // for leaf node
        if left.is_leaf() {
            keys_mut!(p).remove(idx);

            entries_mut!(left).extend(entries_mut!(right).drain(..));

            // update succ
            succ!(left, succ!(right));
            // // update pred
            // if succ!(right).is_some() {
            //     pred!(succ!(right), left.clone());
            // }

            // update max_node
            if right.rc_eq(&self.max_node.upgrade()) {
                // update max_node
                self.max_node = left.downgrade();
            }
        }
        // for internal node
        else {
            keys_mut!(left).push(keys_mut!(p).remove(idx));
            keys_mut!(left).extend(keys_mut!(right).drain(..));

            // merge right's children to the left
            let left_children = children_mut!(left);

            for child in children_mut!(right).drain(..) {
                paren!(child, left.clone());

                left_children.push(child);
            }
        }

        // remove right from p
        children_mut!(p).remove(idx + 1);
    }

    /// parent, x idx of parent
    fn try_rebalancing(p: &Node<K, V>, idx: usize) -> bool
    where
        K: Clone,
    {
        // Try left first
        if idx > 0 && Self::try_node_redistribution_eager(&p, idx - 1, Left) {
            return true;
        }

        // Try right then
        if idx < children!(p).len() - 1
            && Self::try_node_redistribution_eager(&p, idx, Right)
        {
            return true;
        }

        false
    }

    #[allow(unused)]
    fn try_node_redistribution(
        p: &Node<K, V>,
        idx: usize,
        sib_dir: Dir,
    ) -> bool
    where
        K: Clone,
    {
        let children = children!(p);

        let left = &children[idx];
        let right = &children[idx + 1];

        let sib = if sib_dir.is_left() { left } else { right };

        if sib.is_leaf() && entries!(sib).len() > 1 {
            if sib_dir.is_left() {
                entries_mut!(right)
                    .insert(0, entries_mut!(left).pop().unwrap());
            }
            // sib is right
            else {
                entries_mut!(left).push(entries_mut!(right).remove(0));

                keys_mut!(p)[idx] = entries!(right)[0].0.clone();
            }

            return true;
        }

        if sib.is_internal() && keys!(sib).len() > Self::entries_low_bound() {
            if sib_dir.is_left() {
                keys_mut!(right).insert(
                    0,
                    replace(
                        &mut keys_mut!(p)[idx],
                        keys_mut!(left).pop().unwrap(),
                    ),
                );

                let child = children_mut!(left).pop().unwrap();

                paren!(child, right.clone());

                children_mut!(right).insert(0, child);
            } else {
                keys_mut!(left).push(replace(
                    &mut keys_mut!(p)[idx],
                    keys_mut!(right).remove(0),
                ));

                let child = children_mut!(right).remove(0);

                paren!(child, left.clone());

                children_mut!(left).push(child);
            }

            return true;
        }

        false
    }

    /// Try redistribute until all equal of two nodes instead of just lazy rebalancing
    ///
    /// B* tree property
    #[allow(unused)]
    fn try_node_redistribution_eager(
        p: &Node<K, V>,
        idx: usize,
        sib_dir: Dir,
    ) -> bool
    where
        K: Clone,
    {
        use common::vec_even_up;
        // use common::vec_even_up_1;

        let children = children!(p);

        let left = &children[idx];
        let right = &children[idx + 1];

        let (sib, x) = if sib_dir.is_left() {
            (left, right)
        } else {
            (right, left)
        };

        if sib.is_leaf()
            && (entries!(x).len() == 0 && entries!(sib).len() > 1
                || entries!(x).len() == Self::entries_high_bound()
                    && entries!(sib).len() < Self::entries_high_bound() - 1)
        {
            vec_even_up(entries_mut!(left), entries_mut!(right));
            // vec_even_up_1(entries_mut!(left), entries_mut!(right));

            keys_mut!(p)[idx] = entries!(right)[0].0.clone();

            return true;
        }

        if sib.is_internal()
            && (keys!(x).len() < Self::entries_low_bound()
                && keys!(sib).len() > Self::entries_low_bound()
                || keys!(x).len() == Self::entries_high_bound()
                    && keys!(sib).len() < Self::entries_high_bound() - 1)
        {
            let left_old_len = children!(left).len();
            let right_old_len = children!(right).len();

            // for vec_even_up
            keys_mut!(left).push(keys_mut!(p)[idx].clone());
            vec_even_up(keys_mut!(left), keys_mut!(right));
            keys_mut!(p)[idx] = keys_mut!(left).pop().unwrap();

            // for vec_even_up_1
            // if left_old_len < right_old_len {
            //     keys_mut!(left).push(replace(
            //         &mut keys_mut!(p)[idx],
            //         keys_mut!(right).remove(0),
            //     ));
            // }
            // else {
            //     keys_mut!(right).insert(
            //         0,
            //         replace(
            //             &mut keys_mut!(p)[idx],
            //             keys_mut!(left).pop().unwrap(),
            //         ),
            //     );
            // }

            vec_even_up(children_mut!(left), children_mut!(right));
            // vec_even_up_1(children_mut!(left), children_mut!(right));

            // for vec_even_up
            if (children!(left).len() + children!(right).len()) % 2 > 0 {
                children_mut!(right)
                    .insert(0, children_mut!(left).pop().unwrap());
            }

            if left_old_len < right_old_len  {
                children_revref!(left, left_old_len..children!(left).len());
            } else {
                children_revref!(right, 0..children!(right).len()-right_old_len);
            }

            return true;
        }

        false
    }
}


impl<K: Ord + Debug, V, const M: usize> Debug for BPT<K, V, M> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.debug_struct("BPT")
            .field("root", &self.root)
            .field("cnt", &self.cnt)
            .field("min_node", &self.min_node.upgrade())
            .field("max_node", &self.max_node.upgrade())
            .finish()
    }
}


impl<K, V> Node_<K, V> {
    fn is_leaf(&self) -> bool {
        matches!(self, Leaf { .. })
    }

    def_node__heap_access!(internal, keys, Vec<K>);
    def_node__heap_access!(internal, children, Vec<Node<K, V>>);
    def_node__heap_access!(leaf, entries, Vec<KVEntry<K, V>>);

    def_node__wn_access!(both, paren);
    def_node__wn_access!(leaf, succ);
    // def_node__wn_access!(leaf, pred);
}


impl<K, V> Node<K, V> {
    fn is_leaf(&self) -> bool {
        self.is_some() && attr!(self | self).is_leaf()
    }

    fn is_internal(&self) -> bool {
        self.is_some() && !attr!(self | self).is_leaf()
    }

    fn max_key(&self) -> Option<&K> {
        if self.is_none() {
            None
        } else if self.is_internal() {
            keys!(self).last()
        } else {
            entries!(self).last().map(|ent| &ent.0)
        }
    }

    fn min_key(&self) -> Option<&K> {
        if self.is_none() {
            None
        } else if self.is_internal() {
            keys!(self).first()
        } else {
            entries!(self).first().map(|ent| &ent.0)
        }
    }
}


impl<K: Debug, V> Debug for Node<K, V> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        if self.is_some() {
            let kn;

            if self.is_leaf() {
                kn = entries!(self).len();
            } else {
                kn = keys!(self).len();
            }

            for i in 0..kn - 1 {
                write!(f, "{:?}, ", key!(self, i))?;
            }
            write!(f, "{:?}", key!(self, kn - 1))?;
        } else {
            write!(f, "nil")?;
        }

        Ok(())
    }
}


////////////////////////////////////////////////////////////////////////////////
//// Test && Stats Method

#[cfg(test)]
#[allow(unused)]
// use std::collections::HashMap;
use indexmap::IndexMap;


#[cfg(test)]
#[allow(unused)]
impl<K: Ord + Debug + Clone, V, const M: usize> BPT<K, V, M> {
    fn search_obsoleted_key(&self) -> Vec<K> {
        let mut obsoleted = vec![];
        let mut q = common::vecdeq![&self.root];

        while let Some(x) = q.pop_front() && x.is_internal() {
            for k in keys!(x) {
                let leaf = Self::search_to_leaf(&x, k);

                if leaf.is_none()
                    || entries!(leaf).binary_search_by_key(
                        &k, |ent| &ent.0
                    ).is_err()
                {
                    obsoleted.push(k.clone());
                }
            }

            if x.lv() > 2 {
                for child in children!(x) {
                    q.push_back(child);
                }
            }
        }

        obsoleted
    }

    fn count_nodes_keys(&self) -> IndexMap<K, usize>
    where
        K: std::hash::Hash,
    {
        let mut map = IndexMap::new();

        for x in self.nodes() {
            for k in entries!(x).iter().map(|ent| ent.0.clone()) {
                let cnt = self.count_key(&k);
                map.insert(k, cnt);
            }
        }

        map
    }

    /// Count key occurance across the tree
    #[track_caller]
    fn count_key(&self, k: &K) -> usize {
        let mut count = 0;
        let mut x = &self.root;

        while x.is_internal() {
            match keys!(x).binary_search_by_key(&k, |k_| k_.borrow()) {
                Ok(idx) => {
                    count += 1;

                    x = &children!(x)[idx + 1];
                }
                Err(idx) => {
                    x = &children!(x)[idx];
                }
            }
        }

        if x.is_none() {
            panic!("There is no {k:?}")
        }

        match entries!(x).binary_search_by_key(&k, |ent| &ent.0) {
            Ok(idx) => {
                if count > 0 {
                    assert_eq!(
                        idx, 0,
                        "has internal index for nodex idx: {idx}"
                    );
                }

                count += 1;
            }
            Err(idx) => panic!("There is no {k:?}"),
        };

        count
    }

    fn debug_write<W: std::fmt::Write>(&self, f: &mut W) -> std::fmt::Result
    where
        V: Debug,
    {
        use common::vecdeq;

        /* print header */

        writeln!(f, "{self:?}")?;

        /* print body */

        if self.root.is_none() {
            return Ok(());
        }

        let mut this_q = vecdeq![vec![self.root.clone()]];
        let mut lv = 1;

        while !this_q.is_empty() {
            writeln!(f)?;
            writeln!(f, "############ Level: {lv} #############")?;
            writeln!(f)?;

            let mut nxt_q = vecdeq![];

            while let Some(children) = this_q.pop_front() {
                for (i, x) in children.iter().enumerate() {
                    let p = paren!(x);

                    if x.is_internal() {
                        nxt_q.push_back(children!(x).clone());
                        writeln!(f, "({i:02}): {x:?} (p: [{p:?}])")?;
                    } else {
                        let succ = succ!(x);
                        writeln!(
                            f,
                            "({i:02}): {x:?} (p: [{p:?}], succ: [{succ:?}])"
                        )?;
                    }
                }

                writeln!(f)?;
            }


            this_q = nxt_q;
            lv += 1;
        }

        writeln!(f, "------------- end --------------\n")?;

        Ok(())
    }

    pub(crate) fn debug_print(&self)
    where
        V: Debug,
    {
        let mut cache = String::new();

        self.debug_write(&mut cache).unwrap();

        println!("{cache}")
    }

    #[track_caller]
    pub(crate) fn validate(&self)
    where
        K: Debug + std::hash::Hash,
        V: Debug,
    {
        if self.root.is_none() {
            return;
        }

        use std::collections::VecDeque;

        use common::vecdeq;

        let mut cur_q = vecdeq![vecdeq![Node::none(), self.root.clone()]];

        while !cur_q.is_empty() {
            let mut nxt_q = vecdeq![];
            let mut leaf_num = 0;
            let mut internal_num = 0;

            while let Some(mut group) = cur_q.pop_front() {
                let p = group.pop_front().unwrap();

                for child in group.iter() {
                    assert!(child.is_some());

                    if child.is_internal() {
                        assert_eq!(
                            keys!(child).len() + 1,
                            children!(child).len(),
                            "{child:?}"
                        );
                        assert!(
                            children!(child).len() <= M,
                            "{child:?}: {}",
                            children!(child).len()
                        );
                        assert!(
                            keys!(child).len() < Self::entries_high_bound()
                        );
                    } else {
                        assert!(
                            entries!(child).len() < Self::entries_high_bound()
                        );
                    }

                    // Exclude leaf
                    if child.is_leaf() {
                        if p.is_some() {
                            let idx = index_of_child!(p, child);

                            if idx > 0 {
                                assert_eq!(
                                    keys!(p)[idx - 1],
                                    entries!(child)[0].0
                                );
                            }
                        }

                        assert!(self.min_key() <= child.min_key());
                        assert!(self.max_key() >= child.max_key());

                        leaf_num += 1;
                    } else {
                        // Exclude the root (which is always one when it's internal node)
                        if paren!(child).is_some() {
                            assert!(
                                keys!(child).len()
                                    >= Self::entries_low_bound()
                            );
                        } else {
                            assert!(children!(child).len() >= 2);
                        }

                        /* search obsoleted key */
                        for k in keys!(child) {
                            let leaf = Self::search_to_leaf(&child, k);

                            if leaf.is_none()
                                || entries!(leaf)
                                    .binary_search_by_key(&k, |ent| &ent.0)
                                    .is_err()
                            {
                                panic!("Found obsoleted key: {k:?}");
                            }
                        }

                        internal_num += 1;

                        let mut nxt_children =
                            VecDeque::from(children!(child).clone());
                        nxt_children.push_front(child.clone());

                        nxt_q.push_back(nxt_children);
                    }
                }

                // All leaves are in same level
                assert!(
                    leaf_num == 0 || internal_num == 0,
                    "leaf: {leaf_num}, internal: {internal_num}"
                );

                // Ordered
                if p.is_some() {
                    let last_child = group.pop_back().unwrap();

                    for (i, child) in group.iter().enumerate() {
                        if child.is_leaf() {
                            assert!(entries!(child).iter().is_sorted());
                        } else {
                            assert!(keys!(child).is_sorted());
                        }

                        let child_max_key = child.max_key().unwrap();
                        let branch_key = key!(&p, i);

                        assert!(
                            child_max_key < branch_key,
                            "child: {child_max_key:?}, branch:{branch_key:?}"
                        );

                        if i > 0 {
                            assert!(key!(&p, i - 1) <= key!(child, 0));
                        }
                    }

                    if p.max_key().unwrap() > key!(&last_child, 0) {
                        self.debug_print();
                    }
                    assert!(
                        p.max_key().unwrap() <= key!(&last_child, 0),
                        "p: {p:?}"
                    );
                }
            }

            cur_q = nxt_q;
        }

        // // test pred
        // let mut cur = self.max_node.upgrade();
        // let mut cnt = 0;

        // while cur.is_some() {
        //     cnt += entries!(cur).len();
        //     cur = pred!(cur)
        // }

        // assert_eq!(cnt, self.len());

        // // test succ
        // let mut cur = self.min_node.upgrade();
        // let mut cnt = 0;

        // while cur.is_some() {
        //     cnt += entries!(cur).len();
        //     cur = succ!(cur)
        // }

        // assert_eq!(cnt, self.len());

        //
        // self.count_nodes_keys();
    }
}


#[cfg(test)]
#[allow(unused)]
impl<K, V> Node<K, V> {
    fn lv(&self) -> usize {
        debug_assert!(self.is_some());

        let mut x = self;
        let mut lv = 1;

        while x.is_internal() {
            x = &children!(x)[0];
            lv += 1;
        }

        lv
    }
}


#[cfg(test)]
pub(crate) mod tests {

    use super::{super::tests::*, *};
    use crate::bst::test_dict;

    macro_rules! prepare_dict {
        ($dict:expr) => {{
            let mut dict = $dict;
            let mut back = std::collections::BTreeMap::new();
            let getone = || common::random::<u16>() % 10_00;

            for _ in 0..1000 {
                let k = getone();

                dict.insert(k, k);
                back.insert(k, k);
            }

            for _ in 0..250 {
                let k = getone();

                dict.remove(&k);
                back.remove(&k);
            }

            (dict, back)
        }};
    }

    macro_rules! assert_select_eq {
        ($dict:expr, $r:expr, $r2:expr) => {{
            use common::{EitherOrBoth::*, Itertools};

            let r = $r;
            let r2 = $r2;

            let u_range = (r);
            let v_range = $dict.select(r2).cloned();

            for x in u_range.zip_longest(v_range) {
                match x {
                    Both(u, v) => {
                        assert_eq!(u, v, "NEQ {u} vs {v} for {:?}", $r)
                    }
                    Left(u) => panic!("range remains: {u}"),
                    Right(v) => panic!("select remains: {v}"),
                }
            }
        }};
    }

    macro_rules! verify_select {
        ($dict:expr) => {{
            let mut dict = $dict;
            let get_lo = |hi| common::random_range((0..hi));

            /* fill */

            let batch = 1000;

            for i in 0..batch {
                dict.insert(i, i);
            }

            /* select verify */

            let mut len = batch * 3 / 4;

            assert_select_eq!(dict, 0..1000, ..);

            loop {
                let lo = get_lo(batch / 4);

                assert_select_eq!(dict, lo..lo + len, &lo..&(lo + len));

                len = len * 3 / 4;
                if len == 0 {
                    break;
                }
            }
        }};
    }

    macro_rules! verify_bulk {
        ($dict:expr) => {{
            let (mut dict, back) = prepare_dict!($dict);

            /* test bulk push */

            let batch = 1000;
            let group = 50;

            for i in 0..group {
                let range = 2_000 + i * batch..2_000 + (i + 1) * batch;

                dict.bulk_push_back(range.clone().map(|x| (x, x)));

                for j in range {
                    assert_eq!(
                        dict.get(&j),
                        Some(&j),
                        "[verify bulk push] verify get range"
                    );
                }
                for k in back.keys() {
                    assert_eq!(
                        dict.get(k),
                        Some(k),
                        "[verify bulk push] verify get back"
                    );
                }

                assert_eq!(
                    dict.len(),
                    back.len() + ((i + 1) * batch) as usize,
                    "[verify bulk push] verify cnt"
                );

                if i % 5 == 0 {
                    dict.validate();
                }
            }


            /* test bulk pop */

            for i in (0..group).rev() {
                let bulk: Vec<(u16, u16)> =
                    dict.bulk_pop(batch as usize).collect();

                for (j, (k, _v)) in bulk.into_iter().enumerate() {
                    // println!("j: {j}");

                    assert_eq!(
                        ((i + 1) * batch) as u16 + 2_000 - 1 - j as u16,
                        k,
                        "[verify bulk pop] verify range"
                    )
                }

                assert_eq!(
                    dict.len(),
                    back.len() + (i * batch) as usize,
                    "[verify bulk push] verify cnt"
                );

                if i % 5 == 0 {
                    dict.validate();
                }
            }
        }};
    }

    macro_rules! verify_pred_succ_etc {
        ($dict:expr) => {{
            let (dict, back) = prepare_dict!($dict);

            // test rank/nth
            for (i, (k, _v)) in back.iter().enumerate() {
                let rk = dict.rank(&k).unwrap();
                let (dk, _v) = dict.nth(rk).unwrap();

                assert_eq!(rk, i, "rank=");
                assert_eq!(dk, k, "dk: {dk}, k: {k}");
            }

            // test pop first
        }};
    }

    macro_rules! verify_bpt_properties {
        ($dict:expr) => {{

            for _ in 0..100 {
                let (dict, _back) = prepare_dict!($dict);

                // 所有 internal key 都是叶子节点 idx=0 的位置的key
                let cnt_keys = dict.count_nodes_keys();

                let min_key_cnt = cnt_keys.values().min().unwrap();
                let max_key_cnt = cnt_keys.values().max().unwrap();

                // 同样的 key 可以出现一次或者两次，也就是说在internal上最多有一个索引
                assert_eq!(min_key_cnt, &1);
                assert_eq!(max_key_cnt, &2);
            }
        }}
    }

    macro_rules! verify_bpt_pop {
        ($dict:expr) => {{
            for _ in 0..20 {
                let (mut dict, mut back) = prepare_dict!($dict);
                dict.validate();

                while let Some((k, _v)) = back.pop_last() {
                    let (dk, _) = dict.pop_last().unwrap();
                    assert_eq!(k, dk, "[pop-last] k: {k}, dk: {dk}");

                    dict.validate();
                }

                // let (mut dict, mut back) = prepare_dict!($dict);

                // while let Some((k, _v)) = back.pop_first() {
                //     let (dk, _) = dict.pop_first().unwrap();
                //     assert_eq!(k, dk, "[pop-first] k: {k}, dk: {dk}");

                //     dict.validate();
                // }
            }
        }};
    }


    pub(crate) use assert_select_eq;
    pub(crate) use prepare_dict;
    pub(crate) use verify_bulk;
    pub(crate) use verify_select;

    #[test]
    fn test_bt_bpt_succ_pred_etc() {
        verify_pred_succ_etc!(BPT::<u16, u16, 3>::new());
        verify_pred_succ_etc!(BPT::<u16, u16, 4>::new());
        verify_pred_succ_etc!(BPT::<u16, u16, 5>::new());
        verify_pred_succ_etc!(BPT::<u16, u16, 10>::new());
        verify_pred_succ_etc!(BPT::<u16, u16, 21>::new());
    }

    #[test]
    fn test_bt_bpt_case_1() {
        let mut dict = BPT::<u16, u16, 3>::new();

        dict_insert!(dict, 52);
        dict_insert!(dict, 47);
        dict_insert!(dict, 3);
        dict_insert!(dict, 35);
        dict_insert!(dict, 24);
        dict_insert!(dict, 44);
        dict_insert!(dict, 66);
        dict_insert!(dict, 38);
        dict_insert!(dict, 30);
        dict_insert!(dict, 28);

        // dict.debug_print();

        /* Test select */
        assert_eq!(
            dict.select(..).cloned().collect::<Vec<_>>(),
            [3, 24, 28, 30, 35, 38, 44, 47, 52, 66]
        );
        assert_eq!(
            dict.select(&19..&19).cloned().collect::<Vec<_>>(),
            [0u16; 0]
        );
        assert_eq!(
            dict.select(&35..).cloned().collect::<Vec<_>>(),
            [35, 38, 44, 47, 52, 66]
        );
        assert_eq!(
            dict.select(&34..).cloned().collect::<Vec<_>>(),
            [35, 38, 44, 47, 52, 66]
        );
        assert_eq!(
            dict.select(&36..).cloned().collect::<Vec<_>>(),
            [38, 44, 47, 52, 66]
        );
        assert_eq!(
            dict.select(&34..&66).cloned().collect::<Vec<_>>(),
            [35, 38, 44, 47, 52]
        );
        assert_eq!(
            dict.select(&34..&67).cloned().collect::<Vec<_>>(),
            [35, 38, 44, 47, 52, 66]
        );
        assert_eq!(
            dict.select(&34..&45).cloned().collect::<Vec<_>>(),
            [35, 38, 44]
        );

        dict_remove!(dict, 24);
        dict_remove!(dict, 44);
        dict_remove!(dict, 66);
        dict_remove!(dict, 38);
        dict_remove!(dict, 52);
        dict_remove!(dict, 47);
        dict_remove!(dict, 3);

        assert_eq!(dict.select(..).cloned().collect::<Vec<_>>(), [28, 30, 35]);

        dict_remove!(dict, 35);
        dict_remove!(dict, 30);
        dict_remove!(dict, 28);

        assert_eq!(dict.select(..).cloned().collect::<Vec<_>>(), [0u16; 0]);

        // dict.debug_print();
    }

    #[test]
    fn test_bt_bpt_case_2() {
        let mut dict = BPT::<u16, u16, 5>::new();

        dict_insert!(dict, 25);
        dict_insert!(dict, 62);
        dict_insert!(dict, 6);
        dict_insert!(dict, 2);
        dict_insert!(dict, 45);
        dict_insert!(dict, 11);
        dict_insert!(dict, 55);
        dict_insert!(dict, 51);

        // dict.debug_print();

        dict_remove!(dict, 45);
        dict_remove!(dict, 55);
        dict_remove!(dict, 51);
        dict_remove!(dict, 25);
        dict_remove!(dict, 11);

        dict.debug_print();

        let mut iter = dict.into_iter();

        assert_eq!(iter.next(), Some((2, 2)));
        assert_eq!(iter.next(), Some((6, 6)));
        assert_eq!(iter.next(), Some((62, 62)));
    }

    #[test]
    fn test_bt_bpt_random() {
        test_dict!(BPT::<u16, u16, 3>::new());
        println!("pass..M=3");

        test_dict!(BPT::<u16, u16, 4>::new());
        println!("pass..M=4");

        test_dict!(BPT::<u16, u16, 5>::new());
        println!("pass..M=5");

        test_dict!(BPT::<u16, u16, 9>::new());
        println!("pass..M=9");

        test_dict!(BPT::<u16, u16, 11>::new());
        println!("pass..M=11");

        test_dict!(BPT::<u16, u16, 20>::new());
        println!("pass..M=20");
    }

    #[test]
    fn test_bt_bpt_select_random() {
        verify_select!(BPT::<u16, u16, 3>::new());
        verify_select!(BPT::<u16, u16, 5>::new());
        verify_select!(BPT::<u16, u16, 10>::new());
        verify_select!(BPT::<u16, u16, 21>::new());
    }

    #[test]
    fn test_bt_bpt_bulk_random() {
        verify_bulk!(BPT::<u16, u16, 3>::new());
        verify_bulk!(BPT::<u16, u16, 5>::new());
        verify_bulk!(BPT::<u16, u16, 10>::new());
        verify_bulk!(BPT::<u16, u16, 21>::new());
    }

    #[test]
    fn test_bt_bpt_properties() {
        verify_bpt_properties!(BPT::<u16, u16, 3>::new());
        verify_bpt_properties!(BPT::<u16, u16, 5>::new());
        verify_bpt_properties!(BPT::<u16, u16, 10>::new());
        verify_bpt_properties!(BPT::<u16, u16, 21>::new());
    }

    #[test]
    fn test_bt_bpt_pop() {
        verify_bpt_pop!(BPT::<u16, u16, 3>::new());
        println!("pass..M=3");

        verify_bpt_pop!(BPT::<u16, u16, 5>::new());
        println!("pass..M=5");

        verify_bpt_pop!(BPT::<u16, u16, 10>::new());
        println!("pass..M=10");

        verify_bpt_pop!(BPT::<u16, u16, 21>::new());
        println!("pass..M=21");
    }
}
