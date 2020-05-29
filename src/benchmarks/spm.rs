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


pub struct TestCaseGenerator {

}

impl TestCaseGenerator {
    // fn gen_pattern () -> Vec<&str> {

    // }
}

/// pattern type:
///     all_same
///
///
// pub struct PatternGenerator {
//     len_range: Range<usize>,
// }

// impl PatternGenerator {
//     pub fn new(range: Range<usize>) -> Self {
//         PatternGenerator {
//             len_range: range
//         }
//     }


// }

pub fn gen_pattern<'a>(len_range:(Range<usize>, usize), n:usize) -> Vec<String> {
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

fn random_char(regular_rand_value: usize) -> char {
    if regular_rand_value < 26 {
        ALPHA_LIST[regular_rand_value]
    } else {
        CN_ALPHA_LIST[regular_rand_value-26]
    }
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

    #[bench]
    fn gen_some_random_text(b: &mut Bencher) {
        b.iter(|| gen_random_text(5_000_000))
    }
}