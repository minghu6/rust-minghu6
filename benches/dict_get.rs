#![feature(test)]


use lazy_static::lazy_static;

use minghu6::collections::{
    bt::bst::*,
    bst2,
    Dictionary,
};


extern crate test;
use test::Bencher;

mod dict_common;

const BATCH_NUM: usize = 4_000;

enum OP {
    Insert(u64, u64),
    Remove(u64)
}
use OP::*;


lazy_static! {
    static ref PREPEND_RES: (Vec<OP>, Vec<u64>) = {
        let get_one = || rand::random::<u32>();

        let mut seq = vec![];
        let mut rems = 0;

        let mut aux = vec![];

        let group = 200;

        // pad 25% of batch
        for _ in 0..BATCH_NUM / group {
            let mut j = 0;
            let mut k = get_one() as u64;

            while j < group {
                let v = k + 500;

                aux.push(k);
                seq.push(OP::Insert(k, v));  // push

                rems += 1;
                k += 1;
                j += 1;
            }
        }

        for _ in 0..(1 * BATCH_NUM) / 20 {
            if rand::random::<u8>() % 2 == 0 {
                let k = get_one() as u64;
                let v = k + 500;

                aux.push(k);
                seq.push(OP::Insert(k, v));
                rems += 1;
            }
            else {
                let i = rand::random::<usize>() % aux.len();
                let popped = aux.swap_remove(i);

                seq.push(OP::Remove(popped));
                rems -= 1;
            }

            if rems == 0 {
                break;
            }
        }

        let mut q = vec![];

        for _i in 0..BATCH_NUM * 20 {
            q.push((rand::random::<u32>() % aux.len() as u32) as u64);
            // if i % 2 == 0 {
            //     let i = (rand::random::<u32>() % aux.len() as u32) as u64;
            //     q.push(i);
            // }
            // else {
            //     let i = rand::random::<u64>();
            //     q.push(i);
            // }
        }

        (seq, q)
    };
}


macro_rules! bench_dict_get {
    ($v:ident, $name: ident, $dict: expr) => {
        bench_dict_get!($v, $name, $dict, i: insert, d: remove, q: get);
    };
    ($v:ident, $name: ident, $dict: expr, i: $i:ident, d: $d:ident, q: $q:ident) => {
        concat_idents::concat_idents! (bench_name = bench_dict_get_, $v, _, $name {
            #[allow(non_snake_case)]
            #[bench]
            fn bench_name (b: &mut Bencher) {
                let mut dict = $dict;

                let (seq, q) = &*PREPEND_RES;

                for flag in seq.iter() {
                    match flag {
                        Insert(k, v) => {
                            dict.$i(k, v);
                        },
                        Remove(k) => {
                            dict.$d(&k);
                        }
                    }
                }

                b.iter(|| {
                    for v in q.iter() {
                        dict.$q(&v);
                    }
                })
            }
        });
    };
}


bench_dict_get!(_0_, HASH_MAP, std::collections::HashMap::new());


bench_dict_get!(V1, AVL, avl::AVL::new());
// bench_dict_get!(V1, RAW, rawst::RawST::new());
bench_dict_get!(V1, RB, rb::RB::new());
bench_dict_get!(V1, LLRB, llrb::LLRB::new());
bench_dict_get!(V1, AA, aa::AA::new());
bench_dict_get!(V1, TREAP, treap::Treap::new());
bench_dict_get!(V1, SPLAY, splay::Splay::new());
// bench_dict_get!(V1, LSG, lsg::LSG::new());
// bench_dict_get!(V1, LSG_06, lsg::LSG::with_alpha(0.6));


bench_dict_get!(V2, SG, bst2::sg::SG::new(0.7));
bench_dict_get!(V2, LSG, bst2::lsg::LSG::new(0.7));
bench_dict_get!(V2, AVL, bst2::avl::AVL::new());
// bench_dict_get!(V2, RB, bst2::rb::RB::new());
bench_dict_get!(V2, Splay, bst2::splay::Splay::new());
bench_dict_get!(V2, Treap, bst2::treap::Treap::new());
bench_dict_get!(V2, TreapImproved, bst2::treap::Treap::new().improve_search());
bench_dict_get!(V2, RB, bst2::rb::RB::new());
