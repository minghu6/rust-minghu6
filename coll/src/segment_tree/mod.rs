mod stats;

#[cfg(test)]
mod tests;


use std::{
    cmp::*,
    marker::PhantomData,
    ops::{Add, AddAssign, Index, IndexMut, RangeBounds},
    slice::SliceIndex,
};

pub use stats::*;


////////////////////////////////////////////////////////////////////////////////
//// Macro

macro_rules! tm {
    ($tl:expr, $tr:expr) => {
        ($tl + $tr) / 2
    };
}


macro_rules! left {
    ($c:expr) => {
        $c.godown_left::<L>()
    };
}


macro_rules! right {
    ($c:expr) => {
        $c.godown_right::<L>()
    };
}


/// -> (l, r)
macro_rules! parse_range {
    ($range:expr, $root:expr) => {{
        use std::ops::Bound::*;

        let range = $range;

        let l;
        let r;

        match range.start_bound() {
            Included(v) => l = *v,
            Excluded(v) => l = *v + 1,
            Unbounded => l = 1,
        }

        match range.end_bound() {
            Included(v) => r = *v,
            Excluded(v) => {
                assert!(*v > 0, "range upper is invalid (=0)");
                r = *v - 1
            }
            Unbounded => r = $root.tr,
        }

        (l, r)
    }};
}


macro_rules! impl_index {
    (slice_index|
        impl <$($generic:ident $(:$trait:ident)?),+>
        for $struct:ident
        with ($($field:ident).+)) =>
    {
        impl<$($generic $(:$trait)?),+, Idx> Index<Idx> for $struct<$($generic),+>
        where
            Idx: SliceIndex<[T], Output = T>,
        {
            type Output = T;

            fn index(&self, idx: Idx) -> &Self::Output {
                &(self.$($field).+)[idx]
            }
        }
    };
    (index|
        impl <$($generic:ident $(:$trait:path)?),+>
        for $struct:ident : $cursor:ty => $output:ident
        with tuple
        ($($c_field:ident).+)
    ) =>
    {
        impl<$($generic $(:$trait)?),+> Index<$cursor> for $struct<$($generic),+>
        {
            type Output = $output;

            fn index(&self, idx: $cursor) -> &Self::Output {
                &(self.0)[idx.$($c_field).+]
            }
        }

        impl<$($generic $(:$trait)?),+> IndexMut<$cursor> for $struct<$($generic),+>
        {
            fn index_mut(&mut self, idx: $cursor) -> &mut T {
                &mut (self.0)[idx.$($c_field).+]
            }
        }
    };
    (index|
        impl <$($generic:ident $(:$trait:path)?),+>
        for $struct:ident : $cursor:ty => $output:ident
        with
        ($($field:ident).+)
        ($($c_field:ident).+)
    ) =>
    {
        impl<$($generic $(:$trait)?),+> Index<$cursor> for $struct<$($generic),+>
        {
            type Output = $output;

            fn index(&self, idx: $cursor) -> &Self::Output {
                &(self.$($field).+)[idx.$($c_field).+]
            }
        }

        impl<$($generic $(:$trait)?),+> IndexMut<$cursor> for $struct<$($generic),+>
        {
            fn index_mut(&mut self, idx: $cursor) -> &mut T {
                &mut (self.$($field).+)[idx.$($c_field).+]
            }
        }
    };
}


use left;
use parse_range;
use right;


impl_index!(slice_index|
    impl<T, S: Count, L: TreeLayout> for SegmentTree with (data)
);
impl_index!(index|
    impl<T, S: Count, L: TreeLayout> for SegmentTree: Cursor => T
    with (data) (i)
);
impl_index!(index|
    impl<T, S: Count<Stats = T>, L: TreeLayout> for UpdaterAdd: Cursor => T
    with tuple (i)
);
// impl_index!(index|
//     impl<T, S: Count<Stats = T>> for UpdaterAssign: Cursor => T
//     with tuple (i)
// );


////////////////////////////////////////////////////////////////////////////////
//// Trait

/// Raw data type Into spec Segment Stats Type
pub trait RawIntoStats<S: Count<Stats = Self::Stats>> {
    type Stats;

    fn raw_into_stats(self) -> Self::Stats;
}


pub trait Count {
    type Stats;

    fn combine<'a>(l: &'a Self::Stats, r: &'a Self::Stats) -> Self::Stats;
    fn zero() -> Self::Stats;
}


pub trait TreeLayout: private::Sealed {
    fn left_i(c: Cursor) -> usize;
    fn right_i(c: Cursor) -> usize;
    fn size(cap: usize) -> usize;
}


mod private {
    pub trait Sealed {}

    impl Sealed for super::BFS {}
    impl Sealed for super::DFS {}
}


////////////////////////////////////////////////////////////////////////////////
//// Structure

pub struct SegmentTree<T, C: Count, L: TreeLayout = DFS> {
    data: Vec<T>,
    root: Cursor,
    _note: PhantomData<(C, L)>,
}


/// (i, tl, tr)
#[derive(Debug, Clone, Copy)]
pub struct Cursor {
    i: usize,
    tl: usize,
    tr: usize,
}


pub struct BFS;
/// Euler tour traversal (memory efficient)
pub struct DFS;


#[repr(transparent)]
pub struct UpdaterAdd<T, S: Count<Stats = T>, L: TreeLayout>(
    Vec<T>,
    PhantomData<(S, L)>,
);


// #[repr(transparent)]
// pub struct UpdaterAssign<T, S: Count<Stats = T>>(Vec<T>, PhantomData<S>);


// pub struct Updaters<T, S: Count<Stats = T>> {
//     add: Vec<T>,
//     assign: Vec<T>,
//     node_ : PhantomData<S>
// }


////////////////////////////////////////////////////////////////////////////////
//// Implementation

impl TreeLayout for BFS {
    #[inline(always)]
    fn left_i(c: Cursor) -> usize {
        c.i * 2
    }

    #[inline(always)]
    fn right_i(c: Cursor) -> usize {
        c.i * 2 + 1
    }

    fn size(cap: usize) -> usize {
        4 * cap
    }
}



impl TreeLayout for DFS {
    #[inline(always)]
    fn left_i(c: Cursor) -> usize {
        c.i + 1
    }

    #[inline(always)]
    fn right_i(c: Cursor) -> usize {
        // left_i + 2(n(left)) - 1
        Self::left_i(c) + 2 * (tm!(c.tl, c.tr) - c.tl + 1) - 1
    }

    fn size(cap: usize) -> usize {
        2 * cap - 1 + 1
    }
}


impl<T: Clone, S: Count<Stats = T>, L: TreeLayout> SegmentTree<T, S, L> {
    pub fn new<U>(raw: &[U]) -> Self
    where
        U: Clone + RawIntoStats<S, Stats = T>,
    {
        assert!(!raw.is_empty());

        let mut data = vec![S::zero(); L::size(raw.len())];

        let root = Cursor::init(raw.len());

        Self::build(&mut data, raw, root);

        Self {
            data,
            root,
            _note: PhantomData::<(S, L)>,
        }
    }

    pub fn query<R: RangeBounds<usize>>(&self, range: R) -> T {
        self.query_(parse_range!(range, self.root), self.root)
    }

    pub fn assoc(&mut self, i: usize, new_val: T) {
        assert!(i <= self.root.tr);

        self.assoc_(self.root, i, new_val)
    }

    /// Create an batch add updater
    pub fn create_updater(&self) -> UpdaterAdd<T, S, L> {
        UpdaterAdd(vec![S::zero(); self.data.len()], PhantomData)
    }

    ////////////////////////////////////////////////////////////////////////////
    /// Inner Method

    fn build<U>(data: &mut [T], arr: &[U], c: Cursor)
    where
        U: Clone + RawIntoStats<S, Stats = T>,
    {
        if c.is_end() {
            data[c.i] = arr[c.tl].clone().raw_into_stats();
        } else {
            Self::build(data, arr, left!(c));
            Self::build(data, arr, right!(c));

            data[c.i] = S::combine(&data[L::left_i(c)], &data[L::right_i(c)])
        }
    }

    fn query_(&self, (l, r): (usize, usize), c: Cursor) -> T {
        if l > r {
            return S::zero();
        }

        if c.is_matched((l, r)) {
            self[c].clone()
        } else {
            S::combine(
                &self.query_((l, min(left!(c).tr, r)), left!(c)),
                &self.query_((max(right!(c).tl, l), r), right!(c))
            )
        }
    }

    fn assoc_(&mut self, c: Cursor, ti: usize, new_val: T) {
        if c.is_end() {
            self.data[c.i] = new_val;
        } else {
            let clf = left!(c);
            let crh = right!(c);

            if ti <= clf.tr {
                self.assoc_(clf, ti, new_val);
            } else {
                self.assoc_(crh, ti, new_val);
            }

            self.data[c.i] = S::combine(&self.data[clf.i], &self.data[crh.i]);
        }
    }
}



impl Cursor {
    fn init(raw_len: usize) -> Self {
        debug_assert!(raw_len > 0);

        Self {
            i: 1,
            tl: 0,
            tr: raw_len - 1,
        }
    }

    #[inline(always)]
    fn is_end(&self) -> bool {
        self.tl == self.tr
    }

    #[inline(always)]
    fn is_matched(&self, (l, r): (usize, usize)) -> bool {
        self.tl == l && self.tr == r
    }

    #[inline(always)]
    fn godown_left<L: TreeLayout>(self) -> Self {
        Self {
            i: L::left_i(self),
            tl: self.tl,
            tr: tm!(self.tl, self.tr),
        }
    }

    #[inline(always)]
    fn godown_right<L: TreeLayout>(self) -> Self {
        Self {
            i: L::right_i(self),
            tl: tm!(self.tl, self.tr) + 1,
            tr: self.tr,
        }
    }
}



impl<T, C: Count<Stats = T>, L: TreeLayout> UpdaterAdd<T, C, L>
where
    T: Default + Ord + Clone + Add<Output = T> + AddAssign,
    for<'a> &'a T: Ord + Add<&'a T, Output = T> + 'a,
    for<'a> T: 'a,
{
    pub fn assoc<R: RangeBounds<usize>>(
        &mut self,
        tree: &mut SegmentTree<T, C, L>,
        range: R,
        addend: T,
    ) {
        self.assoc_(
            tree,
            parse_range!(range, tree.root),
            addend,
            tree.root,
        )
    }

    pub fn query<R: RangeBounds<usize>>(
        &mut self,
        tree: &mut SegmentTree<T, C, L>,
        range: R,
    ) -> T
    {
        self.query_(
            tree,
            parse_range!(range, tree.root),
            tree.root,
        )
    }

    fn is_marked(&self, c: Cursor) -> bool {
        if self[c] != C::zero() {
            true
        } else {
            false
        }
    }

    /// Lazy propagation
    fn propagate(&mut self, tree: &mut SegmentTree<T, C, L>, c: Cursor) {
        if self.is_marked(c) {
            tree[left!(c)] += self[c].clone();
            self[left!(c)] = &self[left!(c)] + &self[c];

            tree[right!(c)] += self[c].clone();
            self[right!(c)] = &self[right!(c)] + &self[c];

            self[c] = C::zero();
        }
    }

    fn assoc_(
        &mut self,
        tree: &mut SegmentTree<T, C, L>,
        (l, r): (usize, usize),
        addend: T,
        c: Cursor,
    )
    {
        if l > r {
            return;
        }

        if c.is_matched((l, r)) {
            tree[c] += addend.clone();
            self[c] += addend;
        } else {
            self.propagate(tree, c);

            self.assoc_(
                tree,
                (l, min(r, left!(c).tr)),
                addend.clone(),
                left!(c),
            );
            self.assoc_(
                tree,
                (max(l, right!(c).tl), r),
                addend,
                right!(c),
            );

            tree[c] = C::combine(&tree[left!(c)], &tree[right!(c)]);
        }
    }

    fn query_(
        &mut self,
        tree: &mut SegmentTree<T, C, L>,
        (l, r): (usize, usize),
        c: Cursor,
    ) -> T
    {
        if l > r {
            return C::zero();
        }

        if c.is_matched((l, r)) {
            tree[c].clone()
        } else {
            self.propagate(tree, c);

            C::combine(
                &self.query_(
                    tree,
                    (l, min(r, left!(c).tr)),
                    left!(c),
                ),
                &self.query_(
                    tree,
                    (max(l, right!(c).tl), r),
                    right!(c)
                )
            )
        }
    }
}




////////////////////////////////////////////////////////////////////////////////
//// Function
