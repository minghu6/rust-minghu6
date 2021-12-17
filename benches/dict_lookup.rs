#![feature(test)]
#![allow(dead_code)]
#![allow(unused_imports)]

use std::collections::HashMap;

use itertools::Itertools;
use minghu6::collections::bt::bst::*;
use minghu6::collections::Dictionary;
use minghu6::test::dict::*;
use minghu6::test::Provider;

extern crate test;
use rand::prelude::SliceRandom;
use rand::thread_rng;
use test::Bencher;

const BATCH_NUM: usize = 1000;


#[bench]
fn bench_dict_lookup_prepare(b: &mut Bencher) {
    let provider = InodeProvider {};
    let mut rng = thread_rng();
    let batch = provider.prepare_batch(BATCH_NUM);
    let mut keys = batch.iter().map(|(k, _)| k.clone()).collect_vec();

    b.iter(|| {
        keys.shuffle(&mut rng);
    })
}

#[bench]
fn bench_avl_lookup(b: &mut Bencher) {
    let provider = InodeProvider {};
    let mut rng = thread_rng();
    let batch = provider.prepare_batch(BATCH_NUM);
    let mut keys = batch.iter().map(|(k, _)| k.clone()).collect_vec();

    let dict = &mut avl::AVL::new() as &mut (dyn Dictionary<u32, Inode>);

    for (k, v) in batch.into_iter() {
        dict.insert(k, v);
    }

    b.iter(|| {
        keys.shuffle(&mut rng);
        provider.bench_dict_lookup(dict, &keys)
    })
}


#[bench]
fn bench_vecdict_lookup(b: &mut Bencher) {
    let provider = InodeProvider {};
    let mut rng = thread_rng();
    let batch = provider.prepare_batch(BATCH_NUM);
    let mut keys = batch.iter().map(|(k, _)| k.clone()).collect_vec();

    let dict = &mut Vec::new() as &mut (dyn Dictionary<u32, Inode>);

    for (k, v) in batch.into_iter() {
        dict.insert(k, v);
    }

    b.iter(|| {
        keys.shuffle(&mut rng);
        provider.bench_dict_lookup(dict, &keys)
    })
}

#[bench]
fn bench_hashmapdict_lookup(b: &mut Bencher) {
    let provider = InodeProvider {};
    let mut rng = thread_rng();
    let batch = provider.prepare_batch(BATCH_NUM);
    let mut keys = batch.iter().map(|(k, _)| k.clone()).collect_vec();

    let dict = &mut HashMap::new() as &mut (dyn Dictionary<u32, Inode>);

    for (k, v) in batch.into_iter() {
        dict.insert(k, v);
    }

    b.iter(|| {
        keys.shuffle(&mut rng);
        provider.bench_dict_lookup(dict, &keys)
    })
}
