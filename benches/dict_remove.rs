#![feature(test)]


use lazy_static::lazy_static;
use minghu6::collections::{bt::bst::*, Dictionary, bst2};

extern crate test;
use rand::{prelude::SliceRandom, thread_rng};
use test::Bencher;


mod dict_common;


lazy_static! {
    static ref INSERT_DATA: Vec<(u64, u64)> = {
        let get_one = || rand::random::<u64>();

        gen_data!(get_one, 50, 200)
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
        concat_idents::concat_idents! (bench_name = bench_dict_remove_, $v, _, $name {
            #[allow(non_snake_case)]
            #[bench]
            fn bench_name (b: &mut Bencher) {
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
        });
   };
}


bench_dict_remove!(_0_, HASH_MAP, std::collections::HashMap::new());


bench_dict_remove!(V1, AVL, avl::AVL::new());
// bench_dict_remove!(V1, RAW, rawst::RawST::new());
bench_dict_remove!(V1, AA, aa::AA::new());
bench_dict_remove!(V1, TREAP, treap::Treap::new());
bench_dict_remove!(V1, SPLAY, splay::Splay::new());
// bench_dict_remove!(V1, LSG, lsg::LSG::new());
// bench_dict_remove!(V1, LSG_06, lsg::LSG::with_alpha(0.6));

// /// Too Slow
// bench_dict_remove!(V2, LSG, bst2::lsg::LSG::new(0.7));
