#![allow(dead_code)]

use std::{fmt::Debug, marker::PhantomData};

use coll::{ptr::*, KVEntry};


////////////////////////////////////////////////////////////////////////////////
//// Structures

enum Node<K, V, const M: usize> {
    Leaf {
        entries: Vec<KVEntry<K, OwnedPtr<V>>>,
        next: Option<Ptr<Self>>,
        paren: Option<Ptr<Self>>,
        _marker: PhantomData<[(); M]>
    },
    Internal {
        children: Vec<KVEntry<K, OwnedPtr<Self>>>,
        paren: Option<Ptr<Self>>,
        _marker: PhantomData<[(); M]>
    },
}

pub struct BPT<K, V, const M: usize = 32> {
    len: usize,
    root: OwnedPtr<Node<K, V, M>>,
}

////////////////////////////////////////////////////////////////////////////////
//// Implementations

impl<K, V, const M: usize> BPT<K, V, M> {
    // pub fn new() -> Self {
    //     // so that after split internal, each has at least two child
    //     debug_assert!(M >= 4);

    //     let root = ;

    //     Self { data, nodes, root }
    // }

    pub const fn len(&self) -> usize {
        self.len
    }

    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// `>= 2`
    const fn internal_cap_low_bound() -> usize {
        M.div_floor(2)
    }
}

/// Bounded public methods
impl<K: Clone + Ord + Debug, V, const M: usize> BPT<K, V, M> {
    pub fn bulk_build<T: IntoIterator<Item = (K, V)>>(sorted_iter: T) -> Self {
        /* dedup */

        let kv_vec0: Vec<(K, V)> = sorted_iter.into_iter().collect();

        debug_assert!(kv_vec0.iter().is_sorted_by_key(|(k, _)| k));

        let mut kv_vec = Vec::with_capacity(kv_vec0.len());

        let mut maybe_pre = None;

        for (k, v) in kv_vec0 {
            if let Some((k0, _v0)) = &maybe_pre {
                if k == *k0 {
                    maybe_pre = Some((k, v));
                } else {
                    kv_vec.push(maybe_pre.replace((k, v)).unwrap());
                }
            } else {
                maybe_pre = Some((k, v))
            }
        }

        /* preset capcity */

        let node_redundancy = 1;
        // avoid tree promote and unpromote
        let leaf_min_cap = 1 + node_redundancy;
        let internal_min_cap = Self::internal_cap_low_bound() + node_redundancy;

        /* compute nodes number per level */

        struct Group {
            len: usize,
            num: usize,
        }

        let divide_into = |cnt_cur_lv: usize, min_cap: usize| -> Vec<Group> {
            // for root
            if cnt_cur_lv <= min_cap {
                return vec![Group {
                    len: cnt_cur_lv,
                    num: 1,
                }];
            }

            let mut tot = cnt_cur_lv / min_cap;
            let rem = cnt_cur_lv % min_cap;

            // hold nodes number, increase some of size
            if rem <= min_cap - node_redundancy * 2 {
                let common_inc = rem / tot;
                let unique_inc_num = rem % tot;

                vec![
                    Group {
                        len: min_cap + common_inc + 1,
                        num: unique_inc_num,
                    },
                    Group {
                        len: min_cap + common_inc,
                        num: tot - unique_inc_num,
                    },
                ]
            }
            // add one additional node, decrease some of size
            else {
                tot += 1;

                let negrem = min_cap - rem;
                let common_dec = negrem / tot;
                let unique_dec_num = negrem % tot;

                vec![
                    Group {
                        len: min_cap - common_dec - 1,
                        num: unique_dec_num,
                    },
                    Group {
                        len: min_cap - common_dec,
                        num: tot - unique_dec_num,
                    },
                ]
            }
        };

        let cnt_data = kv_vec.len();

        let mut leaf_groups = divide_into(cnt_data, leaf_min_cap);
        let cnt_leaf = leaf_groups.iter().fold(0, |acc, x| acc + x.num);

        // include root
        let mut all_lv_internal_grps = vec![];
        let mut cnt_cur_lv = cnt_leaf;

        while cnt_cur_lv > 1 {
            let cur_lv_int_grps = divide_into(cnt_cur_lv, internal_min_cap);
            cnt_cur_lv = cur_lv_int_grps.iter().fold(0, |acc, x| acc + x.num);

            all_lv_internal_grps.push(cur_lv_int_grps);
        }

        /* build tree upwards */

        /* build leaves (kventry-leaf) */

        let mut nodeq = Vec::with_capacity(cnt_leaf);

        let Group { len: mut leaf_len, num: mut leaf_num } = leaf_groups.pop().unwrap();

        let mut kvi = 0;
        let mut ingroupi = 0;
        let mut entries = Vec::with_capacity(M);
        let mut maybe_next_leaf = None;

        for (k, v) in kv_vec.into_iter().rev() {

            entries.push(KVEntry(k, OwnedPtr::new(v)));

            kvi += 1;

            if kvi == leaf_len {
                let cur_leaf = OwnedPtr::new(
                    Node::<K, V, M>::new_leaf()
                        .with_entries(
                            entries
                        )
                        .with_next(maybe_next_leaf),
                );

                maybe_next_leaf = Some(cur_leaf.ptr());
                entries = Vec::with_capacity(M);

                // nodeq.push(cur_leaf);

                kvi = 0;
                ingroupi += 1;

                if ingroupi == leaf_num {
                    if let Some(Group { len, num }) = leaf_groups.pop() {
                        leaf_len = len;
                        leaf_num = num;
                        ingroupi = 0;
                    }
                    else {
                        break;
                    }
                }
            }
        }

        // for cur_lv_grps in all_lv_internal_grps.into_iter() {
        //     for Group { len, num } in cur_lv_grps {
        //         for _ in 0..num {
        //             let paren_node = OwnedPtr::new(
        //                 Node::new_internal().with_children_iter(
        //                     cur_lv_nodes
        //                         .drain(..len)
        //                         .map(|node| KVEntry(node.as_ref().k(), node)),
        //                 ),
        //             );

        //             // let paren_ptr = paren_node.ptr();

        //             // paren_node.as_ref().get_children().iter().for_each(
        //             //     |KVEntry(_, child)| {
        //             //         *child.as_mut().get_paren_mut() = Some(paren_ptr);
        //             //     },
        //             // );

        //             cur_lv_nodes.push_back(paren_node);
        //         }
        //     }
        // }

        // debug_assert_eq!(cur_lv_nodes.len(), 1);

        let len = cnt_data;
        let root = if let Some(node) = nodeq.pop() {
            node
        } else {
            OwnedPtr::new(Node::new_leaf())
        };

        Self { len, root }
    }
}

impl<K: Clone + Ord + Debug, V, const M: usize> FromIterator<(K, V)>
    for BPT<K, V, M>
{
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        let mut inputs = iter.into_iter().collect::<Vec<_>>();

        inputs.sort_by(|a, b| a.0.cmp(&b.0));

        Self::bulk_build(inputs.into_iter())
    }
}

impl<K, V, const M: usize> Node<K, V, M> {
    pub fn new_leaf() -> Self {
        Self::Leaf {
            entries: Vec::with_capacity(M),
            next: None,
            paren: None,
            _marker: PhantomData
        }
    }

    pub fn new_internal() -> Self {
        Self::Internal {
            children: Vec::with_capacity(M),
            paren: None,
            _marker: PhantomData
        }
    }

    pub fn with_entries(
        mut self,
        entries: Vec<KVEntry<K, OwnedPtr<V>>>,
    ) -> Self {
        self.get_entries_mut().extend(entries);
        self
    }

    /// Leaf build method
    pub fn with_next(mut self, new_next: Option<Ptr<Self>>) -> Self {
        *self.get_next_mut() = new_next;
        self
    }

    pub fn with_children<
        I: Iterator<Item = KVEntry<K, OwnedPtr<Self>>>,
    >(
        mut self,
        children: Vec<KVEntry<K, OwnedPtr<Self>>>,
    ) -> Self {
        self.get_children_mut().extend(children);
        self
    }

    #[allow(unused)]
    pub fn with_paren(mut self, paren: Option<Ptr<Self>>) -> Self {
        *self.get_paren_mut() = paren;

        self
    }

    pub const fn len(&self) -> usize {
        match self {
            Self::Internal { children, .. } => children.len(),
            Self::Leaf { entries, .. } => entries.len(),
        }
    }

    pub const fn is_full(&self) -> bool {
        self.len() == M
    }

    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn k(&self) -> K
    where
        K: Clone,
    {
        debug_assert!(!self.is_empty());

        match self {
            Self::Internal { children, .. } => children[0].0.clone(),
            // entries.arr[0].as_ref().unwrap().0.clone()
            Self::Leaf { entries, .. } => entries[0].0.clone(),
        }
    }

    pub fn get_keys(&self) -> Vec<&K> {
        match self {
            Self::Internal { children, .. } => {
                children[..].iter().map(|KVEntry(k, _)| k).collect()
            }
            Self::Leaf { entries, .. } => {
                entries[..].iter().map(|KVEntry(k, _)| k).collect()
            }
        }
    }

    pub fn next(&self) -> Option<Ptr<Self>> {
        match self {
            Self::Internal { .. } => unreachable!(),
            Self::Leaf { next, .. } => *next,
        }
    }

    pub fn get_next_mut(&mut self) -> &mut Option<Ptr<Self>> {
        match self {
            Self::Internal { .. } => unreachable!(),
            Self::Leaf { next, .. } => next,
        }
    }

    pub fn paren(&self) -> Option<Ptr<Self>> {
        match self {
            Self::Internal { paren, .. } => *paren,
            Self::Leaf { paren, .. } => *paren,
        }
    }

    pub fn get_paren_mut(&mut self) -> &mut Option<Ptr<Self>> {
        match self {
            Self::Internal { paren, .. } => paren,
            Self::Leaf { paren, .. } => paren,
        }
    }

    pub fn children(&self) -> Vec<Ptr<Self>> {
        self.get_children()
            .iter()
            .map(|KVEntry(_, child)| child.ptr())
            .collect()
    }

    pub fn get_children(&self) -> &Vec<KVEntry<K, OwnedPtr<Self>>> {
        match self {
            Self::Internal { children, .. } => children,
            Self::Leaf { .. } => unreachable!(),
        }
    }

    pub fn get_children_mut(
        &mut self,
    ) -> &mut Vec<KVEntry<K, OwnedPtr<Self>>> {
        match self {
            Self::Internal { children, .. } => children,
            Self::Leaf { .. } => unreachable!(),
        }
    }

    pub fn get_entries(&self) -> &Vec<KVEntry<K, OwnedPtr<V>>> {
        match self {
            Self::Internal { .. } => unreachable!(),
            Self::Leaf { entries, .. } => entries,
        }
    }

    pub fn get_entries_mut(
        &mut self,
    ) -> &mut Vec<KVEntry<K, OwnedPtr<V>>> {
        match self {
            Self::Internal { .. } => unreachable!(),
            Self::Leaf { entries, .. } => entries,
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bpt_bulk_load() {
        let _ = BPT::<_, _>::from_iter((0..5).map(|v| (v, v)));
    }
}
