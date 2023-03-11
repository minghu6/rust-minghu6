//! BP2 M: 10-20


#![feature(test)]
use test::Bencher;

use lazy_static::lazy_static;

use m6_coll_st::{
    // bst,
    bt
};


extern crate test;

mod dict_common;

const BATCH_NUM: usize = 4_000;

enum OP {
    Insert(u64, u64),
    Remove(u64)
}
use OP::*;


lazy_static! {
    static ref PREPEND_RES: (Vec<OP>, Vec<u64>) = {
        use common::random;

        let get_one = || random::<u32>();

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
            if random::<u8>() % 2 == 0 {
                let k = get_one() as u64;
                let v = k + 500;

                aux.push(k);
                seq.push(OP::Insert(k, v));
                rems += 1;
            }
            else {
                let i = random::<usize>() % aux.len();
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
            q.push((random::<u32>() % aux.len() as u32) as u64);
        }

        (seq, q)
    };
}


macro_rules! bench_dict_get {
    ($v:ident, $name: ident, $dict: expr) => {
        bench_dict_get!($v, $name, $dict, i: insert, d: remove, q: get);
    };
    ($v:ident, $name: ident, $dict: expr, i: $i:ident, d: $d:ident, q: $q:ident) => {
        coll::paste! (
            #[allow(non_snake_case)]
            #[bench]
            fn [<bench_dict_get_ $v _ $name>] (b: &mut Bencher) {
                let mut dict = $dict;

                let (seq, q) = &*PREPEND_RES;

                for flag in seq.iter() {
                    match flag {
                        Insert(k, v) => {
                            dict.$i(*k, *v);
                        },
                        Remove(k) => {
                            dict.$d(k);
                        }
                    }
                }

                b.iter(|| {
                    for v in q.iter() {
                        dict.$q(&v);
                    }
                })
            }
        );
    };
}




// bench_dict_get!(V2, SG, bst::sg::SG::new(0.7));
// bench_dict_get!(V2, LSG, bst::lsg::LSG::new(0.7));
// bench_dict_get!(V2, AVL, bst::avl::AVL::new());
// bench_dict_get!(V2, RB, bst::rb::RB::new());
// bench_dict_get!(V2, Splay, bst::splay::Splay::new());
// bench_dict_get!(V2, Treap, bst::treap::Treap::new());
// bench_dict_get!(V2, TreapImproved, bst::treap::Treap::new().improve_search());
// bench_dict_get!(V2, RB, bst::rb::RB::new());
// bench_dict_get!(V2, AA, bst::aa::AA::new());

bench_dict_get!(_0__07, B, bt::bt::BT::<u64, u64, 7>::new());
bench_dict_get!(_0__11, B, bt::bt::BT::<u64, u64, 11>::new());
bench_dict_get!(_0__20, B, bt::bt::BT::<u64, u64, 20>::new());
bench_dict_get!(_0__30, B, bt::bt::BT::<u64, u64, 30>::new());
bench_dict_get!(_0__100, B, bt::bt::BT::<u64, u64, 100>::new());

bench_dict_get!(_V1__07, BP, bt::bpt::BPT::<u64, u64, 7>::new());
bench_dict_get!(_V1__11, BP, bt::bpt::BPT::<u64, u64, 11>::new());
bench_dict_get!(_V1__20, BP, bt::bpt::BPT::<u64, u64, 20>::new());
bench_dict_get!(_V1__30, BP, bt::bpt::BPT::<u64, u64, 30>::new());
bench_dict_get!(_V1__100, BP, bt::bpt::BPT::<u64, u64, 100>::new());

bench_dict_get!(_V2__07, BP, bt::bpt2::BPT2::<u64, u64, 7>::new());
bench_dict_get!(_V2__11, BP, bt::bpt2::BPT2::<u64, u64, 11>::new());
bench_dict_get!(_V2__20, BP, bt::bpt2::BPT2::<u64, u64, 20>::new());
bench_dict_get!(_V2__30, BP, bt::bpt2::BPT2::<u64, u64, 30>::new());
bench_dict_get!(_V2__100, BP, bt::bpt2::BPT2::<u64, u64, 100>::new());

bench_dict_get!(_0_, HASH_MAP, std::collections::HashMap::new());
