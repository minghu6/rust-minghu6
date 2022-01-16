#![allow(path_statements)]


use std::{
    alloc::{alloc, dealloc, Layout},
    ops::{Index, IndexMut},
};



////////////////////////////////////////////////////////////////////////////////
//// Structure

#[derive(Clone)]
#[repr(C)]
pub struct Array<T> {
    len: usize,
    ptr: *mut T,
}


////////////////////////////////////////////////////////////////////////////////
//// Implement

/// Heap Array
impl<T> Array<T> {
    pub fn new(cap: usize) -> Self {
        unsafe {
            let ptr = alloc(Self::layout(cap)) as *mut T;
            let len = cap;

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

    pub fn layout(cap: usize) -> Layout {
        Layout::array::<T>(cap).unwrap()
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// src, dst, len
    pub fn copy(src: &Self, dst: &Self, len: usize) {
        debug_assert!(src.len() >= len);
        debug_assert!(dst.len() >= len);

        unsafe { dst.ptr.copy_from(src.ptr, len) }
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

    pub fn as_ptr(&self) -> *mut Self {
        self as *const Self as *mut Self
    }

    // pub unsafe fn from_ptr(ptr: *mut T) -> Self {

    //     let len = *(ptr as *const usize).sub(1);

    //     Self {
    //         len,
    //         ptr
    //     }

    // }
}

impl<T> Drop for Array<T> {
    fn drop(&mut self) {
        unsafe {
            if self.len > 0 {
                dealloc(self.ptr as *mut u8, Self::layout(self.len));
                self.len = 0;
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
