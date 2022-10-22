#![feature(box_syntax)]


use std::fmt::Debug;
use minghu6::{
    collections::persistent::vector::{trie, Vector},
    test::{
        *,
        persistent::VectorProvider,
    },
    elapsed,
    bench
};

const BATCH_NUM: usize = 7;


fn bench_vec_push<'a, T: PartialEq + Debug + Clone>
(
    label: &str,
    vec_new: fn() -> Box<(dyn Vector<'a, T>)>,
    provider: &(dyn VectorProvider<T>),
)
{
    let batch = provider.prepare_batch(BATCH_NUM);

    bench! {label,
        let mut vec = vec_new();
        let batch = batch.clone();

        // let mut i = 0;
        for e in batch.into_iter() {
            vec = vec.push(e);
        }

    };
}


fn bench_vec_tran_push<'a, T: PartialEq + Debug + Clone>
(
    label: &str,
    vec_new: fn() -> Box<(dyn Vector<'a, T>)>,
    provider: &(dyn VectorProvider<T>),
)
{
    let batch = provider.prepare_batch(BATCH_NUM);

    bench! {label,
        let mut vec = vec_new();
        let mut batch = batch.clone();

        for _ in 0..BATCH_NUM / 2 {
            vec = vec.push(batch.pop().unwrap());
        }

        let mut tvec = vec.transient().unwrap();

        for e in batch.into_iter() {
            tvec = tvec.push(e);
        }
    }
}


fn bench_vec_pop<'a, T: PartialEq + Debug + Clone>
(
    label: &str,
    vec_new: fn() -> Box<(dyn Vector<'a, T>)>,
    provider: &(dyn VectorProvider<T>),
)
{
    let batch = provider.prepare_batch(BATCH_NUM);
    let mut vec = vec_new();
    for e in batch.into_iter() {
        vec = vec.push(e);
    }

    bench! {label,
        let total = vec.len();
        let mut vec = vec.duplicate();

        // println!("vec push&pop: {:?}", vec);

        for _ in 0..total {
            vec = vec.pop().unwrap();
        }
    }
}



pub fn bench() {
    // bench_vec_push::<Inode>("bench ptrie_vec push", || box trie::PTrieVec::empty(), &InodeProvider{});
    // bench_vec_push::<Inode>("bench ttrie_vec push", || box trie::TTrieVec::empty(), &InodeProvider{});
    // bench_vec_tran_push::<Inode>("bench ptrie_vec transition push", || box trie::PTrieVec::empty(), &InodeProvider{});

    bench_vec_pop::<usize>("bench ttrie_vec push && pop", || box trie::TTrieVec::empty(), &UZProvider{});

}

