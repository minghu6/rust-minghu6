#![feature(test)]
#![allow(dead_code)]
#![allow(unused_imports)]

use minghu6::collections::bst::*;
use minghu6::collections::bst::avl::Aavl;
use minghu6::test::dict::*;
use minghu6::test::Provider;

extern crate test;
use test::Bencher;

const BATCH_NUM: usize = 1000;



#[bench]
fn bench_aavl_insert_remove(b: &mut Bencher) {
    let provider = InodeProvider {};
    let mut adict =  Aavl::new();
    let batch = provider.prepare_batch(BATCH_NUM);

    b.iter( || {
        provider.bench_adict_insert_remove(&mut adict, &batch[..])
    })
}
