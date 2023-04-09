#![feature(test)]

extern crate test;

use std::{ops::Range, hint::black_box};
use test::Bencher;

use lazy_static::lazy_static;

use common::random_range;
use m6_coll::segment_tree::{SegmentTree, Sum, BFS, DFS};


const ARR_CAP: usize = 0_100_000;
const Q_BATCH: usize = 0_050_000;


lazy_static! {
    static ref FIXED_ARR: Vec<i32> = {
        let mut arr = vec![0; ARR_CAP];

        for j in 0..ARR_CAP {
            arr[j] = random_range!(0..1000) as i32;
        }

        arr
    };
    static ref QS: Vec<Range<usize>> = {
        let mut qs = vec![];

        for _ in 0..Q_BATCH {
            let i = random_range!(0..ARR_CAP);
            let len = random_range!(Q_BATCH / 10..=Q_BATCH);

            qs.push(i..std::cmp::min(i + len, ARR_CAP));
        }

        qs
    };
}


#[bench]
fn bench_segement_tree_sum_bfs(b: &mut Bencher) {
    let st = SegmentTree::<i32, Sum<_>, BFS>::new(&FIXED_ARR);

    b.iter(|| {
        for q in QS.iter() {
            black_box(st.query(q.clone()));
        }
    })
}


#[bench]
fn bench_segement_tree_sum_dfs(b: &mut Bencher) {
    let st = SegmentTree::<i32, Sum<_>, DFS>::new(&FIXED_ARR);

    b.iter(|| {
        for q in QS.iter() {
            black_box(st.query(q.clone()));
        }
    })
}


#[bench]
fn bench_segement_tree_sum_raw(b: &mut Bencher) {
    b.iter(|| {
        for q in QS.iter() {
            black_box(FIXED_ARR[q.clone()].into_iter().sum::<i32>());
        }
    })
}


#[ignore = "10 times slower than others"]
#[bench]
fn bench_segement_tree_sum_manually(b: &mut Bencher) {
    b.iter(|| {
        for q in QS.iter() {
            let mut sum = 0;

            for i in q.clone() {
                black_box(sum += FIXED_ARR[i]);
            }

            black_box(sum);
        }
    })
}
