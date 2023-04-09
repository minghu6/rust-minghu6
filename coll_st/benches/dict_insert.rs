#![feature(test)]

use lazy_static::lazy_static;

#[allow(unused_imports)]
use m6_coll_st::{
    bst,
    bt
};


extern crate test;
use test::Bencher;

mod dict_common;


lazy_static! {
    static ref INSERT_DATA: Vec<(u64, u64)> = {
        let get_one = || common::random::<u64>();

        gen_data!(get_one, 50, 200)
    };
}


macro_rules! bench_dict_insert {
    ($v:ident, $name: ident, $dict: expr) => {
        bench_dict_insert!($v, $name, $dict, i: insert);
    };
    ($v:ident, $name: ident, $dict: expr, i: $i:ident) => {
        coll::paste!(
            #[allow(non_snake_case)]
            #[bench]
            fn [<bench_dict_insert_ $v _ $name>] (b: &mut Bencher) {
                b.iter(|| {
                    let mut dict = $dict;

                    for (k, v) in INSERT_DATA.iter().cloned() {
                        dict.$i(k, v);
                    }
                });
            }
        );
   };
}


bench_dict_insert!(_0_, HASH_MAP, std::collections::HashMap::new());


// #[ignore="Too Slow"]
// bench_dict_insert!(V2, SG, bst::sg::SG::new(0.7));

// #[ignore="Too Slow"]
// bench_dict_insert!(V2, LSG, bst::lsg::LSG::new(0.7));

// #[ignore="Too Slow"]
// bench_dict_insert!(V2, Splay, bst::splay::Splay::new());

bench_dict_insert!(V2, AVL, bst::avl::AVL::new());
bench_dict_insert!(V2, RB, bst::rb::RB::new());
bench_dict_insert!(V2, Treap, bst::treap::Treap::new());
bench_dict_insert!(V2, TreapImproved, bst::treap::Treap::new().improve_search());
bench_dict_insert!(V2, AA, bst::aa::AA::new());


bench_dict_insert!(_0__60, B, bt::bt::BT::<u64, u64, 60>::new());
bench_dict_insert!(_0__100, B, bt::bt::BT::<u64, u64, 100>::new());
bench_dict_insert!(_0__200, B, bt::bt::BT::<u64, u64, 200>::new());
bench_dict_insert!(_0__300, B, bt::bt::BT::<u64, u64, 300>::new());


bench_dict_insert!(_V1__60, BP, bt::bpt::BPT::<u64, u64, 60>::new());
bench_dict_insert!(_V1__105, BP, bt::bpt::BPT::<u64, u64, 105>::new());
bench_dict_insert!(_V1__100, BP, bt::bpt::BPT::<u64, u64, 100>::new());
bench_dict_insert!(_V1__200, BP, bt::bpt::BPT::<u64, u64, 200>::new());
bench_dict_insert!(_V1__300, BP, bt::bpt::BPT::<u64, u64, 300>::new());
bench_dict_insert!(_V1__95, BP, bt::bpt::BPT::<u64, u64, 95>::new());

bench_dict_insert!(_V2__20, BP, bt::bpt2::BPT2::<u64, u64, 20>::new());
bench_dict_insert!(_V2__30, BP, bt::bpt2::BPT2::<u64, u64, 30>::new());
bench_dict_insert!(_V2__60, BP, bt::bpt2::BPT2::<u64, u64, 60>::new());
bench_dict_insert!(_V2__105, BP, bt::bpt2::BPT2::<u64, u64, 105>::new());
bench_dict_insert!(_V2__100, BP, bt::bpt2::BPT2::<u64, u64, 100>::new());
bench_dict_insert!(_V2__200, BP, bt::bpt2::BPT2::<u64, u64, 200>::new());
bench_dict_insert!(_V2__300, BP, bt::bpt2::BPT2::<u64, u64, 300>::new());
bench_dict_insert!(_V2__95, BP, bt::bpt2::BPT2::<u64, u64, 95>::new());


////////////////////////////////////////////////////////////////////////////////
//// Nodes Distribution Bench

/// performance cross for u64 key:
///
/// | batch | M |
/// | -- | -- |
/// | 1150 | 105(main) |
/// | 2500 | 10(sub) | (at most 105)
///
const DISTRBUTE_BATCH: u64 = 2500;

#[bench]
fn bench_distribute_vec (b: &mut Bencher) {
    use std::hint::black_box;

    b.iter(|| {
        let mut v1 = Vec::with_capacity(DISTRBUTE_BATCH as usize);
        let mut v2 = Vec::with_capacity(DISTRBUTE_BATCH as usize);

        for i in 0..DISTRBUTE_BATCH {
            black_box(v1.push(i));
        }

        for _ in 0..DISTRBUTE_BATCH {
            black_box(v2.push(black_box(v1.remove(0))));
        }
    });
}

#[bench]
fn bench_distribute_bpt (b: &mut Bencher) {
    use std::hint::black_box;

    b.iter(|| {
        let mut v1 = bt::bpt::BPT::<u64, u64, 10>::new();
        let mut v2 = bt::bpt::BPT::<u64, u64, 10>::new();

        for i in 0..DISTRBUTE_BATCH {
            black_box(v1.push_back(i, i));
        }

        for _ in 0..DISTRBUTE_BATCH {
            let (k, v) = black_box(v1.pop_first().unwrap());
            black_box(v2.push_back(k, v));
        }
    });
}
