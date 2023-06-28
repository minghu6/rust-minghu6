#![feature(test)]
#![allow(dead_code)]

use m6_algs::string::{
    b5s::B5STimePattern, bm::BMPattern, gen_random_dna_text,
    gen_square_periodic_dna_pattern, sunday::SundayPattern,
};

extern crate test;

use test::Bencher;

fn gen_tested_text() -> Vec<String> {
    let mut result = vec![];
    result.push(gen_random_dna_text(500_000));

    result
}

fn gen_tested_pattern() -> Vec<String> {
    let mut result = vec![];

    for pattern in gen_square_periodic_dna_pattern((2..400, 20), 2) {
        result.push(pattern)
    }

    result
}

#[bench]
fn gen_some_random_dna_text(b: &mut Bencher) {
    b.iter(|| {
        gen_tested_text();
        gen_tested_pattern();
    })
}

#[bench]
fn bm_periodic_spm(b: &mut Bencher) {
    let gen = || {
        let tested_texts = gen_tested_text();
        let tested_patterns = gen_tested_pattern();
        for text in &tested_texts {
            for pattern in &tested_patterns {
                BMPattern::new(pattern.as_str()).find_all(text.as_str());
            }
        }
    };

    b.iter(|| gen())
}

#[bench]
fn b5stime_periodic_spm(b: &mut Bencher) {
    let gen = || {
        let tested_texts = gen_tested_text();
        let tested_patterns = gen_tested_pattern();
        for text in &tested_texts {
            for pattern in &tested_patterns {
                B5STimePattern::new(pattern.as_str()).find_all(text.as_str());
            }
        }
    };

    b.iter(|| gen())
}

#[bench]
fn sunday_periodic_spm(b: &mut Bencher) {
    let gen = || {
        let tested_texts = gen_tested_text();
        let tested_patterns = gen_tested_pattern();
        for text in &tested_texts {
            for pattern in &tested_patterns {
                SundayPattern::new(pattern.as_str()).find_all(text.as_str());
            }
        }
    };

    b.iter(|| gen())
}
