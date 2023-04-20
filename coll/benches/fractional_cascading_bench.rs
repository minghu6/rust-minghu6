#![feature(test)]
#![feature(closure_lifetime_binder)]

extern crate test;

use std::{ops::Range, hint::black_box};
use m6_coll::frac_casc::FractionalCascading;
use test::Bencher;

use lazy_static::lazy_static;

use m6_coll::paste;
use common::random_range;


const K: usize = 3200;
const N: usize = 6_250;

const V_RANGE: Range<usize> = 0..10_00;
const Q_LEN: usize = 1000;


macro_rules! sample_as_ref {
    ($sample:expr) => {
        {
            let mut sample2 = vec![];

            for sub in $sample.iter() {
                sample2.push(&sub[..]);
            }

            sample2
        }
    };
}


lazy_static! {
    static ref K_ARR: Vec<Vec<usize>> = {
        let mut arr = Vec::with_capacity(K);

        for _ in 0..K {
            let mut sub = Vec::with_capacity(N);

            for _ in 0..N {
                sub.push(random_range!(V_RANGE));
            }

            sub.sort();

            arr.push(sub);
        }

        arr
    };

    static ref K_ARR_UNIQUE: Vec<Vec<usize>> = {
        let mut arr = Vec::with_capacity(K);

        for _ in 0..K {
            let mut sub = Vec::with_capacity(N);

            for _ in 0..N {
                sub.push(random_range!(V_RANGE));
            }

            sub.sort();
            sub.dedup();

            arr.push(sub);
        }

        arr
    };

    static ref K_ARR_REF: Vec<&'static [usize]> = {
        sample_as_ref!(K_ARR)
    };

    static ref K_ARR_UNIQUE_REF: Vec<&'static [usize]> = {
        sample_as_ref!(K_ARR)
    };


    static ref QS: Vec<usize> = {
        (0..Q_LEN).map(|_| random_range!(V_RANGE)).collect()
    };
}



macro_rules! bench_k_binary_search {
    (raw|$name:ident, $data:ident) => {
        paste! (
            #[bench]
            fn [<bench_k_binary_search_ $name>] (b: &mut Bencher) {
                b.iter(|| {
                    let mut expect = vec![];

                    for q in QS.iter() {
                        for sub in $data.iter() {
                            black_box(expect.push(sub.binary_search(&q)));
                        }
                    }

                    black_box(expect);
                })
            }
        );
    };
    (fc|$name:ident, $data:ident, $frac:literal) => {
        paste! (
            #[bench]
            fn [<bench_k_binary_search_ $name>] (b: &mut Bencher) {
                let fc = FractionalCascading::<_, $frac>::new(&$data);

                b.iter(|| {
                    for q in QS.iter() {
                        black_box(fc.find(q));
                    }
                })
            }
        );
    };
    (fc-q|$name:ident, $data:ident, $frac:literal) => {
        paste! (
            #[bench]
            fn [<bench_k_binary_search_ $name>] (b: &mut Bencher) {
                let fc = FractionalCascading::<_, $frac>::new(&$data);

                b.iter(|| {
                    for q in QS.iter() {
                        black_box(fc.quick_find(q));
                    }
                })
            }
        );
    };
}


// equal vs bisect down
// dup vs unique

bench_k_binary_search!(raw|dup_raw, K_ARR_REF);
bench_k_binary_search!(fc|dup_fc1, K_ARR_REF, 1);
bench_k_binary_search!(fc|dup_fc2, K_ARR_REF, 2);
bench_k_binary_search!(fc|dup_fc3, K_ARR_REF, 3);
bench_k_binary_search!(fc|dup_fc4, K_ARR_REF, 4);
// bench_k_binary_search!(fc|dup_f6, K_ARR_REF, 6);



bench_k_binary_search!(raw|unique_raw, K_ARR_UNIQUE_REF);
bench_k_binary_search!(fc|unique_fc1, K_ARR_UNIQUE_REF, 1);
bench_k_binary_search!(fc|unique_fc2, K_ARR_UNIQUE_REF, 2);
bench_k_binary_search!(fc|unique_fc4, K_ARR_UNIQUE_REF, 4);
