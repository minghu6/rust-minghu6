#![feature(test)]
#![feature(generators, generator_trait)]
#![feature(ptr_internals)]
#![feature(stmt_expr_attributes)]
#![feature(is_sorted)]
#![feature(type_ascription)]
#![feature(trait_alias)]
#![feature(box_syntax)]
#![feature(trait_upcasting)]
#![feature(int_roundings)]
#![feature(int_abs_diff)]


use proc_macros::{
    make_vec_macro_rules,
    make_simple_error_rules
};


pub mod text;
pub mod collections;
pub mod algs;
pub mod test;
pub mod etc;
pub mod error_code;


make_vec_macro_rules!(vecdeq , std::collections::VecDeque, push_back);

make_simple_error_rules!(XXXError);
