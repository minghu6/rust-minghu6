use std::{cmp::Ordering::*, ops::Range};

use common::random_range;
use math::gcd_rem as gcd;

use super::*;

const TEST_CAP_RANGE: Range<usize> = 100..500;
const TEST_VALUE_UPPER_LIMIT: usize = 100;


macro_rules! gen_arr {
    (N-i64) => {
        gen_arr!(200, TEST_CAP_RANGE, 0..TEST_VALUE_UPPER_LIMIT, i64)
    };
    (N) => {
        gen_arr!(200, TEST_CAP_RANGE, 0..TEST_VALUE_UPPER_LIMIT, i32)
    };
    (I) => {
        gen_arr!(
            50,
            TEST_CAP_RANGE,
            -(TEST_VALUE_UPPER_LIMIT as isize)..TEST_VALUE_UPPER_LIMIT as isize,
            i32
        )
    };
    (Z+) => {
        gen_arr!(200, TEST_CAP_RANGE, 1..TEST_VALUE_UPPER_LIMIT, usize)
    };
    ($batch:expr, $cap_range:expr, $v_range:expr, $ty:ty) => {{
        use common::random_range;

        let mut batch_arr = vec![];

        for _ in 0..$batch {
            let cap = random_range!($cap_range);
            let mut arr = vec![0; cap];

            for j in 0..cap {
                arr[j] = random_range!($v_range) as $ty;
            }

            batch_arr.push(arr);
        }

        batch_arr
    }};
}

macro_rules! gen_query {
    ($cap:expr) => {
        gen_query!(500, $cap)
    };
    ($batch:expr, $cap:expr) => {
        gen_query!($batch, 0..$cap, 1..=$cap, $cap)
    };
    ($batch:expr, $i_range:expr, $len_range:expr, $cap:expr) => {{
        use common::random_range;

        let mut batch_vec = vec![];

        for _ in 0..$batch {
            let i = random_range!($i_range);
            let len = random_range!($len_range);

            batch_vec.push(i..std::cmp::min(i + len, $cap));
        }

        batch_vec
    }};
}

macro_rules! gen_update {
    (N-i64| $cap:expr) => {
        gen_update!(100, 0..$cap, 0..TEST_VALUE_UPPER_LIMIT, i64)
    };
    (N| $cap:expr) => {
        gen_update!(100, 0..$cap, 0..TEST_VALUE_UPPER_LIMIT, i32)
    };
    (I| $cap:expr) => {
        gen_update!(
            100,
            0..$cap,
            -(TEST_VALUE_UPPER_LIMIT as isize)..TEST_VALUE_UPPER_LIMIT as isize,
            i32
        )
    };
    (range|$batch:expr, $cap:expr, $v_range:expr, $ty:ty) => {{
        use common::random_range;

        let mut batch_vec = vec![];

        for q in gen_query!($batch, $cap) {
            let v = random_range!($v_range) as $ty;

            batch_vec.push((q, v));
        }

        batch_vec
    }};
    ($batch:expr, $i_range:expr, $v_range:expr, $ty:ty) => {{
        use common::random_range;

        let mut batch_vec = vec![];

        for _ in 0..$batch {
            let i = random_range!($i_range);
            let v = random_range!($v_range) as $ty;

            batch_vec.push((i, v));
        }

        batch_vec
    }};
}


pub(crate) use gen_arr;
pub(crate) use gen_query;
pub(crate) use gen_update;


#[test]
fn test_segment_tree_build() {
    /* Manually Test */

    let arr = [1, 2, 3, 4, 5, 6];

    let st = SegmentTree::<i32, BFS>::new(&arr);
    let stm = SegmentTree::<i32, DFS>::new(&arr);

    assert_eq!(st[1], stm[1]);

    for i in 1..1000 {
        for arr in gen_arr!(1, i..i + 1, 1..1000, i32) {
            let st = SegmentTree::<i32, BFS>::new(&arr);
            let stm = SegmentTree::<i32, DFS>::new(&arr);

            assert_eq!(
                st[1], stm[1],
                "st:{} / stm:{}  failed at {i}",
                st[1], stm[1]
            );
        }
    }
}


#[test]
fn test_segment_tree_sum_nth() {
    for mut arr in gen_arr!(N) {
        let mut st = SegmentTree::<i32, BFS>::new(&arr);
        let mut stm = SegmentTree::<i32, DFS>::new(&arr);

        /* update */

        for (i, v) in gen_update!(N | arr.len()) {
            arr[i] = v;
            st.assoc(i, v);
            stm.assoc(i, v);
        }

        /* update-query */

        for q in gen_query!(arr.len()) {
            let expect: i32 = arr[q.clone()].into_iter().sum();

            let res = st.query(q.clone());
            let resm = stm.query(q);

            assert_eq!(res, expect, "res / expect");
            assert_eq!(resm, expect, "resM / expect")
        }

        /* verify nth prefix sum */

        let prefix_acc = arr[..]
            .into_iter()
            .scan(0, |acc, &x| {
                *acc += x;

                Some(*acc)
            })
            .collect::<Vec<_>>();

        for i in (0..st.query(..)).step_by(100) {
            let expect = prefix_acc
                .iter()
                .enumerate()
                .find(|(_, x)| **x >= i)
                .map(|x| x.0);

            let res = RangeSum::find_nth(&st, &i);

            assert_eq!(res, expect, "{res:#?} / {expect:#?} for {i}");
        }
    }
}


#[test]
fn test_segment_tree_max_nth_updater() {
    for mut arr in gen_arr!(N - i64) {
        let mut st = SegmentTree::<RangeMax<i64>>::new(&arr);

        /* update */

        for (i, v) in gen_update!(N - i64 | arr.len()) {
            arr[i] = v;
            st.assoc(i, v.into());
        }

        /* query */

        for q in gen_query!(arr.len()) {
            let expect = arr[q.clone()].into_iter().max().cloned().unwrap();

            let res = st.query(q.clone());

            assert_eq!(res, expect, "res / expect");

            /* verify query_nth_gt */

            let qx = arr[random_range!(0..arr.len())]
                + arr[random_range!(0..arr.len())];

            let expect = arr[q.clone()]
                .into_iter()
                .enumerate()
                .find(|&x| x.1 > &qx)
                .map(|x| x.0 + q.start);

            let res = RangeMax::<i64>::query_first_gt(&st, q, &qx.into());

            assert_eq!(res, expect, "res / expect");
        }

        /* batch update add */

        let mut updater = st.create_updater();

        for (_i, q) in gen_query!(50, arr.len()).into_iter().enumerate() {
            let addend = random_range!(0..100);

            // println!("{_i:3} +{addend}");

            for i in q.clone() {
                if addend > arr[i] {
                    arr[i] = addend;
                }
            }

            updater.assoc(&mut st, q, addend.into());

            for q2 in gen_query!(50, arr.len()) {
                let expect =
                    arr[q2.clone()].into_iter().max().cloned().unwrap();

                let res = updater.query(&mut st, q2.clone());

                assert_eq!(res, expect, "res / expect");
            }
        }
    }
}


#[test]
fn test_segment_tree_max_stats() {
    for mut arr in gen_arr!(N) {
        let mut st = SegmentTree::<RangeMaxStats<_>>::new(&arr);

        /* update */
        for (i, v) in gen_update!(N | arr.len()) {
            arr[i] = v;
            st.assoc(i, v.into());
        }

        /* query */
        for q in gen_query!(arr.len()) {
            let expect = arr[q.clone()]
                .into_iter()
                .map(|x| (*x, 1))
                .reduce(|mut acc, x| {
                    match acc.0.cmp(&x.0) {
                        Less => acc = x,
                        Equal => acc.1 += 1,
                        Greater => (),
                    };

                    acc
                })
                .unwrap();

            let res = st.query(q.clone());

            assert_eq!(res, expect.into(), "res / expect");
        }
    }
}


#[test]
fn test_segment_tree_gcd() {
    for mut arr in gen_arr!(N) {
        let mut st = SegmentTree::<RangeGCD<i32>>::new(&arr);

        /* update */
        for (i, v) in gen_update!(N | arr.len()) {
            arr[i] = v;
            st.assoc(i, v.into());
        }

        /* update-query */
        for q in gen_query!(arr.len()) {
            let expect = arr[q.clone()]
                .into_iter()
                .cloned()
                .reduce(|acc, x| gcd!(acc.abs(), x.abs()))
                .unwrap()
                .abs();

            let res = st.query(q.clone());

            assert_eq!(res, expect.into(), "res / expect");
        }
    }
}


#[test]
fn test_segment_tree_zero_nth() {
    for mut arr in gen_arr!(I) {
        let mut st = SegmentTree::<RangeZeroStats<_>>::new(&arr);

        /* update */
        for (i, v) in gen_update!(I | arr.len()) {
            arr[i] = v;
            st.assoc(i, v.into());
        }

        /* update-query */
        for q in gen_query!(arr.len()) {
            let expect =
                arr[q.clone()].into_iter().filter(|&&x| x == 0).count();

            let res = st.query(q.clone());

            assert_eq!(res, expect, "res / expect");
        }

        /* verify nth */
        let nth_acc = arr[..]
            .into_iter()
            .scan(0, |acc, &x| {
                if x == 0 {
                    *acc += 1
                }

                Some(*acc)
            })
            .collect::<Vec<_>>();

        for i in 0..st.query(..).into() {
            let nth_res = RangeZeroStats::find_nth(&st, i);

            let nth_expect = nth_acc
                .iter()
                .enumerate()
                .find(|&nth| *nth.1 == i + 1)
                .map(|x| x.0);

            assert_eq!(nth_res, nth_expect, "res / expect");
        }
    }
}


#[test]
fn test_segment_tree_sub_seg_max_sum() {
    let calc_sub_seg_max_sum = |arr: &[i32]| {
        arr
        .into_iter()
        // DP algorithm
        .scan(0, |st, &x| {
            *st = max(x, x+*st);
            Some(*st)
        })
        .max()
    };

    for mut arr in gen_arr!(I) {
        let mut st =
            SegmentTree::<SubSegMaxSumStats<i32>>::new(&arr);

        /* update */
        for (i, v) in gen_update!(I | arr.len()) {
            arr[i] = v;
            st.assoc(
                i,
                v.into(),
            );
        }

        /* update-query */
        for q in gen_query!(50, arr.len()) {
            let expect = calc_sub_seg_max_sum(&arr[q.clone()]).unwrap();
            let res = st.query(q.clone());

            assert_eq!(res.ans, expect, "res / expect {res:#?}");
        }
    }
}
