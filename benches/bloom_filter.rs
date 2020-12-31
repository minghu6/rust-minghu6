#![feature(test)]
#![allow(dead_code)]

use bit_vec::BitVec;

use minghu6::collections::bloom_filter::{ BytesBloomFilter, FastBloomFilter, BloomFilter };
use minghu6::test::spm::{  gen_random_text };


extern crate test;

use test::Bencher;

fn gen_test_text() -> String {
    gen_random_text(1000)
}

#[bench]
fn simple_bloom_filter_basic_op(b: &mut Bencher) {
    let gen = || {
        let mut bloom_filter = BytesBloomFilter::new();

        for b in gen_test_text().as_bytes() {
            bloom_filter.insert(&b);

            bloom_filter.contains(&b);
        }
    };

    b.iter(|| gen())
}

#[bench]
fn fast_bloom_filter_basic_op(b: &mut Bencher) {
    let gen = || {
        let mut bloom_filter = FastBloomFilter::with_rate(100, 0.15);

        for b in gen_test_text().as_bytes() {
            bloom_filter.insert(&b);

            bloom_filter.contains(&b);
        }
    };

    b.iter(|| gen())
}

#[bench]
fn bitvec_bloom_filter_basic_op(b: &mut Bencher) {
    let gen = || {
        let mut bits = BitVec::from_elem(128, false);

        for b in gen_test_text().as_bytes() {
            for i in 0..3 {
                bits.set((b+i & 127).into(), true);
            }

            for i in 0..3 {
                if let Some(v) = bits.get((b+i & 127) as usize) {
                    if v {
                        let a = 1+1;
                    } else{
                        let b = 2+2;
                    }

                } else {
                    panic!("");
                }
            }
        }
    };

    b.iter(|| gen())
}