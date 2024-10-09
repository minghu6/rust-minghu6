#![feature(stmt_expr_attributes)]
#![feature(trait_alias)]
#![feature(test)]
#![feature(macro_metavar_expr)]
#![feature(int_roundings)]
#![feature(negative_impls)]


pub mod error_code;
pub mod rand;
pub mod timeit;
mod traits;
pub mod utils;

pub use error_code::*;
pub use itertools::{EitherOrBoth, Itertools};
pub use rand::*;
pub use traits::*;
pub use utils::{gen, gen_unique, *};


pub mod tests {
    pub const RESOURCE_DIR: &str = "res";
    pub const ZH_EN_POEMS: &str = "zh_en_poems.txt";
}
