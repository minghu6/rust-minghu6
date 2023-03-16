//! B Tree
//!

use std::{borrow::Borrow, fmt::*, mem::swap};

use coll::{node as aux_node, *};

use crate::{
    bst::{Left, Right},
    bt::*,
};


def_attr_macro!(clone | paren);
def_attr_macro!(ref|
    (entries, Vec<KVEntry<K, V>>),
    (children, Vec<Node<K, V>>)
);

impl_node!();
impl_tree!(
    /// B-Trees
    ///
    /// Panic: M > 2
    ///
    /// Recommend: maybe 60, 90, 250
    /// (Rust use M=12 (B=6, M=2B-1+1) maybe increase it in the futer)
    #[derive(Debug)]
    BT {}
);


////////////////////////////////////////////////////////////////////////////////
//// Macros

macro_rules! node {
    (kv| $k:expr, $v:expr) => {{
        // overflow is M
        let mut entries = Vec::with_capacity(M);
        entries.push(KVEntry($k, $v));

        let children = vec![Node::none(); 2];

        node!(basic | entries, children, WeakNode::none())
    }};
    (basic| $entries:expr, $children:expr, $paren:expr) => {{
        aux_node!(FREE {
            entries: $entries,
            children: $children,
            paren: $paren
        })
    }};
}


macro_rules! key {
    ($x:expr, $idx:expr) => {
        &entries!($x)[$idx].0
    };
}



/// O(logM) search by key
macro_rules! index_of_child {
    ($p: expr, $child: expr) => {{
        let p = &$p;
        let child = &$child;

        debug_assert!(child.is_some());

        match entries!(p).binary_search(child.last_entry()) {
            Ok(oldidx) => {
                panic!("Dup key on {oldidx}");
            }
            Err(inseridx) => inseridx,
        }
    }};
}


macro_rules! children_revref {
    ($x:expr) => {{
        let x = &$x;
        let children = children_mut!(x);

        if children[0].is_some() {
            for child in children {
                paren!(child, x.downgrade());
            }
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

        // merge right's children to the left

        let left_children = children_mut!(left);

        for child in children_mut!(right).drain(..) {
            if child.is_some() {
                paren!(child, left.downgrade());
            }

            left_children.push(child);
        }

        // merge entries
        entries_mut!(left).extend(entries_mut!(right).drain(..));

        // remove right from p
        children_mut!(p).remove(idx + 1);
    }};
}


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

        if children!(sib)[0].is_none() && entries!(sib).len() > 1
            || entries!(sib).len() > Self::entries_low_bound()
        {
            if sib_dir.is_left() {
                entries_mut!(right).insert(
                    0,
                    replace(
                        &mut entries_mut!(p)[idx],
                        entries_mut!(left).pop().unwrap(),
                    ),
                );

                let child = children_mut!(left).pop().unwrap();

                if child.is_some() {
                    paren!(child, right.downgrade());
                }

                children_mut!(right).insert(0, child);
            } else {
                entries_mut!(left).push(replace(
                    &mut entries_mut!(p)[idx],
                    entries_mut!(right).remove(0),
                ));

                let child = children_mut!(right).remove(0);

                if child.is_some() {
                    paren!(child, left.downgrade());
                }

                children_mut!(left).push(child);
            }

            return Ok(());
        }
    }};
}



////////////////////////////////////////////////////////////////////////////////
//// Structure

struct Node_<K, V> {
    entries: Vec<KVEntry<K, V>>,
    /// 即使是叶子节点，也要保持孩子数量 = k-v 数量 + 1
    children: Vec<Node<K, V>>,
    paren: WeakNode<K, V>,
}



////////////////////////////////////////////////////////////////////////////////
//// Implementation

impl<K: Ord, V, const M: usize> BT<K, V, M> {
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
        self.root
            .search(k)
            .map(|(node, idx)| &entries!(node)[idx].1)
    }

    pub fn get_mut<Q>(&mut self, k: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.root
            .search(k)
            .map(|(node, idx)| &mut entries_mut!(node)[idx].1)
    }

    pub fn insert(&mut self, k: K, v: V) -> Option<V> {
        let mut y = &Node::none();
        let mut x = &self.root;
        let mut idx = 0;

        while x.is_some() {
            match entries!(x).binary_search_by_key(&&k, |ent| &ent.0) {
                Ok(idx_) => {
                    idx = idx_;

                    break;
                }
                Err(idx_) => {
                    idx = idx_;

                    y = x;
                    x = &children!(y)[idx];
                }
            }
        }

        if x.is_some() {
            return Some(replace(&mut entries_mut!(x)[idx].1, v));
        }

        if y.is_none() {
            self.root = node!(kv | k, v);
        } else {
            /* insert into leaf */

            entries_mut!(y).insert(idx, KVEntry(k, v));
            children_mut!(y).push(Node::none());

            if entries!(y).len() == M {
                self.promote(y.clone());
            }
        }

        None
    }

    pub fn remove<Q>(&mut self, k: &Q) -> Option<V>
    where
        K: Borrow<Q> + Debug,
        Q: Ord + ?Sized,
        V: Debug,
    {
        if let Some((mut x, mut idx)) = self.root.search(k) {
            /* Swap to its successor leaf node */

            if children!(x)[0].is_some() {
                let (succ, succ_idx) = children!(x)[idx + 1].minimum();

                swap(
                    &mut entries_mut!(x)[idx],
                    &mut entries_mut!(succ)[succ_idx],
                );

                x = succ;
                idx = succ_idx;
            }

            debug_assert!(children!(x)[0].is_none());

            let popped = entries_mut!(x).remove(idx);
            children_mut!(x).pop();

            if entries!(x).is_empty() {
                if paren!(x).is_none() {
                    self.root = Node::none();
                } else {
                    self.unpromote(x);
                }
            }

            Some(popped.1)
        } else {
            None
        }
    }



    ////////////////////////////////////////////////////////////////////////////
    //// Assistant Method

    /// 漂亮的尾递归
    fn promote(&mut self, x: Node<K, V>) {
        debug_assert_eq!(entries!(x).len(), Self::entries_high_bound());

        /* split node */

        let split_pos = M.div_ceil(2);
        let entries_x = entries_mut!(x);

        let entries_x2 = entries_x.split_off(split_pos);
        let entry_head = entries_x.pop().unwrap();

        let children_x2 = children_mut!(x).split_off(split_pos);

        let x2 = node!(basic | entries_x2, children_x2, WeakNode::none());
        children_revref!(x2);

        let p = paren!(x).upgrade();

        if p.is_none() {
            /* push new level */

            let entries = vec![entry_head];
            let children = vec![x, x2];

            self.root = node!(basic | entries, children, WeakNode::none());

            children_revref!(self.root);
        } else {
            /* insert into paren node */

            let x_idx = index_of_child!(p, x);

            entries_mut!(p).insert(x_idx, entry_head);
            children_mut!(p).insert(x_idx + 1, x2.clone());

            paren!(x2, p.downgrade());

            if entries!(p).len() == Self::entries_high_bound() {
                self.promote(p);
            }
        }
    }

    fn unpromote(&mut self, x: Node<K, V>) {
        // Exclude leaf node
        debug_assert!(
            children!(x)[0].is_none()
                || entries!(x).len() == Self::entries_low_bound() - 1
        );

        if let Err((p, idx)) = self.try_rebalancing(&x) {
            /* merge with sib node (rebalance failed means that each node are small) */

            // merge left child
            if idx > 0 {
                entries_mut!(x).insert(0, entries_mut!(p).remove(idx - 1));
                merge_node!(p, idx - 1);
            }
            // merge right child
            else {
                entries_mut!(x).push(entries_mut!(p).remove(0));
                merge_node!(p, idx);
            }

            if paren!(p).is_none() {
                // pop new level
                if entries!(p).is_empty() {
                    debug_assert!(children!(p)[0].is_some());
                    debug_assert_eq!(children!(p).len(), 1);

                    self.root = children_mut!(p).pop().unwrap();
                    paren!(self.root, WeakNode::none());
                }
            } else {
                if entries!(p).len() < Self::entries_low_bound() {
                    self.unpromote(p);
                }
            }
        }
    }

    /// -> (idx, idx)
    fn try_rebalancing(
        &mut self,
        x: &Node<K, V>,
    ) -> std::result::Result<(), (Node<K, V>, usize)> {
        debug_assert!(paren!(x).is_some());

        let p = paren!(x).upgrade();
        let idx = index_of_child_by_rc!(p, x);

        /* Check if siblings has remains */

        if idx > 0 {
            try_node_redistribution!(p, idx - 1, Left);
        }

        if idx < children!(p).len() - 1 {
            try_node_redistribution!(p, idx, Right);
        }

        Err((p, idx))
    }
}


impl<K, V> Node<K, V> {
    #[track_caller]
    fn last_entry(&self) -> &KVEntry<K, V> {
        if let Some(ent) = entries!(self).last() {
            ent
        } else {
            panic!("EMPTY entries");
        }
    }

    /// 漂亮的尾递归
    fn search<Q>(&self, k: &Q) -> Option<(Self, usize)>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        if self.is_some() {
            match entries!(self).binary_search_by_key(&k, |ent| ent.0.borrow())
            {
                Ok(idx) => Some((self.clone(), idx)),
                Err(idx) => children!(self)[idx].search(k),
            }
        } else {
            None
        }
    }

    /// Left most
    fn minimum(&self) -> (Self, usize) {
        let mut x = self;
        let mut y = Node::none();

        while x.is_some() {
            y = x.clone();
            x = children!(x).first().unwrap();
        }

        (y, 0)
    }
}


impl<K: Debug, V> Debug for Node<K, V> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        if self.is_some() {
            let kn = entries!(self).len();

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



#[cfg(test)]
mod tests {

    use super::{super::tests::*, *};
    use crate::bst::test_dict;

    impl<K: Ord, V, const M: usize> BT<K, V, M> {
        ////////////////////////////////////////////////////////////////////////////
        //// Test Method

        #[cfg(test)]
        fn debug_write<W: std::fmt::Write>(
            &self,
            f: &mut W,
        ) -> std::fmt::Result
        where
            K: std::fmt::Debug,
            V: std::fmt::Debug,
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
                        let p = paren!(x).upgrade();

                        if x.is_some() && children!(x)[0].is_some() {
                            nxt_q.push_back(children!(x).clone());
                        }

                        writeln!(f, "({i:02}): {x:?} (p: {p:?})")?;
                    }

                    writeln!(f)?;
                }


                this_q = nxt_q;
                lv += 1;
            }

            writeln!(f, "------------- end --------------\n")?;

            Ok(())
        }

        #[cfg(test)]
        fn debug_print(&self)
        where
            K: std::fmt::Debug,
            V: std::fmt::Debug,
        {
            let mut cache = String::new();

            self.debug_write(&mut cache).unwrap();

            println!("{cache}")
        }

        #[cfg(test)]
        fn validate(&self)
        where
            K: Debug,
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

                        assert_eq!(
                            entries!(child).len() + 1,
                            children!(child).len(),
                            "{child:?}"
                        );
                        assert!(
                            children!(child).len() <= M,
                            "{child:?}: {}",
                            children!(child).len()
                        );
                        assert!(
                            entries!(child).len() < Self::entries_high_bound()
                        );

                        // Exclude leaf
                        if children!(child)[0].is_none() {
                            assert!(entries!(child).len() > 0);
                            leaf_num += 1;
                        } else {
                            // Exclude the root (which is always one when it's internal node)
                            if paren!(child).is_some() {
                                assert!(
                                    entries!(child).len()
                                        >= Self::entries_low_bound()
                                );
                            } else {
                                assert!(children!(child).len() >= 2);
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
                            assert!(entries!(child).is_sorted());

                            let child_max_key = &child.last_entry().0;
                            let branch_key = key!(p, i);

                            assert!(
                                child_max_key < branch_key,
                                "child: {child_max_key:?}, branch:{branch_key:?}"
                            );
                        }

                        assert!(key!(last_child, 0) > &p.last_entry().0);
                    }
                }

                cur_q = nxt_q;
            }
        }
    }

    #[test]
    fn test_bt_bt_case_1() {
        let mut dict = BT::<u16, u16, 3>::new();

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
        dict_remove!(dict, 35);
        dict_remove!(dict, 30);
        dict_remove!(dict, 28);

        dict.debug_print();
    }


    #[test]
    fn test_bt_bt_random() {
        test_dict!(BT::<u16, u16, 3>::new());
        test_dict!(BT::<u16, u16, 4>::new());
        test_dict!(BT::<u16, u16, 5>::new());
        test_dict!(BT::<u16, u16, 11>::new());
        test_dict!(BT::<u16, u16, 20>::new());
    }
}
