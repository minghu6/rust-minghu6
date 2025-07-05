#![feature(test)]

use std::hint::black_box;

use paste::paste;

use m6_algs::string::{
    bm::*, gen_random_dna_string,
    gen_square_periodic_dna_pattern,
    FindStr
};

extern crate test;

use test::Bencher;


macro_rules! bench {
    ($name:ident, $patten:path) => {
        paste! {
            #[bench]
            fn [<bench_ $name periodic_spm>](b: &mut Bencher) {
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
    result.push(gen_random_dna_string(500_000));

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


bench!(bm, BMPattern);

bench!(sunday, SundayPattern);

bench!(b5s_time, B5STimePattern);
