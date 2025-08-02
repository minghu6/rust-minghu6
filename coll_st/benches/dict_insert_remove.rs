#![feature(test)]

use lazy_static::lazy_static;
use rand::prelude::*;

#[allow(unused_imports)]
use m6_coll_st::{
    bst,
    bt
};


extern crate test;

use test::Bencher;
use std::hint::black_box;

mod dict_common;


lazy_static! {
    static ref INSERT_DATA: Vec<(u64, u64)> = {
        let get_one = || common::random::<u64>();

        gen_data!(get_one, 1000, 4000)
    };

    static ref KEYS: Vec<u64> = {
        let mut keys = vec![];

        for (k, _) in INSERT_DATA.iter() {
            keys.push(k.clone());
        }

        keys.shuffle(&mut thread_rng());

        keys
    };
}

macro_rules! bench_dict_insert {
    ($v:ident, $name: ident, $dict: expr ; $($tail:tt)*) => {
        bench_dict_insert!($v, $name, $dict, i: insert; $($tail)*);
    };
    ($v:ident, $name: ident, $dict: expr, i: $i:ident ; $($tail:tt)*) => {
        coll::paste!(
            #[allow(non_snake_case)]
            #[bench]
            fn [<bench_dict_insert_ $v _ $name>] (b: &mut Bencher) {
                b.iter(|| {
                    let mut dict = $dict;

                    for (k, v) in INSERT_DATA.iter().cloned() {
                        black_box(dict.$i(k, v));
                    }

                    // dict_introspection!(dict $($tail)*);
                });
            }
        );
   };
}

macro_rules! bench_dict_remove {
    ($v:ident, $name:ident, $dict:expr) => {
        bench_dict_remove!($v, $name, $dict, i: insert, r: remove);
    };
    ($v:ident, $name:ident, $dict:expr, i: $i:ident, r: $r:ident) => {
        coll::paste!(
            #[allow(non_snake_case)]
            #[bench]
            fn [<bench_dict_remove_ $v _ $name>] (b: &mut Bencher) {
                let mut dict = $dict;

                b.iter(|| {
                    for (k, v) in INSERT_DATA.iter().cloned() {
                        black_box(dict.$i(k, v));
                    }

                    for k in KEYS.iter() {
                        black_box(dict.$r(k));
                    }
                });
            }
        );
   };
}


////////////////////////////////////////////////////////////////////////////////
//// Insert Bench

bench_dict_insert!(_0_, HASH_MAP, std::collections::HashMap::new(); );


// #[ignore="Too Slow"]
// bench_dict_insert!(V2, SG, bst::sg::SG::new(0.7));

// #[ignore="Too Slow"]
// bench_dict_insert!(V2, LSG, bst::lsg::LSG::new(0.7));

// #[ignore="Too Slow"]
// bench_dict_insert!(V2, Splay, bst::splay::Splay::new());

// bench_dict_insert!(V2, AVL, bst::avl::AVL::new());
// bench_dict_insert!(V2, RB, bst::rb::RB::new());
// bench_dict_insert!(V2, Treap, bst::treap::Treap::new());
// bench_dict_insert!(V2, TreapImproved, bst::treap::Treap::new().improve_search());
// bench_dict_insert!(V2, AA, bst::aa::AA::new());


// bench_dict_insert!(_0__60, B, bt::bt::BT::<u64, u64, 60>::new());
// bench_dict_insert!(_0__100, B, bt::bt::BT::<u64, u64, 100>::new());
// bench_dict_insert!(_0__200, B, bt::bt::BT::<u64, u64, 200>::new());
// bench_dict_insert!(_0__300, B, bt::bt::BT::<u64, u64, 300>::new());


// bench_dict_insert!(_V1__60, BP, bt::bpt::BPT::<u64, u64, 60>::new());
// bench_dict_insert!(_V1__90, BP, bt::bpt::BPT::<u64, u64, 90>::new());

// bench_dict_insert!(_V2__20, BP, bt::bpt2::BPT2::<u64, u64, 20>::new());
// bench_dict_insert!(_V2__30, BP, bt::bpt2::BPT2::<u64, u64, 30>::new());
// bench_dict_insert!(_V2__60, BP, bt::bpt2::BPT2::<u64, u64, 60>::new());
// bench_dict_insert!(_V2__90, BP, bt::bpt2::BPT2::<u64, u64, 90>::new());

bench_dict_insert!(_0_, BTree_MAP, std::collections::BTreeMap::new(); );
// bench_dict_insert!(_V3__03, BP, bt::bpt3::BPT::<u64, u64, 3>::new(); @height);
// bench_dict_insert!(_V3__07, BP, bt::bpt3::BPT::<u64, u64, 7>::new(); @height);
// bench_dict_insert!(_V3__11, BP, bt::bpt3::BPT::<u64, u64, 11>::new(); @height);
// bench_dict_insert!(_V3__26, BP, bt::bpt3::BPT::<u64, u64, 26>::new(); @height);
bench_dict_insert!(_V3__32, BP, bt::bpt3::BPT::<u64, u64, 32>::new(); @height);
bench_dict_insert!(_V3__48, BP, bt::bpt3::BPT::<u64, u64, 48>::new(); @height);
// bench_dict_insert!(_V3__64, BP, bt::bpt3::BPT::<u64, u64, 64>::new(); @height);


// bench_dict_insert!(_0__07, FBPT, bt::flatbpt::FlatBPT::<u64, u64, 7>::new());
// bench_dict_insert!(_0__11, FBPT, bt::flatbpt::FlatBPT::<u64, u64, 11>::new());
// bench_dict_insert!(_0__20, FBPT, bt::flatbpt::FlatBPT::<u64, u64, 20>::new());
// bench_dict_insert!(_0__30, FBPT, bt::flatbpt::FlatBPT::<u64, u64, 30>::new());
// bench_dict_insert!(_0__90, FBPT, bt::flatbpt::FlatBPT::<u64, u64, 90>::new());

////////////////////////////////////////////////////////////////////////////////
//// Remove Bench

bench_dict_remove!(_0_, HASH_MAP, std::collections::HashMap::new());

// bench_dict_remove!(V2, AVL, bst::avl::AVL::new());
// bench_dict_remove!(V2, Splay, bst::splay::Splay::new());
// bench_dict_remove!(V2, Treap, bst::treap::Treap::new());
// bench_dict_remove!(V2, TreapImproved, bst::treap::Treap::new().improve_search());
// /// Too Slow
// bench_dict_remove!(V2, LSG, bst::lsg::LSG::new(0.7));
// bench_dict_remove!(V2, SG, bst::sg::SG::new(0.7));
// bench_dict_remove!(V2, RB, bst::rb::RB::new());
// bench_dict_remove!(V2, AA, bst::aa::AA::new());
// bench_dict_remove!(_0__11, B, bt::bt::BT::<u64, u64, 11>::new());
// // bench_dict_remove!(_0__20, B, bt::bt::BT::<u64, u64, 20>::new());
// bench_dict_remove!(_0__60, B, bt::bt::BT::<u64, u64, 60>::new());
// bench_dict_remove!(_0__100, B, bt::bt::BT::<u64, u64, 100>::new());

// bench_dict_remove!(_V1__60, BP, bt::bpt::BPT::<u64, u64, 60>::new());
// bench_dict_remove!(_V1__100, BP, bt::bpt::BPT::<u64, u64, 100>::new());
// bench_dict_remove!(_V1__300, BP, bt::bpt::BPT::<u64, u64, 300>::new());
// bench_dict_remove!(_V1__500, BP, bt::bpt::BPT::<u64, u64, 500>::new());

// bench_dict_remove!(_V2__60, BP, bt::bpt2::BPT2::<u64, u64, 60>::new());
// bench_dict_remove!(_V2__90, BP, bt::bpt2::BPT2::<u64, u64, 90>::new());

bench_dict_remove!(_0_, BTree_MAP, std::collections::BTreeMap::new());
// // Too Slow
// bench_dict_remove!(_V3__03, BP, bt::bpt3::BPT::<u64, u64, 3>::new());
// bench_dict_remove!(_V3__11, BP, bt::bpt3::BPT::<u64, u64, 11>::new());
// bench_dict_remove!(_V3__20, BP, bt::bpt3::BPT::<u64, u64, 20>::new());
// bench_dict_remove!(_V3__26, BP, bt::bpt3::BPT::<u64, u64, 26>::new());
bench_dict_remove!(_V3__32, BP, bt::bpt3::BPT::<u64, u64, 32>::new());
bench_dict_remove!(_V3__48, BP, bt::bpt3::BPT::<u64, u64, 48>::new());
// bench_dict_remove!(_V3__64, BP, bt::bpt3::BPT::<u64, u64, 64>::new());

// bench_dict_remove!(_0__07, FBPT, bt::flatbpt::FlatBPT::<u64, u64, 7>::new());
// bench_dict_remove!(_0__11, FBPT, bt::flatbpt::FlatBPT::<u64, u64, 11>::new());
// bench_dict_remove!(_0__20, FBPT, bt::flatbpt::FlatBPT::<u64, u64, 20>::new());
// bench_dict_remove!(_0__30, FBPT, bt::flatbpt::FlatBPT::<u64, u64, 30>::new());
// bench_dict_remove!(_0__90, FBPT, bt::flatbpt::FlatBPT::<u64, u64, 90>::new());


#[cfg(tprofile)]
#[test]
fn bench_bp_remove() {
    let mut dict = bt::bpt::BPT::<u64, u64, 100>::new();

    for _ in 0..5 {
        for (k, v) in INSERT_DATA.iter().cloned() {
            dict.insert(k, v);
        }

        for k in KEYS.iter() {
            dict.remove(k);
        }
    }

    use minghu6::{ etc::timeit::TPROFILE_STATS, get };

    let map = TPROFILE_STATS.take();

    println!("bpt_remove: {:?}", get!(map => "bpt_remove"));
    println!("ob_loss: {:?}", get!(map => "ob_loss"));
    println!("bpt_remove_stats_loss: {:?}", get!(map => "bpt_remove_root_stats"));
    println!("bpt_remove_search_internal: {:?}", get!(map => "bpt_remove_search_internal"));
    println!("bpt_remove_search_leaf: {:?}", get!(map => "bpt_remove_search_leaf"));
    // println!("bpt_remove_update_key: {:?}", get!(map => "bpt_remove_update_key"));
    // println!("bpt_remove_entry: {:?}", get!(map => "bpt_remove_entry"));
    // println!("bpt_remove_unpromote: {:?}", get!(map => "bpt_remove_unpromote"));

    TPROFILE_STATS.set(map);
}


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
