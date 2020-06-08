#![allow(dead_code)]

use std::collections::{HashMap, VecDeque};

pub struct HorsPoolPattern<'a> {
    pat_bytes: &'a [u8],
    bc: [usize; 256],
}

impl<'a> HorsPoolPattern<'a> {
    pub fn new(pat: &'a str) -> Self {
        assert_ne!(pat.len(), 0);

        let pat_bytes = pat.as_bytes();
        let bc = HorsPoolPattern::build_bc(pat_bytes);

        HorsPoolPattern { pat_bytes, bc }
    }

    fn build_bc(p: &'a [u8]) -> [usize; 256] {
        let mut bc_table = [p.len(); 256];

        for i in 0..p.len() - 1 {
            bc_table[p[i] as usize] = p.len() - i - 1;
        }

        bc_table
    }

    pub fn find_all(&self, string: &str) -> Vec<usize> {
        let mut result = vec![];
        let string_bytes = string.as_bytes();
        let mut string_index = (self.pat_bytes.len() - 1) as isize;
        let mut pat_index = (self.pat_bytes.len() - 1) as isize;

        while string_index < string_bytes.len() as isize {
            if pat_index < 0 {
                result.push((string_index + 1) as usize);

                string_index += 1 + self.pat_bytes.len() as isize;
                pat_index = (self.pat_bytes.len() - 1) as isize;

                continue;
            }

            if string_bytes[string_index as usize] == self.pat_bytes[pat_index as usize] {
                pat_index -= 1;
                string_index -= 1;
                continue;
            }

            let last_char_pos =
                string_index as usize + (self.pat_bytes.len() as isize - 1 - pat_index) as usize;
            // println!(
            //     "string_index:{}, last_char_pos:{}, last_char:{}, bc:{}",
            //     string_index,
            //     last_char_pos,
            //     string_bytes[last_char_pos],
            //     self.bc[string_bytes[last_char_pos] as usize]
            // );

            string_index = (last_char_pos + self.bc[string_bytes[last_char_pos] as usize]) as isize;
            pat_index = (self.pat_bytes.len() - 1) as isize;
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::super::test::spm;
    use super::*;
    #[test]
    fn horspool_find_all_fixeddata_works() {
        let p1 = HorsPoolPattern::new("abbaaba");
        assert_eq!(p1.find_all("abbaabbaababbaaba"), vec![4, 10]);

        let p2 = HorsPoolPattern::new("aaa");
        assert_eq!(p2.find_all("aaaaa"), vec![0, 1, 2]);

        let p3 = HorsPoolPattern::new("b");
        assert_eq!(p3.find_all("aaaaa"), vec![]);
    }

    #[test]
    fn horspool_find_all_randomdata_works() {
        for (pat, text, result) in spm::gen_test_case() {
            assert_eq!(
                HorsPoolPattern::new(pat.as_str()).find_all(text.as_str()),
                result
            )
        }
    }
}
