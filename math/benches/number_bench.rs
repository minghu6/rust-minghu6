#![feature(test)]

extern crate test;

use std::{hint::black_box, ops::Range};
use test::Bencher;

use lazy_static::lazy_static;

use common::random_range;
use m6_math::gcd;


const GCD_PAIRS_LEN: usize = 1_000;
const GCD_U_RANGE: Range<u64> = 0x000F_FFFF_FFFF_FFFF..0x00FF_FFFF_FFFF_FFFF;
// const GCD_U_RANGE: Range<u64> = 0x0000_0000_0000_FFFF..0x0000_0000_00FF_FFFF;

const GCD_V_RANGE: Range<u64> = 0x0000_0FFF_FFFF_FFFF..0x0000_FFFF_FFFF_FFFF;
// const GCD_V_RANGE: Range<u64> = 0x0000_0000_0000_FFFF..0x0000_0000_00FF_FFFF;


lazy_static! {
    static ref GCD_PAIRS: Vec<(u64, u64)> = {
        let mut arr = vec![(0, 0); GCD_PAIRS_LEN];

        for j in 0..GCD_PAIRS_LEN {
            let u = random_range!(GCD_U_RANGE);
            let v = random_range!(GCD_V_RANGE);

            arr[j] = (u, v)
        }

        arr
    };

}


#[bench]
fn bench_gcd_mod(b: &mut Bencher) {
    b.iter(|| {
        for (m, n) in GCD_PAIRS.iter().cloned() {
            black_box(gcd!(mod| m, n));
        }
    })
}

#[ignore = "too slow"]
#[bench]
fn bench_gcd_sub(b: &mut Bencher) {
    b.iter(|| {
        for (m, n) in GCD_PAIRS.iter().cloned() {
            black_box(gcd!(sub| m, n));
        }
    })
}


#[bench]
fn bench_gcd_smart(b: &mut Bencher) {
    b.iter(|| {
        for (m, n) in GCD_PAIRS.iter().cloned() {
            black_box(gcd!(u64| m, n));
        }
    })
}
