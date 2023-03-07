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
        def_attr_macro_bpt,
        def_node__heap_access,
        def_node__wn_access,
        BPT,
        bpt
    },
    node as aux_node, *,
};


impl_node!();
impl_tree!(
    /// B+ Trees powered by B+ tree
    ///
    BPT2 { cnt: usize, min_node: WeakNode<K, V> }
);

def_attr_macro_bpt!(paren, succ, entries, children);

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
    (basic-internal| $children:expr, $paren:expr) => {{
        aux_node!(FREE-ENUM Internal {
            children: $children,
            paren: $paren
        })
    }};
}


macro_rules! min_key {
    ($x:expr) => {
        if let Some(k) = $x.min_key() {
            k.clone()
        }
        else {
            unreachable!("No min-key {:?}", $x)
        }
    };
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
            min_node: WeakNode::none(),
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
        let x = Self::search_to_leaf_r(&self.root, k);

        // Nil
        if x.is_none() {
            None
        }
        // Leaf
        else {
            entries_mut!(x).get_mut(k)
        }
    }

    pub fn select<'a, Q, R>(&self, range: R) -> impl Iterator<Item = &V>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized + 'a,
        R: RangeBounds<&'a Q> + Clone,
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
        R: RangeBounds<&'a Q> + Clone,
    {
        /* find start_node */

        let mut cur;

        if self.cnt == 0 {
            cur = Node::none();
        }
        else {
            match range.start_bound() {
                Included(&k) => {
                    cur = Self::search_to_leaf_r(&self.root, k.borrow());

                    if cur.is_none() {
                        cur = self.min_node.upgrade();
                    }
                },
                Excluded(_) => unimplemented!(),
                Unbounded => {
                    cur = self.min_node.upgrade();
                }
            }
        }

        std::iter::from_generator(move || {
            while cur.is_some() {
                let mut entries =
                    entries_mut!(cur).select_mut(range.clone()).peekable();

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
        K: Clone + Debug,
        V: Debug,
    {

        /* empty none */
        if self.root.is_none() {
            self.root = node!(kv | k, v);
            self.min_node = self.root.downgrade();

            self.cnt += 1;

            return None;
        }

        let mut x = Self::search_to_leaf_r(&self.root, &k);

        /* new min none */

        if x.is_none() {
            x = self.min_node.upgrade();
        }

        /* insert into leaf */

        debug_assert!(x.is_leaf());

        let old_k = min_key!(x);

        let popped = entries_mut!(x).insert(k, v);

        Self::update_index(old_k, x.clone());

        if entries!(x).len() == M {
            self.promote(x);
        }

        if popped.is_none() {
            self.cnt += 1;
        }

        popped
    }

    ////////////////////////////////////////////////////////////////////////////
    //// Assistant Method

    /// search to leaf restricted version (with short-circuit evaluation)
    #[inline(always)]
    fn search_to_leaf_r<Q>(mut x: &Node<K, V>, k: &Q) -> Node<K, V>
    where
        K: Borrow<Q> + Ord,
        Q: Ord + ?Sized,
    {
        while x.is_internal() {
            if let Some(x_) = children!(x).approx_search(k) {
                x = x_;
            } else {
                return Node::none();
            }
        }

        x.clone()
    }

    // fn nodes(&self) -> impl Iterator<Item = Node<K, V>> {
    //     let mut cur = self.min_node.upgrade();

    //     std::iter::from_generator(move || {
    //         while cur.is_some() {
    //             yield cur.clone();

    //             cur = succ!(cur);
    //         }
    //     })
    // }
    fn update_index(old_k: K, x: Node<K, V>)
    where
        K: Clone + Debug,
    {
        let k = min_key!(x);
        let p = paren!(x);

        if old_k != k && p.is_some() {
            /* update x key */

            let old_p_k = min_key!(p);

            children_mut!(p).remove(&old_k);
            children_mut!(p).insert(k.clone(), x.clone());

            Self::update_index(old_p_k, p);
        }
    }

    fn promote(&mut self, x: Node<K, V>)
    where
        K: Clone + Debug,
        V: Debug,
    {
        debug_assert!(x.is_some());

        /* split node */

        let x2;

        if x.is_leaf() {
            debug_assert_eq!(entries!(x).len(), Self::entries_high_bound());

            // keep key for leaf
            let entries_x2 = entries_mut!(x).split_off(M.div_floor(2));

            x2 = node!(
                basic - leaf | entries_x2,
                succ!(x).downgrade(),
                WeakNode::none()
            );

            succ!(x, x2.clone());
        } else {
            debug_assert_eq!(children!(x).len(), Self::entries_high_bound() + 1);

            let children_x2 = children_mut!(x).split_off((M+1).div_floor(2));

            x2 = node!(basic - internal | children_x2, WeakNode::none());

            for child in children_mut!(x2).values() {
                paren!(child, x2.clone());
            }
        }

        let p = paren!(x);
        let x_k = min_key!(x);
        let x2_k = min_key!(x2);

        /* push new level */
        if p.is_none() {
            let children = bpt!{
                x_k => x,
                x2_k => x2
            };

            self.root =
                node!(basic - internal | children, WeakNode::none());

            for child in children_mut!(self.root).values() {
                paren!(child, self.root.clone());
            }
        }
        /* insert into paren node */
        else {
            paren!(x2, p.clone());

            // insert new or update
            children_mut!(p).insert(
                x2_k,
                x2
            );

            if children!(p).len() == Self::entries_high_bound() + 1 {
                self.promote(p);
            }
        }
    }


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
        /// 维护 K 总为Node最小值
        /// 维护： 可能需要更新index在insert、remove，查询可以短路终止
        /// 不维护： 没有短路终止而且查询量加倍
        ///
        /// 还是维护的好
        children: BPT<K, Node<K, V>, SUB_M>,
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

    def_node__heap_access!(internal, children, BPT<K, Node<K, V>, SUB_M>);
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

    fn max_key(&self) -> Option<&K>
    where
        K: Ord
    {
        if self.is_none() {
            None
        } else if self.is_internal() {
            children!(self).max_key()
        } else {
            entries!(self).max_key()
        }
    }

    fn min_key(&self) -> Option<&K>
    where
        K: Ord
    {
        if self.is_none() {
            None
        } else if self.is_internal() {
            children!(self).min_key()
        } else {
            entries!(self).min_key()
        }
    }
}


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
            let mut peek = children!(self).keys().peekable();

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
                        nxt_q.push_back(
                            children!(x).values().cloned().collect(),
                        );
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
                        assert!(
                            children!(child).len() < Self::entries_high_bound() + 1
                        );
                    } else {
                        assert!(
                            entries!(child).len() < Self::entries_high_bound()
                        );
                    }

                    // Exclude leaf
                    if child.is_leaf() {
                        leaf_num += 1;
                    } else {
                        // Exclude the root (which is always one when it's internal node)
                        if paren!(child).is_some() {
                            assert!(
                                children!(child).len()
                                    >= Self::entries_low_bound() + 1
                            );
                        } else {
                            assert!(children!(child).len() >= 2);
                        }

                        internal_num += 1;

                        let mut nxt_children = VecDeque::from_iter(
                            children!(child).values().cloned(),
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
            }

            cur_q = nxt_q;
        }
    }
}



#[cfg(test)]
mod tests {

    use super::{super::tests::*, *, bpt::tests::*};
    use crate::collections::bst2::test_dict;

    #[test]
    fn test_bt2_bpt2_case_1() {
        let mut dict = BPT2::<u16, u16, 3>::new();

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

        dict_insert!(dict, 52, 520);
        dict_insert!(dict, 47, 470);
        dict_insert!(dict, 3, 30);

        // dict.debug_print();
    }

    #[test]
    fn test_bt2_bpt2_case_2() {
        let mut dict = BPT2::<u16, u16, 3>::new();

        dict_insert!(dict, 54);
        dict_insert!(dict, 48);
        dict_insert!(dict, 36);
        dict_insert!(dict, 18);
        dict_insert!(dict, 24);
        dict_insert!(dict, 27);
        dict_insert!(dict, 07);
        dict_insert!(dict, 18);

        dict_get!(dict, 54);
        dict_get!(dict, 48);
        dict_get!(dict, 36);
        dict_get!(dict, 18);
        dict_get!(dict, 24);
        dict_get!(dict, 27);
        dict_get!(dict, 07);
        dict_get!(dict, 18);


        dict.debug_print();
    }

    #[test]
    fn test_bt2_bpt2_random() {
        test_dict!(BPT2::<u16, u16, 3>::new());
        test_dict!(BPT2::<u16, u16, 4>::new());
        test_dict!(BPT2::<u16, u16, 5>::new());
        test_dict!(BPT2::<u16, u16, 11>::new());
        test_dict!(BPT2::<u16, u16, 20>::new());
        test_dict!(BPT2::<u16, u16, 100>::new());
    }

    // #[test]
    // fn test_bt2_bpt_select_random() {
    //     verify_select!(BPT::<u16, u16, 3>::new());
    //     verify_select!(BPT::<u16, u16, 5>::new());
    //     verify_select!(BPT::<u16, u16, 10>::new());
    //     verify_select!(BPT::<u16, u16, 21>::new());
    // }
}
