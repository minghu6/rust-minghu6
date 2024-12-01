mod sa;
mod sa16;
mod sais;

pub use sa::*;
pub use sa16::*;
pub use sais::*;

use super::gen_pattern;


/// <pat, sa>
pub fn gen_sa_test_case() -> Vec<(String, Vec<usize>)> {
    let mut cases = vec![];
    for pat in gen_pattern((1..6000, 100), 1) {
        let sa = super::sa::compute_suffix_array_naive(pat.as_bytes());
        cases.push((pat, sa));
    }

    cases
}
