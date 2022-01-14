
//! Linked List
//!


use std::{ptr::null_mut, marker::PhantomData};

use super::List;


////////////////////////////////////////////////////////////////////////////////
//// Structure

/// Persistent List
pub struct PList<'a, T> {
    ptr: *mut Node<T>,
    _ph: &'a PhantomData<()>
}


struct Node<T> {
    elem: *mut T,
    nxt: *mut Node<T>,
    // _ph: PhantomData<P>
}


////////////////////////////////////////////////////////////////////////////////
//// Implement


impl<T> Node<T> {

    fn new_ptr(elem: *mut T, nxt: *mut Node<T>) -> *mut Self {

        Box::into_raw(box Self {
            elem,
            nxt
        })
    }

}


impl<'a, T: 'a> PList<'a, T> {

    fn new(ptr: *mut Node<T>) -> Self {
        Self {
            ptr,
            _ph: &PhantomData
        }
    }

    #[allow(unused)]
    fn nil() -> Self {
        Self {
            ptr: null_mut(),
            _ph: &PhantomData
        }
    }
}

impl<'a, T: 'a> Clone for PList<'a, T> {
    fn clone(&self) -> Self {
        Self { ptr: self.ptr.clone(), _ph: &PhantomData }
    }
}



impl<'a, T: 'a> List<'a, T> for PList<'a, T> {
    fn cons(&self, head: *mut T) -> Box<dyn List<'a, T> + 'a> {

        box PList::new(Node::new_ptr(head, self.ptr))

    }

    fn ht(&self) -> (*mut T, Box<dyn List<'a, T> + 'a>) {

        unsafe {

            ((*self.ptr).elem, box PList::new((*self.ptr).nxt))

        }

    }

    fn duplicate(&self) -> Box<dyn List<'a, T> + 'a> {
        box self.clone()
    }

}




#[cfg(test)]
mod tests {
    use crate::test::{persistent::ListProvider, dict::InodeProvider};

    use super::PList;

    #[test]
    fn test_plist_randomedata() {
        unsafe {
            InodeProvider{}.test_list(|| box PList::nil())
        }

    }


}
