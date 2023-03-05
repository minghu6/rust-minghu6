//! B+ Tree using B+ itself for index
//!


use std::{
    borrow::Borrow,
    cmp::Ordering::{self, *},
    fmt::*,
    ops::{
        Bound::{self, *},
        RangeBounds,
    },
};

use super::{
    super::bst2::{Left, Right},
    bpt::{
        def_attr_macro_bpt, def_node__heap_access, def_node__wn_access, BPT,
    },
    node as aux_node, *,
};


impl_node!();
impl_tree!(
    /// B+ Trees powered by B+ tree
    ///
    BPT2 { cnt: usize }
);

def_attr_macro_bpt!(paren, succ, keys, entries, children);

const SUB_M: usize = 10;


////////////////////////////////////////////////////////////////////////////////
//// Macro

macro_rules! node {
    (kv| $k:expr, $v:expr) => {{
        // overflow is M
        let mut entries = BPT::new();
        entries.insert($k, $v);

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


macro_rules! max_key {
    ($x:expr) => {{
        let x = $x.clone();

        if $x.is_leaf() {
            entries!(x).max_key().unwrap()
        } else {
            keys!(x).max_key().unwrap()
        }
    }};
}


macro_rules! min_key {
    ($x:expr) => {{
        let x = $x.clone();

        if $x.is_leaf() {
            entries!(x).min_key().unwrap()
        } else {
            entries!(x).min_key().unwrap()
        }
    }};
}


////////////////////////////////////////////////////////////////////////////////
//// Implementation

impl<K: Ord, V, const M: usize> BPT2<K, V, M> {
    ////////////////////////////////////////////////////////////////////////////
    //// Public API

    pub fn new() -> Self {
        assert!(M > 2, "M should be greater than 2");

        Self {
            root: Node::none(),
            cnt: 0,
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
        let x = self.root.search_to_leaf(k);

        // Nil
        if x.is_none() {
            None
        }
        // Leaf
        else {
            entries_mut!(x).get_mut(k)
        }
    }

    pub fn select<'a, Q, R>(
        &self,
        range: R,
    ) -> impl Iterator<Item = &V>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized + 'a,
        R: RangeBounds<&'a Q> + Clone
    {
        mut_self!(self).select_mut(range).map(|v| &*v)
    }

    pub fn select_mut<'a, Q, R>(
        &mut self,
        range: R,
    ) -> impl Iterator<Item = &mut V>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized + 'a,
        R: RangeBounds<&'a Q> + Clone
    {
        /* find start_node */

        let mut cur;
        match range.start_bound() {
            Included(&k) => {
                cur = self.root.search_to_leaf(k.borrow())
            }
            Excluded(_) => unimplemented!(),
            Unbounded => {
                cur = self.min_node()
            }
        }

        std::iter::from_generator(move || {
            while cur.is_some() {
                let mut entries = entries_mut!(cur)
                    .select_mut(range.clone())
                    .peekable();

                if entries.peek().is_none() {
                    return;
                }

                for ent in entries {
                    yield ent
                }

                cur = succ!(cur);
            }
        })
    }

    pub fn insert(&mut self, k: K, v: V) -> Option<V>
    where
        K: Clone,
    {
        let x = self.root.search_to_leaf(&k);

        /* NonInternal Node */

        /* Nil */
        if x.is_none() {
            self.root = node!(kv | k, v);
        }
        /* Leaf */
        else {
            /* insert into leaf */

            entries_mut!(x).insert(k, v);

            if entries!(x).len() == M {
                // self.promote(x);
            }
        }

        self.cnt += 1;

        None
    }

    ////////////////////////////////////////////////////////////////////////////
    //// Assistant Method

    fn nodes(&self) -> impl Iterator<Item = Node<K, V>> {
        let mut cur = self.min_node();

        std::iter::from_generator(move || {
            while cur.is_some() {
                yield cur.clone();

                cur = succ!(cur);
            }
        })
    }

    #[inline(always)]
    fn min_node(&self) -> Node<K, V> {
        let mut x = &self.root;

        while x.is_internal() {
            x = &children!(x).min_key().unwrap();
        }

        x.clone()
    }

    #[inline]
    fn max_node(&self) -> Node<K, V> {
        let mut x = &self.root;

        while x.is_internal() {
            x = &children!(x).max_key().unwrap();
        }

        x.clone()
    }

    // fn promote(&mut self, x: Node<K, V>)
    // where
    //     K: Clone,
    // {
    //     debug_assert!(x.is_some());

    //     /* split node */

    //     let lpos = M.div_floor(2);
    //     let hpos = M.div_ceil(2);

    //     let head_key;
    //     let x2;

    //     if x.is_leaf() {
    //         debug_assert_eq!(entries!(x).len(), Self::entries_high_bound());

    //         // keep key for leaf
    //         let entries_x2 = entries_mut!(x).split_off(lpos);

    //         head_key = entries_x2[0].0.clone();

    //         x2 = node!(
    //             basic - leaf | entries_x2,
    //             succ!(x).downgrade(),
    //             WeakNode::none()
    //         );

    //         succ!(x, x2.clone());
    //     } else {
    //         debug_assert_eq!(keys!(x).len(), Self::entries_high_bound());

    //         let keys_x2 = keys_mut!(x).split_off(hpos);

    //         head_key = keys_mut!(x).pop().unwrap();

    //         let children_x2 = children_mut!(x).split_off(hpos);

    //         x2 = node!(
    //             basic - internal | keys_x2,
    //             children_x2,
    //             WeakNode::none()
    //         );

    //         children_revref!(&x2);
    //     }


    //     let p = paren!(x);

    //     /* push new level */
    //     if p.is_none() {
    //         let keys = vec![head_key];
    //         let children = vec![x, x2];

    //         self.root =
    //             node!(basic - internal | keys, children, WeakNode::none());

    //         children_revref!(&self.root);
    //     }
    //     /* insert into paren node */
    //     else {
    //         keys_mut!(p).insert(head_key, ());
    //         children_mut!(p).insert(x_idx + 1, x2.clone());

    //         paren!(x2, p.clone());

    //         if keys!(p).len() == Self::entries_high_bound() {
    //             self.promote(p);
    //         }
    //     }
    // }

}


impl<K: Ord + Debug, V, const M: usize> Debug for BPT2<K, V, M> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.debug_struct("BPT2")
            .field("root", &self.root)
            .field("cnt", &self.cnt)
            .finish()
    }
}


////////////////////////////////////////////////////////////////////////////////
//// Structure

enum Node_<K, V> {
    Internal {
        keys: BPT<K, (), SUB_M>,
        children: BPT<Node<K, V>, (), SUB_M>,
        paren: WeakNode<K, V>,
    },
    Leaf {
        entries: BPT<K, V, SUB_M>,
        /// Successor (Leaf)
        succ: WeakNode<K, V>,
        paren: WeakNode<K, V>,
    },
}
use Node_::*;


impl<K, V> Node_<K, V> {
    fn is_leaf(&self) -> bool {
        matches!(self, Leaf { .. })
    }

    def_node__heap_access!(internal, keys, BPT<K, (), SUB_M>);
    def_node__heap_access!(internal, children, BPT<Node<K, V>, (), SUB_M>);
    def_node__heap_access!(leaf, entries, BPT<K, V, SUB_M>);

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

    #[inline(always)]
    fn search_to_leaf<Q>(&self, k: &Q) -> Self
    where
        K: Borrow<Q> + Ord,
        Q: Ord + ?Sized,
    {
        let mut x = self.clone();

        while x.is_internal() {
            let rk = keys!(x).rank(k);
            x = children!(x).nth(rk).unwrap().0.clone();
        }

        x.clone()
    }
}



impl<K, V> PartialEq for Node<K, V> {
    fn eq(&self, other: &Self) -> bool {
        self.rc_eq(other)
    }
}


impl<K, V> PartialOrd for Node<K, V> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(match (&self.0, &other.0) {
            (Some(rc1), Some(rc2)) => {
                (rc1.as_ptr() as usize).cmp(&(rc2.as_ptr() as usize))
            }
            (Some(_), None) => Greater,
            (None, Some(_)) => Less,
            (None, None) => Equal,
        })
    }
}


impl<K, V> Ord for Node<K, V> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}


impl<K, V> Eq for Node<K, V> {}


impl<K: Debug + Ord, V> Debug for Node<K, V> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        if self.is_leaf() {
            let mut peek = entries!(self).keys().peekable();

            while let Some(k) = peek.next() {
                write!(f, "{k:?}")?;

                if peek.peek().is_some() {
                    write!(f, ", ")?;
                }
            }
        } else if self.is_internal() {
            let mut peek = keys!(self).keys().peekable();

            while let Some(k) = peek.next() {
                write!(f, "{k:?}")?;

                if peek.peek().is_some() {
                    write!(f, ", ")?;
                }
            }
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
impl<K: Ord + Debug + Clone, V, const M: usize> BPT2<K, V, M> {
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
                        nxt_q
                            .push_back(children!(x).keys().cloned().collect());
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
                            let (br_lf, br_it) =
                                keys!(p).key_between(max_key!(child));

                            if let Some(br_lf) = br_lf {
                                assert_eq!(br_lf, min_key!(&child));
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
                        for k in keys!(child).keys() {
                            let leaf = child.search_to_leaf(k);

                            if leaf.is_none()
                                || entries!(leaf)
                                    .keys()
                                    .find(|&leafk| leafk == k)
                                    .is_none()
                            {
                                panic!("Found obsoleted key: {k:?}");
                            }
                        }

                        internal_num += 1;

                        let mut nxt_children = VecDeque::from_iter(
                            children!(child).keys().cloned(),
                        );
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
                        let child_max_key = max_key!(&child);
                        let (br_lf, br_rh) =
                            keys!(p).key_between(child_max_key);

                        assert!(child_max_key < br_rh.unwrap(),);

                        if i > 0 {
                            assert!(br_lf.unwrap() <= min_key!(&child));
                        }
                    }

                    assert!(max_key!(&p) <= min_key!(&last_child));
                }
            }

            cur_q = nxt_q;
        }
    }
}
