#![feature(test)]


use m6_algs::spm::{bm::BMPattern, gen_pattern};

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
    let gen = || {
        let tested_patterns = gen_tested_pattern();
        for pattern in &tested_patterns {
            BMPattern::build_delta2_table_naive(pattern.as_bytes());
        }
    };

    b.iter(|| gen())
}

#[bench]
fn compute_delta2_knuth(b: &mut Bencher) {
    let gen = || {
        let tested_patterns = gen_tested_pattern();
        for pattern in &tested_patterns {
            BMPattern::build_delta2_table_improved_knuth(pattern.as_bytes());
        }
    };

    b.iter(|| gen())
}

#[bench]
fn compute_delta2_ryter(b: &mut Bencher) {
    let gen = || {
        let tested_patterns = gen_tested_pattern();
        for pattern in &tested_patterns {
            BMPattern::build_delta2_table_improved_rytter(pattern.as_bytes());
        }
    };

    b.iter(|| gen())
}

#[bench]
fn compute_delta2_blog(b: &mut Bencher) {
    let gen = || {
        let tested_patterns = gen_tested_pattern();
        for pattern in &tested_patterns {
            BMPattern::build_delta2_table_improved_blog(pattern.as_bytes());
        }
    };

    b.iter(|| gen())
}

#[bench]
fn compute_delta2_minghu6(b: &mut Bencher) {
    let gen = || {
        let tested_patterns = gen_tested_pattern();
        for pattern in &tested_patterns {
            BMPattern::build_delta2_table_improved_minghu6(pattern.as_bytes());
        }
    };

    b.iter(|| gen())
}
