#![feature(test)]

use std::hint::black_box;

use m6_algs::string::{
    ac,
    ac2,
    bm::*,
    // bm_badimpl::BMPattern as BMBadImplPattern,
    create_npows,
    gen_pattern,
    gen_random_string,
    kmp::{ComputeNext, KMPPattern},
    rk::{RabinKarpPatten, RabinKarpText},
    AlphaBet,
    CommonChinese,
    FindStr,
};
use paste::paste;


extern crate test;

use test::Bencher;


macro_rules! bench {
    ($name:ident, $patten:path) => {
        paste! {
            #[bench]
            fn [<bench_ $name _spm>](b: &mut Bencher) {
                let gen = || {
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

                b.iter(|| gen())
            }
        }
    };
}


fn gen_tested_text() -> Vec<String> {
    let mut result = vec![];
    // result.push(gen_random_text(1_000_000));
    // result.push(gen_random_text(1_000_000));
    result.push(gen_random_string(0_500));

    // result.push(gen_random_text(1_000));

    result
}

fn gen_tested_pattern() -> Vec<String> {
    let mut result = vec![];

    for pattern in gen_pattern((1..10, 1), 2000) {
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
fn kmp_spm(b: &mut Bencher) {
    let gen = || {
        let tested_texts = gen_tested_text();
        let tested_patterns = gen_tested_pattern();
        for text in &tested_texts {
            for pattern in &tested_patterns {
                KMPPattern::new(pattern.as_str(), ComputeNext::Improved)
                    .find_all(text.as_str());
            }
        }
    };

    b.iter(|| gen())
}

#[ignore]
#[bench]
fn kmp_spm_naive(b: &mut Bencher) {
    let gen = || {
        let tested_texts = gen_tested_text();
        let tested_patterns = gen_tested_pattern();
        for text in &tested_texts {
            for pattern in &tested_patterns {
                KMPPattern::new(pattern.as_str(), ComputeNext::Naive)
                    .find_all(text.as_str());
            }
        }
    };

    b.iter(|| gen())
}

bench!(horspool, HorspoolPattern);

bench!(bm, BMPattern);

bench!(twoway, String);

bench!(sunday, SundayPattern);

bench!(b5s_time, B5STimePattern);

bench!(b5s_space, B5SSpacePattern);


#[bench]
fn simplified_bm_spm(b: &mut Bencher) {
    let gen = || {
        let tested_texts = gen_tested_text();
        let tested_patterns = gen_tested_pattern();
        for text in &tested_texts {
            for pattern in &tested_patterns {
                SimplifiedBMPattern::new(pattern.as_str())
                    .find_all(text.as_str());
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
        let trie_tree = ac::ACTrie::new(&tested_patterns);

        for text in &tested_texts {
            black_box(trie_tree.index_of(text.as_str()));
        }
    };

    b.iter(|| gen())
}

#[bench]
fn ac_automaton2(b: &mut Bencher) {
    let gen = || {
        let tested_texts = gen_tested_text();
        let tested_patterns = gen_tested_pattern();
        let trie_tree = ac2::ACTrie::with_keys(tested_patterns);

        for text in &tested_texts {
            black_box(trie_tree.search(text.as_str()));
        }
    };

    b.iter(|| gen())
}

#[bench]
fn rk_spm(b: &mut Bencher) {
    let alphabet = CommonChinese;
    let p = alphabet.prime();
    let npows = create_npows(p, 1000);

    let gen = || {
        let tested_texts = gen_tested_text();
        let tested_patterns = gen_tested_pattern();

        for text in &tested_texts {
            let rk_text = RabinKarpText::<1>::new(&text, &alphabet).unwrap();

            for pat in tested_patterns.iter() {
                let pat = RabinKarpPatten::new(&pat, &alphabet).unwrap();

                // extend_npows(p, &mut npows, pat.len());

                black_box(pat.find(&rk_text, &npows).collect::<Vec<usize>>());
            }
        }
    };

    b.iter(|| gen())
}
