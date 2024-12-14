#![feature(test)]

use std::{collections::BTreeMap, hint::black_box, ops::Bound::*};

use lazy_static::lazy_static;
use m6_coll_st::bt::*;
use rand::prelude::*;
use test_suites::{
    bpt_mapping::{A, BPTIU, D, Q, R},
    mapping::RandomInputFacility,
    rand::RandomRoller,
};

extern crate test;
use test::Bencher;

const DATA_SIZE: usize = 5_0000;

lazy_static! {
    static ref INSTALL_DATA: Vec<(i32, i32)> =
        (0..DATA_SIZE).map(|x| (x as _, x as _)).collect();
    static ref TEST_DATA: Vec<BPTIU> = {
        let mut inputs = Vec::with_capacity(DATA_SIZE);

        let mut roller = RandomRoller::with_candicates(vec![
            (30, Q(0)),
            (45, R(Unbounded, Unbounded)),
            (30, A(0, 0)),
            (10, D(0)),
        ]);
        let mut tracer = RandomInputFacility::<i32, i32>::default();

        for (k, v) in INSTALL_DATA.iter().cloned() {
            tracer.insert(k, v);
        }

        let mut g = thread_rng();
        let mut cnt = 0;

        while cnt < DATA_SIZE * 2 {
            cnt += 1;

            inputs.push(match roller.roll() {
                Q(..) => {
                    let k = tracer.randomly_roll_item().0;
                    Q(k)
                }
                R(..) => {
                    let k1 = tracer.randomly_roll_item().0;
                    let k2 = g.gen_range(k1..k1 + 500);

                    R(Included(k1), Included(k2))
                }
                A(..) => {
                    let k = g.gen_range(0..DATA_SIZE * 3) as _;
                    tracer.insert(k, k);
                    A(k, k)
                }
                D(..) => {
                    let k = tracer.randomly_roll_item().0;
                    tracer.remove(&k);
                    D(k)
                }
                _ => unreachable!(),
            });
        }

        inputs
    };
}


macro_rules! bench_dict_all {
    ($name: ident, $dict: path) => {
        coll::paste!(
            #[allow(non_snake_case)]
            #[bench]
            fn [<bench_dict_all_ $name>] (b: &mut Bencher) {
                // let mut dict = $dict::from_iter(INSTALL_DATA.iter().cloned());
                let mut dict = $dict::new();

                for (k, v) in INSTALL_DATA.iter().cloned() {
                    dict.insert(k, v);
                }

                b.iter(|| {
                    for iu in TEST_DATA.iter() {
                        black_box(match iu {
                            Q(k) => {
                                dict.get(&k);
                            }
                            R(start_bound, end_bound) => {
                                match (start_bound, end_bound) {
                                    (Included(k1), Included(k2)) => {
                                        let _ = dict.range(k1..=k2).collect::<Vec<_>>();
                                    }
                                    _ => unimplemented!()
                                }
                            }
                            A(k, v) => {
                                dict.insert(*k, *v);
                            }
                            D(k) => {
                                dict.remove(k);
                            },
                            _ => unimplemented!()
                        });
                    }
                });
            }
        );
   };
}

bench_dict_all!(BTree, BTreeMap);

// bench_dict_all!(BPT_31, bpt::BPT::<_, _, 31>);
bench_dict_all!(BPT_32, bpt::BPT::<_, _, 32>);
// bench_dict_all!(BPT_33, bpt::BPT::<_, _, 33>);

// bench_dict_all!(BPT2_20, bpt2::BPT2::<_, _, 20>);
bench_dict_all!(BPT2_32, bpt2::BPT2::<_, _, 32>);
// bench_dict_all!(BPT2_40, bpt2::BPT2::<_, _, 40>);

// bench_dict_all!(FBPT_11, flatbpt::FlatBPT::<_, _, 11>);
// bench_dict_all!(FBPT_20, flatbpt::FlatBPT::<_, _, 20>);
// bench_dict_all!(FBPT_26, flatbpt::FlatBPT::<_, _, 26>);
bench_dict_all!(FBPT_32, flatbpt::FlatBPT::<_, _, 32>);
