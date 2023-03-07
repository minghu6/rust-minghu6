

pub trait Num{}

pub trait Int: Num {}
pub trait UInt: Int {}
pub trait SInt: Int {}



// impl<T: Int> Num for T {}
// impl<T: SInt> Int for T where T {}
// impl<T: UInt + SInt> Int for T {}

macro_rules! impl_for {
    (uint, $ty:ty) => {
        impl Num for $ty {}
        impl Int for $ty {}
        impl UInt for $ty {}
    };
}


impl_for!(uint, usize);
impl_for!(uint, u32);




// pub fn log2<T: Int>(mut a: T) -> impl Int {
//     let mut i = 0;
//     while a.as_i32() > 0 {
//         i += 1;
//         a >>= 1;
//     }

//     i
// }
