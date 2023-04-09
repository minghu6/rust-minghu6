#![feature(stmt_expr_attributes)]
#![feature(trait_alias)]
#![feature(test)]
#![feature(absolute_path)]
#![feature(macro_metavar_expr)]
#![feature(int_roundings)]
#![feature(negative_impls)]


pub mod error_code;
pub mod timeit;
pub mod rand;
mod r#trait;
pub mod util;


pub use itertools::{ Itertools, EitherOrBoth };

pub use r#trait::*;
pub use rand::*;
pub use error_code::*;
pub use util::{ gen, gen_unique, * };

