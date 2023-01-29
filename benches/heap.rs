#![feature(test)]
#![allow(dead_code)]
#![allow(unused_imports)]

extern crate test;

use std::collections::{BinaryHeap, HashMap, HashSet};

use rand::{prelude::SliceRandom, random, thread_rng};
use indexmap::IndexMap;

use minghu6::{
    collections::{
        heap::{
            dary::{DaryHeap, DaryHeap1, DaryHeap5},
            fib::FibHeap,
        },
        CollKey, Heap,
    },
    hashmap,
    etc::gen_unique, algs::random_range,
};

use test::Bencher;

const BATCH_NUM: usize = 2_000;




fn heap_prepare_basic() -> Vec<(usize, isize)> {
    let mut unique = gen_unique();

    (0..BATCH_NUM).map(|_| (unique(), random())).collect()
}


////////////////////////////////////////////////////////////////////////////////
//// Bench Heap Basic

#[bench]
fn bench_heap_basic_binaryheap(b: &mut Bencher) {
    let batch = heap_prepare_basic();

    b.iter(|| {
        let mut heap = BinaryHeap::new();
        let mut index = HashMap::new();
        for (_, v) in batch.iter() {
            heap.push(*v);
            index.insert(*v, *v);
        }

        while let Some(v) = heap.pop() {
            index.remove(&v);
        }
    })
}


#[bench]
fn bench_heap_basic_fibheap(b: &mut Bencher) {
    let batch = heap_prepare_basic();

    b.iter(|| {
        let mut heap = FibHeap::new();
        for (k, v) in batch.iter() {
            heap.push(*k, *v);
        }

        while heap.pop().is_some() {}
    })
}


#[bench]
fn bench_heap_basic_dary5heap(b: &mut Bencher) {
    let batch = heap_prepare_basic();

    b.iter(|| {
        let mut heap = DaryHeap5::new();
        for (k, v) in batch.iter() {
            heap.insert(*k, *v);
        }

        while heap.pop().is_some() {}
    })
}


#[bench]
fn bench_heap_basic_dary1heap(b: &mut Bencher) {
    let batch = heap_prepare_basic();

    b.iter(|| {
        let mut heap = DaryHeap1::new();
        for (k, v) in batch.iter() {
            heap.insert(*k, *v);
        }

        while heap.pop().is_some() {}
    })
}



////////////////////////////////////////////////////////////////////////////////
//// Bench Heap Advanced (classic with decrease-key and pop)

fn heap_prepare_classic() -> (Vec<(usize, usize)>, Vec<Vec<(usize, usize)>>) {
    let mut insert_map = Vec::new();
    let mut update_set = vec![];
    let mut auxheap = FibHeap::new();
    // let mut aux2heap = DaryHeap1::new();
    let mut get_unique_v = gen_unique();
    let mut unique_v_set = HashSet::new();

    for i in 0..BATCH_NUM {
        let v = get_unique_v();
        insert_map.push((i, v));
        auxheap.push(i, v);
        // aux2heap.push(i, v);
        unique_v_set.insert(v);
    }

    for _ in 0..BATCH_NUM {
        let update_num = (random::<usize>() % (BATCH_NUM / 10)) + 1;

        let indexset: HashSet<usize> = (0..update_num)
            .map(|_| {
                let candi: HashSet<&usize> = auxheap.indexes().collect();
                // let candi2: HashSet<&usize> = aux2heap.indexes().collect();
                // assert!(candi == candi2);

                let candi: Vec<&usize> = candi.into_iter().collect();

                **candi.choose(&mut rand::thread_rng()).unwrap()
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


lazy_static::lazy_static!{
    pub static ref CLASSIC_BNECH_SET: (Vec<(usize, usize)>, Vec<Vec<(usize, usize)>>) = {
        heap_prepare_classic()
    };
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
        let mut heap = DaryHeap::<8, usize, usize>::new();
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
        let mut heap = DaryHeap5::new();
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
        let mut heap = DaryHeap1::new();
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
