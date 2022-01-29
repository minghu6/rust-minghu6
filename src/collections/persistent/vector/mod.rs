use crate::collections::Collection;

use std::{fmt::Debug, error::Error};


pub mod trie;
pub mod raw;
pub mod rrb;


pub trait Vector<'a, T: Debug + Clone>: Debug + Collection {
    fn nth(&self, idx: usize) -> &T;

    // get last index of Vector
    fn peek(&self) -> Option<&T>;

    fn push(&self, item: T) -> Box<dyn Vector<'a, T> + 'a>;

    fn pop(&self) -> Result<Box<dyn Vector<'a, T> + 'a>, Box<dyn Error>>;

    fn assoc(&self, idx: usize, item: T) -> Box<dyn Vector<'a, T> + 'a>;

    /// Deep Copy
    fn duplicate(&self) -> Box<dyn Vector<'a, T> + 'a>;

    fn transient(&self) -> Result<Box<dyn Vector<'a, T> + 'a>, ()>;

    fn persistent(&self) -> Result<Box<dyn Vector<'a, T> + 'a>, ()>;

}
