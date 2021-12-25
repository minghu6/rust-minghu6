#![feature(test)]
#![allow(dead_code)]
#![allow(unused_imports)]

use std::collections::HashMap;

use minghu6::collections::DictKey;
use minghu6::collections::Dictionary;
use minghu6::collections::bt::bst::*;
use minghu6::collections::bt::*;
use minghu6::test::dict::*;
use minghu6::test::Provider;

extern crate test;
use rand::prelude::SliceRandom;
use rand::thread_rng;
use test::Bencher;

const BATCH_NUM: usize = 5000;


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


fn bench_insert<
    K: DictKey + Clone,
    V: GetKey<K> + Eq + Clone + std::fmt::Debug,
>(
    b: &mut Bencher,
    dict: &mut (dyn Dictionary<K, V>),
    provider: &(dyn DictProvider<K, V>),
) {
    let mut batch = provider.prepare_batch(BATCH_NUM);
    let mut rng = thread_rng();

    b.iter( || {
        batch.shuffle(&mut rng);
        provider.bench_dict_insert(dict, batch.clone())
    })
}


#[bench]
fn bench_avl_insert(b: &mut Bencher) {
    bench_insert::<u32, Inode>(b, &mut avl::AVL::new(), &InodeProvider{})
}


#[bench]
fn bench_rawst_insert(b: &mut Bencher) {
    bench_insert::<u32, Inode>(b, &mut rawst::RawST::new(), &InodeProvider{})
}


#[bench]
fn bench_vecdict_insert(b: &mut Bencher) {
    bench_insert::<u32, Inode>(b, &mut Vec::new(), &InodeProvider{})
}


#[bench]
fn bench_hashmapdict_insert(b: &mut Bencher) {
    bench_insert::<u32, Inode>(b, &mut HashMap::new(), &InodeProvider{})
}

#[bench]
fn bench_b3_insert(b: &mut Bencher) {
    bench_insert::<u32, Inode>(b, &mut b3::B3::new(), &InodeProvider{})
}

#[bench]
fn bench_b4_insert(b: &mut Bencher) {
    bench_insert::<u32, Inode>(b, &mut b4::B4::new(), &InodeProvider{})
}

#[bench]
fn bench_bstar4_insert(b: &mut Bencher) {
    bench_insert::<u32, Inode>(b, &mut bstar4::BStar4::new(), &InodeProvider{})
}

