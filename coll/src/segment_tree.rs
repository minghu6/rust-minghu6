use std::{
    borrow::Borrow,
    marker::PhantomData,
    ops::{Add, Bound::*, RangeBounds}, cmp::*,
};


////////////////////////////////////////////////////////////////////////////////
//// Macro

macro_rules! tm {
    ($tl:expr, $tr:expr) => {
        ($tl + $tr) / 2
    };
}


////////////////////////////////////////////////////////////////////////////////
//// Trait

pub trait SegStats<T> {
    fn combine<'a>(l: &'a T, r: &'a T) -> T;
    fn default() -> T;
}


pub trait TreeLayout: private::Sealed
{
    fn left_i(c: Cursor) -> usize;
    fn right_i(c: Cursor) -> usize;
    fn size(cap: usize) -> usize;
}


mod private {
    pub trait Sealed {}
}


////////////////////////////////////////////////////////////////////////////////
//// Structure

pub struct SegmentTree<T, S: SegStats<T>, L: TreeLayout = BFS> {
    data: Vec<T>,
    root: Cursor,
    _note: PhantomData<(S, L)>,
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


pub struct Sum<T>(PhantomData<T>);


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

impl private::Sealed for BFS {}


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

impl private::Sealed for DFS {}


impl<T: Clone, S: SegStats<T>, L: TreeLayout> SegmentTree<T, S, L> {
    pub fn new(raw: &[T]) -> Self
    where
        T: Default,
    {
        assert!(!raw.is_empty());

        let mut data = vec![T::default(); 4 * raw.len()];

        let root = Cursor::init(raw.len());

        Self::build(&mut data, raw, root);

        Self {
            data,
            root,
            _note: PhantomData::<(S, L)>,
        }
    }

    pub fn query<R: RangeBounds<usize>>(&self, range: R) -> T {
        let l;
        let r;

        match range.start_bound() {
            Included(v) => l = *v,
            Excluded(v) => l = *v + 1,
            Unbounded => panic!("Unsupported unbound range"),
        }

        match range.end_bound() {
            Included(v) => r = *v,
            Excluded(v) => {
                assert!(*v > 0, "range upper is invalid (=0)");
                r = *v - 1
            }
            Unbounded => r = self.root.tr,
        }

        self.query_(self.root, l, r)
    }

    pub fn assoc(&mut self, i: usize, new_val: T) {
        assert!(i <= self.root.tr);

        self.assoc_(self.root, i, new_val)
    }

    ////////////////////////////////////////////////////////////////////////////
    /// Inner Method

    fn assoc_(&mut self, c: Cursor, ti: usize, new_val: T) {
        if c.is_end() {
            self.data[c.i] = new_val;
        }
        else {
            let clf = c.godown_left::<L>();
            let crh = c.godown_right::<L>();

            if ti <= clf.tr {
                self.assoc_(clf, ti, new_val);
            }
            else {
                self.assoc_(crh, ti, new_val);
            }

            self.data[c.i] = S::combine(&self.data[clf.i], &self.data[crh.i]);
        }
    }

    fn query_(&self, c: Cursor, l: usize, r: usize) -> T {
        if l == c.tl && r == c.tr {
            self.data[c.i].clone()
        } else {
            let clf = c.godown_left::<L>();
            let crh = c.godown_right::<L>();

            let lf_r = min(clf.tr, r);
            let rh_l = max(crh.tl, l);

            let mut sum = S::default();

            if l <= lf_r {
                sum = S::combine(&sum, &self.query_(clf, l, lf_r))
            }

            if rh_l <= r {
                sum = S::combine(&sum, &self.query_(crh, rh_l, r))
            }

            sum
        }
    }

    fn build(data: &mut [T], arr: &[T], c: Cursor)
    where
        T: Clone,
    {
        if c.is_end() {
            data[c.i] = arr[c.tl].clone();
        } else {
            Self::build(data, arr, c.godown_left::<L>());
            Self::build(data, arr, c.godown_right::<L>());

            data[c.i] =
                S::combine(&data[L::left_i(c)], &data[L::right_i(c)])
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


impl<T> SegStats<T> for Sum<T>
where
    T: Default,
    for<'a> &'a T: Add<&'a T, Output = T> + 'a,
    for<'a> T: 'a,
{
    fn combine(l: &T, h: &T) -> T {
        l.borrow() + h.borrow()
    }

    fn default() -> T {
        T::default()
    }
}



////////////////////////////////////////////////////////////////////////////////
//// Function

// fn count_segment_tree_size(cap: usize) -> usize {
//     assert!(cap > 0);

//     fn count(tl: usize, tr: usize) -> usize {
//         if tl == tr { 1 }
//         else {
//             let tm = tm!(tl, tr);

//             1 + count(tl, tm) + count(tm+1, tr)
//         }
//     }

//     count(0, cap - 1)

// }




#[cfg(test)]
mod tests {
    use std::ops::Range;

    use common::random_range;

    use super::*;

    const TEST_CAP_RANGE: Range<usize> = 100..500;
    const TEST_VALUE_UPPER_LIMIT: usize = 100;


    macro_rules! gen_arr {
        (test | N) => {
            gen_arr!(200, TEST_CAP_RANGE, 0..TEST_VALUE_UPPER_LIMIT, i32)
        };
        (test | Z+) => {
            gen_arr!(200, TEST_CAP_RANGE, 1..TEST_VALUE_UPPER_LIMIT, usize)
        };
        ($batch:expr, $cap_range:expr, $v_range:expr, $ty:ty) => {{
            let mut batch_arr = vec![];

            for _ in 0..$batch {
                let cap = random_range($cap_range);
                let mut arr = vec![0; cap];

                for j in 0..cap {
                    arr[j] = random_range($v_range) as $ty;
                }

                batch_arr.push(arr);
            }

            batch_arr
        }};
    }

    macro_rules! gen_query {
        (test| $cap:expr) => {
            gen_query!(500, 0..$cap, 1..=$cap, $cap)
        };
        ($batch:expr, $i_range:expr, $len_range:expr, $cap:expr) => {{
            let mut batch_vec = vec![];

            for _ in 0..$batch {
                let i = random_range($i_range);
                let len = random_range($len_range);

                batch_vec.push(i..std::cmp::min(i + len, $cap));
            }

            batch_vec
        }};
    }

    macro_rules! gen_update {
        (test| $cap:expr) => {
            gen_update!(100, 0..$cap, 0..TEST_VALUE_UPPER_LIMIT, i32)
        };
        ($batch:expr, $i_range:expr, $v_range:expr, $ty:ty) => {{
            let mut batch_vec = vec![];

            for _ in 0..$batch {
                let i = random_range($i_range);
                let v = random_range($v_range) as $ty;

                batch_vec.push((i, v));
            }

            batch_vec
        }};
    }


    #[test]
    fn test_segment_tree_build() {
        /* Manually Test */

        let arr = [1, 2, 3, 4, 5, 6];

        let st = SegmentTree::<i32, Sum<_>>::new(&arr);
        let stm = SegmentTree::<i32, Sum<_>, DFS>::new(&arr);

        assert_eq!(st.data[1], stm.data[1]);

        for i in 1..1000 {
            for arr in gen_arr!(1, i..i+1, 1..1000, i32) {
                let st = SegmentTree::<i32, Sum<_>>::new(&arr);
                let stm = SegmentTree::<i32, Sum<_>, DFS>::new(&arr);

                assert_eq!(
                    st.data[1], stm.data[1],
                    "st:{} / stm:{}  failed at {i}",
                    st.data[1], stm.data[1]
                );
            }
        }
    }


    #[test]
    fn test_segment_tree_sum() {
        for mut arr in gen_arr!(test | N) {
            let mut st = SegmentTree::<i32, Sum<_>>::new(&arr);
            let mut stm = SegmentTree::<i32, Sum<_>, DFS>::new(&arr);

            /* update */

            for (i, v) in gen_update!(test| arr.len()) {
                arr[i] = v;
                st.assoc(i, v);
                stm.assoc(i, v);
            }

            /* update-query */

            for q in gen_query!(test | arr.len()) {
                let expect: i32 = arr[q.clone()].into_iter().sum();

                let res = st.query(q.clone());
                let resm = stm.query(q);

                assert_eq!(res, expect, "res: {res} / expect: {expect}");
                assert_eq!(resm, expect, "resM: {resm} / expect: {expect}")
            }
        }
    }

}
