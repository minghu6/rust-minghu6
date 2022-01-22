#![feature(test)]
#![feature(box_syntax)]
#![allow(dead_code)]


extern crate test;

use std::fmt::Debug;

use minghu6::test::*;
use minghu6::test::persistent::VectorProvider;
use minghu6::collections::persistent::vector::{ *, Vector };


use test::Bencher;

const BATCH_NUM: usize = 15_;


#[bench]
#[allow(unused)]
fn bench_pvec_push_prepare(b: &mut Bencher) {
    let provider = InodeProvider {};
    let mut batch = provider.prepare_batch(BATCH_NUM);

    b.iter( || {
        batch.clone();
    })
}


fn bench_vec_push<'a, T: PartialEq + Debug + Clone>
(
    b: &mut Bencher,
    vec_new: fn() -> Box<(dyn Vector<'a, T>)>,
    provider: &(dyn VectorProvider<T>),
)
{
    let batch = provider.prepare_batch(BATCH_NUM);

    b.iter( || {
        let mut vec = vec_new();
        let batch = batch.clone();

        // let mut i = 0;
        for e in batch.into_iter() {
            vec = vec.push(e);

            // println!("{}", i);
            // i += 1;
        }

    })
}


fn bench_vec_tran_push<'a, T: PartialEq + Debug + Clone>
(
    b: &mut Bencher,
    vec_new: fn() -> Box<(dyn Vector<'a, T>)>,
    provider: &(dyn VectorProvider<T>),
)
{
    let batch = provider.prepare_batch(BATCH_NUM);

    b.iter( || {
        let mut vec = vec_new();
        let mut batch = batch.clone();

        for _ in 0..BATCH_NUM / 2 {
            vec = vec.push(batch.pop().unwrap());
        }

        let mut tvec = vec.transient().unwrap();

        for e in batch.into_iter() {
            tvec = tvec.push(e);
        }
    })
}


#[bench]
fn bench_ptrie_vec_push(b: &mut Bencher) {
    bench_vec_push::<Inode>(b, || box trie::PTrieVec::empty(), &InodeProvider{})
}


#[bench]
fn bench_ttrie_vec_push(b: &mut Bencher) {
    bench_vec_push::<Inode>(b, || box trie::TTrieVec::empty(), &InodeProvider{})
}


#[bench]
fn bench_trie_tran_vec_push(b: &mut Bencher) {
    bench_vec_tran_push::<Inode>(b, || box trie::PTrieVec::empty(), &InodeProvider{})
}

