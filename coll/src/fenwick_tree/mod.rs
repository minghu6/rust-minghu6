#[cfg(test)]
mod tests;


use std::{
    iter::Sum,
    ops::{Add, AddAssign, Range, RangeBounds, Sub},
};

macro_rules! range_len {
    ($i:ident) => {
        (($i as isize + 1) & -($i as isize + 1)) as usize
    };
}

macro_rules! zero {
    () => {
        [].into_iter().sum()
    };
}

////////////////////////////////////////////////////////////////////////////////
//// Structures

#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct BIT<T> {
    data: Vec<T>,
}


#[repr(transparent)]
pub struct RangeAddQuerySum<T>(BIT<T>)
where
    T: Sum,
    for<'a> &'a T: Add<&'a T, Output = T> + 'a,
    for<'a> T: 'a;


////////////////////////////////////////////////////////////////////////////////
//// Implementations

impl<T> BIT<T> {
    pub fn len(&self) -> usize {
        self.data.len()
    }
}

impl<T> BIT<T>
where
    T: Sum + Clone + Add<Output = T> + AddAssign,
{
    pub fn new<U>(raw: &[U]) -> Self
    where
        U: Clone + Into<T>,
    {
        assert!(!raw.is_empty());

        let data = Self::build(raw);

        Self { data }
    }

    fn build<U>(raw: &[U]) -> Vec<T>
    where
        U: Clone + Into<T>,
    {
        let mut data: Vec<T> = vec![zero!(); raw.len()];

        for i in 0..raw.len() {
            data[i] += raw[i].clone().into();

            // direct parent
            let j = i + range_len!(i);

            if j < raw.len() {
                let data_i = data[i].clone();

                data[j] += data_i;
            }
        }

        data
    }

    pub fn prefix(&self, mut i: usize) -> T {
        i = std::cmp::min(i, self.data.len() - 1);

        let mut acc = zero!();

        loop {
            acc += self.data[i].clone();

            if i < range_len!(i) {
                break;
            }

            i -= range_len!(i);
        }

        acc
    }

    pub fn add(&mut self, mut i: usize, addend: T) {
        while i < self.data.len() {
            self.data[i] += addend.clone();

            i += range_len!(i);
        }
    }

    pub fn query<R: RangeBounds<usize>>(&self, range: R) -> T
    where
        T: Sub<T, Output = T>,
    {
        let Range { start, end } = std::slice::range(range, ..self.len());

        let (l, r) = (start, end - 1);

        self.prefix(r) - if l > 0 { self.prefix(l - 1) } else { zero!() }
    }
}


impl BIT<i32> {
    pub fn range_add_for_origin<R: RangeBounds<usize>>(
        &mut self,
        range: R,
        addend: i32,
    ) {
        let Range { start, end } = std::slice::range(range, ..self.len());
        let (l, r) = (start, end - 1);

        self.add(l, addend);
        self.add(r + 1, -addend);
    }

    pub fn create_range_add_query_sum_aux(&self) -> RangeAddQuerySum<i32> {
        RangeAddQuerySum::new(self.len())
    }
}


impl RangeAddQuerySum<i32> {
    pub fn new(cap: usize) -> Self {
        Self(BIT { data: vec![0; cap] })
    }

    pub fn add<R: RangeBounds<usize>>(&mut self, range: R, addend: i32) {
        let Range { start, end } = std::slice::range(range, ..self.0.len());

        let (l, r) = (start, end - 1);
        self.0.add(l, addend * (l as i32 - 1));
        self.0.add(r + 1, -addend * r as i32);
    }

    /// Get update accumulation
    pub fn query<R: RangeBounds<usize>>(
        &self,
        range: R,
        bit1: &BIT<i32>,
    ) -> i32 {
        let Range { start, end } = std::slice::range(range, ..self.0.len());

        let (l, r) = (start, end - 1);
        self.prefix(r, bit1) - if l > 0 { self.prefix(l - 1, bit1) } else { 0 }
    }

    fn prefix(&self, i: usize, bit1: &BIT<i32>) -> i32 {
        bit1.prefix(i) * i as i32 - self.0.prefix(i)
    }
}
