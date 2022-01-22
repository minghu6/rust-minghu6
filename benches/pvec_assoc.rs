#![feature(test)]
#![feature(box_syntax)]
#![allow(dead_code)]


extern crate test;

use std::fmt::Debug;

use itertools::Itertools;
use minghu6::collections::persistent::vector::{ *, Vector };
use minghu6::test::*;
use minghu6::test::persistent::VectorProvider;


use rand::prelude::SliceRandom;
use rand::thread_rng;
use test::Bencher;

const BATCH_NUM: usize = 10_00;


#[bench]
#[allow(unused)]
fn bench_pvec_assoc_prepare(b: &mut Bencher) {
    let provider = InodeProvider {};
    let mut rng = thread_rng();
    let mut batch_0 = provider.prepare_batch(BATCH_NUM / 2);
    let mut batch_1 = provider.prepare_batch(BATCH_NUM / 2);

    b.iter( || {

        batch_0.shuffle(&mut rng);
        let batch_0 = batch_0
            .clone()
            .into_iter()
            .enumerate()
            .collect_vec();

        batch_1.shuffle(&mut rng);
        let batch_1 = batch_1
            .clone()
            .into_iter()
            .enumerate()
            .collect_vec();

    })
}


fn bench_vec_assoc<'a, T: PartialEq + Debug + Clone>
(
    b: &mut Bencher,
    vec_new: fn() -> Box<(dyn Vector<'a, T>)>,
    provider: &(dyn VectorProvider<T>),
)
{
    let mut batch = provider.prepare_batch(BATCH_NUM);
    let mut vec = vec_new();

    for e in batch.iter() {
        vec = vec.push(e.clone());
    }

    let mut rng = thread_rng();

    b.iter( || {
        batch.shuffle(&mut rng);
        let batch = batch.clone().into_iter().enumerate().collect_vec();

        for (idx, e) in batch.into_iter() {
            vec = vec.assoc(idx, e);
        }

    })
}


fn bench_vec_tran_assoc<'a, T: PartialEq + Debug + Clone>
(
    b: &mut Bencher,
    vec_new: fn() -> Box<(dyn Vector<'a, T>)>,
    provider: &(dyn VectorProvider<T>),
)
{
    let mut batch_0 = provider.prepare_batch(BATCH_NUM / 2);
    let mut batch_1 = provider.prepare_batch(BATCH_NUM / 2);

    let mut vec = vec_new();

    for e in batch_0.iter().chain(batch_1.iter()) {
        vec = vec.push(e.clone());
    }

    let mut rng = thread_rng();

    b.iter( || {
        batch_0.shuffle(&mut rng);
        let batch_0 = batch_0
            .clone()
            .into_iter()
            .enumerate()
            .collect_vec();

        batch_1.shuffle(&mut rng);
        let batch_1 = batch_1
            .clone()
            .into_iter()
            .enumerate()
            .collect_vec();


        for (idx, e) in batch_0.into_iter() {
            vec = vec.assoc(idx, e);
        }

        let mut vec = vec.transient().unwrap();

        for (idx, e) in batch_1.into_iter() {
            vec = vec.assoc(idx, e);
        }


    })
}


#[bench]
fn bench_ptrie_vec_assoc(b: &mut Bencher) {
    bench_vec_assoc::<Inode>(b, || box trie::PTrieVec::empty(), &InodeProvider{})
}


#[bench]
fn bench_ttrie_vec_assoc(b: &mut Bencher) {
    bench_vec_assoc::<Inode>(b, || box trie::TTrieVec::empty(), &InodeProvider{})
}


#[bench]
fn bench_trie_tran_vec_assoc(b: &mut Bencher) {
    bench_vec_tran_assoc::<Inode>(b, || box trie::PTrieVec::empty(), &InodeProvider{})
}




#[bench]
fn bench_praw_vec_assoc(b: &mut Bencher) {
    bench_vec_assoc::<Inode>(b, || box raw::PRawVec::empty(), &InodeProvider{})
}


#[bench]
fn bench_traw_vec_assoc(b: &mut Bencher) {
    bench_vec_assoc::<Inode>(b, || box raw::TRawVec::empty(), &InodeProvider{})
}


#[bench]
fn bench_raw_tran_vec_assoc(b: &mut Bencher) {
    bench_vec_tran_assoc::<Inode>(b, || box raw::PRawVec::empty(), &InodeProvider{})
}
