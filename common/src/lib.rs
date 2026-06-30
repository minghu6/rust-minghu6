#![cfg_attr(test, feature(test, stmt_expr_attributes))]
#![feature(macro_metavar_expr)]


pub mod rand;
pub mod timeit;
mod traits;
pub mod utils;

pub use itertools::{EitherOrBoth, Itertools};
pub use rand::*;
pub use traits::*;
pub use utils::{generate, gen_unique, *};
