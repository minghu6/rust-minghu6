use std::{cmp::Reverse, collections::BinaryHeap};

use either::Either::{self, Left, Right};

use super::*;
use crate::{
    algs::random,
    collections::{AdvHeap, CollKey, Heap},
};


////////////////////////////////////////////////////////////////////////////////
//// Structure

pub struct UnionBinHeap<T> {
    inner: Either<BinaryHeap<T>, BinaryHeap<Reverse<T>>>,
}


////////////////////////////////////////////////////////////////////////////////
//// Trait

pub trait HeapProvider<V: CollKey + Clone>: Provider<V> {
    fn test_heap<'a>(
        &self,
        non_dec: bool,
        heap_new: fn() -> Box<(dyn Heap<V>)>,
    ) {
        for _ in 0..20 {
            /* Basic Test */

            let mut heap = heap_new();

            let batch_num = 1000;

            for _ in 0..batch_num {
                let e = self.get_one();
                heap.push(e);
            }

            let mut res = vec![];

            for _ in 0..batch_num {
                res.push(heap.pop().unwrap());
            }

            if !non_dec {
                res.reverse();
            }

            assert!(res.is_sorted());

            /* Accompany Test */

            // In-In Out-Out, generate in/out sequence
            let mut seq = vec![];
            let mut rems = 0;

            // pad 25% of batch
            for _ in 0..batch_num / 4 {
                seq.push(true); // push
                rems += 1;
            }

            for _ in 0..(3 * batch_num) / 4 {
                if random() % 2 == 0 {
                    seq.push(true);
                    rems += 1;
                } else {
                    seq.push(false);
                    rems -= 1;
                }

                if rems == 0 {
                    break;
                }
            }

            let mut refheap = UnionBinHeap::new(non_dec);
            let mut testheap = heap_new();

            for flag in seq {
                if flag {
                    let e = self.get_one();
                    refheap.push(e.clone());
                    testheap.push(e);
                } else {
                    let target = refheap.pop();
                    assert_eq!(testheap.pop(), target);
                }
            }
        }
    }
}


pub trait AdvHeapProvider<V: CollKey + Clone>: Provider<V> {
    fn test_advheap<'a>(
        &self,
        non_dec: bool,
        heap_new: fn() -> Box<(dyn AdvHeap<V>)>,
    ) {
        let batch_num = 1000;

        let mut refheap = UnionBinHeap::new(non_dec);
        let mut testheap = heap_new();

        // pad 25% of batch
        for _ in 0..batch_num / 2 {
            let e = self.get_one();
            refheap.push(e.clone());
            testheap.push(e); // push
        }

        for _ in 0..batch_num / 2 {
            let newkey = self.get_one();

            refheap.dkey(newkey.clone());
            testheap.dkey(newkey.clone());
        }

        while let Some(target) = refheap.pop() {
            assert_eq!(testheap.pop().unwrap(), target);
        }
    }
}


////////////////////////////////////////////////////////////////////////////////
//// Implmentation

impl<T, V> HeapProvider<V> for T
where
    T: Provider<V>,
    V: CollKey + Clone,
{
}

impl<T, V> AdvHeapProvider<V> for T
where
    T: HeapProvider<V>,
    V: CollKey + Clone,
{
}


impl<T: Ord> UnionBinHeap<T> {
    pub fn new(non_dec: bool) -> Self {
        if non_dec {
            Self {
                inner: Either::Right(BinaryHeap::new()),
            }
        } else {
            Self {
                inner: Either::Right(BinaryHeap::new()),
            }
        }
    }
}


impl<T: CollKey> Heap<T> for UnionBinHeap<T> {
    fn top(&self) -> Option<&T> {
        match &self.inner {
            Left(heap) => heap.peek(),
            Right(heap) => heap.peek().map(|rev| &rev.0),
        }
    }

    fn pop(&mut self) -> Option<T> {
        match &mut self.inner {
            Left(heap) => heap.pop(),
            Right(heap) => heap.pop().map(|rev| rev.0),
        }
    }

    fn push(&mut self, val: T) {
        match &mut self.inner {
            Left(heap) => heap.push(val),
            Right(heap) => heap.push(Reverse(val)),
        }
    }
}


impl<T: CollKey + Clone> AdvHeap<T> for UnionBinHeap<T> {
    fn dkey(&mut self, val: T) -> Option<T> {
        match &mut self.inner {
            Left(heap) => {
                if let Some(mut pm) = heap.peek_mut() {
                    let old = (*pm).clone();
                    *pm = val;
                    Some(old)
                } else {
                    None
                }
            }
            Right(heap) => {
                if let Some(mut pm) = heap.peek_mut() {
                    let old = (*pm).clone();
                    *pm = Reverse(val);
                    Some(old.0)
                } else {
                    None
                }
            }
        }
    }
}
