#![feature(test)]

use minghu6::algs::spm::sa::*;
use minghu6::test::spm::gen_random_text;

extern crate test;

use test::Bencher;


fn gen_tested_text() -> Vec<String> {
    let mut result = vec![];
    for i in 1..300 {
        result.push(gen_random_text(i));
    }

    result
}

#[bench]
fn gen_some_random_text(b: &mut Bencher) {
    b.iter(|| {
        gen_tested_text();
    })
}

#[bench]
fn compute_sa_naive(b: &mut Bencher) {
    b.iter(|| {
        for text in gen_tested_text() {
            compute_suffix_array_naive(text.as_bytes());
        }
    })
}

#[bench]
fn compute_sa_doubling(b: &mut Bencher) {
    b.iter(|| {
        for text in gen_tested_text() {
            compute_suffix_array_doubling(text.as_bytes());
        }
    })
}

#[bench]
fn compute_sa_doubling_radix(b: &mut Bencher) {
    b.iter(|| {
        for text in gen_tested_text() {
            compute_suffix_array_doubling_radix(text.as_bytes());
        }
    })
}

#[bench]
fn compute_sa_doubling_radix_improved(b: &mut Bencher) {
    b.iter(|| {
        for text in gen_tested_text() {
            compute_suffix_array_doubling_radix_improved(text.as_bytes());
        }
    })
}