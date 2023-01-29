pub mod utf8;
pub mod utf16;
pub mod path;
pub mod timeit;


use std::{ptr, fmt::Write, mem::size_of, collections::HashSet};

use either::Either;


////////////////////////////////////////////////////////////////////////////////
//// Traits

pub trait Reverse {
    fn reverse(&self) -> Self;
}


pub trait Conjugation<T> {
    fn adjoint(&self, baseline: &str) -> T;
}

pub trait BitLen {
    fn bit_len(&self) -> usize;
}

pub trait DeepCopy {
    fn deep_copy(&self) -> Self;
}

pub trait StrJoin {
    fn strjoin(&mut self, sep: &str) -> String;
}

pub trait TrimInPlace {
    fn trim_in_place (&mut self);
}


////////////////////////////////////////////////////////////////////////////////
//// Implements

impl Reverse for Either<(), ()> {
    fn reverse(&self) -> Either<(), ()> {
        if self.is_left() {
            Either::Right(())
        } else {
            Either::Left(())
        }
    }
}


impl BitLen for usize {
    fn bit_len(&self) -> usize {
        size_of::<usize>() * 8 - self.leading_zeros() as usize
    }
}

impl BitLen for u32 {
    fn bit_len(&self) -> usize {
        size_of::<u32>() * 8 - self.leading_zeros() as usize
    }
}


impl<T: Default> DeepCopy for Vec<T> {
    fn deep_copy(&self) -> Self {
        if self.is_empty() {
            return Vec::new();
        }

        let cap = self.capacity();

        let mut new_vec = Vec::<T>::with_capacity(cap);

        let new_ptr = new_vec.as_mut_ptr();

        unsafe {
            let self_ptr = self.as_ptr() as *mut T;

            new_vec.resize_with(self.len(), || T::default());

            ptr::copy_nonoverlapping(self_ptr, new_ptr, self.len());
        }

        new_vec
    }
}


impl<'a, T: ToString> StrJoin for dyn Iterator<Item=&T> + 'a {
    fn strjoin(&mut self, sep: &str) -> String {
        let mut seq = vec![];

        for item in self.into_iter() {
            seq.push(item.to_string());
        }

        seq.join(sep)
    }
}


impl TrimInPlace for String {
    fn trim_in_place (&mut self) {
        let trimed_slice = self.trim();
        let start = trimed_slice.as_ptr();
        let newlen = trimed_slice.len();

        unsafe {
            std::ptr::copy(
                start,
                self.as_mut_ptr(), // no str::as_mut_ptr() in std ...
                newlen,
            );
        }
        self.truncate(newlen); // no String::set_len() in std ...
    }
}



////////////////////////////////////////////////////////////////////////////////
//// Utils

pub fn strshift<T: ToString>(it: T, pad: &str) -> String {
    let mut cache = String::new();

    write!(cache, "{}", it.to_string()).unwrap();

    let mut res = vec![];
    for line in cache.split('\n') {
        res.push(pad.to_string() + line);
    }

    res.join("\n")
}


pub fn normalize<T: Ord>(raw_data: &[T]) -> Vec<usize> {
    if raw_data.is_empty() {
        return vec![];
    }

    let mut res = vec![0; raw_data.len()];
    let mut taged: Vec<(usize, &T)> = raw_data
        .into_iter()
        .enumerate()
        .collect();

    taged.sort_by_key(|x| x.1);

    let mut rank = 1;
    let mut iter = taged.into_iter();
    let (i, mut prev) = iter.next().unwrap();
    res[i] = rank;

    for (i, v) in iter {
        if v != prev {
            rank += 1;
        }
        prev = v;
        res[i] = rank;
    }

    res
}


#[cfg(test)]
pub(crate) fn gen() -> impl FnMut() -> usize {
    let mut _inner = 0;
    move || {
        let old = _inner;
        _inner += 1;
        old
    }
}


pub fn gen_unique() -> impl FnMut() -> usize {
    let mut set = HashSet::new();

    move || {
        let mut v = crate::algs::random();

        while set.contains(&v) {
            v = crate::algs::random();
        }

        set.insert(v);

        v
    }
}

////////////////////////////////////////////////////////////////////////////////
//// Declare Macro

#[macro_export]
macro_rules! ht {
    ( $head_expr:expr, $tail_expr:expr ) => {
        {
            let head = $head_expr;
            let tail = $tail_expr;

            let mut _vec = vec![head];
            _vec.extend(tail.iter().cloned());
            _vec
        }
    };
    ( $head:expr) => {
        {
            ht!($head, vec![])
        }
    };

}


/// should be used inner function which return Result<(), Box<dyn Error>>
#[macro_export]
macro_rules! should {
    ($cond:expr $(,)?) => {{
        use crate::XXXError;

        if !$cond {
            return Err(XXXError::new_box_err(
                ""
            ))
        }

     }};
    ($cond:expr, $args:tt) => {{
        use crate::XXXError;

        if !$cond {
            return Err(XXXError::new_box_err(
                format!($args).as_str()
            ))
        }

     }};
}


#[cfg(test)]
mod tests {
    use std::error::Error;

    use super::*;

    #[test]
    fn test_should() -> Result<(), Box<dyn Error>> {

        should!(2 < 2, "2 shoud lt 2");

        Ok(())
    }

    #[test]
    fn test_deep_copy() {

        let mut vec0 = vec!['a', 'b', 'c'];

        let mut vec1 = vec0.deep_copy();

        vec1[1] = 'e';

        vec0[1] = '0';

        println!("{:?}", vec0);
        println!("{:?}", vec1);

    }

    #[test]
    fn test_gen_unique() {

        let mut unique = gen_unique();

        for _ in 0..1000 {
            unique();
        }
    }

}
