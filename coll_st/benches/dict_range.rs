#![feature(test)]

use std::{ collections::BTreeMap, hint::black_box, ops::RangeInclusive};

use rand::prelude::*;
use lazy_static::lazy_static;

use m6_coll_st::bt::*;


extern crate test;
use test::Bencher;

const DATA_SIZE: usize = 50_000;
const TEST_GROUPS_SIZE: usize = 5_000;

lazy_static! {
    static ref INSTALL_DATA: Vec<usize> = {
        let mut inputs = (0..DATA_SIZE).collect::<Vec<usize>>();

        inputs.shuffle(&mut thread_rng());

        inputs
    };

    static ref TEST_DATA: Vec<RangeInclusive<usize>> = {
        let mut rng = thread_rng();

        (0..TEST_GROUPS_SIZE).map(|_| {
            let start = rng.gen_range(0..DATA_SIZE - 1 - 2000);
            let end = rng.gen_range(start + 1..start + 1 + 2000);

            start..=end
        })
        .collect::<Vec<_>>()
    };
}

macro_rules! bench_dict_range {
    ($v:ident, $name: ident, $dict: expr) => {
        bench_dict_range!($v, $name, $dict, range);
    };
    ($v:ident, $name: ident, $dict: expr, $i:path) => {
        coll::paste!(
            #[allow(non_snake_case)]
            #[bench]
            fn [<bench_dict_range_ $v _ $name>] (b: &mut Bencher) {
                let mut dict = $dict;

                for k in INSTALL_DATA.iter().cloned() {
                    dict.insert(k, k);
                }

                b.iter(|| {
                    for range in TEST_DATA.iter() {
                        black_box(dict.$i(range.clone()).collect::<Vec<_>>());
                    }
                });
            }
        );
   };
}

bench_dict_range!(__, BTree, BTreeMap::new());
// bench_dict_range!(V0_11, BPT, bpt::BPT::<_, _, 11>::new());
// bench_dict_range!(V0_20, BPT, bpt::BPT::<_, _, 20>::new());
// bench_dict_range!(_V0_26, BPT, bpt::BPT::<_, _, 26>::new());
// bench_dict_range!(_V0_30, BPT, bpt::BPT::<_, _, 30>::new());

// bench_dict_range!(_V1_11, BPT, bpt2::BPT2::<_, _, 11>::new());
// bench_dict_range!(_V1_20, BPT, bpt2::BPT2::<_, _, 20>::new());
// bench_dict_range!(_V1_26, BPT, bpt2::BPT2::<_, _, 26>::new());
// bench_dict_range!(_V1_30, BPT, bpt2::BPT2::<_, _, 30>::new());

bench_dict_range!(_20, FBPT, flatbpt::FlatBPT::<_, _, 20>::new());
bench_dict_range!(_30, FBPT, flatbpt::FlatBPT::<_, _, 30>::new());
