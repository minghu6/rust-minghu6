#![feature(test)]
#![allow(dead_code)]
#![allow(unused_imports)]

use minghu6::collections::bst::*;
use minghu6::test::dict::*;
use minghu6::test::Provider;

extern crate test;
use rand::prelude::SliceRandom;
use rand::thread_rng;
use test::Bencher;

const BATCH_NUM: usize = 1000;


#[bench]
#[allow(unused)]
fn bench_dict_insert_prepare(b: &mut Bencher) {
    let provider = InodeProvider {};
    let mut rng = thread_rng();
    let mut batch = provider.prepare_batch(BATCH_NUM);

    b.iter( || {
        batch.shuffle(&mut rng);
        let new_batch = batch.clone();
    })
}


#[bench]
fn bench_avl_insert(b: &mut Bencher) {
    let provider = InodeProvider {};
    let mut dict =  avl::AVL::new();
    let mut batch = provider.prepare_batch(BATCH_NUM);
    let mut rng = thread_rng();

    b.iter( || {
        batch.shuffle(&mut rng);
        provider.bench_dict_insert(&mut dict, batch.clone())
    })
}

#[bench]
fn bench_vecdict_insert(b: &mut Bencher) {
    let provider = InodeProvider {};
    let mut dict =  Vec::new();
    let mut batch = provider.prepare_batch(BATCH_NUM);
    let mut rng = thread_rng();

    b.iter( || {
        batch.shuffle(&mut rng);
        provider.bench_dict_insert(&mut dict, batch.clone())
    })
}
