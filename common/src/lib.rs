#![feature(stmt_expr_attributes)]
#![feature(trait_alias)]
#![feature(test)]
#![feature(macro_metavar_expr)]
#![feature(int_roundings)]
#![feature(negative_impls)]


pub mod error_code;
pub mod timeit;
pub mod rand;
mod traits;
pub mod utils;


pub use itertools::{ Itertools, EitherOrBoth };

pub use traits::*;
pub use rand::*;
pub use error_code::*;
pub use utils::{ gen, gen_unique, * };
