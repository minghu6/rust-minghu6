pub mod utfx;

// use num_format::{ Locale, ToFormattedString };
// use std::fmt::{ Debug, self };
use std::mem::size_of;
use std::ptr;
use std::fmt::Write;

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

    use crate::*;
    // use super::NumENDebug;
    use std::error::Error;

    use super::DeepCopy;

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

}