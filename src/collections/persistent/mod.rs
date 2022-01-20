//! Persistent Data Structure

use std::{fmt::Debug, error::Error};

use super::Collection;


pub mod list;
pub mod vector;



pub trait List<'a, T> {
    fn cons(&self, head: *mut T) -> Box<dyn List<'a, T> + 'a>;

    fn ht(&self) -> (*mut T, Box<dyn List<'a, T> + 'a>);

    fn duplicate(&self) -> Box<dyn List<'a, T> + 'a>;

}


pub fn cons<'a, T>(
    h: *mut T,
    t: Box<dyn List<'a, T>>
) -> Box<dyn List<'a, T> + 'a> {
    t.cons(h)
}


pub fn ht<'a, T>(
    l: Box<dyn List<'a, T>>
) -> (*mut T, Box<dyn List<'a, T> + 'a>) {
    l.ht()
}


pub trait Vector<'a, T: Debug>: Debug + Collection {
    fn nth(&self, idx: usize) -> &T;

    // get last index of Vector
    fn peek(&self) -> Option<&T>;

    fn push(&self, item: T) -> Box<dyn Vector<'a, T> + 'a>;

    fn pop(&self) -> Result<Box<dyn Vector<'a, T> + 'a>, Box<dyn Error>>;

    fn assoc(&self, idx: usize, item: T) -> Box<dyn Vector<'a, T> + 'a>;

    fn duplicate(&self) -> Box<dyn Vector<'a, T> + 'a>;

    fn transient(&self) -> Result<Box<dyn Vector<'a, T> + 'a>, ()>;

    fn persistent(&self) -> Result<Box<dyn Vector<'a, T> + 'a>, ()>;

}


// pub trait TransientCapable {
//     fn transient(&mut self);
// }


// pub trait PersistentCapable {
//     fn persistent(&mut self);
// }

