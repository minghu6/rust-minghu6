#![feature(test)]

use std::collections::BTreeMap;

use lazy_static::lazy_static;
use m6_coll_st::bt::{bpt::BPT, flatbpt::FlatBPT};
#[allow(unused_imports)]
use m6_coll_st::{bst, bt};


extern crate test;
use test::{black_box, Bencher};

mod dict_common;


lazy_static! {
    static ref INSERT_DATA: Vec<(u64, u64)> =
        (0..10_0000).map(|x| (x, x)).collect();
}

#[allow(non_snake_case)]
#[bench]
fn bench_dict_bulk_build_BTreeMap(b: &mut Bencher) {
    b.iter(|| black_box(BTreeMap::from_iter(INSERT_DATA.clone())));
}

#[allow(non_snake_case)]
#[bench]
fn bench_dict_bulk_build_FBPT_11(b: &mut Bencher) {
    b.iter(|| black_box(FlatBPT::<_, _, 11>::from_iter(INSERT_DATA.clone())));
}

#[allow(non_snake_case)]
#[bench]
fn bench_dict_bulk_build_FBPT_20(b: &mut Bencher) {
    b.iter(|| black_box(FlatBPT::<_, _, 20>::from_iter(INSERT_DATA.clone())));
}

#[allow(non_snake_case)]
#[bench]
fn bench_dict_bulk_build_FBPT_30(b: &mut Bencher) {
    b.iter(|| black_box(FlatBPT::<_, _, 30>::from_iter(INSERT_DATA.clone())));
}

#[allow(non_snake_case)]
#[bench]
fn bench_dict_bulk_build_FBPT_100(b: &mut Bencher) {
    b.iter(|| black_box(FlatBPT::<_, _, 100>::from_iter(INSERT_DATA.clone())));
}

#[allow(non_snake_case)]
#[bench]
fn bench_dict_bulk_build_BPT_20(b: &mut Bencher) {
    b.iter(|| {
        let mut bpt = BPT::<_, _, 20>::new();

        black_box(bpt.bulk_push_back(INSERT_DATA.clone().into_iter()))
    });
}
