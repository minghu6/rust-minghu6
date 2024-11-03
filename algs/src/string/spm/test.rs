use std::{char, collections::BTreeMap, ops::Range};

use common::*;

use super::FindStr;

const CN_ALPHA_LIST: [char; 26] = [
    '啊', '吧', '从', '的', '俄', '分', '个', '好', '爱', '就', '看', '了',
    '吗', '你', '哦', '牌', '去', '人', '是', '他', '优', '未', '我', '想',
    '有', '在',
];

const ALPHA_LIST: [char; 26] = [
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o',
    'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
];

const DNA_LIST: [char; 4] = ['A', 'C', 'T', 'G'];



////////////////////////////////////////////////////////////////////////////////
//// Random Char

fn random_char() -> char {
    let i = random_range!(0..CN_ALPHA_LIST.len() + ALPHA_LIST.len());

    if i < ALPHA_LIST.len() {
        ALPHA_LIST[i]
    }
    else {
        CN_ALPHA_LIST[i - 26]
    }
}

// fn random_char() -> char {
//     let i = random_range!(0..ALPHA_LIST.len());

//     ALPHA_LIST[i]
// }


fn random_dna_char() -> char {
    DNA_LIST[random_range!(0..4)]
}



////////////////////////////////////////////////////////////////////////////////
//// Mock Pattern Input

pub fn gen_square_periodic_dna_pattern(
    len_range: (Range<usize>, usize),
    n: usize,
) -> Vec<String> {
    let mut result = vec![];
    let __gen = |length| {
        let mut s = String::with_capacity(length);

        let period = (length as f64).sqrt().floor() as usize;
        let mut period_vec = Vec::with_capacity(period);

        for _ in 0..period {
            period_vec.push(random_dna_char());
        }

        for i in 0..length {
            s.push(period_vec[i % period]);
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


pub fn gen_dna_pattern(
    len_range: (Range<usize>, usize),
    n: usize,
) -> Vec<String> {
    let mut result = vec![];
    let __gen = |length| {
        let mut s = String::with_capacity(length);
        for _ in 0..length {
            s.push(random_dna_char());
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


pub fn gen_pattern(len_range: (Range<usize>, usize), n: usize) -> Vec<String> {
    let mut result = vec![];
    let __gen = |length| {
        let mut s = String::with_capacity(length);
        for _ in 0..length {
            s.push(random_char());
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


////////////////////////////////////////////////////////////////////////////////
//// Mock Source

pub fn gen_random_string(size: usize) -> String {
    let mut s = String::with_capacity(size);
    let mut cur_size = 0;

    while cur_size < size {
        let c = random_char();
        cur_size += c.len_utf8();
        s.push(c)
    }

    s
}


pub fn gen_random_dna_string(size: usize) -> String {
    let mut s = String::with_capacity(size);

    let mut rng = thread_rng();
    let mut cur_size = 0;

    while cur_size < size {
        let c = DNA_LIST[rng.gen_range(0..4 )];
        cur_size += 1;
        s.push(c)
    }

    s
}


////////////////////////////////////////////////////////////////////////////////
//// Test Case


pub fn gen_test_case() -> Vec<(String, String, Vec<usize>)> {
    let mut cases = vec![];

    let strings = vec![
        gen_random_string(0),
        gen_random_string(1),
        gen_random_string(10),
        gen_random_string(100),
        gen_random_string(500),
    ];

    for string in strings {
        for pat in gen_pattern((1..24, 1), 100) {
            let result = pat.find_all(&string).collect();
            // let result = BMPattern::new(pat.as_str()).find_all(string.as_str());

            cases.push((pat, string.clone(), result))
        }
    }

    let dna_strings = [
        gen_random_dna_string(5),
        gen_random_dna_string(20),
        gen_random_dna_string(100),
        gen_random_dna_string(500),
    ];

    for string in dna_strings {
        for pat in gen_pattern((1..24, 1), 50) {
            let result = pat.find_all(&string).collect();
            // let result = BMPattern::new(pat.as_str()).find_all(string.as_str());

            cases.push((pat, string.clone(), result))
        }
    }

    cases
}


/// 每份文本查找多个pattern
pub fn gen_test_case_multiple(
) -> Vec<(Vec<String>, String, BTreeMap<String, Vec<usize>>)> {
    let mut cases = vec![];

    let strings = vec![
        gen_random_string(10000),
        gen_random_string(1000),
        gen_random_string(100),
        gen_random_string(10),
        gen_random_string(1),
        gen_random_string(0),
    ];

    for string in strings {
        let mut result = BTreeMap::new();
        let patterns = gen_pattern((1..24, 1), 5);

        for pat in patterns.iter() {
            result.insert(
                pat.clone(),
                pat.find_all(&string).collect(),
            );
        }

        cases.push((patterns, string.clone(), result))
    }

    cases
}



#[cfg(test)]
mod test {

    use super::{super::{ FindStr, compute_k }, *};

    #[test]
    fn test_gen_pattern() {
        let result = gen_pattern((1..8, 1), 3);

        println!("{:?}", result);
    }

    #[test]
    fn test_gen_text() {
        let text = gen_random_string(5_000);
        println!("text size: {}", text.len());
        println!("{}", text);
    }

    #[test]
    fn test_bf_find_all() {
        assert_eq!(
            "abbaaba".find_all("abbaabbaababbaaba")
                .collect::<Vec<_>>(),
            vec![4, 10]
        );
        assert_eq!(
            "aaa".find_all("aaaaa").collect::<Vec<_>>(),
            vec![0, 1, 2]
        );
        assert_eq!(
            "你好a".find_all("aab你好a, 你好a,hahahah")
                .collect::<Vec<_>>(),
            vec![3, 12]
        );
        assert_eq!(
            "k".find_all("yka你").collect::<Vec<_>>(),
            vec![1]
        );
    }

    #[test]
    fn test_gen_square_periodic_dna_pattern() {
        for pat in gen_square_periodic_dna_pattern((2..100, 1), 5) {
            assert!(compute_k(pat.as_bytes()) > 0);
        }
    }
}
