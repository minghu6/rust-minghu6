pub mod ac;
pub mod kmp;
pub mod bm;
pub mod bm_badimpl;
pub mod horspool;
pub mod sunday;
pub mod b5s;
pub mod rk;

mod test;


use std::cmp::PartialEq;



/// Prefix Array
pub fn compute_pi(s: &[impl PartialEq]) -> Vec<usize> {
    let slen = s.len();

    let mut pi = vec![0usize; slen];

    for i in 1..slen {
        let mut j = pi[i-1] as isize;

        while j > 0 && s[i] != s[j as usize] {
            j = pi[j as usize - 1] as isize;
        }

        if s[i] == s[j as usize] {
            j += 1;
        }

        pi[i] = j as usize;
    }

    pi
}


/// AKA Galil rule
/// k in 1..patlen+1
pub fn compute_k(p: &[impl PartialEq]) -> usize {
    let patlen = p.len();
    let lastpos = patlen - 1;

    let pi = compute_pi(p);

    patlen - pi[lastpos]
}


pub use test::*;



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compute_pi_works() {
        let mut pat;

        pat = "aabaaab";
        assert_eq!(compute_pi(&pat.chars().collect::<Vec<char>>()[..]), vec![0, 1, 0, 1, 2, 2, 3]);
        assert_eq!(compute_pi(pat.as_bytes()), vec![0, 1, 0, 1, 2, 2, 3]);

        pat = "abcabcd";
        assert_eq!(compute_pi(&pat.chars().collect::<Vec<char>>()[..]), vec![0, 0, 0, 1, 2, 3, 0]);
        assert_eq!(compute_pi(pat.as_bytes()), vec![0, 0, 0, 1, 2, 3, 0]);

        pat = "abaabaabaa";
        assert_eq!(compute_pi(&pat.chars().collect::<Vec<char>>()[..]), vec![0, 0, 1, 1, 2, 3, 4, 5, 6, 7]);
        assert_eq!(compute_pi(pat.as_bytes()), vec![0, 0, 1, 1, 2, 3, 4, 5, 6, 7]);
    }
}
