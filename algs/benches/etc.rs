#![feature(test)]

extern crate test;
use test::Bencher;

use std::hint::black_box;

use lazy_static::lazy_static;

use common::random;



lazy_static! {
    /// BE CAREFUL ABLOUT NUMBER IN CASE OF O(N^2) SPACE COMPELEXITY
    static ref RANDOM_INTS: Vec<u32> = {
        (0..1_000).map(|_| random::<u32>()).collect()
    };
}


#[bench]
fn bench_loop_inner(b: &mut Bencher) {
    b.iter(|| {
        let mut ans = vec![];

        for i in 0..RANDOM_INTS.len() {
            for j in 0..RANDOM_INTS.len() {
                if RANDOM_INTS[i] % 2 == 0 {
                    ans.push(i);
                }
                else {
                    ans.push(j);
                }
            }
        }

        black_box(ans);
    })
}


#[bench]
fn bench_loop_outter_group(b: &mut Bencher) {
    b.iter(|| {
        let mut ans = vec![];

        for i in 0..RANDOM_INTS.len() {
            if RANDOM_INTS[i] % 2 == 0 {
                for _ in 0..RANDOM_INTS.len() {
                    ans.push(i);
                }
            }
            else {
                for j in 0..RANDOM_INTS.len() {
                    ans.push(j);
                }
            }
        }

        black_box(ans);
    })
}
