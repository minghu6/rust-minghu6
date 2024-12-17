pub mod ac;
pub mod ac2;
pub mod bm;
pub mod kmp;
pub mod rk;

mod test;

use std::{fmt::Debug, str};

////////////////////////////////////////////////////////////////////////////////
//// Traits

pub trait Find<'a, T> {
    fn len(&self) -> usize;

    fn find(&self, string: &[T]) -> Option<usize>;

    /// Find overlapping
    fn find_all(&'a self, string: &'a [T]) -> impl Iterator<Item = usize> + 'a
    where
        T: Debug,
    {
        std::iter::from_coroutine(
            #[coroutine]
            move || {
                let mut suffix = string;
                let mut base = 0;

                while let Some(i) = self.find(suffix) {
                    yield i + base;

                    let shift = i + 1;

                    if shift >= suffix.len() {
                        break;
                    }

                    base += shift;
                    suffix = &suffix[shift..];
                }
            },
        )
    }
}


pub trait FindStr<'a>: Find<'a, u8> {
    fn find(&'a self, string: &str) -> Option<usize> {
        Find::find(self, string.as_bytes())
    }

    /// Find overlapping
    fn find_all(
        &'a self,
        string: &'a str,
    ) -> impl Iterator<Item = usize> + 'a {
        Find::find_all(self, string.as_bytes())
    }
}


////////////////////////////////////////////////////////////////////////////////
//// Implementations

impl<'a, T: Find<'a, u8> + ?Sized> FindStr<'a> for T {}


impl<'a, T: AsRef<str>> Find<'a, u8> for T {
    fn len(&self) -> usize {
        self.as_ref().len()
    }

    fn find(&self, string: &[u8]) -> Option<usize> {
        str::from_utf8(string).unwrap().find(self.as_ref())
    }

    fn find_all(&'a self, string: &'a [u8]) -> impl Iterator<Item = usize> + 'a
    where
        u8: Debug,
    {
        std::iter::from_coroutine(
            #[coroutine]
            move || {
                let pat = self.as_ref();

                if pat.is_empty() {
                    loop {
                        yield 0;
                    }
                }

                let nxtcharlen = pat.chars().next().unwrap().len_utf8();
                let mut suffix = string;
                // let mut suffix = str::from_utf8(string).unwrap();
                let mut base = 0;

                while let Some(i) = Find::find(self, suffix) {
                    yield i + base;

                    let shift =
                        i + nxtcharlen;

                    base += shift;
                    suffix = &suffix[shift..];
                }
            },
        )
    }
}


////////////////////////////////////////////////////////////////////////////////
//// Functions

/// Prefix Array
///
/// `pi[i]` is max length which proper-prefix = proper-suffix of `pat[..=i]`
///
/// `pi[0] = 0`
///
/// `len(pi) = len(pat)`
pub fn compute_pi(pat: &[impl Eq]) -> Vec<usize> {
    let mut pi = vec![0usize; pat.len()];

    for i in 1..pat.len() {
        let mut j = pi[i - 1] as isize;

        while j > 0 && pat[i] != pat[j as usize] {
            j = pi[j as usize - 1] as isize;
        }

        if pat[i] == pat[j as usize] {
            j += 1;
        }

        pi[i] = j as usize;
    }

    pi
}


/// Galil rule shortest period k
///
/// k = patlen - prefixlen
pub fn compute_k(p: &[impl Eq]) -> usize {
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
        assert_eq!(
            compute_pi(&pat.chars().collect::<Vec<char>>()[..]),
            vec![0, 1, 0, 1, 2, 2, 3]
        );
        assert_eq!(compute_pi(pat.as_bytes()), vec![0, 1, 0, 1, 2, 2, 3]);

        pat = "abcabcd";
        assert_eq!(
            compute_pi(&pat.chars().collect::<Vec<char>>()[..]),
            vec![0, 0, 0, 1, 2, 3, 0]
        );
        assert_eq!(compute_pi(pat.as_bytes()), vec![0, 0, 0, 1, 2, 3, 0]);

        pat = "abaabaabaa";
        assert_eq!(
            compute_pi(&pat.chars().collect::<Vec<char>>()[..]),
            vec![0, 0, 1, 1, 2, 3, 4, 5, 6, 7]
        );
        assert_eq!(
            compute_pi(pat.as_bytes()),
            vec![0, 0, 1, 1, 2, 3, 4, 5, 6, 7]
        );
    }

    #[test]
    fn twoway_find_all_fixeddata_works() {
        let p = String::from("abbaaba");
        assert_eq!(
            FindStr::find_all(&p, "abbaabbaababbaaba").collect::<Vec<_>>(),
            vec![4, 10]
        );

        let p = String::from("aaa");
        assert_eq!(FindStr::find_all(&p, "aaaaa").collect::<Vec<_>>(), vec![0, 1, 2]);

        let p = String::from("b");
        assert!(FindStr::find_all(&p, "aaaaa").collect::<Vec<_>>().is_empty());
    }
}
