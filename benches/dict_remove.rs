#![feature(test)]
#![allow(dead_code)]
#![allow(unused_imports)]

use std::collections::HashMap;

use itertools::Itertools;
use minghu6::collections::DictKey;
use minghu6::collections::bt::bst::*;
use minghu6::collections::bt::*;
use minghu6::collections::Dictionary;
use minghu6::test::dict::*;
use minghu6::test::Provider;

extern crate test;
use rand::prelude::SliceRandom;
use rand::thread_rng;
use test::Bencher;

const BATCH_NUM: usize = 9000;


#[bench]
fn bench_dict_remove_prepare(b: &mut Bencher) {
    let provider = InodeProvider {};
    let mut rng = thread_rng();
    let batch = provider.prepare_batch(BATCH_NUM);
    let mut keys = batch.iter().map(|(k, _)| k.clone()).collect_vec();
    keys.shuffle(&mut rng);

    b.iter(|| {
    })
}


fn bench_remove<
    K: DictKey + Clone,
    V: GetKey<K> + Eq + Clone + std::fmt::Debug,
>(
    b: &mut Bencher,
    dict: &mut (dyn Dictionary<K, V>),
    provider: &(dyn DictProvider<K, V>),
) {
    let mut rng = thread_rng();
    let batch = provider.prepare_batch(BATCH_NUM);
    let mut keys = batch.iter().map(|(k, _)| k.clone()).collect_vec();

    for (k, v) in batch.into_iter() {
        dict.insert(k, v);
    }
    keys.shuffle(&mut rng);

    b.iter(|| {
        provider.bench_dict_remove(dict, &keys)
    })
}


#[bench]
fn bench_avl_remove(b: &mut Bencher) {
    bench_remove::<u32, Inode>(b, &mut avl::AVL::new(), &InodeProvider {})
}


#[bench]
fn bench_rawst_remove(b: &mut Bencher) {
    bench_remove::<u32, Inode>(b, &mut rawst::RawST::new(), &InodeProvider {})
}


#[bench]
fn bench_vecdict_remove(b: &mut Bencher) {
    bench_remove::<u32, Inode>(b, &mut Vec::new(), &InodeProvider {})
}


#[bench]
fn bench_hashmapdict_remove(b: &mut Bencher) {
    bench_remove::<u32, Inode>(b, &mut HashMap::new(), &InodeProvider {})
}

#[bench]
fn bench_b3_remove(b: &mut Bencher) {
    bench_remove::<u32, Inode>(b, &mut b3::B3::new(), &InodeProvider {})
}

#[bench]
fn bench_b4_remove(b: &mut Bencher) {
    bench_remove::<u32, Inode>(b, &mut b4::B4::new(), &InodeProvider {})
}

#[bench]
fn bench_bstar4_remove(b: &mut Bencher) {
    bench_remove::<u32, Inode>(b, &mut bstar4::BStar4::new(), &InodeProvider {})
}

#[bench]
fn bench_rb_remove(b: &mut Bencher) {
    bench_remove::<u32, Inode>(b, &mut rb::RB::new(), &InodeProvider {})
}

#[bench]
fn bench_aa_remove(b: &mut Bencher) {
    bench_remove::<u32, Inode>(b, &mut aa::AA::new(), &InodeProvider {})
}

