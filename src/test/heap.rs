use super::{Provider, dict::{Inode, InodeProvider}};
use crate::collections::Heap;


pub trait GetWeight<T> {
    fn get_weight(&self) -> T;
}

pub trait HeapProvider<V: GetWeight<usize>>: Provider<V> {
    fn test_heap<'a>(&self, heap_new: fn() -> Box<(dyn Heap<usize, V>)>) {
        for _ in 0..20 {
            let mut heap = heap_new();

            let batch_num = 1000;

            // let mut weights = (0..batch_num).collect_vec();
            // weights.shuffle(&mut thread_rng());

            // Create
            let mut i = 0;
            while i < batch_num {
                let e = self.get_one();

                heap.insert(e.get_weight(), e);

                i += 1;
            }

            i = 0;
            let mut res = vec![];

            while i < batch_num {
                res.push(heap.pop_top().unwrap().get_weight());

                i += 1;
            }

            res.reverse();
            assert!(res.is_sorted())
        }
    }
}


impl GetWeight<usize> for Inode {
    fn get_weight(&self) -> usize {
        let uid_u32 = self.uid as u32;
        let gid_u32 = self.gid as u32;

        (((gid_u32 << 16) + uid_u32) % 1000_000).try_into().unwrap()
    }
}


impl HeapProvider<Inode> for InodeProvider {

}