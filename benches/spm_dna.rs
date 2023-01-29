#![feature(test)]
#![allow(dead_code)]

use minghu6::algs::spm::{
    ac::TrieTree,
    b5s::{B5SSpacePattern, B5STimePattern},
    bm::{BMPattern, SimplifiedBMPattern},
    brute_force_match, gen_dna_pattern, gen_random_dna_text,
    horspool::HorspoolPattern,
    kmp::{ComputeNext, KMPPattern},
    sunday::SundayPattern,
};

extern crate test;

use test::Bencher;

fn gen_tested_text() -> Vec<String> {
    let mut result = vec![];
    for _ in 0..1 {
        result.push(gen_random_dna_text(500_000));
    }

    result
}

fn gen_tested_pattern() -> Vec<String> {
    let mut result = vec![];

    for pattern in gen_dna_pattern((300..500, 20), 10) {
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
fn bf_dna_spm(b: &mut Bencher) {
    let gen = || {
        let tested_texts = gen_tested_text();
        let tested_patterns = gen_tested_pattern();

        for pattern in &tested_patterns {
            for text in &tested_texts {
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
        for pattern in &tested_patterns {
            let kmppat =
                KMPPattern::new(pattern.as_str(), ComputeNext::Improved);
            for text in &tested_texts {
                kmppat.find_all(text.as_str());
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
        for pattern in &tested_patterns {
            let bmpat = BMPattern::new(pattern.as_str());
            for text in &tested_texts {
                bmpat.find_all(text.as_str());
            }
        }
    };

    b.iter(|| gen())
}

#[bench]
fn simplified_bm_dna_spm(b: &mut Bencher) {
    let gen = || {
        let tested_texts = gen_tested_text();
        let tested_patterns = gen_tested_pattern();
        for pattern in &tested_patterns {
            let simplifiedbmpat = SimplifiedBMPattern::new(pattern.as_str());
            for text in &tested_texts {
                simplifiedbmpat.find_all(text.as_str());
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
        for pattern in &tested_patterns {
            let horspool_pat = HorspoolPattern::new(pattern.as_str());
            for text in &tested_texts {
                horspool_pat.find_all(text.as_str());
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
        for pattern in &tested_patterns {
            let sundaypat = SundayPattern::new(pattern.as_str());
            for text in &tested_texts {
                sundaypat.find_all(text.as_str());
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
        for pattern in &tested_patterns {
            let b5stimepat = B5STimePattern::new(pattern.as_str());
            for text in &tested_texts {
                b5stimepat.find_all(text.as_str());
            }
        }
    };

    b.iter(|| gen())
}

#[ignore]
#[bench]
fn b5s_space_dna_spm(b: &mut Bencher) {
    let gen = || {
        let tested_texts = gen_tested_text();
        let tested_patterns = gen_tested_pattern();
        for pattern in &tested_patterns {
            let b5space = B5SSpacePattern::new(pattern.as_str());
            for text in &tested_texts {
                b5space.find_all(text.as_str());
            }
        }
    };

    b.iter(|| gen())
}

#[ignore]
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
