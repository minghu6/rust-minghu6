#[cfg(test)]
mod tests;


use std::{marker::PhantomData, ops::{RangeBounds, Sub, Add}};

use crate::segment_tree::{Count, RawIntoStats, Sum};


macro_rules! range_len {
    ($i:ident) => {
        (($i as isize + 1) & -($i as isize + 1)) as usize
    };
}


/// [l, r]
macro_rules! parse_range {
    ($range:expr, $len:expr) => {{
        use std::ops::Bound::*;

        let range = $range;
        let len = $len;

        let l;
        let r;

        match range.start_bound() {
            Included(v) => l = *v,
            Excluded(v) => l = *v + 1,
            Unbounded => l = 0,
        }

        match range.end_bound() {
            Included(v) => r = *v,
            Excluded(v) => {
                assert!(*v > 0, "range upper is invalid (=0)");
                r = *v - 1
            }
            Unbounded => r = len - 1,
        }

        (l, r)
    }};
}

////////////////////////////////////////////////////////////////////////////////
//// Structure

#[derive(Debug, Clone)]
pub struct BIT<T, C: Count = Sum<T>> {
    data: Vec<T>,
    _note: PhantomData<C>,
}


#[repr(transparent)]
pub struct RangeAddQuerySum<T>(BIT<T, Sum<T>>)
where
    T: Default,
    for<'a> &'a T: Add<&'a T, Output = T> + 'a,
    for<'a> T: 'a,;


////////////////////////////////////////////////////////////////////////////////
//// Implementation

impl<T: Clone, C: Count<Stats = T>> BIT<T, C> {
    pub fn new<U>(raw: &[U]) -> Self
    where
        U: Clone + RawIntoStats<C, Stats = T>,
    {
        assert!(!raw.is_empty());

        let data = Self::build(raw);

        Self {
            data,
            _note: PhantomData::<C>,
        }
    }

    pub fn prefix(&self, mut i: usize) -> T {
        i = std::cmp::min(i, self.data.len() - 1);

        let mut acc = C::e();

        loop {
            acc = C::combine(&acc, &self.data[i]);

            if i < range_len!(i) {
                break;
            }

            i -= range_len!(i);
        }

        acc
    }

    pub fn add(&mut self, mut i: usize, addend: T) {
        while i < self.data.len() {
            self.data[i] = C::combine(&self.data[i], &addend);

            i += range_len!(i);
        }
    }

    pub fn query<R: RangeBounds<usize>>(&self, range: R) -> T
    where
        T: Sub<T, Output = T>
    {
        let (l, r) = parse_range!(range, self.data.len());

        self.prefix(r) - if l > 0 { self.prefix(l - 1) } else { C::e() }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.data.len()
    }


    fn build<U>(raw: &[U]) -> Vec<T>
    where
        U: Clone + RawIntoStats<C, Stats = T>,
    {
        let mut data = vec![C::e(); raw.len()];

        for i in 0..raw.len() {
            data[i] = C::combine(&data[i], &raw[i].clone().raw_into_stats());

            // direct parent
            let j = i + range_len!(i);

            if j < raw.len() {
                data[j] = C::combine(&data[j], &data[i]);
            }
        }

        data
    }

}


impl BIT<i32, Sum<i32>> {
    pub fn range_add_for_origin<R: RangeBounds<usize>>(&mut self, range: R, addend: i32) {
        let (l, r) = parse_range!(range, self.data.len());

        self.add(l, addend);
        self.add(r + 1, -addend);
    }

    pub fn create_range_add_query_sum_aux(&self)
    -> RangeAddQuerySum<i32>
    {
        RangeAddQuerySum::new(self.len())
    }
}


impl RangeAddQuerySum<i32> {
    pub fn new(cap: usize) -> Self {
        Self(BIT {
            data: vec![0; cap],
            _note: PhantomData,
        })
    }

    pub fn add<R: RangeBounds<usize>>(&mut self, range: R, addend: i32) {
        let (l, r) = parse_range!(range, self.0.len());

        self.0.add(l, addend * (l as i32 - 1));
        self.0.add(r+1, -addend * r as i32);
    }

    /// Get update accumulation
    pub fn query<R: RangeBounds<usize>>(&self, range: R, bit1: &BIT<i32>) -> i32 {
        let (l, r) = parse_range!(range, self.0.len());

        self.prefix(r, bit1) - if l > 0 { self.prefix(l-1, bit1) } else { 0 }
    }

    fn prefix(&self, i: usize, bit1: &BIT<i32>) -> i32 {
        bit1.prefix(i) * i as i32 - self.0.prefix(i)
    }

}

