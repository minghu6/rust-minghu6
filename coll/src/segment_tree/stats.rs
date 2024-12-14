use std::{
    cmp::{Ordering::*, *},
    iter::Sum,
    marker::PhantomData,
    ops::{Add, AddAssign, Range, RangeBounds, Sub},
};

use math::gcd;

use super::{SegmentTree, TreeCursor, TreeLayout};


////////////////////////////////////////////////////////////////////////////////
//// Macros

macro_rules! zero {
    () => {
        [].into_iter().sum()
    };
}

////////////////////////////////////////////////////////////////////////////////
//// Traits

pub(crate) trait Number =
    Clone + Copy + Ord + Add<Output = Self> + AddAssign + Sum + Sub;

////////////////////////////////////////////////////////////////////////////////
//// Structures

#[derive(Clone, Copy)]
pub struct RangeSum<T>(PhantomData<T>);

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
#[repr(transparent)]
pub struct RangeMax<T>(T);

/// (max value, max value count)
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct RangeMaxStats<T> {
    val: T,
    cnt: usize,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(transparent)]
pub struct RangeGCD<T>(T);

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
#[repr(transparent)]
pub struct RangeZeroStats<T> {
    cnt: usize,
    _marker: PhantomData<T>,
}

/// find max positive sum of a range
#[derive(Clone, Copy, Debug)]
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

// macro_rules! impl_sub_seg_max_sum_stats_from {
//     ($($ty:ty),*) => {
//         $(
//             impl From<$ty> for SubSegMaxSumStats<$ty> {
//                 fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
//                     iter.reduce(|acc, x| acc + x).unwrap_or(Self(0))
//                 }
//             }
//         )*
//     };
// }

impl<T: Number> From<T> for SubSegMaxSumStats<T> {
    fn from(value: T) -> Self {
        Self {
            sum: value,
            pref: value,
            suff: value,
            ans: value,
        }
    }
}

impl<T: Sum> Default for SubSegMaxSumStats<T> {
    fn default() -> Self {
        Self {
            sum: zero!(),
            pref: zero!(),
            suff: zero!(),
            ans: zero!(),
        }
    }
}

impl<T: Number> Sum for SubSegMaxSumStats<T> {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.reduce(|acc, x| acc + x).unwrap_or_default()
    }
}

impl<T: Number> Add<Self> for SubSegMaxSumStats<T> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let limit_ans = Self::default().ans;

        if self.ans == limit_ans {
            return rhs;
        } else if rhs.ans == limit_ans {
            return self;
        }

        Self {
            sum: self.sum + rhs.sum,
            pref: max(self.pref, self.sum + rhs.pref),
            suff: max(self.suff + rhs.sum, rhs.suff),
            ans: [self.ans, rhs.ans, (self.suff + rhs.pref)]
                .into_iter()
                .max()
                .unwrap(),
        }
    }
}

impl<'a, T: Number> Add<Self> for &'a SubSegMaxSumStats<T> {
    type Output = SubSegMaxSumStats<T>;

    fn add(self, rhs: Self) -> Self::Output {
        *self + *rhs
    }
}

impl<T> RangeZeroStats<T> {
    pub fn from_cnt(cnt: usize) -> Self {
        Self {
            cnt,
            _marker: PhantomData,
        }
    }
}

impl<T> RangeZeroStats<T>
where
    for<'a> &'a T: Add<&'a T, Output = T> + Sub<&'a T, Output = T> + Ord + 'a,
    for<'a> T: 'a,
{
    pub fn find_nth<L: TreeLayout>(
        tree: &SegmentTree<RangeZeroStats<T>, L>,
        n: usize,
    ) -> Option<usize> {
        if n + 1 > tree[L::root()].cnt {
            None
        } else {
            Some(Self::find_nth_(tree, n, L::root(), tree.root))
        }
    }

    fn find_nth_<L: TreeLayout>(
        tree: &SegmentTree<RangeZeroStats<T>, L>,
        n: usize,
        i: usize,
        t: TreeCursor,
    ) -> usize {
        if t.is_end() {
            t.tl
        } else {
            if tree[L::left(i, t)].cnt > n {
                Self::find_nth_(tree, n, L::left(i, t), t.left())
            } else {
                Self::find_nth_(
                    tree,
                    n - tree[L::left(i, t)].cnt,
                    L::right(i, t),
                    t.right(),
                )
            }
        }
    }
}

impl<T> Into<usize> for RangeZeroStats<T> {
    fn into(self) -> usize {
        self.cnt
    }
}

impl<'a, T> Sub<Self> for &'a RangeZeroStats<T> {
    type Output = RangeZeroStats<T>;

    fn sub(self, rhs: Self) -> Self::Output {
        RangeZeroStats {
            cnt: self.cnt - rhs.cnt,
            _marker: PhantomData,
        }
    }
}

impl<T: Sum + Eq> From<T> for RangeZeroStats<T> {
    fn from(value: T) -> Self {
        Self {
            cnt: if value == zero!() { 1 } else { 0 },
            _marker: PhantomData,
        }
    }
}

impl<T: Sum + Eq> Sum for RangeZeroStats<T> {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.reduce(|acc, x| acc + x).unwrap_or(RangeZeroStats {
            cnt: 0,
            _marker: PhantomData,
        })
    }
}

impl<T> Add<Self> for RangeZeroStats<T> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            cnt: self.cnt + rhs.cnt,
            _marker: PhantomData,
        }
    }
}

impl<'a, T> Add<Self> for &'a RangeZeroStats<T> {
    type Output = RangeZeroStats<T>;

    fn add(self, rhs: Self) -> Self::Output {
        RangeZeroStats {
            cnt: self.cnt + rhs.cnt,
            _marker: PhantomData,
        }
    }
}

impl<'a, T> AddAssign<&'a Self> for RangeZeroStats<T> {
    fn add_assign(&mut self, rhs: &'a Self) {
        self.cnt += rhs.cnt
    }
}

impl<T: Eq> PartialEq<usize> for RangeZeroStats<T> {
    fn eq(&self, other: &usize) -> bool {
        self.cnt.eq(other)
    }
}

impl<T> From<T> for RangeGCD<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}

macro_rules! impl_range_gcd {
    ($($ty:ty),*) => {
        $(
            impl Sum<Self> for RangeGCD<$ty> {
                fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
                    iter.reduce(|acc, x| acc + x).unwrap_or(Self(0))
                }
            }

            impl Add<Self> for RangeGCD<$ty> {
                type Output = Self;

                fn add(self, rhs: Self) -> Self::Output {
                    Self(gcd!(self.0, rhs.0))
                }
            }

            impl<'a> Add<Self> for &'a RangeGCD<$ty> {
                type Output = RangeGCD<$ty>;

                fn add(self, rhs: Self) -> Self::Output {
                    RangeGCD(gcd!(self.0, rhs.0))
                }
            }

            impl<'a> AddAssign<&'a Self> for RangeGCD<$ty> {
                fn add_assign(&mut self, rhs: &'a Self) {
                    self.0 = gcd!(self.0, rhs.0)
                }
            }
        )*
    };
}

impl_range_gcd!(i32, i64, usize);

impl<T> From<T> for RangeMaxStats<T> {
    fn from(value: T) -> Self {
        Self { val: value, cnt: 1 }
    }
}

impl<T> From<(T, usize)> for RangeMaxStats<T> {
    fn from(value: (T, usize)) -> Self {
        Self {
            val: value.0,
            cnt: value.1,
        }
    }
}

impl<T: Sum + Ord + Clone + Default> Sum for RangeMaxStats<T> {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.reduce(|acc, x| acc + x).unwrap_or(Self {
            val: T::default(),
            cnt: 0,
        })
    }
}

impl<T: Ord + Clone> Add<Self> for RangeMaxStats<T> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        match self.val.cmp(&rhs.val) {
            Less => rhs,
            Equal => RangeMaxStats {
                val: rhs.val,
                cnt: self.cnt + rhs.cnt,
            },
            Greater => self,
        }
    }
}

impl<'a, T: Ord + Clone> Add<Self> for &'a RangeMaxStats<T> {
    type Output = RangeMaxStats<T>;

    fn add(self, rhs: Self) -> Self::Output {
        match self.val.cmp(&rhs.val) {
            Less => rhs.clone(),
            Equal => RangeMaxStats {
                val: self.val.clone(),
                cnt: self.cnt + rhs.cnt,
            },
            Greater => self.clone(),
        }
    }
}

impl<'a, T: Ord + Clone> AddAssign<&'a Self> for RangeMaxStats<T> {
    fn add_assign(&mut self, rhs: &'a Self) {
        match self.val.cmp(&rhs.val) {
            Less => *self = rhs.clone(),
            Equal => self.cnt += rhs.cnt,
            Greater => (),
        };
    }
}

impl<T> From<T> for RangeMax<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}

impl<T: Sum + Ord> Sum for RangeMax<T> {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        Self(iter.map(|x| x.0).max().unwrap_or(zero!()))
    }
}

impl<T: Ord> Add for RangeMax<T> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(std::cmp::max(self.0, rhs.0))
    }
}

impl<'a, T: Ord + Clone + 'a> Add for &'a RangeMax<T> {
    type Output = RangeMax<T>;

    fn add(self, rhs: Self) -> Self::Output {
        RangeMax(std::cmp::max(&self.0, &rhs.0).clone())
    }
}

impl<'a, T: Ord + Clone> AddAssign<&'a Self> for RangeMax<T> {
    fn add_assign(&mut self, rhs: &'a Self) {
        if rhs > self {
            *self = rhs.clone();
        }
    }
}

impl<T: Ord> PartialEq<T> for RangeMax<T> {
    fn eq(&self, other: &T) -> bool {
        self.0.eq(other)
    }
}

impl<T: Ord> PartialOrd<T> for RangeMax<T> {
    fn partial_cmp(&self, other: &T) -> Option<Ordering> {
        self.0.partial_cmp(other)
    }
}

impl<T> RangeSum<T>
where
    for<'a> &'a T: Add<&'a T, Output = T> + Sub<&'a T, Output = T> + Ord + 'a,
    for<'a> T: 'a,
{
    pub fn find_nth<L: TreeLayout>(
        tree: &SegmentTree<T, L>,
        x: &T,
    ) -> Option<usize> {
        if x > &tree[L::root()] {
            None
        } else {
            Some(Self::find_nth_(tree, x, L::root(), tree.root))
        }
    }

    fn find_nth_<L: TreeLayout>(
        tree: &SegmentTree<T, L>,
        x: &T,
        i: usize,
        t: TreeCursor,
    ) -> usize {
        if t.is_end() {
            t.tl
        } else {
            let clf = t.left();
            let crh = t.right();

            if &tree[L::left(i, t)] >= x {
                Self::find_nth_(tree, x, L::left(i, t), clf)
            } else {
                Self::find_nth_(
                    tree,
                    &(x - &tree[L::left(i, t)]),
                    L::right(i, t),
                    crh,
                )
            }
        }
    }
}


impl<T> RangeMax<T>
where
    T: Ord,
    for<'a> &'a T: Add<&'a T, Output = T> + 'a,
    for<'a> T: 'a,
{
    pub fn query_first_gt<L: TreeLayout, R: RangeBounds<usize>>(
        tree: &SegmentTree<Self, L>,
        range: R,
        x: &Self,
    ) -> Option<usize> {
        let Range { start, end } = std::slice::range(range, ..tree.len());

        Self::query_first_gt_1(tree, x, (start, end - 1), L::root(), tree.root)
    }

    fn query_first_gt_1<L: TreeLayout>(
        tree: &SegmentTree<Self, L>,
        x: &Self,
        (l, r): (usize, usize),
        i: usize,
        t: TreeCursor,
    ) -> Option<usize> {
        // left or right
        if t.tl > r || t.tr < l {
            None
        }
        // inner
        else if t.tr <= r && t.tl >= l {
            Self::query_first_gt_2(tree, x, i, t)
        }
        // cross
        else {
            // avoid bound overflow

            let left_res = Self::query_first_gt_1(
                tree,
                x,
                (l, r),
                L::left(i, t),
                t.left(),
            );

            if left_res.is_some() {
                return left_res;
            }

            Self::query_first_gt_1(tree, x, (l, r), L::right(i, t), t.right())
        }
    }

    fn query_first_gt_2<L: TreeLayout>(
        tree: &SegmentTree<Self, L>,
        x: &Self,
        i: usize,
        t: TreeCursor,
    ) -> Option<usize> {
        if &tree[i] <= x {
            return None;
        }

        if t.is_end() {
            Some(t.tl)
        } else {
            if &tree[L::left(i, t)] > x {
                Self::query_first_gt_2(tree, x, L::left(i, t), t.left())
            } else {
                Self::query_first_gt_2(tree, x, L::right(i, t), t.right())
            }
        }
    }
}
