use std::{
    fmt::Debug,
    num::NonZeroUsize,
    ops::{Deref, DerefMut},
};

use coll::{KVEntry, OwnedPtr, Ptr};

use crate::bt::StackVec;

////////////////////////////////////////////////////////////////////////////////
//// Macros


////////////////////////////////////////////////////////////////////////////////
//// Traits

pub trait TreeMap: Sized {
    type K;
    type V;

    fn new() -> Self;

    fn len(&self) -> usize;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn first(&self) -> Option<(&Self::K, &Self::V)> {
        self.nth(0)
    }

    fn first_value(&self) -> Option<&Self::V> {
        self.nth(0).map(|(_k, v)| v)
    }

    fn nth(&self, idx: usize) -> Option<(&Self::K, &Self::V)>;

    fn nth_value_mut(&mut self, idx: usize) -> Option<&mut Self::V>;

    fn rank<Q>(&self, k: &Q) -> Result<usize, usize>
    where
        Self::K: std::borrow::Borrow<Q>,
        Q: Ord + ?Sized;

    ///
    /// `Some(Ok(&V)): node.k == k`
    ///
    /// `Some(Err(&V)): node.k < k`
    ///
    /// `None: k < min_key`
    fn lower_bound_search<Q>(
        &self,
        k: &Q,
    ) -> Option<Result<&Self::V, &Self::V>>
    where
        Self::K: std::borrow::Borrow<Q>,
        Q: Ord + ?Sized;

    fn get_min_key(&self) -> Option<&Self::K>;

    fn get<Q>(&self, k: &Q) -> Option<&Self::V>
    where
        Self::K: std::borrow::Borrow<Q>,
        Q: Ord + ?Sized;

    fn get_mut<Q>(&mut self, k: &Q) -> Option<&mut Self::V>
    where
        Self::K: std::borrow::Borrow<Q>,
        Q: Ord + ?Sized;

    fn iter(&self) -> impl Iterator<Item = (&Self::K, &Self::V)>;

    fn drain_all(&mut self) -> impl Iterator<Item = (Self::K, Self::V)>;

    fn keys(&self) -> impl Iterator<Item = &Self::K> {
        self.iter().map(|(k, _v)| k)
    }

    fn values(&self) -> impl Iterator<Item = &Self::V> {
        self.iter().map(|(_k, v)| v)
    }

    fn values_mut(&mut self) -> impl Iterator<Item = &mut Self::V>;

    fn insert(&mut self, k: Self::K, v: Self::V) -> Option<Self::V>
    where
        Self::K: Ord;

    fn remove<Q>(&mut self, k: &Q) -> Option<Self::V>
    where
        Self::K: std::borrow::Borrow<Q>,
        Q: Ord + ?Sized;

    ///
    /// replace `(old_k, v)` to `(new_k, v)`
    ///
    /// if `old_k` == `new_k`, do nothing;
    ///
    /// if `old_k` doesn't exist, do nothing;
    ///
    /// if `new_k` exist,replace and return old value.
    ///
    /// There is a low efficient implementation.
    fn update_index<Q>(&mut self, old_k: &Q, new_k: Self::K) -> Option<Self::V>
    where
        Self::K: std::borrow::Borrow<Q> + Clone + Ord,
        Q: Ord,
    {
        self.remove(old_k).map(|x| self.insert(new_k, x)).flatten()
    }

    fn pop_first(&mut self) -> Option<(Self::K, Self::V)>;

    fn pop_last(&mut self) -> Option<(Self::K, Self::V)>;

    /// k should be less than min_key
    fn push_front(&mut self, k: Self::K, v: Self::V);

    /// k should be greate than max_key
    fn push_back(&mut self, k: Self::K, v: Self::V);

    /// split as [0, at) and return [at, len)
    ///
    /// # Panics
    ///
    /// Panics if at > len.
    fn split_off(&mut self, at: usize) -> Self;

    #[cfg(test)]
    fn validate(&self)
    where
        Self::K: Ord,
    {
        assert!(self.keys().is_sorted())
    }

    fn with_kv(mut self, k: Self::K, v: Self::V) -> Self
    where
        Self::K: Ord,
    {
        self.insert(k, v);
        self
    }
}


////////////////////////////////////////////////////////////////////////////////
//// Structures

/// Sorted Vec
#[repr(transparent)]
pub struct VecMap<K, V, const C: usize>(Vec<KVEntry<K, V>>);

#[repr(transparent)]
struct StackVecMap<K, V, const C: usize>(StackVec<KVEntry<K, V>, C>);

enum Node<K, V, const M: usize> {
    Leaf {
        entries: StackVecMap<K, V, M>,
        next: Option<Ptr<Self>>,
        paren: Option<Ptr<Self>>,
    },
    Internal {
        children: StackVecMap<K, OwnedPtr<Self>, M>,
        paren: Option<Ptr<Self>>,
    },
}

struct Tree<K, V, const M: usize> {
    root: OwnedPtr<Node<K, V, M>>,
    min_node: Ptr<Node<K, V, M>>,
    cnt: NonZeroUsize,
}

#[derive(Debug)]
pub struct BPT<K, V, const M: usize = 32> {
    tree: Option<Tree<K, V, M>>,
}

////////////////////////////////////////////////////////////////////////////////
//// Implementations

impl<K: Debug, V, const M: usize> Debug for Tree<K, V, M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Tree")
            .field("root", &self.root)
            .field("min_node", &self.min_node)
            .field("cnt", &self.cnt)
            .finish()
    }
}

impl<K, V, const C: usize> VecMap<K, V, C> {
    pub const fn len(&self) -> usize {
        self.0.len()
    }

    pub fn pop_second(&mut self) -> Option<(K, V)> {
        if self.len() < 2 {
            None
        } else {
            Some(self.0.remove(1).into())
        }
    }
}

impl<K, V, const C: usize> TreeMap for VecMap<K, V, C> {
    type K = K;
    type V = V;

    fn new() -> Self {
        VecMap(Vec::with_capacity(C))
    }

    fn len(&self) -> usize {
        self.len()
    }

    fn nth(&self, idx: usize) -> Option<(&Self::K, &Self::V)> {
        self.0.get(idx).map(|kv| (&kv.0, &kv.1))
    }

    fn nth_value_mut(&mut self, idx: usize) -> Option<&mut Self::V> {
        self.0.get_mut(idx).map(|kv| &mut kv.1)
    }

    fn rank<Q>(&self, k: &Q) -> Result<usize, usize>
    where
        Self::K: std::borrow::Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.0.binary_search_by_key(&k, |kv| kv.0.borrow())
    }

    fn lower_bound_search<Q>(&self, k: &Q) -> Option<Result<&Self::V, &Self::V>>
    where
        Self::K: std::borrow::Borrow<Q>,
        Q: Ord + ?Sized,
    {
        match self.rank(k) {
            Ok(idx) => Some(Ok(&self.0[idx].1)),
            Err(idx) => {
                if idx == 0 {
                    None
                } else {
                    Some(Err(&self.0[idx - 1].1))
                }
            }
        }
    }

    fn get_min_key(&self) -> Option<&Self::K> {
        self.0.get(0).map(|kv| &kv.0)
    }

    fn get<Q>(&self, k: &Q) -> Option<&Self::V>
    where
        Self::K: std::borrow::Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.rank(k).ok().map(|idx| &self.0[idx].1)
    }

    fn get_mut<Q>(&mut self, k: &Q) -> Option<&mut Self::V>
    where
        Self::K: std::borrow::Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.rank(k).ok().map(|idx| &mut self.0[idx].1)
    }

    fn iter(&self) -> impl Iterator<Item = (&Self::K, &Self::V)> {
        self.0.iter().map(|kv| (&kv.0, &kv.1))
    }

    fn drain_all(&mut self) -> impl Iterator<Item = (Self::K, Self::V)> {
        self.0.drain(..).map(|kv| kv.into())
    }

    fn values_mut(&mut self) -> impl Iterator<Item = &mut Self::V> {
        self.0.iter_mut().map(|kv| &mut kv.1)
    }

    fn insert(&mut self, k: Self::K, v: Self::V) -> Option<Self::V>
    where
        K: Ord,
    {
        match self.rank(&k) {
            Ok(idx) => Some(core::mem::replace(&mut self.0[idx].1, v)),
            Err(idx) => {
                self.0.insert(idx, KVEntry(k, v));
                None
            }
        }
    }

    fn remove<Q>(&mut self, k: &Q) -> Option<Self::V>
    where
        Self::K: std::borrow::Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.rank(&k).ok().map(|idx| self.0.remove(idx).1)
    }

    fn update_index<Q>(&mut self, old_k: &Q, new_k: Self::K) -> Option<Self::V>
    where
        Self::K: std::borrow::Borrow<Q>,
        Q: Ord,
    {
        if new_k.borrow() == old_k {
            return None;
        }

        let Ok(old_idx) = self.rank(old_k) else {
            return None;
        };

        // It's usually better than Enum compare in Rust impl
        if new_k.borrow() < old_k {
            match self.0[..old_idx]
                .binary_search_by_key(&new_k.borrow(), |kv| kv.0.borrow())
            {
                Ok(new_idx) => {
                    let old_v = self.0.remove(old_idx).1;

                    Some(core::mem::replace(&mut self.0[new_idx].1, old_v))
                }
                Err(new_idx) => {
                    // rotate right 1 step to swap
                    self.0[new_idx..=old_idx].rotate_right(1);
                    self.0[new_idx].0 = new_k;

                    None
                }
            }
        } else {
            let base = old_idx + 1;

            match self.0[base..]
                .binary_search_by_key(&new_k.borrow(), |kv| kv.0.borrow())
            {
                Ok(sub_idx) => {
                    let new_idx = base + sub_idx - 1;

                    let old_v = self.0.remove(old_idx).1;

                    Some(core::mem::replace(&mut self.0[new_idx].1, old_v))
                }
                Err(sub_idx) => {
                    let new_idx = base + sub_idx - 1;

                    // rotate left 1 step to swap
                    self.0[old_idx..=new_idx].rotate_left(1);
                    self.0[new_idx].0 = new_k;

                    None
                }
            }
        }
    }

    fn pop_first(&mut self) -> Option<(Self::K, Self::V)> {
        if !self.is_empty() {
            Some(self.0.remove(0).into())
        } else {
            None
        }
    }

    fn pop_last(&mut self) -> Option<(Self::K, Self::V)> {
        self.0.pop().map(|kv| kv.into())
    }

    fn push_front(&mut self, k: Self::K, v: Self::V) {
        self.0.insert(0, KVEntry(k, v));
    }

    fn push_back(&mut self, k: Self::K, v: Self::V) {
        self.0.push(KVEntry(k, v));
    }

    fn split_off(&mut self, at: usize) -> Self {
        Self(self.0.split_off(at))
    }
}

impl<K, V, const C: usize> StackVecMap<K, V, C> {
    pub const fn new() -> Self {
        Self(StackVec::new())
    }

    pub const fn len(&self) -> usize {
        self.0.len()
    }

    pub fn pop_second(&mut self) -> Option<(K, V)> {
        if self.len() < 2 {
            None
        } else {
            Some(self.0.remove(1).into())
        }
    }
}

impl<K, V, const C: usize> TreeMap for StackVecMap<K, V, C> {
    type K = K;
    type V = V;

    fn new() -> Self {
        Self::new()
    }

    fn len(&self) -> usize {
        self.len()
    }

    fn nth(&self, idx: usize) -> Option<(&Self::K, &Self::V)> {
        self.0.get(idx).map(|kv| (&kv.0, &kv.1))
    }

    fn nth_value_mut(&mut self, idx: usize) -> Option<&mut Self::V> {
        self.0.get_mut(idx).map(|kv| &mut kv.1)
    }

    fn rank<Q>(&self, k: &Q) -> Result<usize, usize>
    where
        Self::K: std::borrow::Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.0.binary_search_by_key(&k, |kv| kv.0.borrow())
    }

    fn lower_bound_search<Q>(&self, k: &Q) -> Option<Result<&Self::V, &Self::V>>
    where
        Self::K: std::borrow::Borrow<Q>,
        Q: Ord + ?Sized,
    {
        match self.rank(k) {
            Ok(idx) => Some(Ok(&self.0[idx].1)),
            Err(idx) => {
                if idx == 0 {
                    None
                } else {
                    Some(Err(&self.0[idx - 1].1))
                }
            }
        }
    }

    fn get_min_key(&self) -> Option<&Self::K> {
        self.0.get(0).map(|kv| &kv.0)
    }

    fn get<Q>(&self, k: &Q) -> Option<&Self::V>
    where
        Self::K: std::borrow::Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.rank(k).ok().map(|idx| &self.0[idx].1)
    }

    fn get_mut<Q>(&mut self, k: &Q) -> Option<&mut Self::V>
    where
        Self::K: std::borrow::Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.rank(k).ok().map(|idx| &mut self.0[idx].1)
    }

    fn iter(&self) -> impl Iterator<Item = (&Self::K, &Self::V)> {
        self.0.iter().map(|kv| (&kv.0, &kv.1))
    }

    fn drain_all(&mut self) -> impl Iterator<Item = (Self::K, Self::V)> {
        self.0.drain(..).map(|kv| kv.into())
    }

    fn values_mut(&mut self) -> impl Iterator<Item = &mut Self::V> {
        self.0.iter_mut().map(|kv| &mut kv.1)
    }

    fn insert(&mut self, k: Self::K, v: Self::V) -> Option<Self::V>
    where
        Self::K: Ord,
    {
        match self.rank(&k) {
            Ok(idx) => Some(core::mem::replace(&mut self.0[idx].1, v)),
            Err(idx) => {
                self.0.insert(idx, KVEntry(k, v));
                None
            }
        }
    }

    fn remove<Q>(&mut self, k: &Q) -> Option<Self::V>
    where
        Self::K: std::borrow::Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.rank(&k).ok().map(|idx| self.0.remove(idx).1)
    }

    fn update_index<Q>(&mut self, old_k: &Q, new_k: Self::K) -> Option<Self::V>
    where
        Self::K: std::borrow::Borrow<Q>,
        Q: Ord,
    {
        if new_k.borrow() == old_k {
            return None;
        }

        let Ok(old_idx) = self.rank(old_k) else {
            return None;
        };

        // It's usually better than Enum compare in Rust impl
        if new_k.borrow() < old_k {
            match self.0[..old_idx]
                .binary_search_by_key(&new_k.borrow(), |kv| kv.0.borrow())
            {
                Ok(new_idx) => {
                    let old_v = self.0.remove(old_idx).1;

                    Some(core::mem::replace(&mut self.0[new_idx].1, old_v))
                }
                Err(new_idx) => {
                    // rotate right 1 step to swap
                    self.0[new_idx..=old_idx].rotate_right(1);
                    self.0[new_idx].0 = new_k;

                    None
                }
            }
        } else {
            let base = old_idx + 1;

            match self.0[base..]
                .binary_search_by_key(&new_k.borrow(), |kv| kv.0.borrow())
            {
                Ok(sub_idx) => {
                    let new_idx = base + sub_idx - 1;

                    let old_v = self.0.remove(old_idx).1;

                    Some(core::mem::replace(&mut self.0[new_idx].1, old_v))
                }
                Err(sub_idx) => {
                    let new_idx = base + sub_idx - 1;

                    // rotate left 1 step to swap
                    self.0[old_idx..=new_idx].rotate_left(1);
                    self.0[new_idx].0 = new_k;

                    None
                }
            }
        }
    }

    fn pop_first(&mut self) -> Option<(Self::K, Self::V)> {
        if !self.is_empty() {
            Some(self.0.remove(0).into())
        } else {
            None
        }
    }

    fn pop_last(&mut self) -> Option<(Self::K, Self::V)> {
        self.0.pop().map(|kv| kv.into())
    }

    fn push_front(&mut self, k: Self::K, v: Self::V) {
        self.0.insert(0, KVEntry(k, v));
    }

    fn push_back(&mut self, k: Self::K, v: Self::V) {
        self.0.push(KVEntry(k, v));
    }

    fn split_off(&mut self, at: usize) -> Self {
        Self(self.0.split_off(at))
    }
}

// Unbounded Public Methods
impl<K, V, const M: usize> BPT<K, V, M> {
    pub fn new() -> Self {
        assert!(M > 2, "M should be greater than 2");

        Self { tree: None }
    }

    pub fn len(&self) -> usize {
        self.tree
            .as_ref()
            .map(|tree| tree.cnt.into())
            .unwrap_or_default()
    }

    /// `O(logn)`
    pub fn height(&self) -> usize {
        self.tree
            .as_ref()
            .map(|tree| {
                let mut p = tree.root.ptr();
                let mut h = 1;

                while p.is_internal() {
                    p = p.get_children().first_value().unwrap().ptr();
                    h += 1;
                }

                h
            })
            .unwrap_or_default()
    }

    pub const fn is_empty(&self) -> bool {
        self.tree.is_none()
    }

    /// `>= 2`
    pub const fn internal_cap_lower_bound() -> usize {
        M.div_floor(2)
    }
}

// Bounded Public Methods
impl<K: Ord, V, const M: usize> BPT<K, V, M> {
    pub fn bulk_build<T: IntoIterator<Item = (K, V)>>(_sorted_iter: T) -> Self {
        todo!()
    }

    pub fn get<Q>(&self, k: &Q) -> Option<&V>
    where
        K: std::borrow::Borrow<Q>,
        Q: Ord + ?Sized,
    {
        let Some(root) = self.tree.as_ref().map(|tree| tree.root.ptr()) else {
            return None;
        };

        let Some(x) = Self::down_to_leaf(root, k) else {
            return None;
        };

        unsafe { &*core::ptr::from_ref(x.get_entries()) }.get(k)
    }

    pub fn get_mut<Q>(&mut self, k: &Q) -> Option<&mut V>
    where
        K: std::borrow::Borrow<Q>,
        Q: Ord + ?Sized,
    {
        let Some(root) = self.tree.as_ref().map(|tree| tree.root.ptr()) else {
            return None;
        };

        let Some(mut x) = Self::down_to_leaf(root, k) else {
            return None;
        };

        unsafe { &mut *core::ptr::from_mut(x.get_entries_mut()) }.get_mut(k)
    }

    pub fn insert(&mut self, k: K, v: V) -> Option<V>
    where
        K: Clone,
    {
        let Some(tree) = self.tree.as_ref().map(|tree| tree) else {
            let root = OwnedPtr::new(Node::new_leaf().with_kv(k, v));
            let min_node = root.ptr();
            let cnt = unsafe { NonZeroUsize::new_unchecked(1) };

            let tree = Tree {
                root,
                min_node,
                cnt,
            };

            self.tree = Some(tree);

            return None;
        };

        let x =
            Self::down_to_leaf(tree.root.ptr(), &k).unwrap_or(tree.min_node);

        self.insert_into_leaf(x, k, v)
    }

    pub fn remove<Q>(&mut self, k: &Q) -> Option<V>
    where
        K: std::borrow::Borrow<Q> + Clone,
        Q: Ord + ?Sized,
    {
        let Some(root) = self.tree.as_ref().map(|tree| tree.root.ptr()) else {
            return None;
        };

        let Some(x) = Self::down_to_leaf(root, k) else {
            return None;
        };

        self.remove_on_leaf(x, k)
    }
}

// Bounded Private Methods
impl<K, V, const M: usize> BPT<K, V, M> {
    /// search to leaf restricted version (with short-circuit evaluation)
    fn down_to_leaf<Q>(
        mut x: Ptr<Node<K, V, M>>,
        k: &Q,
    ) -> Option<Ptr<Node<K, V, M>>>
    where
        K: std::borrow::Borrow<Q> + Ord,
        Q: Ord + ?Sized,
    {
        while x.is_internal() {
            if let Some(x_res) = x.get_children().lower_bound_search(k) {
                x = match x_res {
                    Ok(x) => {
                        let mut x = x.ptr();
                        // quick down
                        while x.is_internal() {
                            x = x.get_children().first_value().unwrap().ptr();
                        }

                        return Some(x);
                    }
                    Err(x) => x.ptr(),
                }
            } else {
                return None;
            }
        }

        Some(x)
    }

    fn insert_into_leaf(
        &mut self,
        mut x: Ptr<Node<K, V, M>>,
        k: K,
        v: V,
    ) -> Option<V>
    where
        K: Ord + Clone,
    {
        debug_assert!(x.is_leaf());

        /* for key exists */
        let idx = match x.get_entries().rank(&k) {
            Ok(idx) => {
                // StackVec optmization extension
                return Some(core::mem::replace(
                    &mut x.get_entries_mut().0[idx].1,
                    v,
                ));
            }
            Err(idx) => idx,
        };

        /* for leaf node is full */
        if x.is_full() {
            let x_sib_rh = self.promote(x);

            if idx >= M.div_ceil(2) {
                x = x_sib_rh;
            }
        }

        let mut need_update_min_key = None;

        if idx == 0 || idx == M.div_ceil(2) {
            need_update_min_key = Some(x.min_key());
        }

        x.get_entries_mut().insert(k, v);

        if let Some(old_k) = need_update_min_key {
            Self::update_index(old_k, x);
        }

        self.tree
            .as_mut()
            .map(|tree| tree.cnt = tree.cnt.checked_add(1).unwrap());

        None
    }

    fn remove_on_leaf<Q>(
        &mut self,
        mut x: Ptr<Node<K, V, M>>,
        k: &Q,
    ) -> Option<V>
    where
        K: std::borrow::Borrow<Q> + Ord + Clone,
        Q: Ord + ?Sized,
    {
        debug_assert!(x.is_leaf());

        let old_k = x.min_key();

        let popped = x.get_entries_mut().remove(k);

        if popped.is_some() {
            if self.len() == 1 {
                self.tree.take();
                // early return to avoid access dropped x cause UB
                return popped;
            } else {
                self.tree.as_mut().map(|tree| {
                    tree.cnt = unsafe {
                        NonZeroUsize::new_unchecked(tree.cnt.get() - 1)
                    }
                });
            }
        }

        if x.get_entries().len() == 0 {
            if !self.is_empty() {
                self.unpromote(x, old_k);
            }
        } else {
            Self::update_index(old_k, x);
        }

        popped
    }

    fn update_index(old_k: K, x: Ptr<Node<K, V, M>>)
    where
        K: Clone + Ord, // Ord used in `insert`
    {
        let k = x.min_key();
        let p = x.paren();

        if old_k != k
            && let Some(mut p) = p
        {
            /* update x key */

            // get old_p_k to avoid update_index override it.
            let old_p_k = p.min_key();

            p.get_children_mut().update_index(&old_k, k);

            Self::update_index(old_p_k, p);
        }
    }

    /// return right sibling (always split x into x and x2)
    fn promote(&mut self, mut x: Ptr<Node<K, V, M>>) -> Ptr<Node<K, V, M>>
    where
        K: Ord + Clone,
    {
        debug_assert!(x.is_full());

        let at = M.div_ceil(2);

        /* split node evenly */

        let mut x2 = OwnedPtr::new(match x.deref_mut() {
            Node::Leaf { entries, next, .. } => {
                Node::new_leaf_with_entries(entries.split_off(at))
                    .with_next(*next)
            }
            Node::Internal { children, .. } => {
                Node::new_internal_with_children(children.split_off(at))
            }
        });

        if let Node::Leaf { next, .. } = x.deref_mut() {
            // x -> x2
            *next = Some(x2.ptr());
        } else {
            for mut child in x2.get_children().values().map(|owned| owned.ptr())
            {
                child.get_paren_mut().replace(x2.ptr());
            }
        }

        let p = x.paren();
        let x2_k = x2.min_key();
        let x2_ptr = x2.ptr();

        /* insert into paren node (if it's not root) */
        if let Some(mut p) = p {
            if p.is_full() {
                let p_sib_rh = self.promote(p);
                // correct x2's parent node
                if x2_k >= *p_sib_rh.get_min_key() {
                    p = p_sib_rh;
                }
            }

            *x2.get_paren_mut() = Some(p);

            // insert new or update
            p.get_children_mut().insert(x2_k, x2);
        }
        /* push new level */
        else {
            let Tree {
                mut root,
                min_node,
                cnt,
            } = self.tree.take().unwrap();

            let x_k = x.min_key();

            root = OwnedPtr::new(Node::new_internal_with_children(
                StackVecMap::new().with_kv(x_k, root).with_kv(x2_k, x2),
            ));

            for mut child in
                root.get_children().values().map(|owned| owned.ptr())
            {
                child.get_paren_mut().replace(root.ptr());
            }

            self.tree = Some(Tree {
                root,
                min_node,
                cnt,
            })
        }

        x2_ptr
    }

    fn unpromote(&mut self, mut x: Ptr<Node<K, V, M>>, mut x_k: K)
    where
        K: Ord + Clone,
    {
        // x sholdn't be root
        debug_assert!(x.paren().is_some());
        debug_assert!(
            x.is_leaf()
                || x.get_children().len()
                    == Self::internal_cap_lower_bound() - 1
        );

        fn pop_levels<K, V, const M: usize>(bpt: &mut BPT<K, V, M>) {
            let tree = bpt.tree.as_mut().unwrap();

            debug_assert!(tree.root.len() == 1);
            debug_assert!(tree.root.is_internal());

            tree.root = tree.root.get_children_mut().pop_first().unwrap().1;
            tree.root.get_paren_mut().take();

            if BPT::<K, V, M>::internal_cap_lower_bound() == 1 {
                while let Node::Internal { children, .. } = tree.root.deref()
                    && children.len() == 1
                {
                    tree.root =
                        tree.root.get_children_mut().pop_first().unwrap().1;
                    tree.root.get_paren_mut().take();
                }
            }
        }

        let mut p = x.paren().unwrap();
        let idx = p.get_children().rank(&x_k).unwrap();

        if Self::try_redistribute_nodes(p, x, idx, x_k.clone()) {
            return;
        }

        /* redistribute failed, start to merge */

        // merge with left_sib (no need to update index)
        if idx > 0 {
            let mut sib_lf = p
                .get_children()
                .nth(idx - 1)
                .map(|(_k, v)| v.ptr())
                .unwrap();

            let x = p.get_children_mut().remove(&x_k).unwrap();

            if x.is_leaf() {
                debug_assert!(x.is_empty());
                *sib_lf.get_next_mut() = x.next();
            } else {
                let Node::Internal {
                    children: mut x_children,
                    ..
                } = OwnedPtr::into_inner(x)
                else {
                    unreachable!()
                };

                for (child_k, mut child) in x_children.drain_all() {
                    *child.get_paren_mut() = Some(sib_lf);
                    sib_lf.get_children_mut().push_back(child_k, child);
                }
            }
        }
        // for right_sib (merge into left)
        else {
            debug_assert!(idx == 0);

            /* handle special case for M = 3 */
            if Self::internal_cap_lower_bound() == 1 && p.len() == 1 {
                debug_assert!(x.is_leaf());

                while let Some(next_p) = p.paren() {
                    p = next_p;

                    if p.len() > 1 {
                        break;
                    }
                }

                // p.len > 1 or p is root

                if p.len() > 1 {
                    p.get_children_mut().remove(&x_k);

                    if *p.get_min_key() > x_k {
                        Self::update_index(x_k, p);
                    }

                    // To satisfied root size >= 2
                    if p.len() == 1 && p.paren().is_none() {
                        // pop level
                        pop_levels(self);
                    }
                } else {
                    debug_assert!(p.paren().is_none());

                    self.tree.take();
                }

                return;
            }

            let mut sib_rh = p
                .get_children()
                .nth(idx + 1)
                .map(|(_k, v)| v.ptr())
                .unwrap();

            // Since we don't know previous leaf of x for all time (when x is leftmost of its parent),
            // we merge right leaf into its left, that's merge sib_rh into x
            if x.is_leaf() {
                // if sib_rh size > 1, it would be redistributed as below.
                debug_assert_eq!(sib_rh.get_entries().len(), 1);

                *x.get_next_mut() = sib_rh.next();

                // In p node:
                //
                // [ x_k (obsolete) => x, sib_rh_k => sib_rh, .. ]
                //
                // change to:
                //
                // [ sib_rh_k => x, .. ]

                let (k, v) = sib_rh.get_entries_mut().pop_first().unwrap();

                x.get_entries_mut().push_back(k, v);

                // p.get_children_mut().remove(&k);
                // VecMap extension
                p.get_children_mut().pop_second();

                Self::update_index(x_k, x);
            } else {
                // let sib_rh =
                //     p.get_children_mut().remove(sib_rh.get_min_key()).unwrap();
                // VecMap extension
                let sib_rh = p.get_children_mut().pop_second().unwrap().1;

                let Node::Internal {
                    children: mut sib_rh_children,
                    ..
                } = OwnedPtr::into_inner(sib_rh)
                else {
                    unreachable!()
                };

                for (child_k, mut child) in sib_rh_children.drain_all() {
                    *child.get_paren_mut() = Some(x);
                    x.get_children_mut().push_back(child_k, child);
                }
            }
        }

        x = p;

        if x.paren().is_none() {
            // pop level
            if x.get_children().len() == 1 {
                pop_levels(self);
            }
        } else {
            x_k = x.min_key();

            if x.get_children().len() < Self::internal_cap_lower_bound() {
                self.unpromote(x, x_k);
            }
        }
    }

    ///
    /// ```ignore
    ///       p
    ///  / | ... \ \
    /// c0 c1    cn-1 cn
    /// ```
    ///
    /// return if redistribute is succeeded
    fn try_redistribute_nodes(
        mut p: Ptr<Node<K, V, M>>,
        mut x: Ptr<Node<K, V, M>>,
        idx: usize,
        old_k: K,
    ) -> bool
    where
        K: Ord + Clone,
    {
        // try to redistribute with left
        if idx > 0 {
            let sib_lf = p.get_children_mut().nth_value_mut(idx - 1).unwrap();

            if sib_lf.is_leaf() && sib_lf.get_entries().len() > 1 {
                let (k, v) = sib_lf.get_entries_mut().pop_last().unwrap();

                x.get_entries_mut().push_front(k, v);

                Self::update_index(old_k, x);

                return true;
            }

            if sib_lf.is_internal()
                && sib_lf.get_children().len()
                    > Self::internal_cap_lower_bound()
            {
                let (k, mut v) = sib_lf.get_children_mut().pop_last().unwrap();

                *v.get_paren_mut() = Some(x);

                x.get_children_mut().push_front(k, v);

                Self::update_index(old_k, x);

                return true;
            }
        }
        // try to redistribute with right then
        if idx < p.get_children().len() - 1 {
            let sib_rh = p.get_children_mut().nth_value_mut(idx + 1).unwrap();

            if sib_rh.is_leaf() && sib_rh.get_entries().len() > 1 {
                let (k, v) = sib_rh.get_entries_mut().pop_first().unwrap();
                x.get_entries_mut().push_back(k.clone(), v);

                //
                // In p node:
                //
                // [ old_k (obsolete) => x, k => sib_rh, .. ]
                //
                // change to:
                //
                // [ k => x, sib_rh_2nd_key => sib_rh ]
                //
                // if old_k is p's first key, In p's parent node:
                //
                // [.., old_k => p, ..]
                //
                // p update_index upwards

                let sib_rh_2nd_key = sib_rh.min_key();

                let p_children = p.get_children_mut();

                p_children.update_index(&k, sib_rh_2nd_key);
                p_children.update_index(&old_k, k);

                if idx == 0 {
                    Self::update_index(old_k, p);
                }

                return true;
            }

            if sib_rh.is_internal()
                && sib_rh.get_children().len()
                    > Self::internal_cap_lower_bound()
            {
                let sib_rh_old_key = sib_rh.min_key();

                let (k, mut v) = sib_rh.get_children_mut().pop_first().unwrap();

                *v.get_paren_mut() = Some(x);
                x.get_children_mut().push_back(k, v);

                Self::update_index(sib_rh_old_key, sib_rh.ptr());

                return true;
            }
        }

        false
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
    pub const fn is_internal(&self) -> bool {
        matches!(self, Self::Internal { .. })
    }

    pub const fn is_leaf(&self) -> bool {
        matches!(self, Self::Leaf { .. })
    }

    /// x.len == M
    pub const fn is_full(&self) -> bool {
        self.len() == M
    }

    pub const fn len(&self) -> usize {
        match self {
            Self::Internal { children, .. } => children.len(),
            Self::Leaf { entries, .. } => entries.len(),
        }
    }

    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn new_leaf() -> Self {
        Self::Leaf {
            entries: StackVecMap::new(),
            next: None,
            paren: None,
        }
    }

    pub fn new_leaf_with_entries(entries: StackVecMap<K, V, M>) -> Self {
        Self::Leaf {
            entries,
            next: None,
            paren: None,
        }
    }

    pub fn new_internal_with_children(
        children: StackVecMap<K, OwnedPtr<Self>, M>,
    ) -> Self {
        Self::Internal {
            children,
            paren: None,
        }
    }

    /// Leaf build method
    pub fn with_next(mut self, new_next: Option<Ptr<Self>>) -> Self {
        *self.get_next_mut() = new_next;
        self
    }

    pub fn with_kv(mut self, k: K, v: V) -> Self
    where
        K: Ord,
    {
        self.get_entries_mut().insert(k, v);
        self
    }

    pub fn get_min_key(&self) -> &K {
        match self {
            Self::Internal { children, .. } => children.get_min_key().unwrap(),
            Self::Leaf { entries, .. } => entries.get_min_key().unwrap(),
        }
    }

    pub fn min_key(&self) -> K
    where
        K: Clone,
    {
        self.get_min_key().clone()
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

    pub fn get_children(&self) -> &StackVecMap<K, OwnedPtr<Self>, M> {
        match self {
            Self::Internal { children, .. } => children,
            Self::Leaf { .. } => unreachable!(),
        }
    }

    pub fn get_children_mut(
        &mut self,
    ) -> &mut StackVecMap<K, OwnedPtr<Self>, M> {
        match self {
            Self::Internal { children, .. } => children,
            Self::Leaf { .. } => unreachable!(),
        }
    }

    pub fn get_entries(&self) -> &StackVecMap<K, V, M> {
        match self {
            Self::Internal { .. } => unreachable!(),
            Self::Leaf { entries, .. } => entries,
        }
    }

    pub fn get_entries_mut(&mut self) -> &mut StackVecMap<K, V, M> {
        match self {
            Self::Internal { .. } => unreachable!(),
            Self::Leaf { entries, .. } => entries,
        }
    }
}


impl<K: Debug, V, const M: usize> Debug for Node<K, V, M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_leaf() {
            let mut peek = self.get_entries().keys().peekable();

            while let Some(k) = peek.next() {
                write!(f, "{k:?}")?;

                if peek.peek().is_some() {
                    write!(f, ", ")?;
                }
            }
        } else if self.is_internal() {
            let mut peek = self.get_children().keys().peekable();

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
//// Test && Stats Methods


#[cfg(test)]
impl<K: Ord + Debug + Clone, V, const M: usize> BPT<K, V, M> {
    pub fn debug_print(&self)
    where
        V: Debug,
    {
        use common::vecdeq;

        /* print header */

        println!("{self:?}");

        /* print body */

        let Some(root) = self.tree.as_ref().map(|tree| tree.root.ptr()) else {
            return;
        };

        let mut this_q = vecdeq![vec![root]];
        let mut lv = 1;

        while !this_q.is_empty() {
            println!();
            println!("############ Level: {lv} #############");
            println!();

            let mut nxt_q = vecdeq![];

            while let Some(children) = this_q.pop_front() {
                for (i, x) in children.iter().enumerate() {
                    let p = x.paren();

                    if x.is_internal() {
                        nxt_q.push_back(
                            x.get_children()
                                .values()
                                .map(|own| own.ptr())
                                .collect(),
                        );
                        println!("({i:02}): {x:?} (p: [{p:?}])");
                    } else {
                        let succ = x.next();
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

    pub fn validate(&self)
    where
        K: Debug + std::hash::Hash,
        V: Debug,
    {
        let Some(root) = self.tree.as_ref().map(|tree| tree.root.ptr()) else {
            return;
        };

        if root.is_internal() {
            assert!(root.len() >= 2);
        }

        use std::collections::VecDeque;

        use common::vecdeq;

        // first None indecate parent is None
        let mut cur_q = vecdeq![(None, vecdeq![root])];

        while !cur_q.is_empty() {
            let mut nxt_q = vecdeq![];
            let mut leaf_num = 0;
            let mut internal_num = 0;

            while let Some((_p, group)) = cur_q.pop_front() {
                for child in group.iter() {
                    if child.is_internal() {
                        assert!(child.len() < M + 1);
                        child.get_children().validate();
                    } else {
                        assert!(child.len() < M + 1);
                        child.get_entries().validate();
                    }

                    // Exclude leaf
                    if child.is_leaf() {
                        leaf_num += 1;
                    } else {
                        // Exclude the root (which is always one when it's internal node)
                        if child.paren().is_some() {
                            assert!(
                                child.len() >= Self::internal_cap_lower_bound(),
                                "Internal node has {} entries, which is less than {}",
                                child.len(),
                                Self::internal_cap_lower_bound()
                            );
                        } else {
                            // if child.len() < 2 {
                            //     self.debug_print()
                            // }

                            assert!(
                                child.len() >= 2,
                                "root should have at least 2 children as internal node"
                            );
                        }

                        internal_num += 1;

                        let nxt_children = VecDeque::from_iter(
                            child.get_children().values().map(|own| own.ptr()),
                        );

                        nxt_q.push_back((Some(*child), nxt_children));
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

    use super::{super::tests::*, *};
    use crate::bst::test_dict;

    // #[test]
    // fn test_tree_map() {
    //     fn test_tree_map<Tree: TreeMap<K = usize, V = usize>>() {
    //         let mut map = Tree::new();

    //         map.insert(1, 1);
    //     }
    // }

    #[test]
    fn test_bpt_bulk_load() {
        // let _ = BPT::<_, _>::from_iter((0..5).map(|v| (v, v)));
    }

    #[test]
    fn test_bt_bpt3_case_1() {
        let mut dict = BPT::<u16, u16, 3>::new();

        dict_insert!(dict, 52);
        dict_insert!(dict, 47);
        dict_insert!(dict, 3);
        dict_insert!(dict, 35);
        dict_insert!(dict, 24);
        dict_insert!(dict, 44);
        dict.insert(66, 66);
        dict_insert!(dict, 66);
        dict_insert!(dict, 38);
        dict_insert!(dict, 30);
        dict_insert!(dict, 28);
        // dict.debug_print();
        // assert!(false);
        dict_remove!(dict, 24);
        dict.debug_print();
        dict_remove!(dict, 44);
        dict_remove!(dict, 66);
        dict_remove!(dict, 38);
        dict_remove!(dict, 52);
        dict_remove!(dict, 47);
        dict_remove!(dict, 3);
        // dict.debug_print();
        // assert!(false);
        // assert_eq!(
        //     dict.range(..).map(|(k, _v)| k).cloned().collect::<Vec<_>>(),
        //     [28, 30, 35]
        // );

        dict_remove!(dict, 35);
        dict_remove!(dict, 30);
        dict_remove!(dict, 28);

        // assert_eq!(
        //     dict.range(..).map(|(k, _v)| k).cloned().collect::<Vec<_>>(),
        //     [0u16; 0]
        // );

        dict.debug_print();
    }

    #[test]
    fn test_bt_bpt3_case_1_1() {
        let mut dict = BPT::<u16, u16, 3>::new();

        dict_insert!(dict, 18);
        dict_insert!(dict, 55);
        dict_insert!(dict, 63);
        dict_insert!(dict, 45);
        dict_insert!(dict, 23);
        dict_insert!(dict, 13);

        dict_remove!(dict, 45);
        dict_remove!(dict, 18);
        dict_remove!(dict, 13);
        dict_remove!(dict, 55);
        dict_remove!(dict, 63);

        dict.debug_print();
    }

    #[test]
    fn test_bt_bpt3_case_2() {
        let mut dict = BPT::<u16, u16, 5>::new();

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
    fn test_bt_bpt3_random() {
        test_dict!(BPT::<u16, u16, 3>::new());
        println!("Ok..M=3");

        test_dict!(BPT::<u16, u16, 4>::new());
        println!("Ok..M=4");

        test_dict!(BPT::<u16, u16, 5>::new());
        println!("Ok..M=5");

        test_dict!(BPT::<u16, u16, 11>::new());
        println!("Ok..M=11");

        test_dict!(BPT::<u16, u16, 20>::new());
        println!("Ok..M=20");

        test_dict!(BPT::<u16, u16, 101>::new());
        println!("Ok..M=101");
    }
}
