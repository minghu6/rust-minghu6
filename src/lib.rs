#![allow(incomplete_features)]

#![feature(test)]
#![feature(coroutines, coroutine_trait)]
#![feature(stmt_expr_attributes)]
#![feature(type_ascription)]
#![feature(trait_alias)]
#![feature(int_roundings)]
#![feature(macro_metavar_expr)]
#![feature(rustc_private)]
#![feature(type_alias_impl_trait)]
#![feature(iter_from_coroutine)]
#![feature(trace_macros)]
#![feature(let_chains)]
#![feature(f16)]
#![feature(debug_closure_helpers)]
#![feature(adt_const_params)]
#![feature(const_trait_impl)]
#![feature(f128)]
#![feature(str_from_utf16_endian)]
#![feature(iter_collect_into)]

pub mod text;
pub mod io;
pub mod path;
pub mod unicode;
pub mod float;
pub mod int;
