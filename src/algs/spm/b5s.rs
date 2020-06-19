#![allow(dead_code)]
///! BMHBNFS (fast search algorithm from stringlib of Python)[http://effbot.org/zone/stringlib.htm#BMHBNFS]

use super::super::super::collections::SimpleBloomFilter;
use super::{ compute_k };

pub struct B5STimePattern<'a> {
    pat_bytes: &'a [u8],
    alphabet: [bool;256],
    bm_bc: [usize;256],
    k: usize
}

impl<'a> B5STimePattern<'a> {
    pub fn new(pat: &'a str) -> Self {
        assert_ne!(pat.len(), 0);

        let pat_bytes = pat.as_bytes();
        let (alphabet, bm_bc, k) = B5STimePattern::build(pat_bytes);

        B5STimePattern { pat_bytes, alphabet, bm_bc, k }
    }

    fn build(p: &'a [u8]) -> ([bool;256], [usize;256], usize)  {
        let mut alphabet = [false;256];
        let mut bm_bc = [p.len(); 256];
        let lastpos = p.len() - 1;

        for i in 0..lastpos {
            alphabet[p[i] as usize] = true;
            bm_bc[p[i] as usize] = lastpos - i;
        }

        alphabet[p[lastpos] as usize] = true;

        (alphabet, bm_bc, compute_k(p))
    }

    pub fn find_all(&self, string: &str) -> Vec<usize> {
        let mut result = vec![];
        let string_bytes = string.as_bytes();
        let pat_last_pos = self.pat_bytes.len() - 1;
        let patlen = self.pat_bytes.len();
        let stringlen = string_bytes.len();
        let mut string_index = pat_last_pos;
        let mut offset = pat_last_pos;
        let offset0 = self.k - 1;
        #[derive(PartialEq)]
        enum MatchState {
            Goon, Break
        }
        let mut state;
        let skip = #[inline] |string_index: &mut usize, state: MatchState| {
            if *string_index + 1 == stringlen {
                return false;
            }

            // Galil rule
            if state == MatchState::Goon {
                *string_index += self.k;
                return true;
            }

            if !self.alphabet[string_bytes[*string_index+1] as usize] {
                *string_index += patlen + 1;  // sunday
            } else {
                *string_index += self.bm_bc[string_bytes[*string_index] as usize];  // horspool
            }

            true
        };

        while string_index < stringlen {
            if string_bytes[string_index] == self.pat_bytes[pat_last_pos] {
                if &string_bytes[string_index-offset..string_index] == &self.pat_bytes[pat_last_pos-offset..pat_last_pos] {
                    result.push(string_index-pat_last_pos);

                    offset = offset0;
                    state = MatchState::Goon;
                } else {
                    offset = pat_last_pos;
                    state = MatchState::Break;
                }

                if !skip(&mut string_index, state) {
                    break;
                }
            } else {
                offset = pat_last_pos;
                state = MatchState::Break;

                if !skip(&mut string_index, state) {
                    break;
                }
            }
        }

        result
    }
}


pub struct B5SSpacePattern<'a> {
    pat_bytes: &'a [u8],
    alphabet: SimpleBloomFilter,
    skip: usize,
}

impl<'a> B5SSpacePattern<'a> {
    pub fn new(pat: &'a str) -> Self {
        assert_ne!(pat.len(), 0);

        let pat_bytes = pat.as_bytes();
        let (alphabet, skip) = B5SSpacePattern::build(pat_bytes);

        B5SSpacePattern { pat_bytes, alphabet, skip }
    }

    fn build(p: &'a [u8]) -> (SimpleBloomFilter, usize)  {
        //let mut alphabet = BloomFilter::with_rate(0.15, 256);  // 相对最合适的参数，m=126Byte, k=3

        let mut alphabet = SimpleBloomFilter::new();  // 性能表现比BloomFilter::with_rate(0.15, 256)居然更好
        let lastpos = p.len() - 1;
        let mut skip = p.len();

        for i in 0..p.len()-1 {
            alphabet.add(&p[i]);

            if p[i] == p[lastpos] {
                skip = lastpos - i;
            }
        }

        alphabet.add(&p[lastpos]);

        (alphabet, skip)
    }

    pub fn find_all(&self, string: &str) -> Vec<usize> {
        let mut result = vec![];
        let string_bytes = string.as_bytes();
        let pat_last_pos = self.pat_bytes.len() - 1;
        let patlen = self.pat_bytes.len();
        let stringlen = string_bytes.len();
        let mut string_index = pat_last_pos;

        while string_index < stringlen {
            if string_bytes[string_index] == self.pat_bytes[pat_last_pos] {
                if &string_bytes[string_index-pat_last_pos..string_index] == &self.pat_bytes[..patlen-1] {
                    result.push(string_index-pat_last_pos);
                }

                if string_index + 1 == stringlen {
                    break;
                }

                if !self.alphabet.contains(&string_bytes[string_index+1]) {
                    string_index += patlen + 1;  // sunday
                } else {
                    string_index += self.skip;  // horspool
                }
            } else {
                if string_index + 1 == stringlen {
                    break;
                }

                if !self.alphabet.contains(&string_bytes[string_index+1]) {
                    string_index += patlen + 1;  // sunday
                } else {
                    string_index += 1;
                }
            }

        }

        result
    }
}
#[cfg(test)]
mod tests {
    use super::super::super::super::test::spm;
    use super::*;
    #[test]
    fn b5s_time_find_all_fixeddata_works() {
        let mut p;

        // p = B5STimePattern::new("abbaaba");
        // assert_eq!(p.find_all("abbaabbaababbaaba"), vec![4, 10]);

        // p = B5STimePattern::new("aaa");
        // assert_eq!(p.find_all("aaaaa"), vec![0, 1, 2]);

        // p = B5STimePattern::new("b");
        // assert_eq!(p.find_all("aaaaa"), vec![]);

        // p = B5STimePattern::new("a");
        // assert_eq!(p.find_all("a"), vec![0]);

        p = B5STimePattern::new("abcd");
        assert_eq!(p.find_all("abcdabcdabcabcd"), vec![0, 4, 11]);
    }

    #[test]
    fn b5s_space_find_all_fixeddata_works() {
        let p1 = B5SSpacePattern::new("abbaaba");
        assert_eq!(p1.find_all("abbaabbaababbaaba"), vec![4, 10]);

        let p2 = B5SSpacePattern::new("aaa");
        assert_eq!(p2.find_all("aaaaa"), vec![0, 1, 2]);

        let p3 = B5SSpacePattern::new("b");
        assert_eq!(p3.find_all("aaaaa"), vec![]);
    }

    #[test]
    fn b5s_time_find_all_randomdata_works() {
        for (pat, text, result) in spm::gen_test_case() {
            assert_eq!(
                B5STimePattern::new(pat.as_str()).find_all(text.as_str()),
                result
            )
        }
    }

    #[test]
    fn b5s_space_find_all_randomdata_works() {
        for (pat, text, result) in spm::gen_test_case() {
            assert_eq!(
                B5SSpacePattern::new(pat.as_str()).find_all(text.as_str()),
                result
            )
        }
    }
}