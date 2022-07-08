#![feature(test)]

extern crate test;

use std::{collections::BinaryHeap, cmp::Reverse};
use m6coll::Entry;
use sorted_vec::SortedVec;

use minghu6::test::sort::gen_bench_case;
use test::Bencher;

const UNIT_LEN: usize = 5_000;

fn gen_case() -> Vec<Vec<Entry<usize, usize>>> {
    gen_bench_case(UNIT_LEN, 20)
}


#[bench]
fn bench_heap_sv(b: &mut Bencher) {
    let gen = || {
        for case in gen_case() {
            let mut heap = BinaryHeap::with_capacity(UNIT_LEN);

            for e in case.into_iter() {
                heap.push(Reverse(e));
            }

            let mut res = Vec::with_capacity(UNIT_LEN);
            for _ in 0..UNIT_LEN {
                res.push(heap.pop().unwrap());
            }
        }
    };

    b.iter(|| gen())
}


#[bench]
fn bench_sortedvec_sv(b: &mut Bencher) {
    let gen = || {
        for case in gen_case() {
            let mut sortvec = SortedVec::with_capacity(UNIT_LEN);

            for e in case.into_iter() {
                sortvec.insert(e);
            }
        }
    };

    b.iter(|| gen())
}
