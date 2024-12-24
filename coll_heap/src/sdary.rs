////////////////////////////////////////////////////////////////////////////////
//// Macros

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
    ($x:expr) => {{
        let x = $x;

        // = q^n
        let var = (base!() - 1) * x + 1;

        let mut ln = var.ilog2() as usize / E;

        let mut col: usize = x - total!(ln);

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

macro_rules! paren_pos {
    ($pos:expr) => {{
        let (ln, col) = $pos;

        (ln - 1, (col - 1) / base!() + 1)
    }};
}


/// first child pos
macro_rules! child_pos {
    ($pos:expr) => {{
        let (ln, col) = $pos;

        (ln + 1, (col - 1) * base!() + 1)
    }};
}


/// by idx to idx
macro_rules! paren {
    ($idx:expr) => {{
        let (ln, col) = paren_pos!(pos!($idx + 1));

        total!(ln, col) - 1
    }};
}

macro_rules! child {
    ($idx:expr) => {{
        let (ln, col) = child_pos!(pos!($idx + 1));

        total!(ln, col) - 1
    }};
}


/// Min Heap, I is unique, T is weight
#[derive(Clone)]
pub struct DaryHeap<const E: usize, T> {
    raw: Vec<T>,
}

/// Basic Implementation
impl<const E: usize, T> DaryHeap<E, T> {
    pub const fn len(&self) -> usize {
        self.raw.len()
    }
}

impl<const E: usize, T> DaryHeap<E, T> {
    pub fn new() -> Self {
        Self::with_capacity(0)
    }

    /// Truely entry-point
    pub fn with_capacity(capacity: usize) -> Self {
        debug_assert!(E > 0);

        Self {
            raw: Vec::with_capacity(capacity),
        }
    }

    pub fn top(&self) -> Option<&T> {
        self.raw.first()
    }
}

impl<const E: usize, T> DaryHeap<E, T>
where
    T: Ord,
{
    pub fn push(&mut self, v: T) {
        self.raw.push(v);
        self.sift_up(self.len() - 1);
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.len() == 0 {
            return None;
        }

        let last = self.raw.len() - 1;
        self.raw.swap(0, last);

        let v = self.raw.pop().unwrap();

        self.sift_down(0);

        Some(v)
    }
}

impl<const E: usize, T> DaryHeap<E, T>
where
    T: Ord,
{
    /// return insert_idx
    fn sift_up(&mut self, idx: usize) -> usize {
        let mut cur = idx;

        while cur != 0 {
            let paren = paren!(cur);

            if self.raw[cur] < self.raw[paren] {
                self.raw.swap(cur, paren);
                cur = paren;
            } else {
                break;
            }
        }

        cur
    }

    /// return insert_idx
    fn sift_down(&mut self, idx: usize) -> usize {
        let mut cur_idx = idx;

        while let Some((child_idx, child_w)) = self.min_child(cur_idx)
            && child_w < &self.raw[cur_idx]
        {
            self.raw.swap(cur_idx, child_idx);
            cur_idx = child_idx;
        }

        cur_idx
    }

    fn min_child(&self, idx: usize) -> Option<(usize, &T)> {
        let start = child!(idx);
        let end = std::cmp::min(self.len(), start + base!());

        if end <= start {
            return None;
        }

        self.raw[start..end]
            .iter()
            .enumerate()
            .min_by_key(|(_, w)| *w)
            .map(|(i, w)| (start + i, w))
    }
}


#[cfg(test)]
mod tests {

    use std::{cmp::Reverse, collections::BinaryHeap};

    use common::{thread_rng, Rng};

    use super::*;

    #[test]
    fn test_sdaryheap_fixeddata() {
        let mut heap = DaryHeap::<1, usize>::new();

        heap.push(2);
        heap.push(4);
        heap.push(1);

        assert_eq!(heap.pop().unwrap(), 1);
        assert_eq!(heap.pop().unwrap(), 2);
        assert_eq!(heap.pop().unwrap(), 4);
        assert_eq!(heap.pop(), None);
    }

    #[test]
    fn test_sdaryheap_randomdata() {
        fn do_test<const E: usize>() {

            let mut eg = DaryHeap::<E, u64>::new();
            let mut cg = BinaryHeap::<Reverse<u64>>::new();

            let n = 10_000;
            let mut rng = thread_rng();

            for _ in 0..n {
                let v = rng.gen_range(0..n * 2);

                eg.push(v);
                cg.push(Reverse(v));
            }

            while let Some(v) = cg.pop() {
                assert_eq!(v.0, eg.pop().unwrap());
            }

            assert!(eg.len() == 0);
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
}
