pub mod counting_sort;
pub mod merge_sort;
pub mod heapsort;


use m6entry::KVEntry as Entry;

use common::*;



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


pub fn gen_bench_case(unitlen: usize, rounds: usize) -> Vec<Vec<Entry<usize, usize>>> {
    let mut cases = Vec::with_capacity(rounds);
    let mut rng = thread_rng();

    for _ in 0..rounds {
        let mut one_case: Vec<Entry<usize, usize>> = (0..unitlen).map(|i| Entry(i, i)).collect();

        one_case.shuffle(&mut rng);

        cases.push(one_case);
    }

    cases
}
