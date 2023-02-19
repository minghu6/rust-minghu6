//! B Tree
//!

use std::{fmt::*, borrow::Borrow};

use m6coll::KVEntry;

use super::{ *, node as aux_node};

def_attr_macro!(ref|
    (entries, Vec<KVEntry<K, V>>)
);

impl_node!();
impl_tree!(BT {});


////////////////////////////////////////////////////////////////////////////////
//// Macros

macro_rules! node {
    (kv| $k:expr, $v:expr) => {{
        let mut entries = Vec::with_capacity(M-1);
        entries.push(KVEntry($k, $v));

        let children = vec![Node::none() ; 2];

        node!(basic| entries, children, WeakNode::none())
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


////////////////////////////////////////////////////////////////////////////////
//// Structure

struct Node_<K, V> {
    entries: Vec<KVEntry<K, V>>,
    /// 即使是叶子节点，也要保持孩子数量 = k-v 数量 + 1
    children: Vec<Node<K, V>>,
    paren: WeakNode<K, V>
}



////////////////////////////////////////////////////////////////////////////////
//// Implementation

impl<K: Ord, V, const M: usize> BT<K, V, M> {

    ////////////////////////////////////////////////////////////////////////////
    //// Public API

    pub fn new() -> Self {
        Self { root: Node::none() }
    }

    pub fn get<Q>(&self, k: &Q) -> Option<&V>
    where K: Borrow<Q>, Q: Ord + ?Sized
    {
        self
        .root
        .search(k)
        .map(|(node, idx)| &entries!(node)[idx].1)
    }

    pub fn get_mut<Q>(&mut self, k: &Q) -> Option<&mut V>
    where K: Borrow<Q>, Q: Ord + ?Sized
    {
        self
        .root
        .search(k)
        .map(|(node, idx)| &mut entries_mut!(node)[idx].1)
    }

    pub fn insert(&mut self, k: K, v: V) -> Option<V>
    where K: Debug
    {

        let mut y = Node::none();
        let mut x = self.root.clone();
        let mut idx = 0;

        while x.is_some() {
            match entries!(x).binary_search_by_key(&&k, |ent| &ent.0) {
                Ok(idx_) => {
                    idx = idx_;

                    break;
                },
                Err(idx_) => {
                    idx = idx_;

                    y = x;
                    x = children!(y)[idx].clone();
                },
            }
        }

        if x.is_some() {
            return Some(
                replace(&mut entries_mut!(x)[idx].1, v)
            );
        }

        if y.is_none() {
            self.root = node!(kv| k, v);
        }
        else {
            /* insert into leaf */

            entries_mut!(y).insert(idx, KVEntry(k, v));
            children_mut!(y).push(Node::none());

            if entries!(y).len() == M {
                self.promote(y);
            }
        }

        None

    }

    ////////////////////////////////////////////////////////////////////////////
    //// Assistant Method

    /// 漂亮的尾递归
    fn promote(&mut self, x: Node<K, V>)
    where K: Debug
    {
        debug_assert_eq!(entries!(x).len(), M);

        /* split node */

        let split_pos = M.div_ceil(2);
        let entries_x = entries_mut!(x);

        let entries_x2 = entries_x.split_off(split_pos);
        let entry_head = entries_x.pop().unwrap();

        let children_x2 = children_mut!(x).split_off(split_pos);

        let x2 = node!(basic| entries_x2, children_x2, WeakNode::none());
        children_revref!(x2);

        let p = paren!(x).upgrade();

        if p.is_none() {
            /* push new level */

            let entries = vec![entry_head];
            let children = vec![x, x2];

            self.root = node!(basic| entries, children, WeakNode::none());

            children_revref!(self.root);
        }
        else {
            /* insert into paren node */

            let x_idx = index_of_child!(p, x);
            // println!("insert {x:?} into {p:?} at {x_idx}");

            entries_mut!(p).insert(x_idx, entry_head);
            children_mut!(p).insert(x_idx + 1, x2.clone());

            paren!(x2, p.downgrade());

            if entries!(p).len() == M {
                self.promote(p);
            }
        }

    }

    #[cfg(test)]
    fn validate(&self)
    where K: Debug
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

                // make sure leaves are in same level
                for child in group.iter() {
                    assert!(child.is_some());

                    if children!(child)[0].is_none() {
                        leaf_num += 1;
                    }
                    else {
                        internal_num += 1;

                        let mut nxt_children = VecDeque::from(
                            children!(child).clone()
                        );
                        nxt_children.push_front(child.clone());

                        nxt_q.push_back(nxt_children);
                    }
                }

                assert!(
                    leaf_num == 0 || internal_num == 0,
                    "leaf: {leaf_num}, internal: {internal_num}"
                );

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


impl<K, V> Node<K, V> {
    fn last_entry(&self) -> &KVEntry<K, V> {
        if let Some(ent) = entries!(self).last() {
            ent
        }
        else {
            unreachable!("EMPTY entries");
        }
    }

    /// 漂亮的尾递归
    fn search<Q>(&self, k: &Q) -> Option<(Self, usize)>
    where K: Borrow<Q>, Q: Ord + ?Sized
    {
        if self.is_some() {
            match entries!(self).binary_search_by_key(&k, |ent| ent.0.borrow()) {
                Ok(idx) => Some((self.clone(), idx)),
                Err(idx) => children!(self)[idx].search(k),
            }
        }
        else {
            None
        }
    }

    /// Right most
    fn maximum(&self) -> (Self, usize) {
        let mut x = self;
        let mut y = Node::none();

        while x.is_some() {
            y = x.clone();
            x = children!(x).last().unwrap();
        }

        let mut idx = 0;

        if y.is_some() {
            idx = children!(y).len() - 1;
        }

        (y, idx)
    }

    /// Left most
    fn minimum(&self) -> (Self, usize) {
        let mut x = self;
        let mut y = Node::none();

        while x.is_some() {
            y = x.clone();
            x = first_child!(x);
        }

        (y, 0)
    }

    fn successor(&self, idx: usize) -> (Self, usize)
    where K: Ord
    {
        debug_assert!(self.is_some());

        let rh = right!(self, idx);

        if rh.is_some() {
            rh.minimum()
        }
        /* paren: from right-most-up, left */
        else {
            let mut x = self.clone();
            let mut y = paren!(&x).upgrade();

            while y.is_some() && !x.rc_eq(last_child!(y)) {
                x = y.clone();
                y = paren!(y).upgrade();
            }

            let mut idx = 0;

            if y.is_some() {
                idx = index_of_child!(y, x);
            }

            (y, idx)
        }
    }



}


impl<K: Debug, V> Debug for Node<K, V> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        if self.is_some() {
            let kn = entries!(self).len();

            for i in 0..kn-1 {
                write!(f, "{:?}, ", key!(self, i))?;
            }
            write!(f, "{:?}", key!(self, kn-1))?;
        }
        else {
            write!(f, "nil")?;
        }

        Ok(())
    }
}


impl<K, V> Node_<K, V> {

}




#[cfg(test)]
mod tests {

    use crate::collections::bst2::test_dict;

    use super::*;

    macro_rules! dict_insert {
        ($dict:ident, $num:expr) => {
            $dict.insert($num, $num);
            assert!($dict.get(&$num).is_some());
        };
    }

    #[test]
    fn test_bt2_bt_case_1() {
        let mut dict = BT::<u16, u16, 3>::new();

        dict_insert!(dict, 52);
        dict_insert!(dict, 47);
        dict_insert!(dict, 3);
        dict_insert!(dict, 35);
        dict_insert!(dict, 24);
        dict_insert!(dict, 44);
        dict_insert!(dict, 66);
        dict_insert!(dict, 38);

        // dict.debug_print();

        dict.validate();
    }


    // #[test]
    // fn test_bt2_bt_case_2() {
    //     let mut dict = BT::<u16, u16, 3>::new();

    //     dict_insert!(dict, 45);
    //     dict_insert!(dict, 46);
    //     dict_insert!(dict, 47);
    //     dict_insert!(dict, 48);
    //     dict_insert!(dict, 49);
    //     dict_insert!(dict, 10);
    //     dict_insert!(dict, 11);
    //     dict_insert!(dict, 12);
    //     dict_insert!(dict, 13);
    //     dict_insert!(dict, 14);

    //     dict.debug_print();

    //     dict.validate();
    // }


    #[test]
    fn test_bt2_bt_random() {
        test_dict!(BT::<u16, u16, 3>::new());
        test_dict!(BT::<u16, u16, 4>::new());
        test_dict!(BT::<u16, u16, 5>::new());
        test_dict!(BT::<u16, u16, 11>::new());
        test_dict!(BT::<u16, u16, 20>::new());
    }


}
