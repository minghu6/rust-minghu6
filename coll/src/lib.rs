#![feature(macro_metavar_expr)]
#![feature(is_sorted)]
#![feature(exact_size_is_empty)]
#![feature(let_chains)]
#![feature(int_roundings)]


pub mod frac_casc;
pub mod segment_tree;
pub mod easycoll;
pub mod union_find;
pub mod aux;
mod beyond_god;
mod unpack;

pub use beyond_god::*;

pub use paste::paste;
pub use m6arr::*;
pub use m6entry::*;
