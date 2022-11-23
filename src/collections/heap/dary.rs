//! D-ary in place heap
//!
//!

use std::{
    cmp::{min, Ordering::*},
    collections::HashMap,
    hash::Hash, borrow::Borrow,
};

use m6coll::Array;

use crate::collections::{AdvHeap, Coll, CollKey, Heap};


////////////////////////////////////////////////////////////////////////////////
//// Macro

macro_rules! normalize_cap {
    ($raw_cap:expr) => {{
        let raw_cap = $raw_cap;
        // Note!! '!' is bitnot = C '~'
        // raw_cap >> E + if raw_cap & !(!0 << E) > 0 { 1 } else { 0 }

        // quick mod
        let e = 4;
        let extra = if raw_cap & !(!0 << e) > 0 { 1 } else { 0 };

        (raw_cap >> e + extra) << e
    }};
}

macro_rules! base {
    () => {{
        1 << E
    }};
}

macro_rules! basen {
    ($n:expr) => {{
        1 << E * ($n)
    }};
}

/// get level from no. of node. (no. = idx + 1)
macro_rules! pos {
    ($n:expr) => {{
        let n = $n;

        // = q^n
        let var = (base!() - 1) * n + 1;

        let mut ln = var.ilog2() as usize / E;

        let mut col: usize = n - total!(ln);

        // println!("n: {n}, ln: {ln}, total(ln): {}", total!(ln));

        if col > 0 {
            ln += 1;
        } else {
            col = basen!(ln - 1);
        }

        (ln, col)
    }};
}

/// total number for lv complete d-ary
macro_rules! total {
    ($ln:expr) => {{
        let ln = $ln;

        debug_assert!(ln > 0);
        debug_assert!(E > 0);

        // 1 * 1-q^n / (1-q)
        (basen!(ln) - 1) / (base!() - 1)
    }};
    ($ln:expr, $col:expr) => {{
        let ln = $ln;
        let col = $col;

        debug_assert!(ln > 0);

        if ln == 1 {
            col
        } else {
            total!(ln - 1) + col
        }
    }};
}

/// paren's idx
macro_rules! paren_pos {
    ($n:expr) => {{
        let n = $n;

        let (ln, col) = pos!(n);
        debug_assert!(ln > 1);

        (ln - 1, (col - 1) / base!() + 1)
    }};
}

/// by idx to idx
macro_rules! paren {
    ($idx:expr) => {{
        let (ln, col) = paren_pos!($idx + 1);

        total!(ln, col) - 1
    }};
}

/// first child idx
macro_rules! child_pos {
    ($n:expr) => {{
        let n = $n;

        let (ln, col) = pos!(n);

        (ln + 1, (col - 1) * base!() + 1)
    }};
}

macro_rules! child {
    ($idx:expr) => {{
        let (ln, col) = child_pos!($idx + 1);

        total!(ln, col) - 1
    }};
}



////////////////////////////////////////////////////////////////////////////////
//// Structure

/// Min Heap, I is unique, T is weight
#[derive(Clone)]
pub struct DaryHeap<const E: usize, I, T> {
    len: usize,
    index: HashMap<I, usize>,
    raw: Array<Option<(I, T)>>,
}

pub type DaryHeap1<I, T> = DaryHeap<1, I, T>;

pub type DaryHeap5<I, T> = DaryHeap<5, I, T>;


////////////////////////////////////////////////////////////////////////////////
//// Implementation


/// Basic Implementation
impl<const E: usize, I, T> DaryHeap<E, I, T> {
    pub fn cap(&self) -> usize {
        self.raw.len()
    }

    pub fn len(&self) -> usize {
        self.len
    }

    fn w(&self, idx: usize) -> &T {
        &self.raw[idx].as_ref().unwrap().1
    }
}


/// New and Init Implementation
impl<const E: usize, I: Clone, T: Clone> DaryHeap<E, I, T> {
    pub fn new() -> Self {
        Self::with_capacity(E)
    }

    /// Truely entry-point
    pub fn with_capacity(cap: usize) -> Self {
        debug_assert!(E > 0);

        Self {
            len: 0,
            index: HashMap::with_capacity(cap),
            raw: Array::new_with_clone(None, normalize_cap!(cap)),
        }
    }

    pub fn top_item(&self) -> Option<&(I, T)> {
        if self.len > 0 {
            Some(&self.raw[0].as_ref().unwrap())
        } else {
            None
        }
    }

    pub fn top(&self) -> Option<&T> {
        self.top_item().map(|x| &(*x).1)
    }
}


/// Main Algorithm Implementation
impl<const E: usize, I: CollKey + Hash + Clone, T: CollKey + Clone>
    DaryHeap<E, I, T>
{
    ////////////////////////////////////////////////////////////////////////////
    //// Public method

    pub fn insert(&mut self, i: I, v: T) -> Option<T> {
        if self.index.contains_key(&i) {
            Some(self.update(i, v))
        } else {
            if self.cap() == 0 {
                self.recap(E);
            } else if self.len >= self.cap() {
                self.recap(self.cap() << 1);
            }

            let ent = self.len;

            self.raw[ent] = Some((i.clone(), v));
            self.index.insert(i, ent);

            self.sift_up(ent);

            self.len += 1;

            None
        }
    }

    pub fn pop_item(&mut self) -> Option<(I, T)> {
        if self.len == 0 {
            return None;
        }

        if self.len == 1 {
            Some(self.remove(0))
        } else {
            self.swap(0, self.len - 1);

            let (i, v) = self.remove(self.len - 1);

            self.sift_down(0);

            Some((i, v))
        }
    }

    pub fn get<Q>(&self, i: &Q) -> Option<&T>
    where
        I: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.index.get(i).map(|&idx| self.w(idx))
    }


    pub fn indexes(&self) -> impl Iterator<Item=&I> {
        self.index.keys()
    }


    ////////////////////////////////////////////////////////////////////////////
    //// Public method alias

    pub fn pop(&mut self) -> Option<T> {
        self.pop_item().map(|x| x.1)
    }

    #[inline]
    pub fn decrease_key(&mut self, i: I, v: T) -> T {
        self.update(i, v)
    }


    ////////////////////////////////////////////////////////////////////////////
    //// Assistant method

    fn update(&mut self, i: I, v: T) -> T {
        debug_assert!(self.index.contains_key(&i), "No {i:?}");

        let idx = self.index.remove(&i).unwrap();

        let (_, oldv) = self.raw[idx].replace((i.clone(), v)).unwrap();

        let newidx = match self.w(idx).cmp(&oldv) {
            Less => self.sift_up(idx),
            Equal => idx,
            Greater => self.sift_down(idx),
        };

        self.index.insert(i, newidx);

        oldv
    }

    fn recap(&mut self, new_cap: usize) {
        self.raw.resize(new_cap);
    }

    /// return insert_idx
    fn sift_up(&mut self, idx: usize) -> usize {
        let mut cur = idx;

        while cur != 0 {
            let paren = paren!(cur);

            if self.w(cur) < self.w(paren) {
                self.swap(cur, paren);
                cur = paren;
            } else {
                break;
            }
        }

        cur
    }

    /// return insert_idx
    fn sift_down(&mut self, idx: usize) -> usize {
        let mut cur = idx;

        loop {
            if let Some(child) = self.min_child(cur) {
                if self.w(child) < self.w(cur) {
                    self.swap(cur, child);
                    cur = child;
                    continue;
                }
            }

            break cur;
        }
    }

    fn swap(&mut self, idx1: usize, idx2: usize) {
        let (k1, _v1) = self.raw[idx1].as_ref().unwrap();
        let (k2, _v2) = self.raw[idx2].as_ref().unwrap();

        self.index.insert(k1.clone(), idx2);
        self.index.insert(k2.clone(), idx1);

        let tmp1 = self.raw[idx1].take();
        self.raw[idx1] = self.raw[idx2].take();
        self.raw[idx2] = tmp1;
    }

    fn min_child(&self, idx: usize) -> Option<usize> {
        let start = child!(idx);
        let end = min(self.len, start + base!());

        (start..end).min_by_key(|&x| self.w(x))
    }

    /// remain None, update self.len
    fn remove(&mut self, idx: usize) -> (I, T) {
        let (i, v) = self.raw[idx].take().unwrap();
        self.index.remove(&i);
        self.len -= 1;

        (i, v)
    }
}


impl<const E: usize, I, T> Coll for DaryHeap<E, I, T> {
    fn len(&self) -> usize {
        self.len
    }
}


impl<const E: usize, I: CollKey + Hash + Clone, T: CollKey + Clone> Heap<I, T>
    for DaryHeap<E, I, T>
{
    fn top(&self) -> Option<&T> {
        self.top()
    }

    fn pop(&mut self) -> Option<T> {
        self.pop()
    }

    fn push(&mut self, key: I, val: T) {
        self.insert(key, val);
    }
}


impl<const E: usize, I: CollKey + Hash + Clone, T: CollKey + Clone>
    AdvHeap<I, T> for DaryHeap<E, I, T>
{
    fn update(&mut self, index: I, val: T) -> Option<T> {
        self.insert(index, val)
    }
}




#[cfg(test)]
mod tests {
    use crate::{
        collections::{
            Coll,
            heap::{dary::DaryHeap, fib::FibHeap}
        },
        test::{
            gen,
            heap::{AdvHeapProvider, HeapProvider},
            UZProvider, gen_unique,
        }, algs::random,
    };

    use super::DaryHeap1;


    #[test]
    fn test_daryheap_fixeddata() {
        // use std::collections::BinaryHeap;
        // let h = BinaryHeap::<usize>::new();

        let mut auto = gen();
        let mut heap = DaryHeap::<1, usize, usize>::new();

        heap.insert(auto(), 2);
        heap.insert(auto(), 4);
        heap.insert(auto(), 1);

        assert_eq!(heap.pop().unwrap(), 1);
        assert_eq!(heap.pop().unwrap(), 2);
        assert_eq!(heap.pop().unwrap(), 4);
        assert_eq!(heap.pop(), None);

        // println!("heap top {:?}", heap.top_item())
    }

    #[test]
    fn test_daryheap_randomdata() {
        fn do_test<const E: usize>() {
            let provider = UZProvider {};

            (&provider as &dyn HeapProvider<usize>)
                .test_heap(true, || box DaryHeap::<E, usize, usize>::new());

            (&provider as &dyn AdvHeapProvider<usize>)
                .test_advheap(true, || box DaryHeap::<E, usize, usize>::new());
        }

        do_test::<1>();
        do_test::<2>();
        do_test::<3>();
        do_test::<4>();
        do_test::<5>();
        do_test::<6>();
        do_test::<7>();
        do_test::<8>();
    }

    #[test]
    fn test_daryheap_randomdata_extra() {

        fn do_test<const E: usize>() {
            let validate = |heap: &DaryHeap<E, i32, usize>, non_dec: bool| {
                let mut heap = (*heap).clone();
                let mut storage = vec![];

                while let Some(e) = heap.pop() {
                    storage.push(e);
                }

                if !non_dec {
                    storage.reverse();
                }

                // println!("storage: {storage:?}");

                let mut iter = storage.into_iter().enumerate();
                let mut prev = iter.next().unwrap().1;

                for (_i, e) in iter {
                    // println!("{i}: {:?}", e);
                    assert!(prev <= e, "prev: {prev:?}, e: {e:?}");
                    prev = e;
                }
            };

            // let batch_num = 10;

            let non_dec = true;
            let get_one = || random::<usize>() % 1000;


            for _ in 0..1 {
                let batch_num = 100 * 3;

                let mut heap = DaryHeap::<E, i32, usize>::new();

                // pad 50% of batch
                for i in 0..batch_num {
                    let e = get_one();
                    heap.insert(i, e); // push
                }

                for _ in 0..batch_num / 3 {
                    let newkey = get_one();
                    let i = random::<usize>() % heap.len;
                    // println!("update: i:{i}, w:{newkey}");
                    heap.update(i as i32, newkey.clone());

                    validate(&heap, non_dec);
                }
            }
        }


        do_test::<1>();

    }

    #[test]
    /// Pacing with Fibheap
    fn test_daryheap_randomdata_extra2() {
        let batch_num = 1000 * 3;
        let mut get_one = gen_unique();

        let mut daryheap = DaryHeap1::new();
        let mut fibheap = FibHeap::new();

        // pad 50% of batch
        for i in 0..batch_num {
            let e = get_one();
            fibheap.insert(i, e); // push
            daryheap.insert(i, e);
        }

        for _ in 0..batch_num / 3 {
            let newkey = get_one();

            let mut i = random::<usize>() % fibheap.len();

            while fibheap.get(&i).is_none() {
                i = random::<usize>() % fibheap.len();
            }

            // println!("update: i:{i}, w:{newkey}");

            let fibres = fibheap.insert(i, newkey.clone());
            let daryres = daryheap.insert(i, newkey.clone());

            assert_eq!(fibres, daryres);

            let fibres = fibheap.pop_item();
            let daryres = daryheap.pop_item();

            assert_eq!(fibres, daryres);
        }

        loop {
            let fibres = fibheap.pop_item();
            let daryres = daryheap.pop_item();

            assert_eq!(fibres, daryres);

            if fibres.is_none() {
                break;
            }
        }

    }


    #[test]
    fn test_dary_stats() {
        fn do_test<const E: usize>() {
            let mut i = 1usize;
            let mut ln = 1usize;
            let mut col = 1usize;

            loop {
                let (x_ln, x_col) = pos!(i);

                /* Verify pos */
                assert_eq!(x_ln, ln,);
                assert_eq!(x_col, col,);

                /* Verify total */
                assert_eq!(i, total!(ln, col));

                i += 1;
                col += 1;

                if (col - 1) % basen!(ln - 1) == 0 {
                    ln += 1;
                    col = 1;
                }

                if i > 1 << 20 {
                    break;
                }
            }

            println!("E: {E} ok");
        }

        println!("!1: {:0b}", !0usize << 2);

        do_test::<1>();
        do_test::<2>();
        do_test::<3>();
        do_test::<4>();
        do_test::<5>();
        do_test::<6>();
        do_test::<7>();
        do_test::<8>();
    }
}
