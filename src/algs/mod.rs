#![allow(dead_code)]

pub mod spm;
pub mod sort;

use std::cmp::Ordering;

#[cfg(
    all(
        any(target_arch = "x86", target_arch = "x86_64"),
    )
)]
pub fn hardware_randvalue() -> usize {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        #[cfg(target_arch="x86")]
        use core::arch::x86::_rdrand32_step as _rdrandsize_step;

        #[cfg(target_arch="x86_64")]
        use core::arch::x86_64::_rdrand64_step as _rdrandsize_step;

        let mut rand_value = 0;
        unsafe {
            match _rdrandsize_step(&mut rand_value) {
                1 => {
                   return rand_value as usize;
                },
                _ => assert!(false),
            }
        }
    }

    0
}

// 使用字典序进行cmp比较
fn lexi_cmp<E: Ord>(l1: &[E], l2: &[E]) -> Ordering {
    for (x, y) in l1.iter().zip(l2.iter()) {
        let cmp_res = x.cmp(&y);

        if cmp_res != Ordering::Equal { return cmp_res; }
    }

    l1.len().cmp(&l2.len())
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn hardware_randvalue_works() {
        let times = 100;
        let mut result = Vec::with_capacity(times);

        for _ in 0..times {
            result.push(hardware_randvalue());
        }

        assert_ne!(result, vec![0;times]);
    }

    #[test]
    fn test_lexi_cmp_works_basic() {
        assert_eq!(lexi_cmp(&[1, 2, 3][..], &[1, 2][..]), Ordering::Greater);
        assert_eq!(lexi_cmp(&vec![1, 2, 3][..], &vec![1, 2, 4][..]), Ordering::Less);
        assert_eq!(lexi_cmp(&vec![1, 2, 3][..], &vec![1, 2, 3][..]), Ordering::Equal);
        assert_eq!(lexi_cmp(&vec![4, 2, 3][..], &vec![4, 3, 2][..]), Ordering::Less);
    }
}