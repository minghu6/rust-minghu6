#[cfg(test)]
mod tests;


use std::{marker::PhantomData, ops::RangeBounds};

use crate::segment_tree::{Count, RawIntoStats};


macro_rules! range_len {
    ($i:ident) => {
        (($i as isize) & -($i as isize)) as usize
    };
}


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

pub struct BIT<T, C: Count> {
    data: Vec<T>,
    _note: PhantomData<C>,
}



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


    /// index base-1
    pub fn prefix_count(&self, mut i: usize) -> T {
        assert!(i <= self.data.len());

        let mut acc = C::e();

        loop {
            acc = C::combine(&acc, &self.data[i]);

            if i <= range_len!(i) {
                break;
            }

            i -= range_len!(i);
        }

        acc
    }


    pub fn query<R: RangeBounds<usize>>(&self, range: R) -> T {
        let (l, r) = parse_range!(range, self.data.len());

        todo!()
    }


    fn build<U>(raw: &[U]) -> Vec<T>
    where
        U: Clone + RawIntoStats<C, Stats = T>,
    {
        let mut data = vec![C::e(); raw.len() + 1];

        for i in 1..=raw.len() {
            data[i] = raw[i].clone().raw_into_stats();

            // direct parent
            let j = i + range_len!(i);

            if j <= raw.len() {
                data[j] = C::combine(&data[j], &data[i]);
            }
        }

        data
    }

}

