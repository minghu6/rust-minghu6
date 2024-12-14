mod stats;

#[cfg(test)]
pub(crate) mod tests;


use std::{
    cmp::*,
    iter::Sum,
    marker::PhantomData,
    ops::{Add, AddAssign, Index, IndexMut, Range, RangeBounds},
    slice::SliceIndex,
};

pub use stats::*;


////////////////////////////////////////////////////////////////////////////////
//// Macros

macro_rules! zero {
    () => {
        [].into_iter().sum()
    };
}

////////////////////////////////////////////////////////////////////////////////
//// Traits

pub trait TreeLayout: private::Sealed {
    fn left(i: usize, t: TreeCursor) -> usize;
    fn right(i: usize, t: TreeCursor) -> usize;
    fn root() -> usize {
        1
    }
    fn size(cap: usize) -> usize;
}


mod private {
    pub trait Sealed {}

    impl Sealed for super::BFS {}
    impl Sealed for super::DFS {}
}


////////////////////////////////////////////////////////////////////////////////
//// Structures

pub struct SegmentTree<T, L = DFS> {
    data: Vec<T>,
    root: TreeCursor,
    _marker: PhantomData<L>,
}

/// (tl, tr)
#[derive(Debug, Clone, Copy)]
pub struct TreeCursor {
    tl: usize,
    tr: usize,
}

pub struct BFS;
/// Euler tour traversal (memory efficient)
pub struct DFS;

#[repr(transparent)]
pub struct UpdaterAdd<T, L>(Vec<T>, PhantomData<L>);

////////////////////////////////////////////////////////////////////////////////
//// Implementations

impl<T, L, I: SliceIndex<[T]>> Index<I> for UpdaterAdd<T, L> {
    type Output = I::Output;

    fn index(&self, index: I) -> &Self::Output {
        &self.0[index]
    }
}

impl<T, L, I: SliceIndex<[T]>> IndexMut<I> for UpdaterAdd<T, L> {
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        &mut self.0[index]
    }
}

impl TreeLayout for BFS {
    fn left(i: usize, _t: TreeCursor) -> usize {
        i * 2
    }

    fn right(i: usize, _t: TreeCursor) -> usize {
        i * 2 + 1
    }

    fn size(cap: usize) -> usize {
        4 * cap
    }
}

impl TreeLayout for DFS {
    fn left(i: usize, _t: TreeCursor) -> usize {
        i + 1
    }

    fn right(i: usize, t: TreeCursor) -> usize {
        // i + 2(n(left))
        i + 2 * ((t.tl + t.tr) / 2 - t.tl + 1)
    }

    fn size(cap: usize) -> usize {
        2 * cap - 1 + 1
    }
}

impl<T, L> SegmentTree<T, L> {
    /// Tree len size
    pub fn len(&self) -> usize {
        self.root.tr + 1
    }
}

impl<T, L> SegmentTree<T, L>
where
    L: TreeLayout,
    T: Sum + Clone + Add<Output = T>,
    for<'a> &'a T: Add<&'a T, Output = T>,
{
    pub fn new<U: Clone + Into<T>>(raw: &[U]) -> Self {
        assert!(!raw.is_empty());

        let mut data = vec![zero!(); L::size(raw.len())];

        let root = TreeCursor::root(raw.len());

        Self::build(&mut data, raw, L::root(), root);

        Self {
            data,
            root,
            _marker: PhantomData::<L>,
        }
    }

    pub fn query<R: RangeBounds<usize>>(&self, range: R) -> T {
        let Range { start, end } = std::slice::range(range, ..self.root.tr + 1);

        self.query_((start, end - 1), L::root(), self.root)
    }

    /// Update
    pub fn assoc(&mut self, i: usize, new_val: T) {
        assert!(i <= self.root.tr);

        self.assoc_(L::root(), self.root, i, new_val)
    }

    /// Create an batch add updater
    pub fn create_updater(&self) -> UpdaterAdd<T, L> {
        UpdaterAdd(vec![zero!(); self.data.len()], PhantomData)
    }

    ////////////////////////////////////////////////////////////////////////////
    /// Inner Method

    fn build<U: Clone + Into<T>>(
        data: &mut [T],
        arr: &[U],
        i: usize,
        t: TreeCursor,
    ) {
        if t.is_end() {
            data[i] = arr[t.tl].clone().into();
        } else {
            Self::build(data, arr, L::left(i, t), t.left());
            Self::build(data, arr, L::right(i, t), t.right());

            data[i] = &data[L::left(i, t)] + &data[L::right(i, t)];
        }
    }

    fn query_(&self, (l, r): (usize, usize), i: usize, t: TreeCursor) -> T {
        if l > r {
            return zero!();
        }

        if t.is_matched((l, r)) {
            self[i].clone()
        } else {
            let lf =
                self.query_((l, min(t.left().tr, r)), L::left(i, t), t.left());

            let rh = self.query_(
                (max(t.right().tl, l), r),
                L::right(i, t),
                t.right(),
            );

            lf + rh
        }
    }

    fn assoc_(&mut self, i: usize, t: TreeCursor, ti: usize, new_val: T) {
        if t.is_end() {
            self.data[i] = new_val;
        } else {
            let lfi = L::left(i, t);
            let rhi = L::right(i, t);

            if ti <= t.left().tr {
                self.assoc_(lfi, t.left(), ti, new_val);
            } else {
                self.assoc_(rhi, t.right(), ti, new_val);
            }

            self.data[i] = &self.data[lfi] + &self.data[rhi];
        }
    }
}

impl<T, L, I: SliceIndex<[T]>> Index<I> for SegmentTree<T, L> {
    type Output = I::Output;

    fn index(&self, index: I) -> &Self::Output {
        &self.data[index]
    }
}

impl<T, L, I: SliceIndex<[T]>> IndexMut<I> for SegmentTree<T, L> {
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        &mut self.data[index]
    }
}

impl TreeCursor {
    fn root(raw_len: usize) -> Self {
        debug_assert!(raw_len > 0);

        Self {
            tl: 0,
            tr: raw_len - 1,
        }
    }

    fn is_end(&self) -> bool {
        self.tl == self.tr
    }

    fn is_matched(&self, (l, r): (usize, usize)) -> bool {
        self.tl == l && self.tr == r
    }

    fn left(&self) -> Self {
        Self {
            tl: self.tl,
            tr: (self.tl + self.tr) / 2,
        }
    }

    fn right(&self) -> Self {
        Self {
            tl: (self.tl + self.tr) / 2 + 1,
            tr: self.tr,
        }
    }
}

impl<T, L: TreeLayout> UpdaterAdd<T, L>
where
    T: Clone + Sum + Add<Output = T> + Ord,
    for<'a> T: AddAssign<&'a T>,
    for<'a> &'a T: Add<&'a T, Output = T>,
{
    pub fn assoc<R: RangeBounds<usize>>(
        &mut self,
        tree: &mut SegmentTree<T, L>,
        range: R,
        addend: T,
    ) {
        let Range { start, end } = std::slice::range(range, ..tree.len());

        self.assoc_(
            tree,
            (start, end - 1),
            addend,
            L::root(),
            tree.root,
        )
    }

    pub fn query<R: RangeBounds<usize>>(
        &mut self,
        tree: &mut SegmentTree<T, L>,
        range: R,
    ) -> T {
        let Range { start, end } = std::slice::range(range, ..tree.len());

        self.query_(
            tree,
            (start, end - 1),
            L::root(),
            tree.root,
        )
    }

    /// Lazy propagation
    fn propagate(
        &mut self,
        tree: &mut SegmentTree<T, L>,
        i: usize,
        t: TreeCursor,
    ) {
        if self[i] != zero!() {
            let lfi = L::left(i, t);
            let rhi = L::right(i, t);

            tree[lfi] += &self[i];
            self[lfi] = &self[lfi] + &self[i];

            tree[rhi] += &self[i];
            self[rhi] = &self[rhi] + &self[i];

            self[i] = zero!();
        }
    }

    fn assoc_(
        &mut self,
        tree: &mut SegmentTree<T, L>,
        (l, r): (usize, usize),
        addend: T,
        i: usize,
        t: TreeCursor,
    ) {
        if l > r {
            return;
        }

        if t.is_matched((l, r)) {
            tree[i] += &addend;
            self[i] += &addend;
        } else {
            self.propagate(tree, i, t);

            self.assoc_(
                tree,
                (l, min(r, t.left().tr)),
                addend.clone(),
                L::left(i, t),
                t.left(),
            );
            self.assoc_(
                tree,
                (max(l, t.right().tl), r),
                addend,
                L::right(i, t),
                t.right(),
            );

            tree[i] = &tree[L::left(i, t)] + &tree[L::right(i, t)];
        }
    }

    fn query_(
        &mut self,
        tree: &mut SegmentTree<T, L>,
        (l, r): (usize, usize),
        i: usize,
        t: TreeCursor,
    ) -> T {
        if l > r {
            return zero!();
        }

        if t.is_matched((l, r)) {
            tree[i].clone()
        } else {
            self.propagate(tree, i, t);

            let lf = self.query_(
                tree,
                (l, min(r, t.left().tr)),
                L::left(i, t),
                t.left(),
            );
            let rh = self.query_(
                tree,
                (max(l, t.right().tl), r),
                L::right(i, t),
                t.right(),
            );

            lf + rh
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
//// Functions
