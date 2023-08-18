use std::{mem::size_of, ptr::copy_nonoverlapping};

////////////////////////////////////////////////////////////////////////////////
//// Macro

macro_rules! impl_for_num {
    ($name:ident|all) => {
        impl_for_num!($name | int);
        impl_for_num!($name | float);
    };
    ($name:ident|int) => {
        impl_for_num!($name | sint);
        impl_for_num!($name | uint);
    };
    ($name:ident|float) => {
        impl_for_num!($name | f32);
        impl_for_num!($name | f64);
    };
    ($name:ident|uint) => {
        impl_for_num!($name | u128);
        impl_for_num!($name | u64);
        impl_for_num!($name | usize);
        impl_for_num!($name | u32);
        impl_for_num!($name | u16);
        impl_for_num!($name | u8);
    };
    ($name:ident|sint) => {
        impl_for_num!($name | i128);
        impl_for_num!($name | i64);
        impl_for_num!($name | isize);
        impl_for_num!($name | i32);
        impl_for_num!($name | i16);
        impl_for_num!($name | i8);
    };
    (max| $for_ty:ty) => {
        impl Max<$for_ty> for $for_ty {
            fn max() -> $for_ty {
                <$for_ty>::MAX
            }
        }
    };
    (min| $for_ty:ty) => {
        impl Min<$for_ty> for $for_ty {
            fn min() -> $for_ty {
                <$for_ty>::MIN
            }
        }
    };
    (float| $for_ty:ty) => {
        impl Float<$for_ty> for $for_ty {}
        impl_for_num!(num | $for_ty);
    };
    (uint| $for_ty:ty) => {
        impl UInt<$for_ty> for $for_ty {}
        impl_for_num!(int | $for_ty);
    };
    (sint| $for_ty:ty) => {
        impl SInt<$for_ty> for $for_ty {}
        impl_for_num!(int | $for_ty);
    };
    (int| $for_ty:ty) => {
        impl Int<$for_ty> for $for_ty {}
        impl_for_num!(num | $for_ty);
    };
    (num| $for_ty:ty) => {
        impl Num<$for_ty> for $for_ty {}
    };
}


impl_for_num!(sint | sint);
impl_for_num!(uint | uint);
impl_for_num!(float | float);


impl_for_num!(max | all);
impl_for_num!(min | all);


////////////////////////////////////////////////////////////////////////////////
//// Traits


pub trait Reverse {
    fn reverse(&self) -> Self;
}


pub trait Conjugation<T> {
    fn adjoint(&self, baseline: &str) -> T;
}

/// Minimal representation bits
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
    fn trim_in_place(&mut self);
}

pub trait Max<T> {
    fn max() -> T;
}

pub trait Min<T> {
    fn min() -> T;
}

pub trait Num<T> {}

pub trait Int<T> {}

pub trait UInt<T> {}

pub trait SInt<T> {}

pub trait Float<T> {}


////////////////////////////////////////////////////////////////////////////////
//// Implements

// no negative bounds
// impl<T: Int<T>> Num<T> for T {}

// impl<T: SInt<T>> Int<T> for T {}

// impl<T: SInt<T>> !UInt<T> for T {}

// impl<T: UInt<T>> !SInt<T> for T {}

// impl<T: UInt<T>> Int<T> for T {}


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


impl<'a, T: ToString> StrJoin for dyn Iterator<Item = &T> + 'a {
    fn strjoin(&mut self, sep: &str) -> String {
        let mut seq = vec![];

        for item in self.into_iter() {
            seq.push(item.to_string());
        }

        seq.join(sep)
    }
}


impl TrimInPlace for String {
    fn trim_in_place(&mut self) {
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
