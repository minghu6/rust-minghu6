#![allow(dead_code)]

pub mod hash;
pub mod math;
pub mod sort;
pub mod spm;

pub(crate) use std::cmp::Ordering;
use std::ops::Range;

use rand::{
    distributions::uniform::{SampleBorrow, SampleUniform},
    Rng,
};


#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub fn hardware_random() -> usize {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        #[cfg(target_arch = "x86")]
        use core::arch::x86::_rdrand32_step as _rdrandsize_step;
        #[cfg(target_arch = "x86_64")]
        use core::arch::x86_64::_rdrand64_step as _rdrandsize_step;

        let mut rand_value = 0;
        unsafe {
            match _rdrandsize_step(&mut rand_value) {
                1 => {
                    return rand_value as usize;
                }
                _ => assert!(false),
            }
        }
    }

    0
}


pub fn random_range<T: SampleUniform, B>(range: Range<B>) -> T
where
    B: SampleBorrow<T> + Sized,
{
    rand::thread_rng().gen_range(range.start, range.end)
}


pub use rand::random;

// pub fn random<T>() -> T where Standard: Distribution<T> {
//     if cfg!(any(
//         target_arch = "x86",
//         target_arch = "x86_64"
//     ))
//     {
//         hardware_random()
//     }
//     else {
//         rand::random()
//     }
// }


// 使用字典序进行cmp比较
fn lexi_cmp<E: Ord>(l1: &[E], l2: &[E]) -> Ordering {
    for (x, y) in l1.iter().zip(l2.iter()) {
        let cmp_res = x.cmp(&y);

        if cmp_res != Ordering::Equal {
            return cmp_res;
        }
    }

    l1.len().cmp(&l2.len())
}


#[cfg(test)]
mod test {
    use super::*;
    use crate::hashset;
    extern crate test;
    use test::Bencher;

    #[test]
    fn hardware_randvalue_works() {
        let mut result = Vec::with_capacity(TIMES);

        for _ in 0..TIMES {
            result.push(random::<usize>());
        }

        assert_ne!(result, vec![0; TIMES]);
    }

    static TIMES: usize = 10000;

    // #[bench]
    // fn bench_soft_random(b: &mut Bencher) {
    //     b.iter(||{
    //         for _ in 0..TIMES {
    //             software_random::<usize>();
    //         }
    //     })
    // }

    #[bench]
    fn bench_hard_random(b: &mut Bencher) {
        b.iter(|| {
            for _ in 0..TIMES {
                hardware_random();
            }
        })
    }

    #[test]
    fn test_lexi_cmp_works_basic() {
        assert_eq!(lexi_cmp(&[1, 2, 3][..], &[1, 2][..]), Ordering::Greater);
        assert_eq!(
            lexi_cmp(&vec![1, 2, 3][..], &vec![1, 2, 4][..]),
            Ordering::Less
        );
        assert_eq!(
            lexi_cmp(&vec![1, 2, 3][..], &vec![1, 2, 3][..]),
            Ordering::Equal
        );
        assert_eq!(
            lexi_cmp(&vec![4, 2, 3][..], &vec![4, 3, 2][..]),
            Ordering::Less
        );
    }


    #[ignore = "just see see"]
    #[test]
    fn stats_random() {
        let mut coll = hashset!();
        let mut odd = 0;
        let mut even = 0;
        let mut dup = 0;
        let mut cont_dup = 0;

        let batch = 1000;
        let range = 100;

        let mut prev = range;

        for _ in 0..batch {
            let v = random::<usize>() % range;

            if random::<usize>() % 2 == 0 {
                even += 1;
            } else {
                odd += 1;
            }

            if !coll.insert(v) {
                dup += 1;
            }

            if prev == v {
                cont_dup += 1;
            }
            prev = v;
        }

        println!("{batch} rounds in 0..{range}");
        println!("odd: {odd}, even: {even}");
        println!("dup: {dup}");
        println!("cont_dup: {cont_dup}");
    }
}
