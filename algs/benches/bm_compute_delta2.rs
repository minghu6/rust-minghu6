#![feature(test)]


use m6_algs::string::{bm::*, gen_pattern};

extern crate test;

use test::Bencher;

fn gen_tested_pattern() -> Vec<String> {
    let mut result = vec![];

    for pattern in gen_pattern((1..100, 1), 1) {
        result.push(pattern)
    }

    result
}


#[bench]
fn gen_some_random_pattern(b: &mut Bencher) {
    b.iter(|| {
        gen_tested_pattern();
    })
}


#[ignore]
#[bench]
fn compute_delta2_naive(b: &mut Bencher) {
    let generate = || {
        let tested_patterns = gen_tested_pattern();
        for pattern in &tested_patterns {
            build_delta2_table_naive(pattern.as_bytes());
        }
    };

    b.iter(|| generate())
}


#[bench]
fn compute_delta2_minghu6(b: &mut Bencher) {
    let generate = || {
        let tested_patterns = gen_tested_pattern();
        for pattern in &tested_patterns {
            build_delta2_table_improved_minghu6(pattern.as_bytes());
        }
    };

    b.iter(|| generate())
}
