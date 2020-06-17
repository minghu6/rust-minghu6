#![allow(dead_code)]

use std::collections::{ HashMap };
use std::collections::hash_map;
use std::hash::{Hash, Hasher};
use std::iter:: { Map };
use std::fmt;

pub type GetKeyType<T> = fn(&T) -> String;
pub type Map2SetType<T> = fn((String, T)) -> T;

pub struct CustomHashSet<T> {
    get_key: GetKeyType<T>,
    _value_map: HashMap<String, T>,
}

impl<T> CustomHashSet<T> where T:Hash + Clone {
    fn default_get_key(value:&T) -> String where T: Hash {
        let mut hasher = hash_map::DefaultHasher::new();
        value.hash(&mut hasher);
        hasher.finish().to_string()
    }

    pub fn new(get_key_option:Option<GetKeyType<T>>) -> Self {
        let _value_map:HashMap<String, T> = HashMap::new();
        let get_key;

        if let Some(passed_get_key) = get_key_option {
            get_key = passed_get_key;
        } else {
            get_key = CustomHashSet::default_get_key;
        }

        CustomHashSet {
            get_key,
            _value_map,
        }
    }

    pub fn insert(&mut self, value:T) {
        let key = (self.get_key)(&value);

        self._value_map.insert(key, value);
    }

    pub fn contains(&self, value: &T) -> bool {
        let key = &(self.get_key)(value);

        self._value_map.contains_key(key)
    }

    // Rust doesn't open the constructor method for struct Draw
    pub fn drain(&mut self) -> IteratorWrapper<Map<hash_map::Drain<String, T>, Map2SetType<T>>, T> {
        IteratorWrapper::new(self._value_map.drain().map(|(_, v)| v))
    }

    pub fn remove(&mut self, value:&T) -> bool {
        let key = &(self.get_key)(value);

        match self._value_map.remove(key) {
            None => false,
            _ => true
        }
    }

    pub fn take(&mut self, value:&T) -> Option<T> {
        let key = &(self.get_key)(value);

        self._value_map.remove(key)
    }

    pub fn get(&mut self, value:&T) -> Option<&T> {
        let key = &(self.get_key)(value);

        self._value_map.get(key)
    }

    pub fn len(&self) -> usize {
        return self._value_map.len();
    }

    pub fn iter(&self) -> hash_map::Values<String, T> {
        self._value_map.values()
    }

    pub fn is_subset(&self, other: &Self) -> bool {
        self.iter().all(|x| other.contains(&x))
    }

    pub fn is_superset(&self, other: &Self) -> bool {
        other.iter().all(|x| self.contains(&x))
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn is_disjoint(&self, other: &Self) -> bool {
        self.union(other).is_empty()
    }

    pub fn intersection<'a>(&'a self, other: &'a Self) -> Self {
        let mut new_set = CustomHashSet::new(Some(self.get_key));
        for v in self.iter().chain(other.iter()) {
            new_set.insert(v.clone())
        }

        new_set
    }

    pub fn union<'a>(&'a self, other: &'a Self) -> Self {
        let mut new_set = CustomHashSet::new(Some(self.get_key));

        for v in self.iter().filter(|v| other.contains(v)) {
            new_set.insert(v.clone())
        }

        new_set
    }

    pub fn difference<'a>(&'a self, other: &'a Self) -> Self {
        let mut new_set = CustomHashSet::new(Some(self.get_key));

        for v in self.iter().filter(|v| !other.contains(v)) {
            new_set.insert(v.clone())
        }

        new_set
    }

    pub fn symmetric_difference<'a>(&'a self, other: &'a Self) -> Self {
        let mut new_set = CustomHashSet::new(Some(self.get_key));

        for v in self.iter().filter(|v| !other.contains(v)) {
            new_set.insert(v.clone())
        }

        for v in other.iter().filter(|v| !self.contains(v)) {
            new_set.insert(v.clone())
        }

        new_set
    }
}


impl<T> IntoIterator for CustomHashSet<T> {
    type Item = T;
    type IntoIter = Map<hash_map::IntoIter<String, T>, Map2SetType<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self._value_map.into_iter().map(|(_, v)| v)
    }
}

impl<T> PartialEq for CustomHashSet<T> where T: Hash + Clone {
    fn eq(&self, other: &Self) -> bool {
        self.is_subset(other) && other.is_subset(self)
    }
}

impl<T> fmt::Debug for CustomHashSet<T> where T: fmt::Debug {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CustomHashSet")
         .field("_value_map", &self._value_map)
         .finish()
    }
}

/// Just for hide abstraction
pub struct IteratorWrapper<I, T> where I: Iterator<Item=T> {
    iter: I,
}

impl<I, T> IteratorWrapper<I, T> where I: Iterator<Item=T> {
    pub fn new(iter: I) -> IteratorWrapper<I, T> where I: Iterator {
        IteratorWrapper {
            iter,
        }
    }
}

impl<I, T> Iterator for IteratorWrapper<I, T>  where I: Iterator<Item=T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        self.iter.next()
    }
}

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

/// k=1, fp_rate很高, 不建议使用， 支持0-255的输入。
impl SimpleBloomFilter {
    pub fn new() -> Self {
        SimpleBloomFilter {
            mask: 0,
        }
    }

    pub fn add(&mut self, char: &u8) {
        // let t = char & (self.bloom_width -1);
        // println!("{}", t);
        // 100u128 << 127u8;
        (self.mask) |= 1u128 << (char & (127u8));
    }

    pub fn contains(&self, char: &u8) -> bool {
        (self.mask & (1u128<< (char & (127u8)))) != 0
    }
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn simple_bloom_works() {
        let mut sbf = SimpleBloomFilter::new();
        sbf.add(&0u8);
        for i in 0..255u8 {
            sbf.add(&(i+1));

            for j in i+2..255u8 {
                if sbf.contains(&(j+1)) {
                    println!("false negative case i:{}, j:{}", i+2, j+1);
                    break;
                }
            }
        }

        sbf.add(&255u8);
        assert!(false);
    }

    #[test]
    fn bloom_filter_fp_rate() {
        let fp_rate = BloomFilter::fp_rate(1024, 256, 1);

        println!("fp_rate: {}", fp_rate);
    }

    #[test]
    fn bloom_filter_proper_params() {
        let (k, m) = BloomFilter::find_proper_params(256, 0.15);

        println!("capacity: {} Bytes, k: {}", m/8, k);
    }
}