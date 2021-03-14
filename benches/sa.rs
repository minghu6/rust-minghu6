#![feature(test)]

use minghu6::algs::spm::sa::*;
use minghu6::algs::spm::sais::suffix_array_sais;
use minghu6::algs::spm::sa16::suffix_array_16;
use minghu6::test::spm::gen_random_text;

extern crate test;

use test::Bencher;


fn gen_tested_text() -> Vec<String> {
    let mut result = vec![];
    for i in 8000..8100 {
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
            suffix_array_bl(text.as_bytes());
        }
    })
}

#[bench]
fn compute_sa_doubling_radix(b: &mut Bencher) {
    b.iter(|| {
        for text in gen_tested_text() {
            suffix_array_bl_radix(text.as_bytes());
        }
    })
}

#[bench]
fn compute_sa_doubling_radix_improved(b: &mut Bencher) {
    b.iter(|| {
        for text in gen_tested_text() {
            suffix_array_bl_radix_improved(text.as_bytes());
        }
    })
}

#[bench]
fn compute_sa_is(b: &mut Bencher) {
    b.iter(|| {
        for text in gen_tested_text() {
            suffix_array_sais(text.as_bytes());
        }
    })
}

#[bench]
fn compute_sa_16(b: &mut Bencher) {
    b.iter(|| {
        for text in gen_tested_text() {
            suffix_array_16(text.as_bytes());
        }
    })
}