use std::hash::Hash;




pub trait Num: Copy + Clone + Hash {
    fn as_i32(&self) -> i32;
}

pub trait Int: Num {}
pub trait UInt: Int {}
pub trait SInt: Int {}

// impl<T: Int> Num for T {}
impl Num for i32 {
    fn as_i32(&self) -> i32 {
        *self
    }


}

impl<T: SInt> Int for T {}

// impl SInt for isize {}
impl SInt for i32 {}



// impl Int for isize {}
// impl Int for isize {}


pub fn log2<T: Int>(mut a: T) -> impl Int {
    let mut i = 0;
    while a.as_i32() > 0 {
        i += 1;
        a >>= 1;
    }

    i
}
