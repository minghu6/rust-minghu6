#![feature(test)]

use paste::paste;

use m6_algs::string::{
    ac::ACTrie,
    bm::*,
    gen_dna_pattern, gen_random_dna_string,
    kmp::{ComputeNext, KMPPattern},
    FindStr,
};

extern crate test;

use test::{black_box, Bencher};


macro_rules! bench {
    ($name:ident, $patten:path) => {
        paste! {
            #[bench]
            fn [<$name _dna_spm>](b: &mut Bencher) {
                let generate = || {
                    let tested_strings = gen_tested_text();
                    let tested_patterns = gen_tested_pattern();
                    for pattern in &tested_patterns {
                        let pat = $patten::from(pattern);

                        for text in &tested_strings {
                            black_box(pat
                                .find_all(&text)
                                .collect::<Vec<_>>()
                                );
                        }
                    }
                };

                b.iter(|| generate())
            }
        }
    };
}

fn gen_tested_text() -> Vec<String> {
    let mut result = vec![];
    for _ in 0..1 {
        result.push(gen_random_dna_string(500_000));
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


#[ignore]
#[bench]
fn kmp_dna_spm(b: &mut Bencher) {
    let generate = || {
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

    b.iter(|| generate())
}


#[bench]
fn simplified_bm_dna_spm(b: &mut Bencher) {
    let generate = || {
        let tested_texts = gen_tested_text();
        let tested_patterns = gen_tested_pattern();
        for pattern in &tested_patterns {
            let simplifiedbmpat = SimplifiedBMPattern::new(pattern.as_str());
            for text in &tested_texts {
                simplifiedbmpat.find_all(text.as_str());
            }
        }
    };

    b.iter(|| generate())
}


bench!(horspool, HorspoolPattern);

bench!(bm, BMPattern);

bench!(twoway, String);

bench!(sunday, SundayPattern);

bench!(b5s_space, B5SSpacePattern);

// bench!(b5s_time, B5STimePattern);


#[ignore]
#[bench]
fn ac_automaton_dna(b: &mut Bencher) {
    let generate = || {
        let tested_texts = gen_tested_text();
        let tested_patterns = gen_tested_pattern();
        let trie_tree = ACTrie::new(&tested_patterns);

        for text in &tested_texts {
            trie_tree.index_of(text.as_str());
        }
    };

    b.iter(|| generate())
}
