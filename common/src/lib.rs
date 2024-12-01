#![feature(stmt_expr_attributes)]
#![feature(trait_alias)]
#![feature(test)]
#![feature(macro_metavar_expr)]
#![feature(int_roundings)]
#![feature(negative_impls)]


pub mod rand;
pub mod timeit;
mod traits;
pub mod utils;

pub use itertools::{EitherOrBoth, Itertools};
pub use rand::*;
pub use traits::*;
pub use utils::{gen, gen_unique, *};
