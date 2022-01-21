
use std::fmt::{ Debug, self };
use std::rc::Rc;
use std::cell::RefCell;

use crate::collections::Collection;

use super::Vector;


pub struct TRawVec<T> {
    raw: Rc<RefCell<Vec<T>>>
}


impl<'a, T: 'a + Debug> Vector<'a, T> for TRawVec<T> {
    fn nth(&self, idx: usize) -> &T {
        &self.raw.as_ref().borrow()[idx]
    }

    fn peek(&self) -> Option<&T> {
        let len = self.len();

        if len == 0 {
            None
        } else {
            Some(&self.raw.as_ref().borrow()[len - 1])
        }
    }

    fn push(&self, item: T) -> Box<dyn Vector<'a, T> + 'a> {
        unsafe {
            let mut_self = &mut *(self as *const Self as *mut Self);

            self.raw.as_ref().borrow_mut().push(item);
        }

        box (self.clone())
    }

    fn pop(&self) -> Result<Box<dyn Vector<'a, T> + 'a>, Box<dyn std::error::Error>> {
        todo!()
    }

    fn assoc(&self, idx: usize, item: T) -> Box<dyn Vector<'a, T> + 'a> {
        todo!()
    }

    fn duplicate(&self) -> Box<dyn Vector<'a, T> + 'a> {
        todo!()
    }

    fn transient(&self) -> Result<Box<dyn Vector<'a, T> + 'a>, ()> {
        Err(())
    }

    fn persistent(&self) -> Result<Box<dyn Vector<'a, T> + 'a>, ()> {
        Err(())
    }
}


impl<T> Collection for TRawVec<T> {
    fn len(&self) -> usize {
        self.raw.as_ref().borrow().len()
    }
}

impl<T> Debug for TRawVec<T> {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> fmt::Result {
         todo!()
    }
}

impl<T> Clone for TRawVec<T> {
    fn clone(&self) -> Self {
        Self { raw: self.raw.clone() }
    }
}
