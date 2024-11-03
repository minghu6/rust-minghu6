#[macro_export]
macro_rules! random_range {
    ($r:expr) => {{
        use std::{
            borrow::Borrow,
            ops::{Add, Bound::*, RangeBounds},
        };

        use common::rand::{
            thread_rng,
            Rng,
            distributions::uniform::{
                SampleBorrow, SampleUniform,
            }
        };

        let range = $r;

        let start;
        let end;

        match range.start_bound() {
            Included(v) => start = *SampleBorrow::borrow(&v),
            Excluded(v) => start = *SampleBorrow::borrow(&v) + 1,
            Unbounded => panic!("Unsupported unbound range"),
        }

        match range.end_bound() {
            Included(v) => end = *SampleBorrow::borrow(&v) + 1,
            Excluded(v) => end = *SampleBorrow::borrow(&v),
            Unbounded => panic!("Unsupported unbound range"),
        }

        thread_rng().gen_range(start..end)
    }};
}


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


// pub fn random_range<T, B, R>(range: R) -> T
// where
//     T: SampleUniform + Add<usize, Output = T> + Copy,
//     B: SampleBorrow<T> + Sized,
//     R: RangeBounds<B>
// {
//     let start;
//     let end;

//     match range.start_bound() {
//         Included(v) => start = *v.borrow(),
//         Excluded(v) => start = *v.borrow() + 1,
//         Unbounded => panic!("Unsupported unbound range"),
//     }

//     match range.end_bound() {
//         Included(v) => end = *v.borrow() + 1,
//         Excluded(v) => end = *v.borrow(),
//         Unbounded => panic!("Unsupported unbound range"),
//     }

//     extern_rand::thread_rng().gen_range(start, end)
// }


pub use extern_rand::{prelude::*, *};



#[cfg(test)]
mod test {
    use std::collections::HashSet;

    use super::*;
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

    #[bench]
    fn bench_hard_random(b: &mut Bencher) {
        b.iter(|| {
            for _ in 0..TIMES {
                hardware_random();
            }
        })
    }

    #[ignore = "just see see"]
    #[test]
    fn stats_random() {
        let mut coll = HashSet::new();
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
