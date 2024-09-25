use std::{
    borrow::Borrow,
    cmp::{Ordering::*, *},
    marker::PhantomData,
    ops::{Add, RangeBounds, Sub},
};

use common::{ Min, max, parse_range };
use math::gcd;

use super::{
    left, right, Count, Cursor, RawIntoStats, SegmentTree,
    TreeLayout,
};

////////////////////////////////////////////////////////////////////////////////
//// Macros

macro_rules! impl_raw_into_stats {
    (monomorphic | $struct:ident, $for_ty:ty, $stats_ty:ty { $fn:item }) => {
        impl RawIntoStats<$struct> for $for_ty {
            type Stats = $stats_ty;

            $fn
        }
    };
    ($struct:ident, $for_ty:ty, $stats_ty:ty { $fn:item }) => {
        impl RawIntoStats<$struct<$for_ty>> for $for_ty {
            type Stats = $stats_ty;

            $fn
        }
    };
}


macro_rules! impl_for_num {
    ($name:ident|all) => {
        impl_for_num!($name|int);
        impl_for_num!($name|float);
    };
    ($name:ident|int) => {
        impl_for_num!($name|sint);
        impl_for_num!($name|uint);
    };
    ($name:ident|float) => {
        impl_for_num!($name|f32);
        impl_for_num!($name|f64);
    };
    ($name:ident|uint) => {
        impl_for_num!($name|u128);
        impl_for_num!($name|u64);
        impl_for_num!($name|usize);
        impl_for_num!($name|u32);
        impl_for_num!($name|u16);
        impl_for_num!($name|u8);
    };
    ($name:ident|sint) => {
        impl_for_num!($name|i128);
        impl_for_num!($name|i64);
        impl_for_num!($name|isize);
        impl_for_num!($name|i32);
        impl_for_num!($name|i16);
        impl_for_num!($name|i8);
    };
    (sum_stats| $for_ty:ty) => {
        impl_raw_into_stats!(Sum, $for_ty, $for_ty {
            fn raw_into_stats(self) -> Self::Stats {
                self
            }
        });
    };
    (max_stats| $for_ty:ty) => {
        impl_raw_into_stats!(Max, $for_ty, $for_ty {
            fn raw_into_stats(self) -> Self::Stats {
                self
            }
        });
    };
    (max_stats_stats| $for_ty:ty) => {
        impl_raw_into_stats!(MaxStats, $for_ty, ($for_ty, usize) {
            fn raw_into_stats(self) -> Self::Stats {
                (self, 1)
            }
        });
    };
    (gcd_stats| $for_ty:ty) => {
        impl_raw_into_stats!(GCD, $for_ty, $for_ty {
            fn raw_into_stats(self) -> Self::Stats {
                self
            }
        });
    };
    (gcd_count_sint| $for_ty:ty) => {
        impl Count for GCD<$for_ty>
        {
            type Stats = $for_ty;

            fn combine<'a>(l: &'a Self::Stats, r: &'a Self::Stats) -> Self::Stats {
                gcd!(l.abs(), r.abs()).abs()
            }

            fn e() -> Self::Stats {
                0
            }
        }
    };
    (gcd_count_uint| $for_ty:ty) => {
        impl Count for GCD<$for_ty>
        {
            type Stats = $for_ty;

            fn combine<'a>(l: &'a Self::Stats, r: &'a Self::Stats) -> Self::Stats {
                gcd!(l.clone(), r.clone())
            }

            fn e() -> Self::Stats {
                0
            }
        }
    };
    (zero_stats| $for_ty:ty) => {
        impl_raw_into_stats!(monomorphic|ZeroStats, $for_ty, usize {
            fn raw_into_stats(self) -> Self::Stats {
                if self == 0 { 1 } else { 0 }
            }
        });
    };
    (sub_seg_max_sum_stats| $for_ty:ty) => {
        impl_raw_into_stats!(SubSegMaxSum, $for_ty, SubSegMaxSumStats<$for_ty> {
            fn raw_into_stats(self) -> Self::Stats {
                SubSegMaxSumStats {
                    sum: self,
                    pref: self,
                    suff: self,
                    ans: self
                }
            }
        });
    };
}


impl_for_num!(sum_stats | all);
impl_for_num!(max_stats | int);
impl_for_num!(max_stats_stats | int);
impl_for_num!(gcd_stats | int);
impl_for_num!(zero_stats | int);
impl_for_num!(sub_seg_max_sum_stats | sint);


impl_for_num!(gcd_count_sint | sint);
impl_for_num!(gcd_count_uint | uint);

////////////////////////////////////////////////////////////////////////////////
//// Structures

#[derive(Clone, Copy)]
pub struct Sum<T>(PhantomData<T>);

#[derive(Clone, Copy)]
pub struct Max<T>(PhantomData<T>);

#[derive(Clone, Copy)]
pub struct MaxStats<T>(PhantomData<T>);

#[derive(Clone, Copy)]
pub struct GCD<T>(PhantomData<T>);

#[derive(Clone, Copy)]
pub struct ZeroStats;

/// find max positive sum of a range
pub struct SubSegMaxSum<T>(PhantomData<T>);



#[derive(Clone, Copy, Debug, Default)]
pub struct SubSegMaxSumStats<T> {
    /// total sum of the segment
    pub sum: T,
    /// max prefix sum
    pub pref: T,
    /// max suffix sum
    pub suff: T,
    /// query answer of max sum of the segment
    pub ans: T,
}


////////////////////////////////////////////////////////////////////////////////
//// Implementations

impl<T: Min<T>> Min<SubSegMaxSumStats<T>> for SubSegMaxSumStats<T> {
    fn min() -> Self {
        Self {
            sum: T::min(),
            pref: T::min(),
            suff: T::min(),
            ans: T::min(),
        }
    }
}

impl<T> Count for Sum<T>
where
    T: Default,
    for<'a> &'a T: Add<&'a T, Output = T> + 'a,
    for<'a> T: 'a,
{
    type Stats = T;

    fn combine(l: &Self::Stats, r: &Self::Stats) -> Self::Stats {
        l.borrow() + r.borrow()
    }

    fn e() -> Self::Stats {
        T::default()
    }
}

impl<T> Sum<T>
where
    T: Default,
    for<'a> &'a T: Add<&'a T, Output = T> + Sub<&'a T, Output = T> + Ord + 'a,
    for<'a> T: 'a,
{
    pub fn find_nth<L: TreeLayout>(
        tree: &SegmentTree<T, Self, L>,
        x: &T,
    ) -> Option<usize> {
        if x > &tree[tree.root] {
            None
        } else {
            Some(Self::find_nth_(tree, x, tree.root))
        }
    }

    fn find_nth_<L: TreeLayout>(
        tree: &SegmentTree<T, Self, L>,
        x: &T,
        c: Cursor,
    ) -> usize {
        if c.is_end() {
            c.tl
        } else {
            let clf = left!(c);
            let crh = right!(c);

            if &tree[clf] >= x {
                Self::find_nth_(tree, x, clf)
            } else {
                Self::find_nth_(tree, &(x - &tree[clf]), crh)
            }
        }
    }
}


impl<T> Count for Max<T>
where
    T: Default + Ord + Clone,
    for<'a> &'a T: Add<&'a T, Output = T> + 'a,
    for<'a> T: 'a,
{
    type Stats = T;

    fn combine(l: &Self::Stats, r: &Self::Stats) -> Self::Stats {
        match l.cmp(&r) {
            Less => r,
            Equal => l,
            Greater => l,
        }
        .clone()
    }

    fn e() -> Self::Stats {
        Default::default()
    }
}


impl<T> Count for MaxStats<T>
where
    T: Min<T> + Ord + Clone,
    for<'a> &'a T: Add<&'a T, Output = T> + 'a,
    for<'a> T: 'a,
{
    type Stats = (T, usize);

    fn combine(l: &Self::Stats, r: &Self::Stats) -> Self::Stats {
        match l.0.cmp(&r.0) {
            Less => r.clone(),
            Equal => (l.0.clone(), l.1 + r.1),
            Greater => l.clone(),
        }
    }

    fn e() -> Self::Stats {
        (<T as Min<T>>::min(), 0)
    }
}


impl<T> Max<T>
where
    T: Default+ Ord + Clone,
    for<'a> &'a T: Add<&'a T, Output = T> + 'a,
    for<'a> T: 'a,
{
    pub fn query_first_gt<L: TreeLayout, R: RangeBounds<usize>>(
        tree: &SegmentTree<T, Self, L>,
        range: R,
        x: &T,
    ) -> Option<usize> {
        Self::query_first_gt_1(
            tree,
            x,
            parse_range!(range, tree.root.tr + 1),
            tree.root,
        )
    }

    fn query_first_gt_1<L: TreeLayout>(
        tree: &SegmentTree<T, Self, L>,
        x: &T,
        (l, r): (usize, usize),
        c: Cursor,
    ) -> Option<usize> {
        // left or right
        if c.tl > r || c.tr < l {
            None
        }
        // inner
        else if c.tr <= r && c.tl >= l {
            Self::query_first_gt_2(tree, x, c)
        }
        // cross
        else {
            // avoid bound overflow

            let left_res = Self::query_first_gt_1(tree, x, (l, r), left!(c));

            if left_res.is_some() {
                return left_res;
            }

            Self::query_first_gt_1(tree, x, (l, r), right!(c))
        }
    }

    fn query_first_gt_2<L: TreeLayout>(
        tree: &SegmentTree<T, Self, L>,
        x: &T,
        c: Cursor,
    ) -> Option<usize> {
        if &tree[c] <= x {
            return None;
        }

        if c.is_end() {
            Some(c.tl)
        } else {
            if &tree[left!(c)] > x {
                Self::query_first_gt_2(tree, x, left!(c))
            } else {
                Self::query_first_gt_2(tree, x, right!(c))
            }
        }
    }
}


impl Count for ZeroStats {
    type Stats = usize;

    fn combine(l: &Self::Stats, r: &Self::Stats) -> Self::Stats {
        *l + *r
    }

    fn e() -> Self::Stats {
        0
    }
}


impl ZeroStats {
    /// Start from 0
    pub fn find_nth<L: TreeLayout>(
        tree: &SegmentTree<usize, Self, L>,
        n: usize,
    ) -> Option<usize> {
        if n + 1 > tree[tree.root] {
            None
        } else {
            Some(Self::find_nth_(tree, n, tree.root))
        }
    }

    fn find_nth_<L: TreeLayout>(
        tree: &SegmentTree<usize, Self, L>,
        n: usize,
        c: Cursor,
    ) -> usize {
        if c.is_end() {
            c.tl
        } else {
            let clf = left!(c);
            let crh = right!(c);

            if tree[clf] > n {
                Self::find_nth_(tree, n, clf)
            } else {
                Self::find_nth_(tree, n - tree[clf], crh)
            }
        }
    }
}


impl<T> Count for SubSegMaxSum<T>
where
    T: Default + Ord + Clone + Min<T>,
    for<'a> &'a T: Add<&'a T, Output = T> + Ord + 'a,
    for<'a> T: 'a,
{
    type Stats = SubSegMaxSumStats<T>;

    fn combine(l: &Self::Stats, r: &Self::Stats) -> Self::Stats {
        let limit_ans = Self::e().ans;

        if l.ans == limit_ans {
            return r.clone();
        }
        else if r.ans == limit_ans {
            return l.clone();
        }

        SubSegMaxSumStats {
            sum: &l.sum + &r.sum,
            pref: max(&l.pref, &(&l.sum + &r.pref)).clone(),
            suff: max(&(&l.suff + &r.sum), &r.suff).clone(),
            ans: max!(&l.ans, &r.ans, &(&l.suff + &r.pref)).clone(),
        }
    }

    fn e() -> Self::Stats {
        SubSegMaxSumStats::<T>::min()
    }
}



