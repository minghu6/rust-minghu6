#![allow(incomplete_features)]

#![feature(test)]
#![feature(coroutines, coroutine_trait)]
#![feature(stmt_expr_attributes)]
#![feature(type_ascription)]
#![feature(trait_alias)]
#![feature(trait_upcasting)]
#![feature(int_roundings)]
#![feature(const_option_ext)]
#![feature(macro_metavar_expr)]
#![feature(rustc_private)]
#![feature(type_alias_impl_trait)]
#![feature(iter_from_coroutine)]
#![feature(trace_macros)]
#![feature(let_chains)]
#![feature(cell_update)]


use proc_macros::make_simple_error_rules;


pub mod text;
pub mod io;
pub mod path;
pub mod unicode;

make_simple_error_rules!(XXXError);
