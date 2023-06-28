#![allow(incomplete_features)]

#![feature(test)]
#![feature(generator_trait)]
#![feature(generators)]
#![feature(iter_from_generator)]
#![feature(is_sorted)]
#![feature(exclusive_range_pattern)]
#![feature(generic_const_exprs)]
#![feature(inline_const)]
#![feature(maybe_uninit_uninit_array)]
#![feature(generic_arg_infer)]
#![feature(associated_const_equality)]
// #![feature(const_for)]
// #![feature(const_trait_impl)]
// #![feature(const_iter)]
// #![feature(const_intoiterator_identity)]
// #![feature(const_mut_refs)]


use std::cmp::Ordering::{ self, * };


pub mod hash;
pub mod sort;
pub mod string;
pub mod bloom_filter;


/// 使用字典序进行cmp比较
pub fn lexi_cmp<E: Ord>(l1: &[E], l2: &[E]) -> Ordering {
    for (x, y) in l1.iter().zip(l2.iter()) {
        let cmp_res = x.cmp(&y);

        if cmp_res != Equal {
            return cmp_res;
        }
    }

    l1.len().cmp(&l2.len())
}



#[cfg(test)]
mod test {

    use std::cmp::Ordering::*;

    use super::*;

    #[test]
    fn test_lexi_cmp_works_basic() {
        assert_eq!(lexi_cmp(&[1, 2, 3][..], &[1, 2][..]), Greater);
        assert_eq!(
            lexi_cmp(&vec![1, 2, 3][..], &vec![1, 2, 4][..]),
            Less
        );
        assert_eq!(
            lexi_cmp(&vec![1, 2, 3][..], &vec![1, 2, 3][..]),
            Equal
        );
        assert_eq!(
            lexi_cmp(&vec![4, 2, 3][..], &vec![4, 3, 2][..]),
            Less
        );
    }
}
