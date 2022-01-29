#![allow(unused_imports)]

use std::fmt::{ Debug, self, Display };
use std::ops::{Index, Add};
use std::cmp::max;
use std::ptr::*;

use crate::collections::{Collection, as_ptr};
use crate::etc::DeepCopy;
use crate::{should, XXXError};

use super::Vector;


////////////////////////////////////////////////////////////////////////////////
//// Structure

pub struct PRawVec<T> {
    raw: Vec<T>,
}


pub struct TRawVec<T> {
    cap: usize,
    len: usize,
    _holder: *mut Vec<T>,
    ptr: *mut T
}



////////////////////////////////////////////////////////////////////////////////
//// Implement

impl<T> PRawVec<T> {

    fn new(vec: Vec<T>) -> Self {
        Self {
            raw: vec,
        }
    }

    pub fn empty() -> Self {
        Self {
            raw: Vec::new()
        }
    }

}

impl<'a, T: 'a + Debug + Default + Clone> Vector<'a, T> for PRawVec<T> {
    fn nth(&self, idx: usize) -> &T {
        &self.raw[idx]
    }

    fn peek(&self) -> Option<&T> {
        todo!()
    }

    fn push(&self, item: T) -> Box<dyn Vector<'a, T> + 'a> {
        let mut new_vec = self.raw.deep_copy();

        new_vec.push(item);

        box Self::new(new_vec)
    }

    fn pop(&self) -> Result<Box<dyn Vector<'a, T> + 'a>, Box<dyn std::error::Error>> {
        let mut new_vec = self.raw.deep_copy();

        if let Some(_) = new_vec.pop() {
            Ok(box Self::new(new_vec))
        } else {
            Err(XXXError::new_box_err("pop an empty vector"))
        }

    }

    fn assoc(&self, idx: usize, item: T) -> Box<dyn Vector<'a, T> + 'a> {
        let mut new_vec = self.raw.deep_copy();

        if self.len() == idx {
            new_vec.push(item);
        } else {
            new_vec[idx] = item;
        }

        box Self::new(new_vec)
    }

    fn duplicate(&self) -> Box<dyn Vector<'a, T> + 'a> {
        box Self::new(self.raw.deep_copy())
    }

    fn transient(&self) -> Result<Box<dyn Vector<'a, T> + 'a>, ()> {
        let vec = self.raw.deep_copy();

        Ok(box TRawVec::from_vec(vec))
    }

    fn persistent(&self) -> Result<Box<dyn Vector<'a, T> + 'a>, ()> {
        todo!()
    }
}

impl<T: Debug + Default> Debug for PRawVec<T> {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        todo!()
        // for i in 0..self.len() {
        //     write!(f, "{:#?}, ", self.nth(i))?;
        // }

        // Ok(())
    }
}

impl<T> Collection for PRawVec<T> {
    fn len(&self) -> usize {
        self.raw.len()
    }
}



impl<'a, T: 'a + Debug + Default + Clone> TRawVec<T> {

    pub fn empty() -> Self {
        Self {
            cap: 0,
            len: 0,
            _holder: null_mut(),
            ptr: null_mut(),
        }
    }

    fn from_vec(vec: Vec<T>) -> Self {
        let mut vec = vec;
        let len = vec.len();
        let cap = vec.capacity();
        let ptr = vec.as_mut_ptr();

        Self {
            cap,
            len,
            _holder: as_ptr(vec),
            ptr,
        }
    }

    pub fn into_vec(self) -> Vec<T> {
        unsafe {
            Vec::from_raw_parts(self.ptr, self.len, self.cap)
        }
    }

    fn expand(&mut self) {
        unsafe {
            let new_cap = max(self.cap * 1.5 as usize, self.cap + 1);
            let mut new_vec = Vec::with_capacity(new_cap);
            let new_ptr = new_vec.as_mut_ptr();

            if self.len > 0 {
                copy(self.ptr, new_ptr, self.len);
            }

            self.cap = new_cap;
            self.ptr = new_ptr;
            self._holder = as_ptr(new_vec);
        }
    }

    fn clone_head(&self) -> Self {

        Self {
            cap: self.cap,
            len: self.len,
            _holder: self._holder,
            ptr: self.ptr,
        }
    }

}


impl<'a, T: 'a + Debug + Default + Clone> Vector<'a, T> for TRawVec<T> {
    fn nth(&self, idx: usize) -> &T {
        debug_assert!(self.len > idx);

        unsafe { &*self.ptr.add(idx) }
    }

    fn peek(&self) -> Option<&T> {
        // let len = self.len();

        // if len == 0 {
        //     None
        // } else {
        //     Some(&self.raw.as_ref().borrow()[len - 1])
        // }
        todo!()
    }

    fn push(&self, item: T) -> Box<dyn Vector<'a, T> + 'a> {
        unsafe {
            let mut_self = &mut *(self as *const Self as *mut Self);

            if self.len == self.cap {
                mut_self.expand();
            }

            *mut_self.ptr.add(self.len) = item;
            mut_self.len += 1;
        }

        box self.clone_head()

    }

    fn pop(&self) -> Result<Box<dyn Vector<'a, T> + 'a>, Box<dyn std::error::Error>> {
        should!(self.len > 0, "Can't pop empty vector");

        unsafe {
            let mut_self = &mut *(self as *const Self as *mut Self);
            mut_self.len -= 1;
        }

        Ok(box self.clone_head())
    }

    fn assoc(&self, idx: usize, item: T) -> Box<dyn Vector<'a, T> + 'a> {
        debug_assert!(self.len >= idx);

        unsafe {

            if self.len == idx {
                return self.push(item);
            }

            *self.ptr.add(idx) = item;
        }

        box self.clone_head()
    }

    fn duplicate(&self) -> Box<dyn Vector<'a, T> + 'a> {
        box unsafe {
            let mut new_vec = Vec::with_capacity(self.cap);
            let new_ptr = new_vec.as_mut_ptr();

            if self.len > 0 {
                copy(self.ptr, new_ptr, self.len);
            }

            Self {
                cap: self.cap,
                len: self.len,
                _holder: as_ptr(new_vec),
                ptr: new_ptr,
            }
        }
    }

    fn transient(&self) -> Result<Box<dyn Vector<'a, T> + 'a>, ()> {
        Err(())
    }

    fn persistent(&self) -> Result<Box<dyn Vector<'a, T> + 'a>, ()> {
        let mut new_vec = Vec::<T>::with_capacity(self.cap);
        let new_ptr = new_vec.as_mut_ptr();
        new_vec.resize_with(self.len(), || T::default());

        unsafe {
            copy_nonoverlapping(self.ptr, new_ptr, self.len());
        }

        Ok(box PRawVec::new(new_vec))
    }

}


impl<T> Collection for TRawVec<T> {
    fn len(&self) -> usize {
        self.len
    }
}

impl<T: Debug> Debug for TRawVec<T> {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        todo!()
        // for i in 0..self.len {
        //     write!(f, "{:#?}, ", self.nth(i))?;
        // }

        // Ok(())
    }
}

// impl<T> Clone for TRawVec<T> {
//     fn clone(&self) -> Self {
//         Self { raw: self.raw.clone() }
//     }
// }


#[cfg(test)]
mod tests {
    use itertools::Itertools;

    use super::{ TRawVec, PRawVec };
    use crate::{
        collections::{as_ptr, persistent::vector::Vector, Collection},
        test::{ persistent::VectorProvider, * },
    };

    #[test]
    fn test_praw_vec_randomedata() {
        unsafe { InodeProvider {}.test_pvec(|| box PRawVec::empty()) }
    }

    #[test]
    fn test_traw_vec_randomedata() {
        // unsafe { InodeProvider {}.test_tvec(|| box TRawVec::empty()) }
        unsafe { UZProvider {}.test_tvec(|| box TRawVec::empty()) }

    }

    #[test]
    fn test_ptraw_tran_vec_randomedata() {
        unsafe { InodeProvider {}.test_pttran(|| box PRawVec::empty()) }
    }

    #[test]
    fn test_traw_vec_manually() {
        // let tv = TRawVec::empty();

        // let mut btv = (box tv) as Box<dyn Vector<usize>>;

        // let provider = UZProvider {};
        // let batch_num = 2;

        // for i in 0..batch_num {
        //     btv = btv.push(i);
        // }

        // let mut btv = btv.duplicate();
        // for i in 0..batch_num {
        //     btv = btv.assoc(i, i * 10);
        // }

        // for i in (0..batch_num).rev() {
        //     btv = btv.pop().unwrap();

        //     for j in 0..i {
        //         assert_eq!(*btv.nth(j), j * 10);
        //     }
        // }

        // println!("{:?}", btv);

        // let batch_num = 1000;
        // let provider = UZProvider {};
        // let mut vec= (box TRawVec::empty()) as Box<dyn Vector<usize>>;

        // let mut plain_elem_vec = vec![];
        // for i in 0..batch_num {
        //     // plain_elem_vec.push(i);
        //     plain_elem_vec.push(provider.get_one());
        // }

        // for i in 0..batch_num {
        //     vec = vec.push(plain_elem_vec[i].clone());

        //     for j in 0..i+1 {
        //         assert_eq!(vec.nth(j), &plain_elem_vec[j]);
        //     }
        // }

        // let mut uvec = vec.duplicate();
        // let mut uelem_vec = vec![];
        // for i in 0..batch_num {
        //     // uelem_vec.push(i);
        //     uelem_vec.push(provider.get_one());
        // }
        // for i in 0..batch_num {
        //     uvec = uvec.assoc(i, uelem_vec[i].clone());

        //     assert_eq!(uvec.nth(i), &uelem_vec[i])
        // }


        // for i in (0..batch_num).rev() {
        //     vec = vec.pop().unwrap();

        //     for j in 0..i {
        //         // assert_eq!(vec.nth(j), &uelem_vec[j]);
        //         assert_eq!(vec.nth(j), &plain_elem_vec[j]);

        //     }
        // }
    }

}