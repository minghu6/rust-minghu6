#![allow(dead_code)]

pub mod spm;
pub mod sort;
pub mod math;
pub mod hash;

use std::{cmp::Ordering, cell::RefCell};

use rand::Rng;

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub fn hardware_random() -> usize {
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

/// More fast than hardware_random
pub fn software_random() -> usize {
    #[allow(unused_imports)]
    use rand;

    thread_local! {
        static RNG: RefCell<rand::rngs::ThreadRng> = RefCell::new(rand::thread_rng());
    }

    RNG.with(|rngcell| rngcell.borrow_mut().gen::<usize>())
}

pub fn random() -> usize {
    if cfg!(any(
        target_arch = "x86",
        target_arch = "x86_64"
    ))
    {
        hardware_random()
    }
    else {
        software_random()
    }
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
    extern crate test;
    use test::Bencher;

    #[test]
    fn hardware_randvalue_works() {
        let mut result = Vec::with_capacity(TIMES);

        for _ in 0..TIMES {
            result.push(random());
        }

        assert_ne!(result, vec![0;TIMES]);
    }

    static TIMES: usize = 10000;

    #[bench]
    fn bench_soft_random(b: &mut Bencher) {
        b.iter(||{
            for _ in 0..TIMES {
                software_random();
            }
        })
    }

    #[bench]
    fn bench_hard_random(b: &mut Bencher) {
        b.iter(||{
            for _ in 0..TIMES {
                hardware_random();
            }
        })
    }

    #[test]
    fn test_lexi_cmp_works_basic() {
        assert_eq!(lexi_cmp(&[1, 2, 3][..], &[1, 2][..]), Ordering::Greater);
        assert_eq!(lexi_cmp(&vec![1, 2, 3][..], &vec![1, 2, 4][..]), Ordering::Less);
        assert_eq!(lexi_cmp(&vec![1, 2, 3][..], &vec![1, 2, 3][..]), Ordering::Equal);
        assert_eq!(lexi_cmp(&vec![4, 2, 3][..], &vec![4, 3, 2][..]), Ordering::Less);
    }

    #[ignore = "just see see"]
    #[test]
    fn stats_random() {
        let mut odd = 0;
        let mut even = 0;

        for _ in 0..1000 {
            if random() % 2 == 0{
                even += 1;
            }
            else {
                odd += 1;
            }
        }

        println!("odd: {odd}, even: {even}");

    }
}
