//! B+ Tree using B+ itself for index
//!


use std::{
    borrow::Borrow,
    fmt::*,
    ops::{Bound::*, RangeBounds},
};

use super::{
    bpt::{
        BPT, bpt
    },
    *,
};


impl_node!();
impl_tree!(
    /// B+ Trees powered by B+ tree
    ///
    BPT2 { cnt: usize, min_node: WeakNode<K, V> }
);

def_attr_macro!(call | paren, succ, entries, children);

const SUB_M: usize = 20;


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
        aux_node!(ENUM Leaf {
            entries: $entries,
            succ: $succ,
            paren: $paren
        })
    }};
    (basic-internal| $children:expr, $paren:expr) => {{
        aux_node!(ENUM Internal {
            children: $children,
            paren: $paren
        })
    }};
}


macro_rules! min_key {
    ($x:expr) => {
        if let Some(k) = $x.min_key() {
            k.clone()
        } else {
            panic!("No min-key {:?}", $x)
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
        } else {
            match range.start_bound() {
                Included(&k) => {
                    cur = Self::search_to_leaf_r(&self.root, k.borrow());

                    if cur.is_none() {
                        cur = self.min_node.upgrade();
                    }
                }
                Excluded(_) => unimplemented!(),
                Unbounded => {
                    cur = self.min_node.upgrade();
                }
            }
        }

        std::iter::from_coroutine(#[coroutine] move || {
            while cur.is_some() {
                let mut entries = entries_mut!(cur).select_mut(range.clone());

                if let Some(fst) = entries.next() {
                    yield fst;

                    for ent in entries {
                        yield ent
                    }

                    cur = succ!(cur);
                } else {
                    break;
                }
            }
        })
    }

    pub fn insert(&mut self, k: K, v: V) -> Option<V>
    where
        K: Clone + Debug,
        V: Debug,
    {
        let x = Self::search_to_leaf_r(&self.root, &k);

        self.insert_into_leaf(x, k, v)
    }

    pub fn remove<Q>(&mut self, k: &Q) -> Option<V>
    where
        K: Borrow<Q> + Clone + Debug,
        Q: Ord + ?Sized,
        V: Debug,
    {
        let x = Self::search_to_leaf_r(&self.root, k);

        self.remove_on_leaf(x, k)
    }


    ////////////////////////////////////////////////////////////////////////////
    //// Assistant Method

    fn remove_on_leaf<Q>(&mut self, x: Node<K, V>, k: &Q) -> Option<V>
    where
        K: Borrow<Q> + Clone + Debug,
        Q: Ord + ?Sized,
        V: Debug,
    {
        if x.is_none() {
            return None;
        }

        debug_assert!(x.is_leaf());

        let old_k = min_key!(x);

        let popped = entries_mut!(x).remove(k);

        if popped.is_some() {
            self.cnt -= 1;
        }

        if entries!(x).len() == 0 {
            if self.cnt == 0 {
                self.root = Node::none();
                self.min_node = WeakNode::none();
            } else {
                self.unpromote(x, old_k);
            }
        } else {
            Self::update_index(old_k, &x);
        }

        popped
    }

    fn insert_into_leaf(&mut self, mut x: Node<K, V>, k: K, v: V) -> Option<V>
    where
        K: Clone + Debug,
        V: Debug,
    {
        /* new min none */

        if x.is_none() {
            if self.cnt == 0 {
                self.root = node!(kv | k, v);
                self.min_node = self.root.downgrade();

                self.cnt += 1;

                return None;
            } else {
                x = self.min_node.upgrade();
            }
        }

        /* insert into leaf */

        debug_assert!(x.is_leaf());

        let old_k = min_key!(x);

        let popped = entries_mut!(x).insert(k, v);

        Self::update_index(old_k, &x);

        if entries!(x).len() == M {
            self.promote(x);
        }

        if popped.is_none() {
            self.cnt += 1;
        }

        popped
    }


    /// search to leaf restricted version (with short-circuit evaluation)
    #[inline(always)]
    fn search_to_leaf_r<Q>(mut x: &Node<K, V>, k: &Q) -> Node<K, V>
    where
        K: Borrow<Q> + Ord,
        Q: Ord + ?Sized,
    {
        while x.is_internal() {
            if let Some(x_) = children!(x).low_bound_search(k) {
                x = x_;
            } else {
                return Node::none();
            }
        }

        x.clone()
    }

    // fn nodes(&self) -> impl Iterator<Item = Node<K, V>> {
    //     let mut cur = self.min_node.upgrade();

    //     std::iter::from_coroutine(move || {
    //         while cur.is_some() {
    //             yield cur.clone();

    //             cur = succ!(cur);
    //         }
    //     })
    // }
    #[inline(always)]
    fn update_index(old_k: K, x: &Node<K, V>)
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

            Self::update_index(old_p_k, &p);
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
            debug_assert_eq!(
                children!(x).len(),
                Self::entries_high_bound() + 1
            );

            let children_x2 = children_mut!(x).split_off((M + 1).div_floor(2));

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
            let children = bpt! {
                x_k => x,
                x2_k => x2
            };

            self.root = node!(basic - internal | children, WeakNode::none());

            for child in children_mut!(self.root).values() {
                paren!(child, self.root.clone());
            }
        }
        /* insert into paren node */
        else {
            paren!(x2, p.clone());

            // insert new or update
            children_mut!(p).insert(x2_k, x2);

            if children!(p).len() == Self::entries_high_bound() + 1 {
                self.promote(p);
            }
        }
    }

    fn unpromote(&mut self, mut x: Node<K, V>, x_k: K)
    where
        K: Clone + Debug,
        V: Debug,
    {
        debug_assert!(paren!(x).is_some());
        debug_assert!(
            x.is_leaf()
                || children!(x).len() == Self::entries_low_bound() - 1 + 1
        );

        let p = paren!(x);
        let idx = children!(p).rank(&x_k).unwrap();

        if Self::try_rebalancing(&p, &x, idx, x_k.clone()) {
            return;
        }

        // merge with left_sib (no need to update index)
        if idx > 0 {
            let (_sib_k, sib_lf) = children!(p).nth(idx - 1).unwrap();

            children_mut!(p).remove(&x_k);

            if x.is_leaf() {
                succ!(sib_lf, succ!(x));
            } else {
                debug_assert!(x.is_internal());

                // let x_inner = unwrap_into!(x);
                // let (children, _) = x_inner.unpack_as_internal();

                for (child_k, child) in children_mut!(x).drain_all() {
                    paren!(child, sib_lf.clone());
                    children_mut!(sib_lf).push_back(child_k, child);
                }
            }
        }
        // for right_sib (merge into left)
        else {
            let (_sib_k, sib_rh) = children!(p).nth(idx + 1).unwrap();
            let sib_rh = sib_rh.clone();

            // 由于没有prev，只能总是合并到左节点，好在此时叶子节点只有一个元素
            if x.is_leaf() {
                debug_assert_eq!(entries!(sib_rh).len(), 1);

                succ!(x, succ!(sib_rh));

                let (k, v) = entries_mut!(sib_rh).pop_first().unwrap();
                entries_mut!(x).insert(k.clone(), v);

                // remove x from p
                Self::update_index(x_k, &x);

                // 不需要更新索引，因为 sib_rh 值更大
            } else {
                let old_key_sib_rh = min_key!(sib_rh);
                children_mut!(p).remove(&old_key_sib_rh);

                for (child_k, child) in children_mut!(sib_rh).drain_all() {
                    paren!(child, x.clone());
                    children_mut!(x).push_back(child_k, child);
                }

                Self::update_index(x_k, &x);
            }
        }

        let old_key = min_key!(p);
        x = p;

        if paren!(x).is_none() {
            if children!(x).len() == 1 {
                // pop new level
                self.root = children_mut!(x).pop_first().unwrap().1;
                paren!(self.root, Node::none());
            }
        } else {
            if children!(x).len() < Self::entries_low_bound() + 1 {
                self.unpromote(x, old_key);
            }
        }
    }

    fn try_rebalancing(
        p: &Node<K, V>,
        x: &Node<K, V>,
        idx: usize,
        old_k: K,
    ) -> bool
    where
        K: Clone + Debug,
        V: Debug,
    {
        // try to redistribute with left
        if idx > 0 {
            let (_sib_k, sib_lf) = children!(p).nth(idx - 1).unwrap();

            if sib_lf.is_leaf() && entries!(sib_lf).len() > 1 {
                let (k, v) = entries_mut!(sib_lf).pop_last().unwrap();
                entries_mut!(x).insert(k, v);

                Self::update_index(old_k, x);

                return true;
            }

            if sib_lf.is_internal()
                && children!(sib_lf).len() > Self::entries_low_bound() + 1
            {
                let (k, v) = children_mut!(sib_lf).pop_last().unwrap();
                paren!(v, x.clone());
                children_mut!(x).insert(k, v);

                Self::update_index(old_k, x);

                return true;
            }
        }
        // try to redistribute with right then
        if idx < children!(p).len() - 1 {
            let (_sib_k, sib_rh) = children!(p).nth(idx + 1).unwrap();
            let sib_rh = sib_rh.clone();

            if sib_rh.is_leaf() && entries!(sib_rh).len() > 1 {
                let (k, v) = entries_mut!(sib_rh).pop_first().unwrap();
                entries_mut!(x).insert(k.clone(), v);

                Self::update_index(old_k, x); // WARNING： it wll replace sib_rh with x
                children_mut!(p).insert(min_key!(sib_rh), sib_rh);

                return true;
            }

            if sib_rh.is_internal()
                && children!(sib_rh).len() > Self::entries_low_bound() + 1
            {
                let sib_rh_old_key = min_key!(sib_rh);

                let (k, v) = children_mut!(sib_rh).pop_first().unwrap();
                paren!(v, x.clone());
                children_mut!(x).insert(k.clone(), v);

                // 不需要像叶子节点那样两段儿更新是因为在叶子更新的时候，已经对 x 向上更新过了
                // Self::update_index(old_k, x);
                // children_mut!(p).insert(min_key!(sib_rh), sib_rh);
                Self::update_index(sib_rh_old_key, &sib_rh);

                return true;
            }
        }

        false
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

    // fn unpack_as_internal(self) -> (BPT<K, Node<K, V>, SUB_M>, WeakNode<K, V>)
    // where
    //     K: Ord + Debug,
    // {
    //     match self {
    //         Internal { children, paren } => (children, paren),
    //         Leaf { entries, .. } => {
    //             panic!("unpack leaf:{:?} as internal", entries)
    //         }
    //     }
    // }
}


impl<K: Debug + Ord, V> Debug for Node_<K, V> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Self::Internal { children, .. } => f
                .debug_struct("Internal")
                .field("children", children)
                .finish(),
            Self::Leaf { entries, .. } => {
                f.debug_struct("Leaf").field("entries", entries).finish()
            }
        }
    }
}


impl<K, V> Node<K, V> {
    fn is_leaf(&self) -> bool {
        self.is_some() && attr!(self | self).is_leaf()
    }

    fn is_internal(&self) -> bool {
        self.is_some() && !attr!(self | self).is_leaf()
    }

    #[allow(unused)]
    fn max_key(&self) -> Option<&K>
    where
        K: Ord,
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
        K: Ord,
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
impl<K: Ord + Debug + Clone, V, const M: usize> BPT2<K, V, M> {
    fn debug_print(&self)
    where
        V: Debug,
    {
        use common::vecdeq;

        /* print header */

        println!("{self:?}");

        /* print body */

        if self.root.is_none() {
            return;
        }

        let mut this_q = vecdeq![vec![self.root.clone()]];
        let mut lv = 1;

        while !this_q.is_empty() {
            println!();
            println!("############ Level: {lv} #############");
            println!();

            let mut nxt_q = vecdeq![];

            while let Some(children) = this_q.pop_front() {
                for (i, x) in children.iter().enumerate() {
                    let p = paren!(x);

                    if x.is_internal() {
                        nxt_q.push_back(
                            children!(x).values().cloned().collect(),
                        );
                        println!("({i:02}): {x:?} (p: [{p:?}])");
                    } else {
                        let succ = succ!(x);
                        println!(
                            "({i:02}): {x:?} (p: [{p:?}], succ: [{succ:?}])"
                        );
                    }
                }

                println!();
            }


            this_q = nxt_q;
            lv += 1;
        }

        println!("------------- end --------------\n");
    }

    fn validate(&self)
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
                        assert!(
                            children!(child).len()
                                < Self::entries_high_bound() + 1
                        );

                        // children!(child).validate();
                    } else {
                        assert!(
                            entries!(child).len() < Self::entries_high_bound()
                        );
                        // entries!(child).validate();
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

    use super::{super::tests::*, bpt::tests::*, *};
    use crate::bst::test_dict;

    #[test]
    fn test_bt_bpt2_case_1() {
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

        // dict.debug_print();

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
    fn test_bt_bpt2_case_2() {
        let mut dict = BPT2::<u16, u16, 5>::new();

        dict_insert!(dict, 22);
        dict_insert!(dict, 18);
        dict_insert!(dict, 12);
        dict_insert!(dict, 58);
        dict_insert!(dict, 20);

        dict_insert!(dict, 42);
        dict_insert!(dict, 34);
        dict_insert!(dict, 32);
        dict_insert!(dict, 13);
        dict_insert!(dict, 45);

        dict_insert!(dict, 10);
        dict_insert!(dict, 46);
        dict_insert!(dict, 53);
        dict_insert!(dict, 00);
        dict_insert!(dict, 27);


        dict_remove!(dict, 34);
        dict_remove!(dict, 13);
        dict_remove!(dict, 18);
        dict_remove!(dict, 00);
        dict_remove!(dict, 46);

        // dict.debug_print();

        dict_remove!(dict, 42);
        dict_remove!(dict, 22);

        // dict.debug_print();
    }

    #[test]
    fn test_bt_bpt2_random() {
        test_dict!(BPT2::<u16, u16, 3>::new());
        println!("Ok..M=3");

        test_dict!(BPT2::<u16, u16, 4>::new());
        println!("Ok..M=4");

        test_dict!(BPT2::<u16, u16, 5>::new());
        println!("Ok..M=5");

        test_dict!(BPT2::<u16, u16, 11>::new());
        println!("Ok..M=11");

        test_dict!(BPT2::<u16, u16, 20>::new());
        println!("Ok..M=20");

        test_dict!(BPT2::<u16, u16, 100>::new());
        println!("Ok..M=100");
    }

    #[test]
    fn test_bt_bpt2_select_random() {
        verify_select!(BPT2::<u16, u16, 3>::new());
        verify_select!(BPT2::<u16, u16, 5>::new());
        verify_select!(BPT2::<u16, u16, 10>::new());
        verify_select!(BPT2::<u16, u16, 21>::new());
    }
}
