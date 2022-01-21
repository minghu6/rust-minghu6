#![feature(test)]
#![feature(box_syntax)]
#![allow(dead_code)]


extern crate test;

use std::fmt::Debug;

use itertools::Itertools;
use minghu6::collections::persistent::vector::{ *, Vector };
use minghu6::test::dict::{InodeProvider, Inode};
use minghu6::test::persistent::VectorProvider;


use rand::prelude::SliceRandom;
use rand::thread_rng;
use test::Bencher;

const BATCH_NUM: usize = 10_000;


#[bench]
#[allow(unused)]
fn bench_pvec_assoc_prepare(b: &mut Bencher) {
    let provider = InodeProvider {};
    let mut rng = thread_rng();
    let mut batch = provider.prepare_batch(BATCH_NUM);

    b.iter( || {
        batch.shuffle(&mut rng);
        let new_batch = batch.clone();
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


#[bench]
fn bench_ptrie_vec_assoc(b: &mut Bencher) {
    bench_vec_assoc::<Inode>(b, || box trie::PTrieVec::empty(), &InodeProvider{})
}


#[bench]
fn bench_ttrie_vec_assoc(b: &mut Bencher) {
    bench_vec_assoc::<Inode>(b, || box trie::TTrieVec::empty(), &InodeProvider{})
}

