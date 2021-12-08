#![feature(test)]

use std::thread;
use std::sync::mpsc;

extern crate num_cpus;

use minghu6::algs::spm::sa::*;
use minghu6::algs::spm::sais::suffix_array_sais;
use minghu6::algs::spm::sa16::suffix_array_16;
use minghu6::test::spm::gen_random_text;
use minghu6::algs::math::*;

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

#[ignore = "too slow 1st"]
#[bench]
fn compute_sa_doubling_radix(b: &mut Bencher) {
    b.iter(|| {
        for text in gen_tested_text() {
            suffix_array_bl_radix(text.as_bytes());
        }
    })
}

#[ignore = "too slow 2nd"]
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

#[bench]
fn compute_sa_16_parallel(b: &mut Bencher) {
    b.iter(|| {
        let texts = gen_tested_text();
        let partial_num = num_cpus::get();
        let texts_chunks = split_improved(&texts[..], partial_num);

        let mut children = vec![];
        let (tx, rx) = mpsc::channel();

        for chunk in texts_chunks.into_iter() {
            let tx_clone = tx.clone();

            children.push(thread::spawn(move || {
                for text in chunk {
                    let subres = suffix_array_16(text.as_bytes());

                    tx_clone.send(subres).unwrap()
                }
            }));
        }

        for child in children {
            child.join().unwrap();
            rx.recv().unwrap();
        }

    })
}

#[bench]
fn compute_sa_16_control_group(b: &mut Bencher) {
    b.iter(|| {
        let texts = gen_tested_text();
        let end = texts.len() / num_cpus::get() + 1;

        for text in &texts[..][0..end] {
            suffix_array_16(text.as_bytes());
        }
    })
}
