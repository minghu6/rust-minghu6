#![allow(path_statements)]


use itertools::Itertools;

use crate::collections::Collection;
use std::{
    alloc::{alloc_zeroed, dealloc, Layout},
    ops::{Index, IndexMut}, ptr::{ null_mut, copy }, fmt::Debug,
};



////////////////////////////////////////////////////////////////////////////////
//// Structure

#[repr(C)]
pub struct Array<T> {
    len: usize,  // and capacity
    ptr: *mut T,
}


////////////////////////////////////////////////////////////////////////////////
//// Implement

/// Heap Array
impl<T> Array<T> {

    ///////////////////////////////////////
    //// static method

    pub fn empty() -> Self {
        Self::new(0)
    }

    pub fn new(cap: usize) -> Self {
        unsafe {
            let len = cap;

            let ptr = if cap == 0 {
                null_mut()
            } else {
                alloc_zeroed(Self::layout(cap)) as *mut T
            };

            Self { len, ptr }
        }
    }

    pub fn new_with(init: T, cap: usize) -> Self
    where
        T: Copy,
    {
        unsafe {
            let it = Self::new(cap);

            for i in 0..cap {
                (*it.ptr.add(i)) = init;
            }

            it
        }
    }


    pub fn merge(lf: &Self, rh: &Self) -> Self {
        let arr = Array::new(lf.len() + rh.len());

        unsafe {
            copy(lf.ptr, arr.ptr, lf.len());
            copy(rh.ptr, arr.ptr.add(lf.len()), rh.len());
        }

        arr
    }

    /// src, dst, len
    pub fn copy(src: &Self, dst: &Self, len: usize) {
        debug_assert!(src.len() >= len);
        debug_assert!(dst.len() >= len);

        unsafe { dst.ptr.copy_from(src.ptr, len) }
    }

    ///////////////////////////////////////
    //// dynamic method

    pub fn layout(cap: usize) -> Layout {
        Layout::array::<T>(cap).unwrap()
    }

    /// cap may greater than len
    pub fn clone_with(&self, len: usize, cap: usize) -> Self where T: Clone {
        debug_assert!(cap >= len);

        let newit = Self::new(cap);
        Self::copy(self, &newit, len);

        newit
    }

    pub fn iter<'a>(&'a self) -> Box<dyn Iterator<Item = &T> + 'a> {
        let mut i = 0;

        box std::iter::from_fn(move || {
            if i == self.len {
                return None;
            }

            let res = Some(&self[i]);
            i += 1;

            res
        })
    }

    pub fn as_ptr(&self) -> *mut T {
        self.ptr
    }

    // pub unsafe fn from_ptr(ptr: *mut T) -> Self {

    //     let len = *(ptr as *const usize).sub(1);

    //     Self {
    //         len,
    //         ptr
    //     }

    // }
}

impl<T> Collection for Array<T> {
    fn len(&self) -> usize {
        self.len
    }
}

impl<T> Drop for Array<T> {
    fn drop(&mut self) {
        unsafe {
            if self.len > 0 {
                dealloc(self.ptr as *mut u8, Self::layout(self.len));
            }
        }
    }
}


impl<T> Index<usize> for Array<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        debug_assert!(index < self.len);

        unsafe { &*self.ptr.add(index) }
    }
}

impl<T> IndexMut<usize> for Array<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        debug_assert!(index < self.len);

        unsafe { &mut *self.ptr.add(index) }
    }
}

impl<T> Clone for Array<T> {
    fn clone(&self) -> Self {
        let cloned = Self::new(self.len);
        Self::copy(self, &cloned, self.len);

        cloned
    }
}

impl Debug for Array<usize> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for chunk in &self.iter().chunks(10) {
            for item in chunk {
                write!(f, "{:>4}, ", item)?;
            }
            writeln!(f)?;
        }

        Ok(())
    }
}


////////////////////////////////////////////////////////////////////////////////
//// Macros


#[macro_export]
macro_rules! array {
    ( $init:expr; $cap:expr ) => {
        {
            use crate::collections::bare::array::Array;

            let init = $init;
            let cap = $cap;

            Array::new_with(init, cap)
        }
    };
    ($($item:expr),*) => {
        {
            use crate::collections::bare::array::Array;

            #[allow(unused_mut)]
            let mut cnt = 0;
            $(
                cnt += 1;

                let _ = $item;
            )*

            #[allow(unused_mut)]
            let mut arr = Array::new(cnt);

            let mut _i = 0;
            $(
                arr[_i] = $item;
                _i += 1;
            )*

            arr
        }
    };

}




#[cfg(test)]
mod tests {
    use super::Array;
    use crate::*;

    #[test]
    fn test_arr() {
        let mut arr = Array::<usize>::new(10);

        arr[2] = 15;
        arr[4] = 20;
        println!("{}", arr[2]);
        println!("{}", arr[1]);

        let arr = [0; 0];

        assert!(arr.is_empty());


        let _arr2 = array![0; 3];
        let arr2 = array!['a', 'b', 'd'];

        for e in arr2.iter() {
            println!("{}", e);
        }

        // test as_ptr/len/from_ptr
        let _ptr = arr2.as_ptr();

        // unsafe {
        //     let arr = Array::from_ptr(ptr);
        //     assert!(arr.len() == 3)
        // }
    }
}
