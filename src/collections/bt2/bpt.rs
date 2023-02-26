//! B+ Tree
//!


use std::{
    borrow::Borrow,
    fmt::*,
    ops::{
        Bound::{self, *},
        RangeBounds,
    }
};

use m6coll::KVEntry;

use super::{
    super::bst2::{Left, Right},
    node as aux_node, *,
};

impl_node!();
impl_tree!(
    /// B+ Trees
    ///
    BPT {}
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


macro_rules! last_key {
    ($x:expr) => {{
        let x = $x;

        if $x.is_leaf() {
            &entries!(x).last().unwrap().0
        } else {
            &keys!(x).last().unwrap()
        }
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


macro_rules! search_to_leaf {
    ($x:expr, $k:expr) => {{
        let mut x = $x;
        let k = $k;

        while x.is_internal() {
            match keys!(x).binary_search_by_key(&k, |k_| k_.borrow()) {
                Ok(idx) => x = &children!(x)[idx + 1],
                Err(idx) => {
                    x = &children!(x)[idx];
                }
            }
        }

        x.clone()
    }};
}


/// WARNING: Search by Key O(logM)
macro_rules! index_of_child {
    ($p: expr, $child: expr) => {{
        let p = &$p;
        let child = $child;

        debug_assert!(child.is_some());

        match keys!(p).binary_search(last_key!(child)) {
            Ok(idx) => idx + 1,
            Err(idx) => idx,
        }
    }};
}


macro_rules! children_revref {
    ($x:expr) => {{
        let x = $x;
        for child in children_mut!(x) {
            paren!(child, x.clone());
        }
    }};
}


/// Node_ heap data field access
macro_rules! def_node__heap_access {
    (internal, $name:ident, $ret:ty) => {
        fn $name(&self) -> &$ret {
            match self {
                Internal { $name, .. } => $name,
                Leaf {..} => unreachable!(
                    "Get `{}` on leaf",
                    stringify!($name)
                )
            }
        }
        paste::paste!(
            fn [<$name _mut>](&mut self) -> &mut $ret {
                match self {
                    Internal { $name, .. } => $name,
                    Leaf {..} => unreachable!(
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
                Internal {..} => unreachable!(
                    "Get `{}` on internal node",
                    stringify!($name)
                ),
                Leaf { $name, ..} => $name
            }
        }
        paste::paste!(
            fn [<$name _mut>](&mut self) -> &mut $ret {
                match self {
                    Internal {..} => unreachable!(
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
        paste::paste!(
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
                Internal {..} => unreachable!(
                    "Get `{}` on internal node",
                    stringify!($name)
                ),
                Leaf { $name, .. } => $name,
            }
            .upgrade()
        }
        paste::paste!(
            fn [<set_ $name>](&mut self, x: Node<K, V>) {
                match self {
                    Internal {..} => unreachable!(
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
            paste::paste!(
                macro_rules! $name {
                    ($node:expr) => {
                        attr!(self | $$node).$name()
                    };
                    ($node:expr, $val:expr) => {
                        attr!(self_mut | $$node).[<set_ $name>]($$val)
                    };
                }
            );

            paste::paste!(
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


def_attr_macro_bpt!(paren, succ, keys, entries, children);


/// (parent, left-idx, sib_dir)
macro_rules! try_node_redistribution {
    ($p:expr, $idx:expr, $sib_dir:expr) => {{
        let p = &$p;
        let idx = $idx;
        let sib_dir = $sib_dir;

        let children = children!(p);

        let left = &children[idx];
        let right = &children[idx + 1];

        let sib = if sib_dir.is_left() { left } else { right };

        if sib.is_leaf() && entries!(sib).len() > 1 {
            if sib_dir.is_left() {
                entries_mut!(right).insert(
                    0,
                    entries_mut!(left).pop().unwrap()
                );
            }
            // sib is right
            else {
                entries_mut!(left).push(
                    entries_mut!(right).remove(0)
                );

                keys_mut!(p)[idx] = entries!(right)[0].0.clone();
            }

            return Ok(());
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

            return Ok(());
        }
    }};
}


/// (parent, left-idx)
macro_rules! merge_node {
    ($p:expr, $idx:expr) => {{
        let p = &$p;
        let idx = $idx;

        let children = children!(p);

        let left = children[idx].clone();
        let right = children[idx + 1].clone();

        // for leaf node
        if left.is_leaf() {
            entries_mut!(left).extend(entries_mut!(right).drain(..));

            // update succ
            succ!(left, succ!(right));
        }
        // for internal node
        else {
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
    }};
}


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

        Self { root: Node::none() }
    }

    pub fn get<Q>(&self, k: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.search_(k).map(|(node, idx)| &entries!(node)[idx].1)
    }

    pub fn get_mut<Q>(&mut self, k: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.search_(k)
            .map(|(node, idx)| &mut entries_mut!(node)[idx].1)
    }

    pub fn select<'a, Q, R: RangeBounds<&'a Q>>(
        &self,
        range: R,
    ) -> impl Iterator<Item = &V>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized + 'a,
    {
        /* find start_node */

        let (mut cur, idx) = self.search_bound_(range.start_bound().cloned());

        std::iter::from_generator(move || {
            if cur.is_none() {
                return;
            }

            let mut entries = &entries!(cur)[idx..];

            loop {
                for ent in entries {
                    if range.contains(&ent.0.borrow()) {
                        yield &ent.1
                    } else {
                        return;
                    }
                }

                cur = succ!(cur);

                if cur.is_none() {
                    break;
                }

                entries = entries!(cur);
            }
        })
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

        let (mut cur, idx) = self.search_bound_(range.start_bound().cloned());

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

                entries = entries_mut!(cur);
            }
        })
    }

    pub fn insert(&mut self, k: K, v: V) -> Option<V>
    where
        K: Clone,
    {
        let x = search_to_leaf!(&self.root, &k);

        /* NonInternal Node */

        /* Nil */
        if x.is_none() {
            self.root = node!(kv | k, v);
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

            if entries!(x).len() == M {
                self.promote(x);
            }
        }

        None
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
                },
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

        let p = paren!(x);

        /* Update internal key with its succsessor key */

        if let Some((internal, i_idx)) = internal_and_idx {
            let new_key;

            if entries!(x).len() > 1 {
                // left first
                if idx > 0 {
                    new_key = entries!(x)[idx-1].0.clone();
                }
                else {
                    new_key = entries!(x)[idx+1].0.clone();
                }
            }
            else {
                let x_idx = index_of_child!(p, x);

                /* check remain node */

                // left first
                if x_idx > 0 && entries!(children!(p)[x_idx - 1]).len() > 1 {
                    new_key = last_key!(&children!(p)[x_idx - 1]).clone();
                }
                // right sib
                else if x_idx < children!(p).len() - 1 && entries!(children!(p)[x_idx + 1]).len() > 1 {
                    new_key = entries!(succ!(x))[0].0.clone();
                }

                /* use default (left first)*/
                else if x_idx > 0 {
                    new_key = last_key!(&children!(p)[x_idx - 1]).clone();
                }
                else {
                    new_key = entries!(succ!(x))[0].0.clone();
                }
            }

            keys_mut!(internal)[i_idx] = new_key;
        }

        let popped = entries_mut!(x).remove(idx);

        if entries!(x).is_empty() {
            if p.is_none() {
                self.root = Node::none();
            } else {
                self.unpromote(x.clone());
            }
        }

        Some(popped.1)

    }

    ////////////////////////////////////////////////////////////////////////////
    //// Assistant Method

    fn min_node(&self) -> Node<K, V> {
        let mut x = &self.root;

        while x.is_internal() {
            x = &children!(x)[0];
        }

        x.clone()
    }

    fn search_bound_<'a, Q>(&self, bound: Bound<&'a Q>) -> (Node<K, V>, usize)
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized + 'a,
    {
        let mut cur;
        let mut idx = 0;

        match bound {
            Included(k) => {
                let x = search_to_leaf!(&self.root, k);

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
                cur = self.min_node();
            }
        }

        if cur.is_some() && idx == entries!(cur).len() {
            cur = succ!(cur);
            idx = 0;
        }

        (cur, idx)
    }

    fn search_<Q>(&self, k: &Q) -> Option<(Node<K, V>, usize)>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        let x = search_to_leaf!(&self.root, k);

        // Nil
        if x.is_none() {
            None
        }
        // Leaf
        else {
            debug_assert!(x.is_leaf());

            match entries!(x).binary_search_by_key(&k, |ent| ent.0.borrow()) {
                Ok(idx) => Some((x, idx)),
                Err(_idx) => None,
            }
        }
    }

    fn promote(&mut self, x: Node<K, V>)
    where
        K: Clone,
    {
        debug_assert!(x.is_some());

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

            succ!(x, x2.clone());
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


        let p = paren!(x);

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

    fn unpromote(&mut self, mut x: Node<K, V>)
    where
        K: Clone,
    {
        // Exclude leaf node and nil node
        debug_assert!(x.is_some());
        debug_assert!(
            x.is_leaf() || keys!(x).len() == Self::entries_low_bound() - 1
        );

        if let Err((p, idx)) = self.try_rebalancing(&x) {
            if idx == 0 {
                merge_node!(p, idx);
            } else {
                merge_node!(p, idx - 1);
            }

            x = p;

            if paren!(x).is_none() {
                // pop new level
                if keys!(x).is_empty() {
                    self.root = children_mut!(x).pop().unwrap();
                    paren!(self.root, Node::none());
                }
            } else {
                if keys!(x).len() < Self::entries_low_bound() {
                    self.unpromote(x);
                }
            }
        }
    }

    /// -> (idx, idx)
    fn try_rebalancing( &mut self, x: &Node<K, V>)
    -> std::result::Result<(), (Node<K, V>, usize)>
    where
        K: Clone,
    {
        debug_assert!(paren!(x).is_some());

        /* Check if siblings has remains */

        let p = paren!(x);
        let idx = index_of_child_by_rc!(p, x);

        // Left first
        if idx > 0 {
            try_node_redistribution!(p, idx - 1, Left);
        }

        if idx < children!(p).len() - 1 {
            try_node_redistribution!(p, idx, Right);
        }

        /* Or else just retrive from parent */

        if x.is_leaf() {
            if idx == 0 {
                keys_mut!(p).remove(0);
            } else {
                // merge right single entry node, remove its corresponding parent key
                keys_mut!(p).remove(idx - 1);
            }
        } else {
            if idx == 0 {
                keys_mut!(x).push(keys_mut!(p).remove(0));
            } else {
                keys_mut!(x).insert(0, keys_mut!(p).remove(idx - 1));
            }
        }

        Err((p, idx))
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
}


impl<K, V> Node<K, V> {
    fn is_leaf(&self) -> bool {
        self.is_some() && attr!(self | self).is_leaf()
    }

    fn is_internal(&self) -> bool {
        self.is_some() && !attr!(self | self).is_leaf()
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
        let mut q = crate::vecdeq![&self.root];

        while let Some(x) = q.pop_front() && x.is_internal() {
            for k in keys!(x) {
                let leaf = search_to_leaf!(x, k);

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
    where K: std::hash::Hash
    {
        let mut map = IndexMap::new();

        for x in self.leafs() {
            for k in entries!(x).iter().map(|ent| ent.0.clone()) {
                let cnt = self.count_key(&k);
                map.insert(k, cnt);
            }
        }

        map
    }

    fn leafs(&self) -> impl Iterator<Item = Node<K, V>> {
        let mut x = self.min_node();

        std::iter::from_generator(move || {
            while x.is_some() {
                yield x.clone();
                x = succ!(x);
            }
        })
    }

    /// Count key occurance across the tree
    fn count_key(&self, k: &K) -> usize {
        let mut count = 0;
        let mut x = &self.root;

        while x.is_internal() {
            match keys!(x).binary_search_by_key(&k, |k_| k_.borrow()) {
                Ok(idx) => {
                    count += 1;

                    // debug_assert!(x.lv() <= 3, "Found lv {}", x.lv());

                    x = &children!(x)[idx + 1];
                }
                Err(idx) => {
                    x = &children!(x)[idx];
                }
            }
        }

        if x.is_none() {
            unreachable!("There is no {k:?}")
        }

        match entries!(x).binary_search_by_key(
            &k, |ent| &ent.0
        )
        {
            Ok(idx) => {
                count += 1;
            }
            Err(idx) => unreachable!("There is no {k:?}"),
        };

        count
    }

    fn debug_write<W: std::fmt::Write>(&self, f: &mut W) -> std::fmt::Result
    where
        V: Debug,
    {
        /* print header */

        writeln!(f, "{self:?}")?;

        /* print body */

        if self.root.is_none() {
            return Ok(());
        }

        let mut this_q = crate::vecdeq![vec![self.root.clone()]];
        let mut lv = 1;

        while !this_q.is_empty() {
            writeln!(f)?;
            writeln!(f, "############ Level: {lv} #############")?;
            writeln!(f)?;

            let mut nxt_q = crate::vecdeq![];

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

    fn debug_print(&self)
    where
        V: Debug,
    {
        let mut cache = String::new();

        self.debug_write(&mut cache).unwrap();

        println!("{cache}")
    }

    fn validate(&self)
    where
        K: Debug,
    {
        if self.root.is_none() {
            return;
        }

        use std::collections::VecDeque;

        use crate::vecdeq;

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
                            let leaf = search_to_leaf!(child, k);

                            if leaf.is_none()
                                || entries!(leaf).binary_search_by_key(
                                    &k, |ent| &ent.0
                                ).is_err()
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
                            assert!(entries!(child).is_sorted());
                        } else {
                            assert!(keys!(child).is_sorted());
                        }

                        let child_max_key = last_key!(child);
                        let branch_key = key!(&p, i);

                        assert!(
                            child_max_key < branch_key,
                            "child: {child_max_key:?}, branch:{branch_key:?}"
                        );

                        if i > 0 {
                            assert!(key!(&p, i - 1) <= key!(child, 0));
                        }
                    }

                    assert!(last_key!(&p) <= key!(&last_child, 0));
                }
            }

            cur_q = nxt_q;
        }
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
mod tests {

    use super::{super::tests::*, *};
    use crate::collections::bst2::test_dict;

    macro_rules! assert_select_eq {
        ($dict:expr, $r:expr, $r2:expr) => {{
            use itertools::{EitherOrBoth::*, Itertools};

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

    macro_rules! test_select {
        ($dict:expr) => {{
            let mut dict = $dict;
            let get_lo = |hi| crate::algs::random_range((0..hi));

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


    #[test]
    fn test_bt2_bpt_case_1() {
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

        dict.debug_print();

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
        dict_remove!(dict, 35);
        dict_remove!(dict, 30);
        dict_remove!(dict, 28);

        // dict.debug_print();
    }

    #[test]
    fn test_bt2_bpt_random() {
        test_dict!(BPT::<u16, u16, 3>::new());
        test_dict!(BPT::<u16, u16, 4>::new());
        test_dict!(BPT::<u16, u16, 5>::new());
        test_dict!(BPT::<u16, u16, 11>::new());
        test_dict!(BPT::<u16, u16, 20>::new());
    }

    #[test]
    fn test_bt2_bpt_select_random() {
        test_select!(BPT::<u16, u16, 3>::new());
        test_select!(BPT::<u16, u16, 5>::new());
        test_select!(BPT::<u16, u16, 10>::new());
        test_select!(BPT::<u16, u16, 21>::new());
    }

    #[test]
    fn verify_bpt_properties() {
        let group = 50;
        let group_num = 100;

        for _ in 0..20 {

            let mut range: Vec<i32> = (1..=group_num).collect();

            use rand::{prelude::SliceRandom, thread_rng};

            range.shuffle(&mut thread_rng());

            let mut dict = BPT::<i32, i32, 3>::new();

            for pos in range {
                for i in pos..pos+group {
                    let k = pos * 100 + i;

                    dict.insert(k, k);
                }
            }

            let cnt_keys = dict.count_nodes_keys();

            let min_key_cnt = cnt_keys.values().min().unwrap();
            let max_key_cnt = cnt_keys.values().max().unwrap();

            // println!("min_key_cnt: {min_key_cnt}, max_key_cnt: {max_key_cnt}");
            assert_eq!(min_key_cnt, &1);
            assert_eq!(max_key_cnt, &2);
        }
    }

    #[test]
    fn test_bt2_bpt_case_2() {
        let mut dict = BPT::<u16, u16, 5>::new();

        dict_insert!(dict, 25);
        dict_insert!(dict, 62);
        dict_insert!(dict, 6);
        dict_insert!(dict, 2);
        dict_insert!(dict, 45);
        dict_insert!(dict, 11);
        dict_insert!(dict, 55);
        dict_insert!(dict, 51);

        dict.debug_print();

        dict_remove!(dict, 45);
        dict_remove!(dict, 55);
        dict_remove!(dict, 51);
        // dict_remove!(dict, 25);
        // dict_remove!(dict, 11);


        dict.debug_print();
    }

}
