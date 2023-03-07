#![feature(test)]
#![feature(local_key_cell_methods)]

extern crate test;
use test::Bencher;

use lazy_static::lazy_static;
use rand::{prelude::SliceRandom, thread_rng};

use minghu6::collections::{bt::bst::*, Dictionary, bst, bt};


mod dict_common;


lazy_static! {
    static ref INSERT_DATA: Vec<(u64, u64)> = {
        let get_one = || rand::random::<u64>();

        gen_data!(get_one, 50, 1_000)
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


macro_rules! bench_dict_remove {
    ($v:ident, $name:ident, $dict:expr) => {
        bench_dict_remove!($v, $name, $dict, i: insert, r: remove);
    };
    ($v:ident, $name:ident, $dict:expr, i: $i:ident, r: $r:ident) => {
        paste::paste!(
            #[allow(non_snake_case)]
            #[bench]
            fn [<bench_dict_remove_ $v _ $name>] (b: &mut Bencher) {
                let mut dict = $dict;

                b.iter(|| {
                    for (k, v) in INSERT_DATA.iter().cloned() {
                        dict.$i(k, v);
                    }

                    for k in KEYS.iter() {
                        dict.$r(k);
                    }
                });
            }
        );
   };
}


bench_dict_remove!(_0_, HASH_MAP, std::collections::HashMap::new());


bench_dict_remove!(V1, AVL, avl::AVL::new());
// bench_dict_remove!(V1, RAW, rawst::RawST::new());
bench_dict_remove!(V1, RB, rb::RB::new());
bench_dict_remove!(V1, LLRB, llrb::LLRB::new());
bench_dict_remove!(V1, AA, aa::AA::new());
bench_dict_remove!(V1, TREAP, treap::Treap::new());
// bench_dict_remove!(V1, SPLAY, splay::Splay::new());
// bench_dict_remove!(V1, LSG, lsg::LSG::new());
// bench_dict_remove!(V1, LSG_06, lsg::LSG::with_alpha(0.6));

bench_dict_remove!(V2, AVL, bst::avl::AVL::new());
// bench_dict_remove!(V2, Splay, bst::splay::Splay::new());
bench_dict_remove!(V2, Treap, bst::treap::Treap::new());
// bench_dict_remove!(V2, TreapImproved, bst::treap::Treap::new().improve_search());
// /// Too Slow
// bench_dict_remove!(V2, LSG, bst::lsg::LSG::new(0.7));
// bench_dict_remove!(V2, SG, bst::sg::SG::new(0.7));
bench_dict_remove!(V2, RB, bst::rb::RB::new());
bench_dict_remove!(V2, AA, bst::aa::AA::new());
// bench_dict_remove!(_0__11, B, bt::bt::BT::<u64, u64, 11>::new());
// bench_dict_remove!(_0__20, B, bt::bt::BT::<u64, u64, 20>::new());
bench_dict_remove!(_0__60, B, bt::bt::BT::<u64, u64, 60>::new());
bench_dict_remove!(_0__100, B, bt::bt::BT::<u64, u64, 100>::new());
bench_dict_remove!(_0__300, B, bt::bt::BT::<u64, u64, 300>::new());
bench_dict_remove!(_0__500, B, bt::bt::BT::<u64, u64, 500>::new());

// bench_dict_remove!(_0__11, BP, bt::bpt::BPT::<u64, u64, 11>::new());
// bench_dict_remove!(_0__20, BP, bt::bpt::BPT::<u64, u64, 20>::new());
bench_dict_remove!(_0__60, BP, bt::bpt::BPT::<u64, u64, 60>::new());
bench_dict_remove!(_0__100, BP, bt::bpt::BPT::<u64, u64, 100>::new());
bench_dict_remove!(_0__300, BP, bt::bpt::BPT::<u64, u64, 300>::new());
// bench_dict_remove!(_0__400, BP, bt::bpt::BPT::<u64, u64, 400>::new());
bench_dict_remove!(_0__500, BP, bt::bpt::BPT::<u64, u64, 500>::new());


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
