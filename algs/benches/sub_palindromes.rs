#![feature(test)]
extern crate test;

use std::hint::black_box;

use lazy_static::lazy_static;
use m6_algs::string::{
    create_npows, find_longest_palindromes_hash_dp,
    find_longest_palindromes_hash_native, find_sub_palindromes_brute_force,
    find_sub_palindromes_manacher, find_sub_palindromes_manacher_unify,
    gen_random_dna_text, sub_palindromes_to_longest_palindromes, AlphaBet,
    DigitsLetters,
};
use test::Bencher;

const TEXT_SZ: usize = 1_000_000;

lazy_static! {
    static ref TEXTS: Vec<Vec<char>> = {
        vec![TEXT_SZ; 100]
            .into_iter()
            .map(|size| gen_random_dna_text(size).chars().collect())
            .collect()
    };
    static ref NPOWS: Vec<[u64; 1]> = create_npows::<1>(DigitsLetters.prime(), TEXT_SZ);
}


#[bench]
fn bench_find_longest_palindromes_brute_force(b: &mut Bencher) {
    b.iter(|| {
        for chars in TEXTS.iter() {
            let (d1, d2) = black_box(find_sub_palindromes_brute_force(chars));
            black_box(sub_palindromes_to_longest_palindromes((&d1, &d2)));
        }
    })
}


#[ignore = "too slow"]
#[bench]
fn bench_find_longest_palindromes_hash_native(b: &mut Bencher) {
    let alphabet = DigitsLetters;

    b.iter(|| {
        for chars in TEXTS.iter() {
            black_box(find_longest_palindromes_hash_native(
                chars, &alphabet, &NPOWS,
            ));
        }
    })
}


#[bench]
fn bench_find_longest_palindromes_hash_dp(b: &mut Bencher) {
    let alphabet = DigitsLetters;

    b.iter(|| {
        for chars in TEXTS.iter() {
            black_box(find_longest_palindromes_hash_dp(
                chars, &alphabet, &NPOWS,
            ));
        }
    })
}


#[bench]
fn bench_find_longest_palindromes_manacher_oddeven(b: &mut Bencher) {
    b.iter(|| {
        for chars in TEXTS.iter() {
            let (d1, d2) = black_box(find_sub_palindromes_manacher(chars));
            black_box(sub_palindromes_to_longest_palindromes((&d1, &d2)));
        }
    })
}


#[bench]
fn bench_find_longest_palindromes_manacher_unify(b: &mut Bencher) {
    b.iter(|| {
        for chars in TEXTS.iter() {
            let (d1, d2) = black_box(find_sub_palindromes_manacher_unify(chars));
            black_box(sub_palindromes_to_longest_palindromes((&d1, &d2)));
        }
    })
}


fn verify_palindrome_whole_compare(s: &[char]) -> bool {
    !s.is_empty() && s.iter().eq(s.iter().rev())
}


fn verify_palindrome_two_ptr(s: &[char]) -> bool {
    if s.is_empty() {
        return false;
    }

    let mut i;
    let mut j;

    if s.len() % 2 == 0 {
        i = s.len() / 2 - 1;
        j = i + 1;
    }
    else {
        i = (s.len() + 1) / 2 - 1;
        j = i;
    }

    loop {
        if s[i] != s[j] {
            break false;
        }

        if i == 0 {
            break true;
        }

        i -= 1;
        j += 1;
    }
}


#[bench]
fn bench_verify_palindrome_whole_compare(b: &mut Bencher) {
    b.iter(|| {
        for chars in TEXTS.iter() {
            black_box(verify_palindrome_whole_compare(&chars));
        }
    })
}


#[bench]
fn bench_verify_palindrome_two_ptr(b: &mut Bencher) {
    b.iter(|| {
        for chars in TEXTS.iter() {
            black_box(verify_palindrome_two_ptr(&chars));
        }
    })
}
