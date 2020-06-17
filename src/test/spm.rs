//! This mod is AKA String-Pattern-Matching
#![allow(dead_code)]

use rand::prelude::*;

use core::arch::x86_64::_rdrand64_step;

use std::ops::Range;
use std::char;


const CN_ALPHA_LIST:[char;26] = [
    '啊','吧','从','的','俄',
    '分','个','好','爱','就',
    '看','了','吗','你','哦',
    '牌','去','人','是','他',
    '优','未','我','想','有',
    '在',
];

const ALPHA_LIST:[char;26] = [
    'a','b','c','d','e',
    'f','g','h','i','j',
    'k','l','m','n','o',
    'p','q','r','s','t',
    'u','v','w','x','y',
    'z',
];

const DNA_LIST:[char;4] = [
    'A', 'C', 'T', 'G'
];


pub fn gen_pattern(len_range:(Range<usize>, usize), n:usize) -> Vec<String> {
    let mut result = vec![];
    let __gen = |length| {
        let mut s = String::with_capacity(3 * length);
        for _ in 0..length {
            let mut rand_value = 0;
            unsafe {
                match _rdrand64_step(&mut rand_value) {
                    1 => {
                        s.push(random_char((rand_value % 52) as usize))
                    },
                    _ => assert!(false),
                }
            }
        }
        s
    };

    let (range, step) = len_range;

    for len in range.step_by(step) {
        for _ in 0..n {
            result.push(__gen(len));
        }
    }

    result
}


pub fn gen_dna_pattern(len_range:(Range<usize>, usize), n:usize) -> Vec<String> {
    let mut result = vec![];
    let __gen = |length| {
        let mut s = String::with_capacity(length);
        for _ in 0..length {
            let mut rand_value = 0;
            unsafe {
                match _rdrand64_step(&mut rand_value) {
                    1 => {
                        s.push(random_dna_char((rand_value % 4) as usize))
                    },
                    _ => assert!(false),
                }
            }
        }
        s
    };

    let (range, step) = len_range;

    for len in range.step_by(step) {
        for _ in 0..n {
            result.push(__gen(len));
        }
    }

    result
}


fn random_char(regular_rand_value: usize) -> char {
    if regular_rand_value < 26 {
        ALPHA_LIST[regular_rand_value]
    } else {
        CN_ALPHA_LIST[regular_rand_value-26]
    }
}


fn random_dna_char(regular_rand_value: usize) -> char {
    DNA_LIST[regular_rand_value]
}


pub fn gen_random_text(size:usize) -> String {
    let mut s = String::with_capacity(size);

    let mut rng = thread_rng();
    let mut cur_size = 0;

    while cur_size < size {
        let c = random_char(rng.gen_range(0, 52));
        cur_size += c.len_utf8();
        s.push(c)
    }

    s
}


pub fn gen_random_dna_text(size:usize) -> String {
    let mut s = String::with_capacity(size);

    let mut rng = thread_rng();
    let mut cur_size = 0;

    while cur_size < size {
        let c = random_dna_char(rng.gen_range(0, 4));
        cur_size += 1;
        s.push(c)
    }

    s
}

pub fn brute_force_match<'a>(pattern:&'a str, text:&'a str) -> Vec<usize> {
    let mut result = vec![];

    for (i, _) in text.char_indices() {
        if let Some(text_slice) = text.get(i..i+pattern.len()) {
            if text_slice == pattern {
                result.push(i);
            }
        }
    }

    result
}

pub fn gen_test_case() -> Vec<(String, String, Vec<usize>)>{
    let mut cases = vec![];
    let texts = vec![
        gen_random_text(1000),
        gen_random_text(100),
        gen_random_text(10),
        gen_random_text(1),
        gen_random_text(0)
    ];

    for text in texts {
        for pat in gen_pattern((1..24, 1), 100) {
            let result = brute_force_match(pat.as_str(), text.as_str());
            cases.push((pat, text.clone(), result))
        }
    }

     cases
}

#[cfg(test)]
mod tests {
    extern crate test;

    use test::Bencher;

    use super::*;

    #[test]
    fn gen_pattern_works() {
        let result = gen_pattern((1..8, 1), 3);

        println!("{:?}", result);
    }

    #[test]
    fn gen_text_works() {
        let text = gen_random_text(5_000);
        println!("text size: {}", text.len());
        println!("{}", text);
    }

    #[test]
    fn bf_find_all_works() {
        assert_eq!(brute_force_match("abbaaba", "abbaabbaababbaaba"), vec![4, 10]);
        assert_eq!(brute_force_match("aaa", "aaaaa"), vec![0, 1, 2]);
        assert_eq!(brute_force_match("你好a", "aab你好a, 你好a,hahahah"), vec![3, 12]);
        assert_eq!(brute_force_match("k", "yka你"), vec![1]);
    }

    #[bench]
    fn gen_some_random_text(b: &mut Bencher) {
        b.iter(|| gen_random_text(5_000_000))
    }
}