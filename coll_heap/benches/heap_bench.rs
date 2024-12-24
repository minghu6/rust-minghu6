#![feature(test)]

extern crate test;

use std::collections::{BinaryHeap, HashSet};

use common::*;

use m6_coll_heap::{
    dary,
    sdary,
    fib::FibHeap,
};

use test::Bencher;

lazy_static::lazy_static!{
    pub static ref BASIC_BNECH_DATA: Vec<(usize, isize)> = {
        let mut unique = gen_unique();
        (0..400_000).map(|_| (unique(), random())).collect()
    };

    pub static ref CLASSIC_BNECH_SET: (Vec<(usize, usize)>, Vec<Vec<(usize, usize)>>) = {
        heap_prepare_classic(2_000)
    };
}


macro_rules! bench_heap_basic {
    ($name:ident, $heap:expr, @$push_method:ident) => {
        coll::paste! (
            #[allow(non_snake_case)]
            #[bench]
            fn [<bench_heap_basic _ $name>] (b: &mut Bencher) {
                b.iter(|| {
                    let mut heap = $heap;

                    #[allow(unused)]
                    for (k, v) in BASIC_BNECH_DATA.iter() {
                        push!(@$push_method heap, k, v);
                    }

                    while heap.pop().is_some() {}
                })
            }
        );
    };
}

macro_rules! push {
    (@indexed_heap $heap:ident, $i:expr, $v:expr) => {
        $heap.insert($i, $v);
    };
    (@basic_heap $heap:ident, $i:expr, $v:expr) => {
        $heap.push($v);
    };
}

////////////////////////////////////////////////////////////////////////////////
//// Bench Heap Basic

bench_heap_basic!(SDaryHeap1, sdary::DaryHeap::<1, _>::new(), @basic_heap);

bench_heap_basic!(SDaryHeap2, sdary::DaryHeap::<2, _>::new(), @basic_heap);

bench_heap_basic!(SDaryHeap3, sdary::DaryHeap::<3, _>::new(), @basic_heap);

bench_heap_basic!(SDaryHeap4, sdary::DaryHeap::<4, _>::new(), @basic_heap);

bench_heap_basic!(SDaryHeap5, sdary::DaryHeap::<5, _>::new(), @basic_heap);

bench_heap_basic!(BinaryHeap, BinaryHeap::new(), @basic_heap);


////////////////////////////////////////////////////////////////////////////////
//// Bench Heap Advanced (classic with decrease-key and pop)

fn heap_prepare_classic(batch_num: usize) -> (Vec<(usize, usize)>, Vec<Vec<(usize, usize)>>) {
    let mut insert_map = Vec::new();
    let mut update_set = vec![];
    let mut auxheap = FibHeap::new();
    // let mut aux2heap = DaryHeap1::new();
    let mut get_unique_v = gen_unique();
    let mut unique_v_set = HashSet::new();

    for i in 0..batch_num {
        let v = get_unique_v();
        insert_map.push((i, v));
        auxheap.push(i, v);
        // aux2heap.push(i, v);
        unique_v_set.insert(v);
    }

    for _ in 0..batch_num {
        let update_num = (random::<usize>() % (batch_num / 10)) + 1;

        let indexset: HashSet<usize> = (0..update_num)
            .map(|_| {
                let candi: HashSet<&usize> = auxheap.indexes().collect();
                // let candi2: HashSet<&usize> = aux2heap.indexes().collect();
                // assert!(candi == candi2);

                let candi: Vec<&usize> = candi.into_iter().collect();

                **candi.choose(&mut common::thread_rng()).unwrap()
            })
            .collect();

        let index_dk = indexset
            .into_iter()
            // filter no unique weight (0)
            .filter_map(|i| {
                let upper = *auxheap.get(&i).unwrap();
                let neww;

                if upper == 0 {
                    neww = 0;
                }
                else {
                    neww = random::<usize>() % upper;
                }

                if unique_v_set.contains(&neww) {
                    return None;
                }

                let _old = auxheap.insert(i, neww);
                // let old2 = aux2heap.insert(i, neww);
                unique_v_set.insert(neww);

                // assert_eq!(old, old2);

                Some((i, neww))
            })
            .collect();

        update_set.push(index_dk);
        let (_, w) = auxheap.pop_item().unwrap();
        // aux2heap.pop_item().unwrap();
        unique_v_set.remove(&w);
    }

    // let update_n = update_set.iter().flatten().count();
    // println!("update_n: {update_n}");

    (insert_map, update_set)
}


#[bench]
fn bench_heap_classic_fibheap(b: &mut Bencher) {
    let (insert_batch, dk_batch) = &*CLASSIC_BNECH_SET;

    b.iter(|| {
        let mut heap = FibHeap::new();
        for (i, w) in insert_batch.iter().cloned() {
            heap.insert(i, w);
        }

        for dks in dk_batch.iter().cloned() {
            for (i, w) in dks {
                heap.decrease_key(i, w);
            }

            heap.pop();
        }
    })
}

#[bench]
fn bench_heap_classic_dary8heap(b: &mut Bencher) {
    let (insert_batch, dk_batch) = &*CLASSIC_BNECH_SET;

    b.iter(|| {
        let mut heap = dary::DaryHeap::<8, usize, usize>::new();
        for (i, w) in insert_batch.iter().cloned() {
            heap.insert(i, w);
        }

        for dks in dk_batch.iter().cloned() {
            for (i, w) in dks {
                heap.decrease_key(i, w);
            }

            heap.pop();
        }
    })
}

#[bench]
fn bench_heap_classic_dary5heap(b: &mut Bencher) {
    let (insert_batch, dk_batch) = &*CLASSIC_BNECH_SET;

    b.iter(|| {
        let mut heap = dary::DaryHeap5::new();
        for (i, w) in insert_batch.iter().cloned() {
            heap.insert(i, w);
        }

        for dks in dk_batch.iter().cloned() {
            for (i, w) in dks {
                heap.decrease_key(i, w);
            }

            heap.pop();
        }
    })
}

#[bench]
fn bench_heap_classic_dary1heap(b: &mut Bencher) {
    let (insert_batch, dk_batch) = &*CLASSIC_BNECH_SET;

    b.iter(|| {
        let mut heap = dary::DaryHeap1::new();
        for (i, w) in insert_batch.iter().cloned() {
            heap.insert(i, w);
        }

        for dks in dk_batch.iter().cloned() {
            for (i, w) in dks {
                heap.decrease_key(i, w);
            }

            heap.pop();
        }
    })
}
