#![feature(test)]
#![feature(generators, generator_trait)]
#![feature(ptr_internals)]
#![feature(const_fn)]
#![feature(stmt_expr_attributes)]
#![feature(is_sorted)]
#![feature(type_ascription)]

pub mod text;
pub mod collections;
pub mod algs;
pub mod test;


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

