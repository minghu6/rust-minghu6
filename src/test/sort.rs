use rand::prelude::*;


pub fn gen_test_case() -> Vec<Vec<usize>> {
    let mut cases = vec![];
    let mut rng = thread_rng();

    for len in 1..100 {
        for _round in 0..100 {
            let mut origin:Vec<usize> = (0..len).collect();
            origin.shuffle(&mut rng);
            cases.push(origin);
        }
    }

    cases
}