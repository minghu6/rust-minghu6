#![feature(test)]
#![feature(box_syntax)]

#![allow(dead_code)]
#![allow(unused_imports)]

use std::collections::HashMap;

use minghu6::collections::DictKey;
use minghu6::collections::Dictionary;
use minghu6::collections::bt::bst::*;
use minghu6::collections::bt::*;
use minghu6::test::Inode;
use minghu6::test::dict::*;
use minghu6::test::{ Provider, InodeProvider };

extern crate test;
use rand::prelude::SliceRandom;
use rand::thread_rng;
use test::Bencher;

const BATCH_NUM: usize = 10_000;


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
    dict_new: fn() -> Box<(dyn Dictionary<K, V>)>,
    provider: &(dyn DictProvider<K, V>),
) {
    let mut batch = provider.prepare_batch(BATCH_NUM);
    let mut rng = thread_rng();

    b.iter( || {
        batch.shuffle(&mut rng);
        provider.bench_dict_insert(dict_new, batch.clone())
    })
}


#[bench]
fn bench_avl_insert(b: &mut Bencher) {
    bench_insert::<u32, Inode>(b, || box avl::AVL::new(), &InodeProvider{})
}


#[bench]
fn bench_rawst_insert(b: &mut Bencher) {
    bench_insert::<u32, Inode>(b, || box rawst::RawST::new(), &InodeProvider{})
}


#[bench]
fn bench_vecdict_insert(b: &mut Bencher) {
    bench_insert::<u32, Inode>(b, || box Vec::new(), &InodeProvider{})
}


#[bench]
fn bench_hashmapdict_insert(b: &mut Bencher) {
    bench_insert::<u32, Inode>(b, || box HashMap::new(), &InodeProvider{})
}

#[bench]
fn bench_b3_insert(b: &mut Bencher) {
    bench_insert::<u32, Inode>(b, || box b3::B3::new(), &InodeProvider{})
}

#[bench]
fn bench_b4_insert(b: &mut Bencher) {
    bench_insert::<u32, Inode>(b, || box b4::B4::new(), &InodeProvider{})
}

#[bench]
fn bench_bstar4_insert(b: &mut Bencher) {
    bench_insert::<u32, Inode>(b, || box bstar4::BStar4::new(), &InodeProvider{})
}

#[bench]
fn bench_rb_insert(b: &mut Bencher) {
    bench_insert::<u32, Inode>(b, || box rb::RB::new(), &InodeProvider{})
}

#[bench]
fn bench_llrb_insert(b: &mut Bencher) {
    bench_insert::<u32, Inode>(b, || box llrb::LLRB::new(), &InodeProvider{})
}

#[bench]
fn bench_aa_insert(b: &mut Bencher) {
    bench_insert::<u32, Inode>(b, || box aa::AA::new(), &InodeProvider{})
}

#[bench]
fn bench_treap_insert(b: &mut Bencher) {
    bench_insert::<u32, Inode>(b, || box treap::Treap::new(), &InodeProvider{})
}

#[bench]
fn bench_splay_insert(b: &mut Bencher) {
    bench_insert::<u32, Inode>(b, || box splay::Splay::new(), &InodeProvider{})
}

#[bench]
fn bench_lsg_insert(b: &mut Bencher) {
    bench_insert::<u32, Inode>(b, || box lsg::LSG::new(), &InodeProvider{})
}

#[bench]
fn bench_lsg_06_insert(b: &mut Bencher) {
    bench_insert::<u32, Inode>(b, || box lsg::LSG::with_alpha(0.6), &InodeProvider{})
}

