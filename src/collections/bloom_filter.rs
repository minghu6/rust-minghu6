/// Bloom filter
pub struct BloomFilter {

}

impl BloomFilter {
    fn optimal_k(m:i32, n:i32) -> i32 {
        let k = (m as f32 / n as f32 * 2f32.ln()).round() as i32;

        if k < 1 {
            1
        } else {
            k
        }
    }

    // calculate false positive rate
    fn fp_rate(m:i32, n:i32, k:i32) -> f32 {
        (1f32 - (1f32 - 1f32 / m as f32).powi(k * n)).powi(k)
    }

    /// -> (k, m)
    fn find_proper_params(n:i32, max_fp_rate:f32) -> (i32, i32) {
        let mut m = n;
        let step = n;
        let mut k = BloomFilter::optimal_k(m, n);

        while BloomFilter::fp_rate(m, n ,k) > max_fp_rate {
            m += step;
            k = BloomFilter::optimal_k(m, n);
        }

        (k, m)
    }
}



pub struct SimpleBloomFilter {
    mask: u128,
}

/// fp rate 0.5
impl SimpleBloomFilter {
    pub fn new() -> Self {
        SimpleBloomFilter {
            mask: 0,
        }
    }

    pub fn insert(&mut self, char: &u8) {
        (self.mask) |= 1u128 << (char & (127u8));
    }

    pub fn contains(&self, char: &u8) -> bool {
        (self.mask & (1u128 << (char & (127u8)))) != 0
    }
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn simple_bloom_works() {
        let mut sbf = SimpleBloomFilter::new();
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
        let mut fp_rate;

        for patlen in &[5, 10, 20, 50, 100, 200, 300, 500, 100] {
            fp_rate = BloomFilter::fp_rate(128, *patlen as i32, 1);
            println!("patlen: {}fp_rate: {}",patlen, fp_rate);
        }
    }

    #[test]
    fn bloom_filter_proper_params() {
        let (k, m) = BloomFilter::find_proper_params(20, 0.15);

        println!("capacity: {} Bytes, k: {}", m/8, k);
    }
}