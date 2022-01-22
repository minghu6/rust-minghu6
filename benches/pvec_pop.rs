#![feature(test)]
#![feature(box_syntax)]
#![allow(dead_code)]


extern crate test;

use std::fmt::Debug;

use minghu6::test::*;
use minghu6::test::persistent::VectorProvider;
use minghu6::collections::persistent::vector::{ *, Vector };


use test::Bencher;

const BATCH_NUM: usize = 10_00;


#[bench]
#[allow(unused)]
fn bench_pvec_pop_prepare(b: &mut Bencher) {
    let provider = InodeProvider {};
    let mut batch = provider.prepare_batch(BATCH_NUM);
    let mut vec = (box trie::PTrieVec::empty()) as Box<dyn Vector<Inode>>;
    for e in batch.into_iter() {
        vec = vec.push(e);
    }

    b.iter( || {
        let vec = vec.duplicate();
        let total = vec.len();
    })
}


fn bench_vec_pop<'a, T: PartialEq + Debug + Clone>
(
    b: &mut Bencher,
    vec_new: fn() -> Box<(dyn Vector<'a, T>)>,
    provider: &(dyn VectorProvider<T>),
)
{
    let batch = provider.prepare_batch(BATCH_NUM);
    let mut vec = vec_new();
    for e in batch.into_iter() {
        vec = vec.push(e);
    }

    b.iter( || {
        let mut vec = vec.duplicate();
        let total = vec.len();

        for _ in 0..total {
            vec = vec.pop().unwrap();
        }

    })
}


fn bench_vec_tran_pop<'a, T: PartialEq + Debug + Clone>
(
    b: &mut Bencher,
    vec_new: fn() -> Box<(dyn Vector<'a, T>)>,
    provider: &(dyn VectorProvider<T>),
)
{
    let batch = provider.prepare_batch(BATCH_NUM);
    let mut vec = vec_new();
    for e in batch.into_iter() {
        vec = vec.push(e);
    }

    b.iter( || {

        let mut vec = vec.duplicate();
        let total = vec.len();

        // for _ in 0..total / 2 {
        //     vec = vec.pop().unwrap();
        // }

        let mut tvec = vec.transient().unwrap();

        for _ in 0..total / 2 {
            tvec = tvec.pop().unwrap();
        }

    })
}


#[bench]
fn bench_ptrie_vec_pop(b: &mut Bencher) {
    bench_vec_pop::<Inode>(b, || box trie::PTrieVec::empty(), &InodeProvider{})
}


#[bench]
fn bench_ttrie_vec_pop(b: &mut Bencher) {
    bench_vec_pop::<Inode>(b, || box trie::TTrieVec::empty(), &InodeProvider{})
}


#[bench]
fn bench_trie_tran_vec_pop(b: &mut Bencher) {
    bench_vec_tran_pop::<Inode>(b, || box trie::PTrieVec::empty(), &InodeProvider{})
}


#[bench]
fn bench_praw_vec_pop(b: &mut Bencher) {
    bench_vec_pop::<Inode>(b, || box raw::PRawVec::empty(), &InodeProvider{})
}


#[bench]
fn bench_traw_vec_pop(b: &mut Bencher) {
    bench_vec_pop::<Inode>(b, || box raw::TRawVec::empty(), &InodeProvider{})
}


#[bench]
fn bench_raw_tran_vec_pop(b: &mut Bencher) {
    bench_vec_tran_pop::<Inode>(b, || box raw::PRawVec::empty(), &InodeProvider{})
}