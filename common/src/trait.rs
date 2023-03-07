////////////////////////////////////////////////////////////////////////////////
//// Traits

use std::{mem::size_of, ptr::copy_nonoverlapping};

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

            copy_nonoverlapping(self_ptr, new_ptr, self.len());
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



#[cfg(test)]
mod tests {
    use super::*;

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
