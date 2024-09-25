#![feature(test)]
#![feature(isqrt)]

extern crate test;

use std::{hint::black_box, ops::Range};

use common::random_range;
use lazy_static::lazy_static;
use m6_math::{gcd, is_prime, number::*};
use test::Bencher;


const GCD_PAIRS_LEN: usize = 1_000;
const GCD_U_RANGE: Range<u64> = 0x000F_FFFF_FFFF_FFFF..0x00FF_FFFF_FFFF_FFFF;
// const GCD_U_RANGE: Range<u64> = 0x0000_0000_0000_FFFF..0x0000_0000_00FF_FFFF;

const GCD_V_RANGE: Range<u64> = 0x0000_0FFF_FFFF_FFFF..0x0000_FFFF_FFFF_FFFF;
// const GCD_V_RANGE: Range<u64> = 0x0000_0000_0000_FFFF..0x0000_0000_00FF_FFFF;

const PRIME_N: usize = 10usize.pow(6);

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
            black_box(gcd!(sub | m, n));
        }
    })
}


#[bench]
fn bench_gcd_smart(b: &mut Bencher) {
    b.iter(|| {
        for (m, n) in GCD_PAIRS.iter().cloned() {
            black_box(gcd!(u64 | m, n));
        }
    })
}

#[ignore = "slow"]
#[bench]
fn bench_pri_brute(b: &mut Bencher) {
    b.iter(|| {
        black_box(
            (0..PRIME_N)
                .filter_map(
                    |v| if is_prime!(v; usize) { Some(v) } else { None },
                )
                .collect::<Vec<usize>>(),
        );
    })
}

#[bench]
fn bench_pri_e_sieve(b: &mut Bencher) {
    b.iter(|| {
        black_box(
                e_sieve(PRIME_N)
                .collect::<Vec<usize>>(),
        );
    })
}

#[bench]
fn bench_pri_e_seg_sieve(b: &mut Bencher) {
    b.iter(|| {
        black_box(
                e_seg_sieve(PRIME_N)
                .collect::<Vec<usize>>(),
        );
    })
}

#[bench]
fn bench_pri_e_seg_sieve_p(b: &mut Bencher) {
    b.iter(|| {
        black_box(
                e_seg_sieve_p(PRIME_N)
                .collect::<Vec<usize>>(),
        );
    })
}

#[ignore = "a little slow"]
#[bench]
fn bench_pri_wheel_sieve(b: &mut Bencher) {
    b.iter(|| {
        black_box(
                wheel_sieve(PRIME_N)
                .collect::<Vec<usize>>(),
        );
    })
}

#[bench]
fn bench_pri_fixed_wheel_seg_sieve(b: &mut Bencher) {
    b.iter(|| {
        black_box(
                fixed_wheel_seg_sieve(PRIME_N)
                .collect::<Vec<usize>>(),
        );
    })
}

#[bench]
fn bench_pri_fixed_wheel_seg_sieve_p(b: &mut Bencher) {
    b.iter(|| {
        black_box(
                fixed_wheel_seg_sieve_p(PRIME_N)
                .collect::<Vec<usize>>(),
        );
    })
}

#[bench]
fn bench_pri_sundaram_sieve(b: &mut Bencher) {
    b.iter(|| {
        black_box(
                sundaram_sieve(PRIME_N)
                .collect::<Vec<usize>>(),
        );
    })
}

#[bench]
fn bench_pri_sundaram_improved_sieve(b: &mut Bencher) {
    b.iter(|| {
        black_box(
                sundaram_sieve_improved(PRIME_N)
                .collect::<Vec<usize>>(),
        );
    })
}

#[bench]
fn bench_pri_atkin_simple_sieve(b: &mut Bencher) {
    b.iter(|| {
        black_box(
                atkin_sieve_simple(PRIME_N)
                .collect::<Vec<usize>>(),
        );
    })
}

// #[bench]
// fn bench_pri_atkin_enumerate_lattice_sieve(b: &mut Bencher) {
//     b.iter(|| {
//         black_box(
//                 atkin_sieve_enum_lattice(PRIME_N)
//                 .collect::<Vec<usize>>(),
//         );
//     })
// }

#[bench]
fn bench_pri_mairson_sieve(b: &mut Bencher) {
    b.iter(|| {
        black_box(
                mairson_sieve(PRIME_N)
                .collect::<Vec<usize>>(),
        );
    })
}

#[bench]
fn bench_pri_inc_e_sieve(b: &mut Bencher) {
    b.iter(|| {
        black_box(
                e_inc_sieve(PRIME_N)
                .collect::<Vec<usize>>(),
        );
    })
}

#[bench]
fn bench_pri_mairson_duel_sieve(b: &mut Bencher) {
    b.iter(|| {
        black_box(
                mairson_dual_sieve(PRIME_N)
                .collect::<Vec<usize>>(),
        );
    })
}

#[bench]
fn bench_pri_gpf_sieve(b: &mut Bencher) {
    b.iter(|| {
        black_box(
                gpf_sieve(PRIME_N)
                .collect::<Vec<usize>>(),
        );
    })
}

#[bench]
fn bench_pri_inc_e_sieve_inf(b: &mut Bencher) {
    b.iter(|| {
        black_box(
                e_inc_sieve_inf()
                .take_while(|v| *v < PRIME_N)
                .collect::<Vec<usize>>(),
        );
    })
}

#[bench]
fn bench_pri_bengelloun_sieve_inf(b: &mut Bencher) {
    b.iter(|| {
        black_box(
                bengelloun_sieve_inf()
                .take_while(|v| *v < PRIME_N)
                .collect::<Vec<usize>>(),
        );
    })
}

#[bench]
fn bench_pri_gpf_sieve_inf(b: &mut Bencher) {
    b.iter(|| {
        black_box(
                gpf_sieve_inf()
                .take_while(|v| *v < PRIME_N)
                .collect::<Vec<usize>>(),
        );
    })
}

