#![feature(test)]
#![feature(generators, generator_trait)]
#![feature(ptr_internals)]


pub mod text;
pub mod collections;
pub mod algs;


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

