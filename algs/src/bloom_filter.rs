//! Bloom filter

use std::f32::consts::LN_2;


////////////////////////////////////////////////////////////////////////////////
//// Trait

pub trait BloomFilter<T> {
    fn insert(&mut self, item: &T);

    fn contains(&self, item: &T) -> bool;
}


////////////////////////////////////////////////////////////////////////////////
//// Structures

pub struct BytesBloomFilter64 {
    data: u64,
    k: u8,
}

////////////////////////////////////////////////////////////////////////////////
//// Implementations

impl BytesBloomFilter64 {
    pub fn with_len(n: usize) -> Self {
        let mask = 0;
        let mut k = ((64.0 / n as f32) * LN_2).round() as u8;
        if k < 1 {
            k = 1
        }

        BytesBloomFilter64 { data: mask, k }
    }
}

/// fp rate 0.5
impl BloomFilter<u8> for BytesBloomFilter64 {
    fn insert(&mut self, elem: &u8) {
        for i in 0..self.k {
            let x = (*elem as u16 + i as u16) % 255;
            (self.data) |= 1u64 << (x & 63)
        }
    }

    fn contains(&self, elem: &u8) -> bool {
        (0..self.k).all(|i| (self.data & (1u64 << (elem + i & 63))) != 0)
    }
}


#[cfg(any(test))]
pub use tests::HashBloomFilter;



#[cfg(test)]
pub mod tests {
    use super::*;
    use std::{
        f32::consts::LN_2,
        hash::{ BuildHasherDefault, Hash },
        marker::PhantomData,
    };

    use std::hash::{ BuildHasher, Hasher };

    use bit_vec::BitVec;
    use twox_hash::XxHash64;


    ////////////////////////////////////////////////////////////////////////////////
    //// Structures

    pub struct HashBloomFilter<T: Hash> {
        bits: BitVec,
        hashbuilder: BuildHasherDefault<XxHash64>,
        _marker: PhantomData<T>,
    }

    ////////////////////////////////////////////////////////////////////////////////
    //// Implementations

    impl<T: Hash> HashBloomFilter<T> {
        pub fn with_size(m: usize, k: usize) -> Self {
            debug_assert!(k > 0);

            let bits = BitVec::from_elem(m, false);
            // let hashbuilder = RandomXxHashBuilder64::default();
            let hashbuilder = BuildHasherDefault::<XxHash64>::default();

            HashBloomFilter {
                bits,
                hashbuilder,
                _marker: PhantomData::<T>,
            }
        }

        pub fn with_rate(n: usize, max_fp_rate: f32) -> Self {
            let (k, m) = find_proper_params(n as usize, max_fp_rate);

            HashBloomFilter::with_size(m as usize, k as usize)
        }
    }

    impl<T: Hash> BloomFilter<T> for HashBloomFilter<T> {
        fn insert(&mut self, item: &T) {
            let bits_len = self.bits.len();
            let bits = &mut self.bits;

            let mut hasher = self.hashbuilder.build_hasher();
            item.hash(&mut hasher);
            bits.set(hasher.finish() as usize % bits_len, true);
        }

        fn contains(&self, item: &T) -> bool {
            let bits_len = self.bits.len();
            let bits = &self.bits;
            let mut hasher = self.hashbuilder.build_hasher();

            item.hash(&mut hasher);
            bits.get(hasher.finish() as usize % bits_len).unwrap()
        }
    }


    ////////////////////////////////////////////////////////////////////////////////
    //// Functions

    fn optimal_k(m: usize, n: usize) -> usize {
        let k = ((m as f32 / n as f32) * LN_2).round() as usize;
        if k < 1 {
            1
        } else {
            k
        }
    }

    // calculate false positive rate
    fn fp_rate(m: usize, n: usize, k: usize) -> f32 {
        (1f32 - (1f32 - 1f32 / m as f32).powi((k * n) as i32)).powi(k as i32)
    }

    /// -> (k, m)
    fn find_proper_params(n: usize, max_fp_rate: f32) -> (usize, usize) {
        let mut m = 1;
        let step = 1;
        let mut k = optimal_k(m, n);

        while fp_rate(m, n, k) > max_fp_rate {
            m += step;
            k = optimal_k(m, n);
        }

        (k, m)
    }


    extern crate test;

    use test::Bencher;

    use crate::string::gen_random_string;

    #[test]
    fn bloom_filter_fp_rate() {
        let mut fp_rate_value;

        for m in [8, 16, 32, 64, 128, 256] {
            for patlen in &[5, 10, 20, 50, 100, 200, 300, 500] {
                fp_rate_value = fp_rate(m, *patlen as usize, 1);

                println!("{m}: patlen: {patlen} fp_rate: {fp_rate_value}");
            }
        }
    }

    #[test]
    fn bloom_filter_proper_params() {
        // let (k, m) = find_proper_params(30, 0.3);

        // println!("capacity: {} Bytes, k: {}", m/8, k);

        println!("k :{}", optimal_k(64, 2));

        // println!("{}", (4 as f32 / 2 as f32) * LN_2);
    }

    #[test]
    fn hash_bloom_filter_works() {
        let mut fbf = HashBloomFilter::with_rate(20, 0.15);

        for i in 0..255u8 {
            fbf.insert(&i);

            assert!(fbf.contains(&i));
        }
    }


    fn gen_test_text() -> String {
        gen_random_string(10000)
    }

    #[bench]
    fn simple_bloom_filter_basic_op(b: &mut Bencher) {
        let generate = || {
            let mut bloom_filter = BytesBloomFilter64::with_len(6);

            for b in gen_test_text().as_bytes() {
                bloom_filter.insert(&b);

                bloom_filter.contains(&b);
            }
        };

        b.iter(|| generate())
    }

    #[bench]
    fn fast_bloom_filter_basic_op(b: &mut Bencher) {
        let generate = || {
            let mut bloom_filter = HashBloomFilter::with_rate(100, 0.15);

            for b in gen_test_text().as_bytes() {
                bloom_filter.insert(&b);

                bloom_filter.contains(&b);
            }
        };

        b.iter(|| generate())
    }

    #[bench]
    fn bitvec_bloom_filter_basic_op(b: &mut Bencher) {
        let generate = || {
            let mut bits = BitVec::from_elem(128, false);

            for b in gen_test_text().as_bytes() {
                for i in 0..3 {
                    bits.set((b+i & 127).into(), true);
                }

                for i in 0..3 {
                    if let Some(v) = bits.get((b+i & 127) as usize) {
                        if v {
                            let _ = 1+1;
                        } else{
                            let _ = 2+2;
                        }

                    } else {
                        panic!("");
                    }
                }
            }
        };

        b.iter(|| generate())
    }
}
