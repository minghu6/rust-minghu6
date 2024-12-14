#![feature(test)]

use std::collections::BTreeMap;

use lazy_static::lazy_static;
use m6_coll_st::bt::{bpt, bpt3, flatbpt};


extern crate test;
use test::{black_box, Bencher};

mod dict_common;


lazy_static! {
    static ref INSERT_DATA: Vec<(u64, u64)> =
        (0..10_0000).map(|x| (x, x)).collect();
}

macro_rules! bench_dict_build {
    ($name:ident, $dict:path) => {
        coll::paste!(
            #[allow(non_snake_case)]
            #[bench]
            fn [<bench_dict_bulk_build_ $name>] (b: &mut Bencher) {
                b.iter(|| black_box($dict::from_iter(INSERT_DATA.clone())));
            }
        );
   };
}

bench_dict_build!(BTreeMap, BTreeMap);
bench_dict_build!(FBPT32, flatbpt::FlatBPT::<_, _, 32>);
bench_dict_build!(BPT_V1_32, bpt::BPT::<_, _, 32>);
bench_dict_build!(BPT_V3_32, bpt3::BPT::<_, _, 32>);
