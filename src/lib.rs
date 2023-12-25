#![allow(incomplete_features)]

#![feature(test)]
#![feature(coroutines, coroutine_trait)]
#![feature(ptr_internals)]
#![feature(stmt_expr_attributes)]
#![feature(is_sorted)]
#![feature(type_ascription)]
#![feature(trait_alias)]
#![feature(trait_upcasting)]
#![feature(int_roundings)]
#![feature(absolute_path)]
#![feature(const_option_ext)]
#![feature(macro_metavar_expr)]
#![feature(rustc_private)]
#![feature(type_alias_impl_trait)]
#![feature(iter_from_coroutine)]
#![feature(trace_macros)]
#![feature(result_option_inspect)]
#![feature(let_chains)]
#![feature(cell_update)]


use proc_macros::make_simple_error_rules;


pub mod text;
pub mod io;
pub mod path;


make_simple_error_rules!(XXXError);
