#![feature(is_sorted)]
#![feature(macro_metavar_expr)]


pub mod fib;
pub mod dary;

use std::{collections::BinaryHeap, cmp::Reverse};



/// Test heap push/pop
#[cfg(test)]
macro_rules! test_heap {
    ($heap:expr, $endian:ident) => {
        test_heap!($heap, $endian, push:push, pop:pop);
    };
    ($heap:expr, $endian:ident, push:$push:ident, pop:$pop:ident) => {
        let get_one = || common::random::<u64>();

        let non_dec = $crate::heap_endian_no_dec!($endian);

        for _ in 0..20 {
            /* Basic Test */

            let mut heap = $heap;
            let mut unique = common::gen_unique();

            let batch_num = 1000;

            for _ in 0..batch_num {
                let e = get_one();
                heap.push(unique(), e);
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
                if random::<usize>() % 2 == 0 {
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

            let mut refheap = $crate::union_heap!($endian);
            let mut testheap = $heap;

            for flag in seq {
                if flag {
                    let e = get_one();
                    let i = unique();

                    refheap.push(i, e.clone());
                    testheap.push(i, e);
                } else {
                    let target = refheap.pop();
                    assert_eq!(testheap.pop(), target);
                }
            }
        }

    }
}


/// Test advanced heap - with `update` method
#[cfg(test)]
macro_rules! test_heap_update {
    ($heap:expr, $endian:ident) => {
        test_heap_update!($heap, $endian, push:push, pop:pop, update:insert);
    };
    ($heap:expr, $endian:ident, push:$push:ident, pop:$pop:ident, update:$update:ident) => {
        let get_one = || common::random::<u64>();
        let non_dec = $crate::heap_endian_no_dec!($endian);

        macro_rules! validate_basic_heap {
            ($testheap:ident, $non_dec:ident) => {
                let mut storage = vec![];

                while let Some(e) = $testheap.pop() {
                    storage.push(e);
                }

                if !$non_dec {
                    storage.reverse();
                }

                let mut iter = storage.into_iter().enumerate();
                let mut prev = iter.next().unwrap().1;

                for (_i, e) in iter {
                    // println!("{i}: {:?}", e);
                    assert!(prev <= e, "prev: {prev:?}, e: {e:?}");
                    prev = e;
                }
            }
        }


        for _ in 0..400 {
            let batch_num = 400;

            let mut testheap = $heap;

            // pad 50% of batch
            for i in 0..batch_num / 2 {
                let e = get_one();
                testheap.push(i, e); // push
            }

            for _ in 0..batch_num / 2 {
                let newkey = get_one();
                let i = common::random_range!(0..testheap.len());

                testheap.$update(i, newkey.clone());
            }

            validate_basic_heap!(testheap, non_dec);
        }
    }
}


#[cfg(test)]
macro_rules! union_heap {
    (MAX) => {
        {
            $crate::MaxDictHeap::new()
        }
    };
    (MIN) => {
        {
            $crate::MinDictHeap::new()
        }
    }
}


#[cfg(test)]
macro_rules! heap_endian_no_dec {
    (MAX) => {
        false
    };
    (MIN) => {
        true
    };
}



#[cfg(test)]
pub(crate) use test_heap;
#[cfg(test)]
pub(crate) use test_heap_update;
#[cfg(test)]
pub(crate) use heap_endian_no_dec;
#[cfg(test)]
pub(crate) use union_heap;


pub struct MinHeap<T>(BinaryHeap<Reverse<T>>);


/// Fake Min Dict Heap
pub struct MinDictHeap<T> {
    inner: BinaryHeap<Reverse<T>>,
    // unique: Box<dyn FnMut() -> usize>
}


/// Fake Max Dict Heap
pub struct MaxDictHeap<T> {
    inner: BinaryHeap<T>,
    // unique: Box<dyn FnMut() -> usize>
}



impl<T: Ord> MinHeap<T> {
    pub fn new() -> Self {
        Self(BinaryHeap::new())
    }

    pub fn push(&mut self, v: T) {
        self.0.push(Reverse(v));
    }

    pub fn pop(&mut self) -> Option<T> {
        self.0.pop().map(|r|r.0)
    }
}


impl<T: Ord> MinDictHeap<T> {
    pub fn new() -> Self {
        Self {
            inner: BinaryHeap::new(),
        }
    }

    pub fn push<I>(&mut self, _i: I, v: T) {
        self.inner.push(Reverse(v));
    }

    pub fn pop(&mut self) -> Option<T> {
        self.inner.pop().map(|r|r.0)
    }
}


impl<T: Ord> MaxDictHeap<T> {
    pub fn new() -> Self {
        Self {
            inner: BinaryHeap::new(),
        }
    }

    pub fn push<I>(&mut self, _i: I, v: T) {
        self.inner.push(v);
    }

    pub fn pop(&mut self) -> Option<T> {
        self.inner.pop()
    }
}
