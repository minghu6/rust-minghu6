#![feature(test)]
#![feature(box_syntax)]

use lazy_static::lazy_static;
use minghu6::{
    collections::{
        bt::bst::*,
        bst2,
        Dictionary
    }
};

extern crate test;
use test::Bencher;

mod dict_common;


lazy_static! {
    static ref INSERT_DATA: Vec<(u64, u64)> = {
        let get_one = || rand::random::<u64>();

        gen_data!(get_one, 50, 200)
    };
}


macro_rules! bench_dict_insert {
    ($v:ident, $name: ident, $dict: expr) => {
        bench_dict_insert!($v, $name, $dict, i: insert);
    };
    ($v:ident, $name: ident, $dict: expr, i: $i:ident) => {
        concat_idents::concat_idents! (bench_name = bench_dict_insert_, $v, _, $name {
            #[allow(non_snake_case)]
            #[bench]
            fn bench_name (b: &mut Bencher) {
                b.iter(|| {
                    let mut dict = $dict;

                    for (k, v) in INSERT_DATA.iter().cloned() {
                        dict.$i(k, v);
                    }
                });
            }
        });
   };
}


bench_dict_insert!(_0_, HASH_MAP, std::collections::HashMap::new());


bench_dict_insert!(V1, AVL, avl::AVL::new());
bench_dict_insert!(V1, RAW, rawst::RawST::new());
bench_dict_insert!(V1, RB, rb::RB::new());
bench_dict_insert!(V1, LLRB, llrb::LLRB::new());
// bench_dict_insert!(B3_V1, bt::b3::B3::new());
// bench_dict_insert!(B4_V1, bt::b4::B4::new());
// bench_dict_insert!(B4_STAR_V1, bt::b4::B4::new());
bench_dict_insert!(V1, AA, aa::AA::new());
bench_dict_insert!(V1, TREAP, treap::Treap::new());
bench_dict_insert!(V1, SPLAY, splay::Splay::new());
bench_dict_insert!(V1, LSG, lsg::LSG::new());
bench_dict_insert!(V1, LSG_06, lsg::LSG::with_alpha(0.6));

// Too Slow
// bench_dict_insert!(V2, SG, bst2::sg::SG::new(0.7));
// bench_dict_insert!(V2, LSG, bst2::lsg::LSG::new(0.7));
bench_dict_insert!(V2, AVL, bst2::avl::AVL::new());
bench_dict_insert!(V2, RB, bst2::rb::RB::new());
bench_dict_insert!(V2, Splay, bst2::splay::Splay::new());
