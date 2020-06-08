#![feature(test)]

use minghu6::test::spm::{ gen_random_text, gen_pattern, brute_force_match };
use minghu6::algs::spm::kmp::{ KMPPattern, ComputeNext };
use minghu6::algs::spm::ac::TrieTree;
use minghu6::algs::spm::bm::BMPattern;
use minghu6::algs::spm::horspool::HorsPoolPattern;


extern crate test;

use test::Bencher;


fn gen_tested_text() -> Vec<String> {
    let mut result = vec![];
    //result.push(gen_random_text(1_000_000));
    result.push(gen_random_text(500_000));
    result.push(gen_random_text(500_000));

    //result.push(gen_random_text(1_000));

    result
}

fn gen_tested_pattern() -> Vec<String> {
    let mut result = vec![];

    for pattern in gen_pattern((1..36, 1), 5) {
        result.push(pattern)
    }

    result
}

#[bench]
fn gen_some_random_text(b: &mut Bencher) {
    b.iter(|| {
        gen_tested_text();
        gen_tested_pattern();
    })
}

#[bench]
fn bf_spm(b: &mut Bencher) {
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


#[bench]
fn kmp_spm(b: &mut Bencher) {
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


#[bench]
fn kmp_spm_naive(b: &mut Bencher) {
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
fn bm_spm(b: &mut Bencher) {
    let gen = || {
        let tested_texts = gen_tested_text();
        let tested_patterns = gen_tested_pattern();
        for text in &tested_texts {
            for pattern in &tested_patterns {
                BMPattern::new(pattern.as_str(), ).find_all(text.as_str());
            }
        }
    };

    b.iter(|| gen())
}


#[bench]
fn horspool_spm(b: &mut Bencher) {
    let gen = || {
        let tested_texts = gen_tested_text();
        let tested_patterns = gen_tested_pattern();
        for text in &tested_texts {
            for pattern in &tested_patterns {
                HorsPoolPattern::new(pattern.as_str(), ).find_all(text.as_str());
            }
        }
    };

    b.iter(|| gen())
}

#[bench]
fn ac_automaton(b: &mut Bencher) {
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
