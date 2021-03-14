#![allow(dead_code)]
///! Bloom filter
use twox_hash:: { RandomXxHashBuilder64 };
use bit_vec::BitVec;

use std::f32::consts::LN_2;
use std::marker::PhantomData;
use std::hash::{ Hash, BuildHasher };
use core::hash::Hasher;


pub trait BloomFilter<T> {
    fn insert(&mut self, item: &T);

    fn contains(&self, item: &T) -> bool;
}

fn optimal_k(m:usize, n:usize) -> usize {
    let k = ((m as f32 / n as f32) * LN_2).round() as usize;
    if k < 1 {
        1
    } else {
        k
    }
}

// calculate false positive rate
fn fp_rate(m:usize, n:usize, k:usize) -> f32 {
    (1f32 - (1f32 - 1f32 / m as f32).powi((k * n) as i32)).powi(k as i32)
}

/// -> (k, m)
fn find_proper_params(n:usize, max_fp_rate:f32) -> (usize, usize) {
    let mut m = 1;
    let step = 1;
    let mut k = optimal_k(m, n);

    while fp_rate(m, n ,k) > max_fp_rate {
        m += step;
        k = optimal_k(m, n);
    }

    (k, m)
}

/// patlen: 5   fp_rate: 0.03
///
/// patlen: 10  fp_rate: 0.07
///
/// patlen: 20  fp_rate: 0.14
///
/// patlen: 50  fp_rate: 0.32
///
/// patlen: 100 fp_rate: 0.54
///
/// patlen: 200 fp_rate: 0.79
///
/// patlen: 300 fp_rate: 0.90
pub struct BytesBloomFilter {
    mask: u64,
}

impl BytesBloomFilter {
    pub fn new() -> Self {
        BytesBloomFilter {
            mask: 0,
        }
    }
}

/// fp rate 0.5
impl BloomFilter<u8> for BytesBloomFilter {
    #[inline]
    fn insert(&mut self, elem: &u8) {
        (self.mask) |= 1u64 << (elem & 63);
    }

    #[inline]
    fn contains(&self, elem: &u8) -> bool {
        (self.mask & (1u64 << (elem & 63))) != 0
    }
}

pub struct BytesBloomFilter64 {
    mask: u64,
    k: u8
}

impl BytesBloomFilter64 {
    pub fn with_len(n: usize) -> Self {
        let mask = 0;
        let mut k = ((64.0 / n as f32) * LN_2).round() as u8;
        if k < 1 {
            k = 1
        }

        BytesBloomFilter64 {
            mask, k
        }
    }
}

/// fp rate 0.5
impl BloomFilter<u8> for BytesBloomFilter64 {
    fn insert(&mut self, elem: &u8) {
        for i in 0..self.k {
            let x = (*elem as u16 + i as u16) % 255;
            (self.mask) |= 1u64 << (x & 63)
        }
    }

    fn contains(&self, elem: &u8) -> bool {
        (0..self.k).all(|i|{
            (self.mask & (1u64 << (elem+i & 63))) != 0
        })
    }
}

// pub struct BytesBloomFilter {
//     bits: BitVec,
//     k: usize,
// }

// impl BytesBloomFilter {
//     pub fn with_size(m:usize, k:usize) -> Self {
//         assert!(k > 0);

//         let bits = BitVec::from_elem(m, false);

//         BytesBloomFilter {
//             bits, k
//         }
//     }

//     pub fn with_rate(n:usize, max_fp_rate: f32) -> Self {
//         let (mut k, mut m) = find_proper_params(n as usize, max_fp_rate);

//         if m >= 256 {
//             m = 256;
//             k = 1;
//         }

//         BytesBloomFilter::with_size(m as usize, k as usize)
//     }
// }

// impl BloomFilter<u8> for BytesBloomFilter {
//     fn insert(&mut self, byte: &u8) {
//         let bitslen = self.bits.len();
//         let mut
//         if bitslen = 256 {

//         }

//         for i in 0..self.k {

//         }
//     }
// }

pub struct FastBloomFilter<T: Hash> {
    bits: BitVec,
    hashbuilders: Vec<RandomXxHashBuilder64>,
    _marker: PhantomData<T>
}

impl<T: Hash> FastBloomFilter<T> {
    pub fn with_size(m:usize, k:usize) -> Self {
        assert!(k > 0);

        let bits = BitVec::from_elem(m, false);
        let mut hashbuilders = Vec::with_capacity(k);

        for _ in 0..k {
            hashbuilders.push(RandomXxHashBuilder64::default());
        }

        FastBloomFilter {
            bits, hashbuilders, _marker: PhantomData::<T>
        }
    }

    pub fn with_rate(n:usize, max_fp_rate: f32) -> Self {
        let (k, m) = find_proper_params(n as usize, max_fp_rate);

        FastBloomFilter::with_size(m as usize, k as usize)
    }
}


impl<T: Hash> BloomFilter<T> for FastBloomFilter<T> {
    fn insert(&mut self, item: &T) {
        let bits_len = self.bits.len();
        let bits = &mut self.bits;

        self.hashbuilders.iter_mut().for_each(|builder| {
            let mut hasher = builder.build_hasher();
            item.hash(&mut hasher);
            bits.set(hasher.finish() as usize % bits_len, true);
        })
    }

    fn contains(&self, item: &T) -> bool {
        let bits_len = self.bits.len();
        let bits = &self.bits;

        self.hashbuilders.iter().all(|builder| {
            let mut hasher = builder.build_hasher();

            item.hash(&mut hasher);
            if let Some(value) = bits.get(hasher.finish() as usize % bits_len) {
                return value
            } else {
                panic!("");
            }
        })
    }
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn simple_bloom_works() {
        // let mut sbf = BytesBloomFilter::new();
        // sbf.insert(&0u8);
        // for i in 0..255u8 {
        //     sbf.insert(&(i+1));

        //     for j in i+2..255u8 {
        //         if sbf.contains(&(j+1)) {
        //             println!("false negative case i:{}, j:{}", i+2, j+1);
        //             break;
        //         }
        //     }
        // }

        // sbf.insert(&255u8);
        // assert!(false);
    }

    #[test]
    fn bloom_filter_fp_rate() {
        let mut fp_rate_value;

        for patlen in &[5, 10, 20, 50, 100, 200, 300, 500] {
            fp_rate_value = fp_rate(128, *patlen as usize, 1);
            println!("128: patlen: {} fp_rate: {}",patlen, fp_rate_value);
        }

        for patlen in &[5, 10, 20, 50, 100, 200, 300, 500] {
            fp_rate_value = fp_rate(64, *patlen as usize, 1);
            println!("64: patlen: {} fp_rate: {}",patlen, fp_rate_value);
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
    fn fast_bloom_filter_works() {
        let mut fbf = FastBloomFilter::with_rate(20, 0.15);

        for i in 0..255u8 {
            fbf.insert(&i);

            assert!(fbf.contains(&i));
        }
    }
}