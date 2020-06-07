#![feature(test)]
#![feature(generators, generator_trait)]
#![feature(ptr_internals)]
#![feature(const_fn)]

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

