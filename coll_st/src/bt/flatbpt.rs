use std::{
    borrow::Borrow,
    cmp::max,
    collections::{HashMap, VecDeque},
    fmt::{Debug, Display},
    mem::{replace, MaybeUninit},
    ops::{Bound::*, Index, IndexMut, RangeBounds},
    ptr,
};

use coll::KVEntry;

////////////////////////////////////////////////////////////////////////////////
//// Traits

pub trait WalkTree<'a> {
    type Node: Borrow<Self::NodeBorrow>;
    type NodeBorrow;

    fn root(&'a self) -> Option<&'a Self::NodeBorrow>;
    fn children(
        &'a self,
        ptr: &'a Self::NodeBorrow,
    ) -> Option<Vec<&'a Self::NodeBorrow>>;

    fn pre_order_walk(
        &'a self,
    ) -> impl Iterator<Item = (LocOnTree, &'a Self::NodeBorrow)> + 'a {
        std::iter::from_coroutine(
            #[coroutine]
            || {
                let Some(root) = self.root() else {
                    return;
                };

                let mut loc = LocOnTree::new();
                let mut curlv = vec![vec![root]];

                while !curlv.is_empty() {
                    loc.ln += 1;
                    loc.col_group = 0;

                    let mut nextlv = vec![];

                    for child_group in curlv.into_iter() {
                        loc.col_group += 1;
                        loc.in_group_id = 0;

                        for child in child_group {
                            loc.in_group_id += 1;

                            yield (loc, child);

                            if let Some(nxt_child_group) = self.children(child)
                            {
                                nextlv.push(nxt_child_group);
                            }
                        }
                    }

                    curlv = nextlv;
                }
            },
        )
    }

    fn display_fault(
        &'a self,
        node: &'a Self::NodeBorrow,
        reason: &str,
    ) -> DisplayFaultNodeOfTree<'a, Self, Self::NodeBorrow> {
        let tree = self;
        let reason = reason.to_string();

        DisplayFaultNodeOfTree { tree, node, reason }
    }
}

////////////////////////////////////////////////////////////////////////////////
//// Structures

enum QueryRangeEndResult {
    /// nodeid, entryid
    Spec(usize, usize),
    Unbound,
}

pub struct PreOrderView<'a, NB> {
    /// persisitent sequence
    pseq: Vec<(LocOnTree, &'a NB)>,
    locmap: HashMap<*const NB, LocOnTree>,
}


#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub struct LocOnTree {
    pub ln: usize,
    pub col_group: usize,
    pub in_group_id: usize,
}

#[derive(Debug, Clone, Copy)]
pub enum Node<K, const M: usize> {
    Leaf {
        /// (key, dataid)
        entries: PartialInitArray<KVEntry<K, usize>, M>,
        next: Option<usize>,
        /// *NOTE:* For root node, `paren` is nonsense.
        paren: usize,
    },
    Internal {
        /// (min-key of child, nodeid)
        children: PartialInitArray<KVEntry<K, usize>, M>,
        paren: usize,
    },
}


#[derive(Clone)]
pub struct FlatBPT<K, V, const M: usize = 30> {
    /// uncouple data manage (avoid unnecessary mutability check)
    data: LazyDeleteVec<*mut V>,

    /// nodes[0] would be root and should always not to be deleted
    nodes: LazyDeleteVec<Node<K, M>>,
    root: usize,
}


#[derive(Default, Clone)]
pub struct LazyDeleteVec<T> {
    data: Vec<Option<T>>,
    deleted: Vec<usize>,
}


#[derive(Debug)]
pub struct PartialInitArray<T, const CAP: usize> {
    len: usize,
    arr: [MaybeUninit<T>; CAP],
}

pub struct DisplayLocOnTree<'a, T> {
    pub revref: &'a T,
    pub max_ln_width: usize,
    pub max_col_group_width: usize,
    pub max_in_group_width: usize,
}

pub struct DisplayFaultNodeOfTree<'a, T: ?Sized, N> {
    tree: &'a T,
    node: &'a N,
    reason: String,
}

////////////////////////////////////////////////////////////////////////////////
//// Implementations

impl<'a, T: WalkTree<'a, NodeBorrow = N> + Display, N> Display
    for DisplayFaultNodeOfTree<'a, T, N>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let view = self.tree.pre_order_walk().collect::<PreOrderView<_>>();
        let node_loc = view[&self.node];

        writeln!(f, "[{node_loc:?}, {}]: {}", self.reason, self.tree)
    }
}

impl<'a, T> DisplayLocOnTree<'a, T> {
    fn display(&self, loc: &LocOnTree) -> String {
        format!(
            "{:0ln_width$}.{:0col_group_width$}.{:0in_group_width$}",
            loc.ln,
            loc.col_group,
            loc.in_group_id,
            ln_width = self.max_ln_width,
            col_group_width = self.max_col_group_width,
            in_group_width = self.max_in_group_width
        )
    }

    #[cfg(test)]
    fn display_fault(&self, loc: &LocOnTree) -> String
    where
        T: Display,
    {
        format!("\n[{}]: {}", self.display(loc), self.revref)
    }

    #[cfg(test)]
    fn display_pass(&self) -> String
    where
        T: Display,
    {
        format!("\n[PASS]: {}", self.revref)
    }
}

impl<'a, NB> PreOrderView<'a, NB> {
    pub fn iter(&'a self) -> impl Iterator<Item = (LocOnTree, &'a NB)> + 'a {
        self.pseq.iter().cloned()
    }
}

impl<'a, NB> FromIterator<(LocOnTree, &'a NB)> for PreOrderView<'a, NB> {
    fn from_iter<T: IntoIterator<Item = (LocOnTree, &'a NB)>>(iter: T) -> Self {
        let pseq = iter.into_iter().collect::<Vec<_>>();

        let locmap = HashMap::from_iter(
            pseq.iter().cloned().map(|(_0, _1)| (_1 as _, _0)),
        );

        Self { pseq, locmap }
    }
}

impl<'a, NB> Index<&NB> for PreOrderView<'a, NB> {
    type Output = LocOnTree;

    fn index(&self, index: &NB) -> &Self::Output {
        &self.locmap[&(index as _)]
    }
}

impl LocOnTree {
    pub fn new() -> Self {
        Default::default()
    }
}

impl Debug for LocOnTree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.ln, self.col_group, self.in_group_id,)
    }
}

////////////////////////////////////////
//// impl PartialInitArray

impl<T, const CAP: usize> PartialInitArray<T, CAP> {
    pub const fn len(&self) -> usize {
        self.len
    }

    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn new() -> Self {
        Self {
            len: 0,
            arr: MaybeUninit::uninit_array(),
        }
    }

    pub fn exact(&self) -> &[T] {
        unsafe { MaybeUninit::slice_assume_init_ref(&self.arr[..self.len]) }
    }

    pub fn insert(&mut self, idx: usize, val: T) {
        debug_assert!(self.len < CAP, "array is full");
        debug_assert!(
            idx <= self.len,
            "index {idx} overflow for length {} ",
            self.len
        );

        if idx < self.len {
            self.arr[idx..self.len + 1].rotate_right(1);
        }

        self.arr[idx].write(val);
        self.len += 1;
    }

    pub fn remove(&mut self, idx: usize) -> T {
        debug_assert!(
            idx < self.len,
            "index {idx} overflow for lenghth {} ",
            self.len
        );

        let val = unsafe { self.arr[idx].assume_init_read() };
        self.arr[idx..self.len].rotate_left(1);

        self.len -= 1;

        val
    }

    pub fn push(&mut self, val: T) {
        self.insert(self.len, val);
    }

    pub fn pop(&mut self) -> T {
        self.remove(self.len - 1)
    }

    /// Binary search and Insert
    pub fn binary_insert(&mut self, val: T) -> Option<T>
    where
        T: Ord,
    {
        match self.exact().binary_search(&val) {
            Ok(idx) => {
                /* Repalce Data */

                Some(unsafe {
                    replace(&mut self.arr[idx], MaybeUninit::new(val))
                        .assume_init()
                })
            }
            Err(idx) => {
                /* Insert Data */

                self.insert(idx, val);

                None
            }
        }
    }

    /// split off within initialized content
    pub fn split_off(&mut self, at: usize) -> &[T] {
        debug_assert!(at < self.len);

        let oldlen = replace(&mut self.len, at);

        unsafe { MaybeUninit::slice_assume_init_ref(&self.arr[at..oldlen]) }
    }

    pub fn init_with_slice(&mut self, slice: &[T]) {
        self.len = 0;
        self.extend_slice(slice);
    }

    /// Extend without trunction (be careful of overflow)
    pub fn extend_slice(&mut self, slice: &[T]) {
        debug_assert!(self.len() + slice.len() <= CAP);

        unsafe {
            std::ptr::copy(
                slice.as_ptr(),
                MaybeUninit::slice_as_mut_ptr(&mut self.arr[self.len..]),
                slice.len(),
            );
        }

        self.len += slice.len();
    }
}


impl<T, const CAP: usize> Extend<T> for PartialInitArray<T, CAP> {
    /// Extend without trunction (be careful of overflow)
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for val in iter {
            self.push(val);
        }
    }
}

impl<T, const CAP: usize> Index<usize> for PartialInitArray<T, CAP> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        debug_assert!(index < self.len);

        unsafe { self.arr[index].assume_init_ref() }
    }
}

impl<T, const CAP: usize> IndexMut<usize> for PartialInitArray<T, CAP> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        debug_assert!(index < self.len);

        unsafe { self.arr[index].assume_init_mut() }
    }
}

impl<T: Clone, const CAP: usize> Clone for PartialInitArray<T, CAP> {
    fn clone(&self) -> Self {
        let mut arr = MaybeUninit::uninit_array();

        unsafe {
            std::ptr::copy(&self.arr, &mut arr, self.len);
        }

        Self { len: self.len, arr }
    }
}

impl<T: Copy, const CAP: usize> Copy for PartialInitArray<T, CAP> {}

impl<T, const CAP: usize> IntoIterator for PartialInitArray<T, CAP> {
    type Item = T;

    type IntoIter = impl Iterator<Item = Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        unsafe { self.arr.into_iter().take(self.len).map(|x| x.assume_init()) }
    }
}

////////////////////////////////////////
//// impl LazyDeleteVec

impl<T> LazyDeleteVec<T> {
    pub const fn len(&self) -> usize {
        self.data.len() - self.deleted.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            data: Vec::with_capacity(capacity),
            deleted: vec![],
        }
    }

    pub fn from_parts(data: Vec<Option<T>>, deleted: Vec<usize>) -> Self {
        Self { data, deleted }
    }

    pub fn push(&mut self, val: T) -> usize {
        if let Some(idx) = self.deleted.pop() {
            self.data[idx] = Some(val);
            idx
        } else {
            self.data.push(Some(val));
            self.data.len() - 1
        }
    }

    pub fn remove(&mut self, idx: usize) -> Option<T> {
        debug_assert!(idx < self.data.len(), "{idx} > len {}", self.data.len());

        let old = self.data[idx].take();

        if old.is_some() {
            self.deleted.push(idx);
        }

        old
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.data
            .iter()
            .filter(|x| x.is_some())
            .map(|x| x.as_ref().unwrap())
    }
}

impl<T> FromIterator<T> for LazyDeleteVec<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self {
            data: iter.into_iter().map(|v| Some(v)).collect(),
            deleted: vec![],
        }
    }
}

impl<T> IntoIterator for LazyDeleteVec<T> {
    type Item = T;
    type IntoIter = impl Iterator<Item = Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        // `Option` impl IntoIterator
        self.data.into_iter().flatten()
    }
}

impl<T> Index<usize> for LazyDeleteVec<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        if let Some(v) = &self.data[index] {
            v
        } else {
            unreachable!("data[{index}] has been deleted")
        }
    }
}

impl<T> IndexMut<usize> for LazyDeleteVec<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        if let Some(v) = &mut self.data[index] {
            v
        } else {
            unreachable!("data[{index}] has been deleted")
        }
    }
}

////////////////////////////////////////
//// impl FlatBPT

/// No bounds public methods
impl<K, V, const M: usize> FlatBPT<K, V, M> {
    pub fn new() -> Self {
        // so that after split internal, each has at least two child
        debug_assert!(M >= 4);

        let data = LazyDeleteVec::with_capacity(0);
        let nodes = vec![Node::new_leaf()].into_iter().collect();
        let root = 0;

        Self { data, nodes, root }
    }

    pub const fn len(&self) -> usize {
        self.data.len()
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
impl<K: Clone + Ord, V, const M: usize> FlatBPT<K, V, M> {
    pub fn bulk_build<T: IntoIterator<Item = (K, V)>>(sorted_iter: T) -> Self
    where
        K: Debug,
    {
        /* dedup */

        let kv_vec0: Vec<(K, V)> = sorted_iter.into_iter().collect();

        debug_assert!(kv_vec0.iter().is_sorted_by_key(|(k, _)| k));

        let mut kv_vec: VecDeque<(K, V)> =
            VecDeque::with_capacity(kv_vec0.len());

        let mut maybe_pre = None;

        for (k, v) in kv_vec0 {
            if let Some((k0, _v0)) = &maybe_pre {
                if k == *k0 {
                    maybe_pre = Some((k, v));
                } else {
                    kv_vec.push_back(maybe_pre.replace((k, v)).unwrap());
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

        let leaf_groups = divide_into(cnt_data, leaf_min_cap);
        let cnt_leaf = leaf_groups.iter().fold(0, |acc, x| acc + x.num);

        // include root
        let mut all_lv_internal_grps = vec![];
        let mut all_lv_cnt_internals = vec![];

        let mut cnt_cur_lv = cnt_leaf;

        while cnt_cur_lv > 1 {
            let cur_lv_int_grps = divide_into(cnt_cur_lv, internal_min_cap);
            cnt_cur_lv = cur_lv_int_grps.iter().fold(0, |acc, x| acc + x.num);

            all_lv_cnt_internals.push(cnt_cur_lv);
            all_lv_internal_grps.push(cur_lv_int_grps);
        }

        let tot = if cnt_leaf == 1 {
            1
        } else {
            cnt_leaf + all_lv_cnt_internals.iter().cloned().sum::<usize>()
        };

        let mut nodes0 = vec![None; tot];
        let mut data = LazyDeleteVec::with_capacity(cnt_data);
        // 0 - root is firendly to cache
        let root = 0;

        /* build tree upwards */

        /* build leaves (kventry-leaf) */

        let leafbase = tot - cnt_leaf;
        let mut leafi = leafbase;

        for Group { len, num } in leaf_groups {
            for _ in 0..num {
                nodes0[leafi] = Some(
                    Node::new_leaf()
                        .with_entries_iter(kv_vec.drain(..len).map(|(k, v)| {
                            KVEntry(k, data.push(Box::into_raw(Box::new(v))))
                        }))
                        .with_next(Some(leafi + 1)),
                );

                leafi += 1;
            }
        }

        *nodes0[leafi - 1].as_mut().unwrap().get_next_mut() = None;

        /* build internals (include root) */

        let mut childbase;
        let mut parenbase = leafbase;

        for (cur_lv_grps, cnt_cur_lv_inernal) in
            all_lv_internal_grps.into_iter().zip(all_lv_cnt_internals)
        {
            childbase = parenbase;
            parenbase = childbase - cnt_cur_lv_inernal;

            let (mut childi, mut pareni) = (childbase, parenbase);

            for Group { len, num } in cur_lv_grps {
                for _ in 0..num {
                    for i in 0..len {
                        *nodes0[childi + i].as_mut().unwrap().get_paren_mut() =
                            pareni;
                    }

                    nodes0[pareni] =
                        Some(Node::new_internal().with_children_iter(
                            (childi..childi + len).map(|nodeid| {
                                KVEntry(
                                    nodes0[nodeid].as_ref().unwrap().k(),
                                    nodeid,
                                )
                            }),
                        ));

                    childi += len;
                    pareni += 1;
                }
            }
        }

        let nodes = LazyDeleteVec::from_parts(nodes0, vec![]);

        let it = Self { data, nodes, root };

        #[cfg(test)]
        {
            it.validate();
        }

        it
    }

    pub fn get<Q>(&self, k: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.shortcut_search(k)
            .map(|(nodeid, entryid, _)| {
                self.nodes[nodeid].get_entries()[entryid].1
            })
            .map(|dataid| self.data(dataid) as _)
    }

    pub fn get_mut<Q>(&mut self, k: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.shortcut_search(k)
            .map(|(nodeid, entryid, _)| {
                self.nodes[nodeid].get_entries()[entryid].1
            })
            .map(|dataid| self.data(dataid))
    }

    pub fn range<Q, R>(&self, range: R) -> impl Iterator<Item = (&K, &V)>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
        R: RangeBounds<Q>,
    {
        self.inner_range(range).map(|(k, v)| (k, v as _))
    }

    pub fn range_mut<Q, R>(
        &mut self,
        range: R,
    ) -> impl Iterator<Item = (&K, &mut V)>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
        R: RangeBounds<Q>,
    {
        self.inner_range(range)
    }

    pub fn push_back(&mut self, key: K, value: V) {
        let nodeid = self.max_node();

        self.inner_insert(key, value, nodeid);
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V>
    {
        let (nodeid, ..) = self.complete_search(&key);

        self.inner_insert(key, value, nodeid)
    }

    pub fn remove<Q>(&mut self, k: &Q) -> Option<V>
    where
        K: Borrow<Q> + Debug,
        Q: Ord + ?Sized,
    {
        let (x, entryid, maybe_x_of_p) =
            if let Some(res) = self.shortcut_search(&k.borrow()) {
                res
            } else {
                return None;
            };

        let entry = self.nodes[x].get_entries_mut().remove(entryid);
        let popped_data = self.remove_data(entry.1);

        if let Some(x_of_p) = maybe_x_of_p {
            if self.nodes[x].is_empty() {
                self.unpromote_leaf(x, x_of_p);
            } else if entryid == 0 {
                let x_new_key = self.nodes[x].k();
                let p = self.nodes[x].paren();

                let x_old_key = replace(
                    &mut self.nodes[p].get_children_mut()[x_of_p].0,
                    x_new_key,
                );

                // p also needs update min key towrads up
                if x_of_p == 0 && p != self.root {
                    self.update_index(p, x_old_key);
                }
            }
        }

        popped_data
    }

    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        self.inner_range(..).map(|(k, v)| (k, v as _))
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&K, &mut V)> {
        self.inner_range(..).map(|(k, v)| (k, v as _))
    }

    pub fn into_iter(mut self) -> impl Iterator<Item = (K, V)> {
        std::iter::from_coroutine(
            #[coroutine]
            move || {
                let mut ptr = self.min_node();

                loop {
                    let Some(Node::Leaf { entries, next, .. }) =
                        self.nodes.remove(ptr)
                    else {
                        unreachable!()
                    };

                    for KVEntry(key, dataid) in entries.into_iter() {
                        yield (key, self.remove_data(dataid).unwrap())
                    }

                    if let Some(next) = next {
                        ptr = next;
                    } else {
                        break;
                    }
                }
            },
        )
    }
}

/// Private auxiliary methods
impl<K, V, const M: usize> FlatBPT<K, V, M> {
    ////////////////////////////////////////////////////////////////////////////
    //// Manage Data Methods

    fn push_data(&mut self, val: V) -> usize {
        self.data.push(Box::into_raw(Box::new(val)))
    }

    fn remove_data(&mut self, idx: usize) -> Option<V> {
        self.data
            .remove(idx)
            .map(|raw| unsafe { Box::into_inner(Box::from_raw(raw)) })
    }

    /// 本来就不应该管数据而只管索引的，只是作为原理性实现，出于方便目的，数据顺便被放到一起，
    /// 实际上具体内容的修改跟我索引有什么关系？（涉及 key 变化的修改应该通过 `remove` 和 `insert` 进行）
    fn data(&self, idx: usize) -> &mut V {
        let valptr = self.data[idx];

        unsafe { &mut *valptr }
    }

    fn replace_data(&mut self, idx: usize, val: V) -> V {
        unsafe {
            let newvalptr = Box::into_raw(Box::new(val));
            let oldvalptr = replace(&mut self.data[idx], newvalptr);

            Box::into_inner(Box::from_raw(oldvalptr))
        }
    }

    fn inner_insert(&mut self, key: K, value: V, nodeid: usize) -> Option<V>
    where
        K: Ord + Clone
    {
        let mut ptr = nodeid;
        let mut maybe_res_entryid = None;

        if self.nodes[ptr].is_full() {
            self.promote(ptr);

            /* Re-search on parent node */

            let p = self.nodes[ptr].paren();
            let p_children = self.nodes[p].get_children();

            (ptr, maybe_res_entryid) = match p_children
                .exact()
                .binary_search_by_key(&&key, |KVEntry(key, _)| key)
            {
                Ok(idx) => (p_children[idx].1, Some(Ok(0))),
                Err(idx) => {
                    if idx == 0 {
                        (p_children[0].1, Some(Err(0)))
                    } else {
                        (p_children[idx - 1].1, None)
                    }
                }
            };
        }

        let res_entryid = maybe_res_entryid.unwrap_or_else(|| {
            self.nodes[ptr]
                .get_entries()
                .exact()
                .binary_search_by_key(&&key, |KVEntry(key, _)| key)
        });

        match res_entryid {
            /* replace data */
            Ok(entryid) => {
                let dataid = self.nodes[ptr].get_entries()[entryid].1;

                Some(self.replace_data(dataid, value))
            }
            /* insert data */
            Err(entryid) => {
                let entry = KVEntry(key, self.push_data(value));

                // if !(ptr == self.root && self.nodes[ptr].is_empty()) {
                //     self.validate();  // avoid break up `len` logic
                // }

                let ptr_entries = self.nodes[ptr].get_entries_mut();

                ptr_entries.insert(entryid, entry);

                if entryid == 0 && ptr != self.root {
                    let ptr_old_key = ptr_entries[1].0.clone();
                    self.update_index(ptr, ptr_old_key);
                }

                None
            }
        }
    }

    ////////////////////////////////////////////////////////////////////////////
    //// Search Methods

    fn inner_range<Q, R>(&self, range: R) -> impl Iterator<Item = (&K, &mut V)>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
        R: RangeBounds<Q>,
    {
        std::iter::from_coroutine(
            #[coroutine]
            move || {
                let (start_nodeid, start_entryid) =
                    if let Some(x) = self.range_start(&range) {
                        x
                    } else {
                        return;
                    };

                let Some(query_end_res) = self.range_end(&range) else {
                    return;
                };

                match query_end_res {
                    QueryRangeEndResult::Spec(end_nodeid, end_entryid) => {
                        let start_k = self.nodes[start_nodeid].get_entries()
                            [start_entryid]
                            .0
                            .borrow();
                        let end_k = self.nodes[end_nodeid].get_entries()
                            [end_entryid]
                            .0
                            .borrow();

                        if start_k > end_k {
                            return;
                        }

                        if start_nodeid == end_nodeid {
                            for KVEntry(key, dataid) in
                                &self.nodes[start_nodeid].get_entries().exact()
                                    [start_entryid..=end_entryid]
                            {
                                yield (key, self.data(*dataid))
                            }
                        } else {
                            for KVEntry(key, dataid) in
                                &self.nodes[start_nodeid].get_entries().exact()
                                    [start_entryid..]
                            {
                                yield (key, self.data(*dataid))
                            }

                            let mut maybe_ptr = self.nodes[start_nodeid].next();

                            while let Some(ptr) = maybe_ptr {
                                let Node::Leaf { entries, next, .. } =
                                    &self.nodes[ptr]
                                else {
                                    unreachable!()
                                };

                                if ptr == end_nodeid {
                                    for KVEntry(key, dataid) in entries.exact()[..=end_entryid].iter() {
                                        yield (key, self.data(*dataid))
                                    }

                                    break;
                                }
                                else {
                                    for KVEntry(key, dataid) in entries.exact() {
                                        yield (key, self.data(*dataid))
                                    }
                                }

                                maybe_ptr = *next;
                            }
                        }
                    }
                    QueryRangeEndResult::Unbound => {
                        for KVEntry(key, dataid) in &self.nodes[start_nodeid]
                            .get_entries()
                            .exact()[start_entryid..]
                        {
                            yield (key, self.data(*dataid))
                        }

                        let mut maybe_ptr = self.nodes[start_nodeid].next();

                        while let Some(ptr) = maybe_ptr {
                            let Node::Leaf { entries, next, .. } =
                                &self.nodes[ptr]
                            else {
                                unreachable!()
                            };

                            for KVEntry(key, dataid) in entries.exact() {
                                yield (key, self.data(*dataid))
                            }

                            maybe_ptr = *next;
                        }
                    },
                }
            },
        )
    }

    /// return Option<(nodeid, entryid)>
    fn range_start<Q, R>(&self, range: &R) -> Option<(usize, usize)>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
        R: RangeBounds<Q>,
    {
        if self.is_empty() {
            return None;
        }

        match range.start_bound() {
            Included(k) => {
                let (ptr, ..) = self.complete_search(k);

                let Node::Leaf { entries, next, .. } = &self.nodes[ptr] else {
                    unreachable!()
                };

                match entries
                    .exact()
                    .binary_search_by_key(&k, |KVEntry(key, _)| key.borrow())
                {
                    Ok(idx) => Some((ptr, idx)),
                    Err(idx) => {
                        if idx == entries.len() {
                            next.map(|next_dataid| (next_dataid, 0))
                        } else {
                            Some((ptr, idx))
                        }
                    }
                }
            }
            Excluded(_) => unimplemented!(),
            Unbounded => Some((self.min_node(), 0)),
        }
    }

    /// return Option<(nodeid, entryid)>
    fn range_end<Q, R>(&self, range: &R) -> Option<QueryRangeEndResult>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
        R: RangeBounds<Q>,
    {
        if self.is_empty() {
            return None;
        }

        match range.end_bound() {
            Included(k) => {
                let (ptr, ..) = self.complete_search(k);

                let Node::Leaf { entries, .. } = &self.nodes[ptr] else {
                    unreachable!()
                };

                match entries
                    .exact()
                    .binary_search_by_key(&k, |KVEntry(key, _)| key.borrow())
                {
                    Ok(idx) => Some(QueryRangeEndResult::Spec(ptr, idx)),
                    Err(idx) => {
                        if idx == 0 {
                            None
                        } else {
                            Some(QueryRangeEndResult::Spec(ptr, idx - 1))
                        }
                    }
                }
            }
            Excluded(k) => {
                let (ptr, maybe_ptr_of_p) = self.complete_search(k);

                let Node::Leaf { entries, .. } = &self.nodes[ptr] else {
                    unreachable!()
                };

                match entries
                    .exact()
                    .binary_search_by_key(&k, |KVEntry(key, _)| key.borrow())
                {
                    Ok(idx) => {
                        if idx == 0 {
                            if let Some(ptr_of_p) = maybe_ptr_of_p {
                                if let Some(prev) =
                                    self.prev_leaf(ptr, ptr_of_p)
                                {
                                    let entryid =
                                        self.nodes[prev].get_entries().len()
                                            - 1;

                                    return Some(QueryRangeEndResult::Spec(
                                        prev, entryid,
                                    ));
                                }
                            }

                            None
                        } else {
                            Some(QueryRangeEndResult::Spec(ptr, idx - 1))
                        }
                    }
                    Err(idx) => {
                        if idx == 0 {
                            None
                        } else {
                            Some(QueryRangeEndResult::Spec(ptr, idx - 1))
                        }
                    }
                }
            }
            Unbounded => Some(QueryRangeEndResult::Unbound),
        }
    }

    fn prev_leaf<Q>(&self, leaf: usize, leaf_of_p: usize) -> Option<usize>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        debug_assert!(matches!(self.nodes[leaf], Node::Leaf { .. }));

        /* trace upwards */

        if leaf == self.root {
            return None;
        }

        let mut p = self.nodes[leaf].paren();
        let mut p_node = &self.nodes[p];

        Some(if leaf_of_p == 0 {
            #[allow(unused_assignments)]
            let mut ptr = leaf;
            let mut ptr_of_p = leaf_of_p;

            loop {
                if ptr_of_p > 0 {
                    ptr = p_node.get_children()[ptr_of_p - 1].1;
                    break;
                }

                if p == self.root {
                    return None;
                }

                let p_node_k = p_node.get_children()[0].0.borrow();

                let pp = p_node.paren();
                let pp_node = &self.nodes[pp];

                let p_of_pp = pp_node
                    .get_children()
                    .exact()
                    .binary_search_by_key(&p_node_k, |KVEntry(k, _)| k.borrow())
                    .unwrap();

                p = pp;
                p_node = pp_node;
                ptr_of_p = p_of_pp;
            }

            let mut ptr_node = &self.nodes[ptr];

            while matches!(ptr_node, Node::Internal { .. }) {
                ptr = ptr_node.get_children().exact().last().unwrap().1;
                ptr_node = &self.nodes[ptr];
            }

            ptr
        } else {
            p_node.get_children()[leaf_of_p - 1].1
        })
    }

    fn min_node(&self) -> usize {
        let mut ptr = self.root;

        while let Node::Internal { children, .. } = &self.nodes[ptr] {
            ptr = children[0].1;
        }

        ptr
    }

    fn max_node(&self) -> usize {
        let mut ptr = self.root;

        while let Node::Internal { children, .. } = &self.nodes[ptr] {
            ptr = children.exact().last().unwrap().1;
        }

        ptr
    }

    /// apply within `insert` or `range`
    ///
    /// return `(nodeid, Option<x_of_p>)`
    fn complete_search<Q>(&self, k: &Q) -> (usize, Option<usize>)
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        let mut ptr = self.root;
        let mut maybe_ptr_of_p = None;

        while let Node::Internal { children, .. } = &self.nodes[ptr] {
            let ptr_of_p = match children
                .exact()
                .binary_search_by_key(&k, |KVEntry(key, _)| key.borrow())
            {
                Ok(idx) => idx,
                Err(idx) => {
                    if idx == 0 {
                        0
                    } else {
                        idx - 1
                    }
                }
            };

            ptr = children[ptr_of_p].1;
            maybe_ptr_of_p = Some(ptr_of_p);
        }

        (ptr, maybe_ptr_of_p)
    }

    /// apply within `get` or `remove`
    ///
    /// return `(nodeid, entryid, x_of_p)`
    fn shortcut_search<Q>(&self, k: &Q) -> Option<(usize, usize, Option<usize>)>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        let mut ptr = self.root;
        let mut maybe_x_of_p = None;

        while let Node::Internal { children, .. } = &self.nodes[ptr] {
            let x_of_p = match children
                .exact()
                .binary_search_by_key(&k, |KVEntry(key, _)| key.borrow())
            {
                Ok(idx) => idx,
                Err(idx) => {
                    if idx == 0 {
                        return None;
                    } else {
                        idx - 1
                    }
                }
            };

            ptr = children[x_of_p].1;
            maybe_x_of_p = Some(x_of_p);
        }

        self.nodes[ptr]
            .get_entries()
            .exact()
            .binary_search_by_key(&k, |KVEntry(key, _)| key.borrow())
            .ok()
            .map(|entryid| (ptr, entryid, maybe_x_of_p))
    }

    ////////////////////////////////////////////////////////////////////////////
    //// unpromote aux methods

    fn redistribute_with_left_leaf(
        &mut self,
        p: usize,
        x: usize,
        sib_lf: usize,
        x_of_p: usize,
    ) where
        K: Clone,
    {
        debug_assert!(self.nodes[x].is_empty());
        debug_assert!(x_of_p > 0);
        debug_assert!(self.nodes[sib_lf].len() > 1);

        let entry = self.nodes[sib_lf].get_entries_mut().pop();

        /* update key */
        self.nodes[p].get_children_mut()[x_of_p].0 = entry.0.clone();

        self.nodes[x].get_entries_mut().push(entry);
    }

    fn merge_into_left_leaf(
        &mut self,
        p: usize,
        x: usize,
        sib_lf: usize,
        x_of_p: usize,
    ) where
        K: Clone + Ord,
    {
        debug_assert!(self.nodes[x].is_empty());
        debug_assert!(x_of_p > 0);
        debug_assert!(self.nodes[sib_lf].len() == 1);

        let x_node = self.nodes.remove(x).unwrap();

        *self.nodes[sib_lf].get_next_mut() = x_node.next();

        /* update key */
        self.nodes[p].get_children_mut().remove(x_of_p);

        // NOTE: since x_of_p > 0, we don't need to update p's key upwards

        /* or else unpromote internal */

        self.or_else_unpromote_internal(p);
    }

    fn extend_right_leaf(
        &mut self,
        p: usize,
        x: usize,
        sib_rh: usize,
        x_of_p: usize,
    ) where
        K: Clone + Ord + Debug,
    {
        debug_assert!(self.nodes[x].is_empty());
        debug_assert!(x_of_p == 0);
        debug_assert!(self.nodes[sib_rh].len() == 1);

        let sib_rh_node = self.nodes.remove(sib_rh).unwrap();
        let Node::Leaf { entries, next, .. } = sib_rh_node else {
            unreachable!()
        };
        let Node::Leaf {
            entries: x_entries,
            next: x_next,
            ..
        } = &mut self.nodes[x]
        else {
            unreachable!()
        };

        x_entries.extend_slice(entries.exact());
        *x_next = next;

        /* update key */

        // remove sib_rh key of p
        let KVEntry(x_new_key, _) = self.nodes[p].get_children_mut().remove(1);

        // update x key of p
        let x_old_key =
            replace(&mut self.nodes[p].get_children_mut()[0].0, x_new_key);

        if p != self.root {
            self.update_index(p, x_old_key);
        }

        /* or else unpromote internal */

        self.or_else_unpromote_internal(p);
    }

    fn redistribute_with_right_leaf(
        &mut self,
        p: usize,
        x: usize,
        sib_rh: usize,
        x_of_p: usize,
    ) where
        K: Clone + Ord + Debug,
    {
        debug_assert!(self.nodes[x].is_empty());
        debug_assert!(x_of_p == 0);
        debug_assert!(self.nodes[sib_rh].len() > 1);

        let sib_rh_entries = self.nodes[sib_rh].get_entries_mut();

        let entry = sib_rh_entries.remove(0);
        let x_new_key = entry.0.clone();
        let sib_rh_new_key = sib_rh_entries[0].0.clone();

        self.nodes[x].get_entries_mut().push(entry);

        /* update key */

        let p_children = self.nodes[p].get_children_mut();

        let x_old_key = p_children[x_of_p].0.clone();
        p_children[x_of_p].0 = x_new_key;
        p_children[x_of_p + 1].0 = sib_rh_new_key;

        if p != self.root {
            self.update_index(p, x_old_key);
        }
    }

    fn unpromote_leaf(&mut self, x: usize, x_of_p: usize)
    where
        K: Clone + Ord + Debug,
    {
        debug_assert!(matches!(self.nodes[x], Node::Leaf { .. }));
        debug_assert!(x != self.root);

        let p = self.nodes[x].paren();

        // Redistribute with left first
        if x_of_p > 0 {
            let sib_lf = self.nodes[p].get_children()[x_of_p - 1].1;

            /* Redistribute with left leaf */
            if self.nodes[sib_lf].len() > 1 {
                self.redistribute_with_left_leaf(p, x, sib_lf, x_of_p);
            }
            /* Merge into left leaf */
            else {
                self.merge_into_left_leaf(p, x, sib_lf, x_of_p);
            }
        } else {
            debug_assert!(x_of_p < self.nodes[p].len() - 1);
            debug_assert!(x_of_p == 0);

            let sib_rh = self.nodes[p].get_children()[x_of_p + 1].1;

            /* Redistribute with right leaf */
            if self.nodes[sib_rh].len() > 1 {
                self.redistribute_with_right_leaf(p, x, sib_rh, x_of_p);
            }
            /* Extend right leaf */
            else {
                self.extend_right_leaf(p, x, sib_rh, x_of_p);
            }
        }
    }

    fn redistribute_with_left_internal(
        &mut self,
        p: usize,
        x: usize,
        sib_lf: usize,
        x_of_p: usize,
    ) where
        K: Clone,
    {
        debug_assert!(
            self.nodes[x].len() == Self::internal_cap_low_bound() - 1
        );
        debug_assert!(x_of_p > 0);
        debug_assert!(
            self.nodes[sib_lf].len() > Self::internal_cap_low_bound()
        );

        let child = self.nodes[sib_lf].get_children_mut().pop();

        /* update parent */

        *self.nodes[child.1].get_paren_mut() = x;

        /* update key */

        self.nodes[p].get_children_mut()[x_of_p].0 = child.0.clone();

        self.nodes[x].get_children_mut().insert(0, child);
    }

    fn merge_into_left_internal(
        &mut self,
        p: usize,
        x: usize,
        sib_lf: usize,
        x_of_p: usize,
    ) where
        K: Clone + Ord,
    {
        debug_assert!(
            self.nodes[x].len() == Self::internal_cap_low_bound() - 1
        );
        debug_assert!(x_of_p > 0);
        debug_assert!(
            self.nodes[sib_lf].len() == Self::internal_cap_low_bound()
        );

        let x_node = self.nodes.remove(x).unwrap();

        let x_children = x_node.get_children().exact();

        self.nodes[sib_lf]
            .get_children_mut()
            .extend_slice(x_children);

        /* update parent */

        for &KVEntry(_, child) in x_children {
            *self.nodes[child].get_paren_mut() = sib_lf;
        }

        /* update key */

        self.nodes[p].get_children_mut().remove(x_of_p);

        /* or else unpromote internal */

        self.or_else_unpromote_internal(p);
    }

    fn extend_right_internal(
        &mut self,
        p: usize,
        x: usize,
        sib_rh: usize,
        x_of_p: usize,
    ) where
        K: Clone + Ord,
    {
        debug_assert!(
            self.nodes[x].len() == Self::internal_cap_low_bound() - 1
        );
        debug_assert!(
            self.nodes[p].len() >= 2 && x_of_p < self.nodes[p].len() - 1
        );
        debug_assert!(
            self.nodes[sib_rh].len() == Self::internal_cap_low_bound()
        );

        let sib_rh_node = self.nodes.remove(sib_rh).unwrap();
        let sib_rh_children = sib_rh_node.get_children().exact();

        self.nodes[x]
            .get_children_mut()
            .extend_slice(sib_rh_children);

        /* update parent */

        for &KVEntry(_, child) in sib_rh_children {
            *self.nodes[child].get_paren_mut() = x;
        }

        /* update key */

        self.nodes[p].get_children_mut().remove(x_of_p + 1);

        /* or else unpromote internal */

        self.or_else_unpromote_internal(p);
    }

    fn redistribute_with_right_internal(
        &mut self,
        p: usize,
        x: usize,
        sib_rh: usize,
        x_of_p: usize,
    ) where
        K: Clone,
    {
        debug_assert!(
            self.nodes[x].len() == Self::internal_cap_low_bound() - 1
        );
        debug_assert!(
            self.nodes[p].len() >= 2 && x_of_p < self.nodes[p].len() - 1
        );
        debug_assert!(
            self.nodes[sib_rh].len() > Self::internal_cap_low_bound()
        );

        let sib_rh_children = self.nodes[sib_rh].get_children_mut();

        /* right remove 0 && x push */
        let child_entry = sib_rh_children.remove(0);
        let sib_rh_new_key = sib_rh_children[0].0.clone();
        let child = child_entry.1;
        self.nodes[x].get_children_mut().push(child_entry);

        /* update parent */

        *self.nodes[child].get_paren_mut() = x;

        /* update key */

        // since x isn't empty, we don't need to update key for it
        self.nodes[p].get_children_mut()[x_of_p + 1].0 = sib_rh_new_key;
    }

    fn or_else_unpromote_internal(&mut self, x: usize)
    where
        K: Clone + Ord,
    {
        debug_assert!(matches!(self.nodes[x], Node::Internal { .. }));
        debug_assert!(
            x == self.root && self.nodes[x].len() >= 1
                || x != self.root
                    && self.nodes[x].len()
                        >= Self::internal_cap_low_bound() - 1
        );

        // x be either root or not root

        let x_len = self.nodes[x].len();
        let bound = Self::internal_cap_low_bound();

        if x == self.root && x_len == 1 {
            /* pop level */
            // TDOO: swap to ensure root == 0
            if let Node::Internal { children, .. } =
                self.nodes.remove(x).unwrap()
            {
                self.root = children[0].1;
            }
        } else if x != self.root && x_len < bound {
            let Node::Internal { children, paren } = &self.nodes[x] else {
                unreachable!()
            };
            let p = *paren;
            let p_children = self.nodes[p].get_children();
            let x_of_p =
                p_children.exact().binary_search(&children[0]).unwrap();

            if x_of_p > 0 {
                let sib_lf = p_children[x_of_p - 1].1;

                if self.nodes[sib_lf].len() > bound {
                    return self
                        .redistribute_with_left_internal(p, x, sib_lf, x_of_p);
                } else if x_of_p < self.nodes[p].len() - 1 {
                    let sib_rh = p_children[x_of_p + 1].1;

                    if self.nodes[sib_rh].len() > bound {
                        return self.redistribute_with_right_internal(
                            p, x, sib_rh, x_of_p,
                        );
                    }
                }

                self.merge_into_left_internal(p, x, sib_lf, x_of_p);
            } else {
                let sib_rh = p_children[x_of_p + 1].1;

                if self.nodes[sib_rh].len() > bound {
                    self.redistribute_with_right_internal(p, x, sib_rh, x_of_p);
                } else {
                    self.extend_right_internal(p, x, sib_rh, x_of_p);
                }
            }
        }
    }

    /// update min key
    ///
    /// `x_old_key` to find x_of_p
    ///
    /// `x` to find `p` and `x_new_key`
    fn update_index(&mut self, x: usize, x_old_key: K)
    where
        K: Clone + Ord,
    {
        debug_assert!(x != self.root);

        let old_key_entry = KVEntry(x_old_key, 0);
        let new_key = self.nodes[x].k();

        let mut ptr = x;

        loop {
            let p = self.nodes[ptr].paren();
            let p_children = self.nodes[p].get_children_mut();

            let Ok(x_of_p) = p_children.exact().binary_search(&old_key_entry)
            else {
                panic!(
                    // "{}",
                    // self.display_fault(
                    //     &self.nodes[ptr],
                    //     &format! {
                    //         "not found {:?} on {:?}",
                    //         old_key_entry.0,
                    //         self.nodes[ptr].get_keys()
                    //     }
                    // )
                )
            };

            p_children[x_of_p].0 = new_key.clone();

            if p != self.root && x_of_p == 0 {
                ptr = p;
            } else {
                break;
            }
        }
    }

    /// Lazy promote
    fn promote(&mut self, x: usize)
    where
        K: Clone + Ord,
    {
        debug_assert!(self.nodes[x].is_full());

        let at = M.div_ceil(2);

        /* split evenly node x into x and x2 */

        let x2_node = match &mut self.nodes[x] {
            Node::Leaf { entries, next, .. } => Node::new_leaf()
                .with_next(*next)
                .with_entries_slice(&entries.split_off(at)),
            Node::Internal { children, .. } => {
                Node::new_internal().with_children_slice(children.split_off(at))
            }
        };

        let x2 = self.nodes.push(x2_node);

        if let Node::Leaf { next, .. } = &mut self.nodes[x] {
            *next = Some(x2);
        } else {
            for child in self.nodes[x2].children() {
                *self.nodes[child].get_paren_mut() = x2;
            }
        }

        let x2_key = self.nodes[x2].k();

        /* push level */
        if x == self.root {
            // TODO: swap node to ensure root = 0

            let x_key = self.nodes[x].k();

            self.root =
                self.nodes.push(Node::new_internal().with_children_slice(&[
                    KVEntry(x_key, x),
                    KVEntry(x2_key, x2),
                ]));

            *self.nodes[x].get_paren_mut() = self.root;
            *self.nodes[x2].get_paren_mut() = self.root;
        }
        /* insert into parent code */
        else {
            let mut p = self.nodes[x].paren();

            if self.nodes[p].is_full() {
                self.promote(p);
                p = self.nodes[x].paren();
            }

            *self.nodes[x2].get_paren_mut() = p;

            self.nodes[p]
                .get_children_mut()
                .binary_insert(KVEntry(x2_key, x2));
        }
    }

    fn height(&self) -> usize {
        let mut ptr = self.root;
        let mut h = 1;

        while let Node::Internal { children, .. } = &self.nodes[ptr] {
            ptr = children[0].1;
            h += 1;
        }

        h
    }

    fn leaves<'a>(&'a self) -> impl Iterator<Item = &'a Node<K, M>> + 'a {
        std::iter::from_coroutine(
            #[coroutine]
            || {
                let mut ptr_node = &self.nodes[self.min_node()];

                while let Some(next) = ptr_node.next() {
                    yield ptr_node;
                    ptr_node = &self.nodes[next];
                }
            },
        )
    }

    fn display_loc_on_tree<'a>(&'a self) -> DisplayLocOnTree<'a, Self> {
        let revref = self;
        let max_ln_width = format!("{}", self.height()).len();
        let max_col_group_width = format!(
            "{}",
            self.leaves()
                .map(|x_node| x_node.paren())
                .collect::<std::collections::HashSet<_>>()
                .len()
        )
        .len();
        let max_in_group_width = format!("{M}").len();

        DisplayLocOnTree {
            revref,
            max_ln_width,
            max_col_group_width,
            max_in_group_width,
        }
    }
}

impl<K, V, const M: usize> Drop for FlatBPT<K, V, M> {
    fn drop(&mut self) {
        for valptr in self.data.iter().cloned() {
            unsafe {
                drop(Box::from_raw(valptr));
            }
        }
    }
}

impl<K: Clone + Ord, V, const M: usize> IntoIterator for FlatBPT<K, V, M> {
    type Item = (K, V);

    type IntoIter = impl Iterator<Item = Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.into_iter()
    }
}

impl<K: Clone + Ord + Debug, V, const M: usize> FromIterator<(K, V)>
    for FlatBPT<K, V, M>
{
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        let mut inputs = iter.into_iter().collect::<Vec<_>>();

        inputs.sort_by(|a, b| a.0.cmp(&b.0));

        Self::bulk_build(inputs.into_iter())

        // let mut flatbpt = FlatBPT::new();

        // for (key, value) in iter {
        //     flatbpt.push_back(key, value);
        // }

        // flatbpt
    }
}

impl<K: Ord + Debug, V, const M: usize> Debug for FlatBPT<K, V, M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(&format!("FlatBPT [M={}]", M))
            .field("len", &self.len())
            .field("(height)", &self.height())
            .field("(leaves)", &self.leaves().count())
            .field("(nodes)", &self.nodes.len())
            .finish()
    }
}

impl<'a, K: Ord, V, const M: usize> WalkTree<'a> for FlatBPT<K, V, M> {
    type Node = Node<K, M>;
    type NodeBorrow = Node<K, M>;

    fn root(&'a self) -> Option<&'a Self::NodeBorrow> {
        if self.is_empty() {
            None
        } else {
            Some(&self.nodes[self.root])
        }
    }

    fn children(
        &'a self,
        ptr: &'a Self::NodeBorrow,
    ) -> Option<Vec<&'a Self::NodeBorrow>> {
        match ptr {
            Node::Leaf { .. } => None,
            Node::Internal { children, .. } => Some(
                children
                    .exact()
                    .iter()
                    .map(|KVEntry(_, nodeid)| &self.nodes[*nodeid])
                    .collect(),
            ),
        }
    }
}

impl<K: Ord + Debug + Clone, V, const M: usize> Display for FlatBPT<K, V, M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{self:#?}")?;

        let pre_order_view = self
            .pre_order_walk()
            .into_iter()
            .collect::<PreOrderView<_>>();

        struct DisplayWalk<'a, K, const M: usize> {
            loc: LocOnTree,
            node: &'a Node<K, M>,
            display_keys: String,
            display_paren: String,
        }

        let dlot = self.display_loc_on_tree();
        let display_line_prefix_sep = |ln: usize| -> String {
            format!("{ln:0ln_width$})", ln_width = dlot.max_ln_width)
        };
        let display_end_line = || -> String {
            format!(
                "{} {:^width$} {}",
                "-".repeat(6),
                "end",
                "-".repeat(6),
                width = dlot.max_ln_width + 2
            )
        };

        let display_walk = pre_order_view
            .iter()
            .map(|(loc, node)| {
                let display_keys = format!("{:?}", node.get_keys());
                let display_paren = if ptr::eq(&self.nodes[self.root], node) {
                    format!("ROOT")
                } else {
                    let paren_node = &self.nodes[node.paren()];

                    format!(
                        "p: {:?}({})",
                        paren_node.k(),
                        dlot.display(&pre_order_view[paren_node])
                    )
                };

                DisplayWalk {
                    loc,
                    node,
                    display_keys,
                    display_paren,
                }
            })
            .collect::<Vec<_>>();

        let (max_display_keys_width, max_display_paren_width) =
            display_walk.iter().fold((0, 0), |(kl, pl), x| {
                (
                    max(kl, x.display_keys.len()),
                    max(pl, x.display_paren.len()),
                )
            });

        let mut preloc = LocOnTree::new();

        for DisplayWalk {
            loc,
            node,
            display_keys,
            display_paren,
        } in display_walk
        {
            let LocOnTree { ln, col_group, .. } = loc;

            if ln > preloc.ln {
                writeln!(f, "\n{}", display_line_prefix_sep(ln))?;
                preloc.col_group = 0;
            }

            let display_loc = dlot.display(&loc);

            if col_group > preloc.col_group {
                let display_ln_col_group =
                    display_loc.rsplit_once('.').unwrap().0;

                writeln!(f, "{display_ln_col_group})")?;
            }

            match node {
                Node::Internal { .. } => {
                    if ptr::eq(node, &self.nodes[self.root]) {
                        writeln!(
                            f,
                            "{}) {:<keys_width$} ROOT",
                            display_loc,
                            display_keys,
                            keys_width = max_display_keys_width,
                        )?;
                    } else {
                        writeln!(
                            f,
                            "{}) {:<keys_width$} {:<paren_width$}",
                            display_loc,
                            display_keys,
                            display_paren,
                            keys_width = max_display_keys_width,
                            paren_width = max_display_paren_width,
                        )?;
                    }
                }
                Node::Leaf { next, .. } => {
                    writeln!(
                        f,
                        "{}) {:<keys_width$} {:<paren_width$} -> {}",
                        display_loc,
                        display_keys,
                        display_paren,
                        if let Some(next) = next {
                            let next_node = &self.nodes[*next];

                            format!(
                                "{:?}({})",
                                next_node.k(),
                                dlot.display(&pre_order_view[next_node])
                            )
                        } else {
                            "END".to_string()
                        },
                        keys_width = max_display_keys_width,
                        paren_width = max_display_paren_width,
                    )?;
                }
            }

            preloc = loc;
        }

        writeln!(f, "\n{}\n", display_end_line())
    }
}



////////////////////////////////////////
//// impl Node

impl<K, const M: usize> Node<K, M> {
    pub fn new_leaf() -> Self {
        Self::Leaf {
            entries: PartialInitArray::new(),
            next: None,
            paren: 0,
        }
    }

    pub fn new_internal() -> Self {
        Self::Internal {
            children: PartialInitArray::new(),
            paren: 0,
        }
    }

    /// Leaf build method
    pub fn with_entries_slice(
        mut self,
        entryslice: &[KVEntry<K, usize>],
    ) -> Self {
        if let Self::Leaf { entries, .. } = &mut self {
            entries.init_with_slice(entryslice);
            self
        } else {
            unreachable!()
        }
    }

    pub fn with_entries_iter<I: Iterator<Item = KVEntry<K, usize>>>(
        mut self,
        entryiter: I,
    ) -> Self {
        self.get_entries_mut().extend(entryiter);

        self
    }

    /// Leaf build method
    pub fn with_next(mut self, new_next: Option<usize>) -> Self {
        if let Self::Leaf { next, .. } = &mut self {
            *next = new_next;

            self
        } else {
            unreachable!()
        }
    }

    /// Internal build method
    pub fn with_children_slice(
        mut self,
        childrenslice: &[KVEntry<K, usize>],
    ) -> Self {
        if let Self::Internal { children, .. } = &mut self {
            children.init_with_slice(childrenslice);
            self
        } else {
            unreachable!()
        }
    }

    pub fn with_children_iter<I: Iterator<Item = KVEntry<K, usize>>>(
        mut self,
        childreniter: I,
    ) -> Self {
        self.get_children_mut().extend(childreniter);

        self
    }

    pub fn with_paren(mut self, paren: usize) -> Self {
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
            Self::Leaf { entries, .. } => entries[0].0.clone(),
        }
    }

    pub fn get_keys(&self) -> Vec<&K> {
        match self {
            Self::Internal { children, .. } => {
                children.exact().iter().map(|KVEntry(k, _)| k).collect()
            }
            Self::Leaf { entries, .. } => {
                entries.exact().iter().map(|KVEntry(k, _)| k).collect()
            }
        }
    }

    pub fn next(&self) -> Option<usize> {
        match self {
            Self::Internal { .. } => unreachable!(),
            Self::Leaf { next, .. } => *next,
        }
    }

    pub fn get_next_mut(&mut self) -> &mut Option<usize> {
        match self {
            Self::Internal { .. } => unreachable!(),
            Self::Leaf { next, .. } => next,
        }
    }

    pub fn paren(&self) -> usize {
        match self {
            Self::Internal { paren, .. } => *paren,
            Self::Leaf { paren, .. } => *paren,
        }
    }

    pub fn get_paren_mut(&mut self) -> &mut usize {
        match self {
            Self::Internal { paren, .. } => paren,
            Self::Leaf { paren, .. } => paren,
        }
    }

    pub fn children(&self) -> Vec<usize> {
        self.get_children()
            .exact()
            .iter()
            .map(|KVEntry(_, child)| child)
            .cloned()
            .collect()
    }

    pub fn get_children(&self) -> &PartialInitArray<KVEntry<K, usize>, M> {
        match self {
            Self::Internal { children, .. } => children,
            Self::Leaf { .. } => unreachable!(),
        }
    }

    pub fn get_children_mut(
        &mut self,
    ) -> &mut PartialInitArray<KVEntry<K, usize>, M> {
        match self {
            Self::Internal { children, .. } => children,
            Self::Leaf { .. } => unreachable!(),
        }
    }

    pub fn get_entries(&self) -> &PartialInitArray<KVEntry<K, usize>, M> {
        match self {
            Self::Internal { .. } => unreachable!(),
            Self::Leaf { entries, .. } => entries,
        }
    }

    pub fn get_entries_mut(
        &mut self,
    ) -> &mut PartialInitArray<KVEntry<K, usize>, M> {
        match self {
            Self::Internal { .. } => unreachable!(),
            Self::Leaf { entries, .. } => entries,
        }
    }
}



#[cfg(test)]
mod tests {
    use log::{info, trace};
    use test_suites::{bpt_mapping::*, *};

    use super::*;

    pub trait Key = Ord + Debug + Clone;

    impl<K: Ord + Debug + Clone, V, const M: usize> FlatBPT<K, V, M> {
        pub fn validate(&self) {
            let dlot = self.display_loc_on_tree();

            if self.is_empty() {
                trace!("{}", dlot.display_pass());
                return;
            }

            let h: usize = self.height();

            let pre_order_view = self
                .pre_order_walk()
                .into_iter()
                .collect::<PreOrderView<_>>();

            for (loc, ptr_node) in pre_order_view.iter() {
                let LocOnTree { ln, .. } = loc;

                match ptr_node {
                    Node::Internal { children, .. } => {
                        assert!(ln < h, "{}", dlot.display_fault(&loc));

                        if ptr::eq(&self.nodes[self.root], ptr_node) {
                            assert!(
                                children.len() >= 2,
                                "{}",
                                dlot.display_fault(&loc)
                            );
                        } else {
                            assert!(
                                children.len()
                                    >= Self::internal_cap_low_bound(),
                                "{}",
                                dlot.display_fault(&loc)
                            );
                        }
                        assert!(
                            children.exact().iter().is_sorted(),
                            "{}",
                            dlot.display_fault(&loc)
                        );

                        for KVEntry(child_k, child) in children.exact().iter() {
                            let child_node = &self.nodes[*child];

                            assert_eq!(
                                self.nodes[child_node.paren()].k(),
                                ptr_node.k(),
                                "{}",
                                dlot.display_fault(&loc)
                            );
                            assert_eq!(
                                child_node.k(),
                                *child_k,
                                "{}",
                                dlot.display_fault(&loc)
                            );
                        }
                    }
                    Node::Leaf { entries, next, .. } => {
                        assert!(ln == h, "{}", dlot.display_fault(&loc));
                        assert!(
                            entries.len() >= 1,
                            "{}",
                            dlot.display_fault(&loc)
                        );
                        assert!(
                            entries.exact().is_sorted(),
                            "{}",
                            dlot.display_fault(&loc)
                        );

                        let lastk = &entries.exact().last().unwrap().0;

                        next.inspect(|next| {
                            assert!(
                                *lastk < self.nodes[*next].k(),
                                "{}",
                                dlot.display_fault(&loc)
                            )
                        });
                    }
                }
            }

            trace!("{}", dlot.display_pass());
        }
    }

    impl<K: Key, V, const M: usize> Validate for FlatBPT<K, V, M> {
        fn validate(&self) {
            self.validate();
        }
    }

    impl<K: Key, V, const M: usize> Collection for FlatBPT<K, V, M> {
        fn len(&self) -> usize {
            self.len()
        }

        fn new() -> Self {
            Self::new()
        }
    }

    impl<K: Key, V, const M: usize> MappingIterable for FlatBPT<K, V, M> {
        type Key = K;
        type Value = V;

        fn iter<'a>(
            &'a self,
        ) -> impl Iterator<Item = (&'a Self::Key, &'a Self::Value)> + 'a
        where
            Self::Key: 'a,
            Self::Value: 'a,
        {
            self.iter()
        }
    }

    impl<K: Key + Borrow<Q>, V, const M: usize, Q: Ord> Mapping<Q>
        for FlatBPT<K, V, M>
    {
        fn get(&self, key: &Q) -> Option<&Self::Value> {
            self.get(key)
        }
    }

    impl<K: Key + Borrow<Q>, V, const M: usize, Q: Ord> MutableMapping<Q>
        for FlatBPT<K, V, M>
    {
        fn insert(
            &mut self,
            key: Self::Key,
            value: Self::Value,
        ) -> Option<Self::Value> {
            self.insert(key, value)
        }

        fn remove(&mut self, key: &Q) -> Option<Self::Value> {
            self.remove(key)
        }
    }

    impl<K: Key + Borrow<Q>, V, const M: usize, Q: Ord> BPTreeMap<Q>
        for FlatBPT<K, V, M>
    {
        fn range<R>(
            &self,
            range: R,
        ) -> impl Iterator<Item = (&Self::Key, &Self::Value)>
        where
            R: RangeBounds<Q>,
        {
            self.range(range)
        }

        fn range_mut<R>(
            &mut self,
            range: R,
        ) -> impl Iterator<Item = (&Self::Key, &mut Self::Value)>
        where
            R: RangeBounds<Q>,
        {
            self.range_mut(range)
        }
    }

    impl<K: Key + Default, V, const M: usize> BulkLoad for FlatBPT<K, V, M> {
        type BulkItem = KVEntry<K, V>;

        fn bulk_load<T: IntoIterator<Item = Self::BulkItem>>(iter: T) -> Self {
            Self::bulk_build(iter.into_iter().map(|KVEntry(k, v)| (k, v)))
        }
    }

    #[test]
    fn test_flatbpt() {
        pretty_env_logger::init();

        test_flatbpt_::<4>();
        test_flatbpt_::<5>();
        test_flatbpt_::<6>();
        test_flatbpt_::<7>();
        test_flatbpt_::<11>();
        test_flatbpt_::<20>();
        test_flatbpt_::<21>();
        test_flatbpt_::<100>();
        test_flatbpt_::<101>();
    }

    fn test_flatbpt_<const M: usize>() {
        let loader =
            MixedLoader::<FlatBPT<i32, i32, M>, _>::new_with_bulkloader(
                BulkLoader::<_, GenerateI32Any>::new_with_upper_bound(2000),
            );
        // let loader: DefaultLoader<FlatBPT<i32, i32, M>> = DefaultLoader::<FlatBPT<i32, i32, M>>::new();
        // let loader =
        //     BulkLoader::<i32, GenerateI32Any>::new_with_upper_bound(20_000);

        let mut test_suit = BPTreeTestSuite::<
            _,
            GenerateI32Any,
            _,
            _,
            FlatBPT<_, _, M>,
        >::new_with_loader(loader);

        test_suit.test_fixeddata();
        test_suit.test_randomdata(100, 2000);

        info!("PASS FlatBPT {M}");
    }
}
