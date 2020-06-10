#![feature(test)]

use minghu6::algs::spm::ac::TrieTree;
use minghu6::algs::spm::b5s::{B5SSpacePattern, B5STimePattern};
use minghu6::algs::spm::bm::BMPattern;
use minghu6::algs::spm::horspool::HorspoolPattern;
use minghu6::algs::spm::kmp::{ComputeNext, KMPPattern};
use minghu6::algs::spm::sunday::SundayPattern;
use minghu6::test::spm::{brute_force_match, gen_dna_pattern, gen_random_dna_text};

extern crate test;

use test::Bencher;

fn gen_tested_text() -> Vec<String> {
    let mut result = vec![];
    result.push(gen_random_dna_text(500_000));

    result
}

fn gen_tested_pattern() -> Vec<String> {
    let mut result = vec![];

    for pattern in gen_dna_pattern((1..100, 4), 2) {
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
fn bf_dna_spm(b: &mut Bencher) {
    let gen = || {
        let tested_texts = gen_tested_text();
        let tested_patterns = gen_tested_pattern();
        for text in &tested_texts {
            for pattern in &tested_patterns {
                brute_force_match(pattern.as_str(), text.as_str());
            }
        }
    };

    b.iter(|| gen())
}

#[ignore]
#[bench]
fn kmp_dna_spm(b: &mut Bencher) {
    let gen = || {
        let tested_texts = gen_tested_text();
        let tested_patterns = gen_tested_pattern();
        for text in &tested_texts {
            for pattern in &tested_patterns {
                KMPPattern::new(pattern.as_str(), ComputeNext::Improved).find_all(text.as_str());
            }
        }
    };

    b.iter(|| gen())
}

#[ignore]
#[bench]
fn kmp_naive_dna_spm(b: &mut Bencher) {
    let gen = || {
        let tested_texts = gen_tested_text();
        let tested_patterns = gen_tested_pattern();
        for text in &tested_texts {
            for pattern in &tested_patterns {
                KMPPattern::new(pattern.as_str(), ComputeNext::Naive).find_all(text.as_str());
            }
        }
    };

    b.iter(|| gen())
}

#[bench]
fn bm_dna_spm(b: &mut Bencher) {
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
fn horspool_dna_spm(b: &mut Bencher) {
    let gen = || {
        let tested_texts = gen_tested_text();
        let tested_patterns = gen_tested_pattern();
        for text in &tested_texts {
            for pattern in &tested_patterns {
                HorspoolPattern::new(pattern.as_str()).find_all(text.as_str());
            }
        }
    };

    b.iter(|| gen())
}

#[bench]
fn sunday_dna_spm(b: &mut Bencher) {
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

#[bench]
fn b5s_time_dna_spm(b: &mut Bencher) {
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
fn b5s_space_spm(b: &mut Bencher) {
    let gen = || {
        let tested_texts = gen_tested_text();
        let tested_patterns = gen_tested_pattern();
        for text in &tested_texts {
            for pattern in &tested_patterns {
                B5SSpacePattern::new(pattern.as_str()).find_all(text.as_str());
            }
        }
    };

    b.iter(|| gen())
}

#[bench]
fn ac_automaton_dna(b: &mut Bencher) {
    let gen = || {
        let tested_texts = gen_tested_text();
        let tested_patterns = gen_tested_pattern();
        let trie_tree = TrieTree::new(&tested_patterns);

        for text in &tested_texts {
            trie_tree.index_of(text.as_str());
        }
    };

    b.iter(|| gen())
}
