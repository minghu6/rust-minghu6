#![allow(dead_code)]

use std::cmp::max;

pub struct BMPattern<'a> {
    pat_bytes: &'a [u8],
    delta1: [usize; 256],
    delta2: Vec<usize>,
}

impl<'a> BMPattern<'a> {
    pub fn new(pat: &'a str) -> Self {
        assert_ne!(pat.len(), 0);

        let pat_bytes = pat.as_bytes();
        let delta1 = BMPattern::build_delta1_table(&pat_bytes);
        let delta2 = BMPattern::build_delta2_table_naive(&pat_bytes);

        BMPattern {
            pat_bytes,
            delta1,
            delta2,
        }
    }

    fn build_delta1_table(p: &'a [u8]) -> [usize; 256] {
        let mut delta1 = [p.len(); 256];
        let lastpos = p.len() - 1;

        for i in 0..lastpos {
            delta1[p[i] as usize] = lastpos - i;
        }

        delta1
    }

    fn build_delta2_table_naive(p: &'a [u8]) -> Vec<usize> {
        // refer to kmp calc_next_improved
        let mut delta2_table = vec![];

        for (i, _) in p.iter().enumerate() {
            let subpatlen = (p.len() - 1 - i) as isize;

            if subpatlen == 0 {
                delta2_table.push(0);
                break;
            }

            for j in (-subpatlen..(i + 1) as isize).rev() {
                // subpat 匹配
                if (j..j + subpatlen)
                    .zip(i + 1..p.len())
                    .all(|(rpr_index, subpat_index)| {
                        if rpr_index < 0 {
                            return true;
                        }

                        if p[rpr_index as usize] == p[subpat_index] {
                            return true;
                        }

                        false
                    })
                    && (j <= 0 || p[(j - 1) as usize] != p[i])
                {
                    delta2_table.push((p.len() as isize - 1 - j) as usize);
                    break;
                }
            }
        }

        delta2_table
    }

    // bad idea! no works...
    fn build_delta2_table_improved(p: &Vec<char>) -> Vec<usize> {
        // refer to kmp calc_next_improved
        let mut rpr = vec![0isize].repeat(p.len());
        rpr[p.len() - 1] = (p.len() - 1) as isize;

        if p.len() > 1 {
            for j in (-1..(p.len() - 1) as isize).rev() {
                if j < 0
                    || p[j as usize] == p[p.len() - 1]
                        && (j == 0 || p[(j - 1) as usize] != p[p.len() - 2])
                {
                    rpr[p.len() - 2] = j;
                    break;
                }
            }
        }

        for i in (1..p.len() - 1).rev() {
            if rpr[i] > 0
                && (p[(rpr[i] - 1) as usize] == p[i]
                    && (rpr[i] == 1 || p[(rpr[i] - 2) as usize] != p[i - 1]))
            {
                rpr[i - 1] = rpr[i] - 1;
                println!("rpr({}): {}", i - 1, rpr[i - 1]);
            } else {
                let start_pos = rpr[i] - 2;
                let subpatlen = (p.len() - 1 - (i - 1)) as isize;

                println!("subpatlen:{}, start_pos:{}", subpatlen, start_pos);

                for j in (-subpatlen..(start_pos + 1) as isize).rev() {
                    // subpat 匹配
                    if (j..j + subpatlen)
                        .zip(i..p.len())
                        .all(|(rpr_index, subpat_index)| {
                            if rpr_index < 0 {
                                return true;
                            }

                            if p[rpr_index as usize] == p[subpat_index] {
                                return true;
                            }

                            false
                        })
                        && (j <= 0 || p[(j - 1) as usize] != p[i - 1])
                    {
                        rpr[i - 1] = j;
                        break;
                    }
                }
            }
        }

        println!("rpr: {:?}", rpr);
        rpr.iter()
            .map(|pos| (p.len() as isize - 1 - pos) as usize)
            .collect()
    }

    pub fn find_all(&self, string: &str) -> Vec<usize> {
        let mut result = vec![];
        let string_bytes = string.as_bytes();
        let stringlen = string_bytes.len();
        let pat_last_pos = self.pat_bytes.len() - 1;
        let mut string_index = pat_last_pos;
        let mut pat_index;

        while string_index < stringlen {
            pat_index = pat_last_pos;

            while pat_index > 0 && string_bytes[string_index] == self.pat_bytes[pat_index] {
                string_index -= 1;
                pat_index -= 1;
            }

            if pat_index == 0 && string_bytes[string_index] == self.pat_bytes[0] {
                result.push(string_index);

                string_index += self.pat_bytes.len();
            } else {
                string_index += max(
                    self.delta1[string_bytes[string_index] as usize],
                    self.delta2[pat_index],
                );
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
    fn bm_delta2_table_built_correctly() {
        let mut p;

        p = BMPattern::new("ABCXXXABC");
        assert_eq!(p.delta2, vec![13, 12, 11, 10, 9, 8, 10, 9, 0]);

        p = BMPattern::new("ABYXCDEYX");
        assert_eq!(p.delta2, vec![16, 15, 14, 13, 12, 11, 6, 9, 0]);

        p = BMPattern::new("abbaaba");
        assert_eq!(p.delta2, vec![11, 10, 9, 8, 4, 2, 0]);

        p = BMPattern::new("aaa");
        assert_eq!(p.delta2, vec![2, 2, 0]);
    }
    #[test]
    fn bm_find_all_fixeddata_works() {
        let p1 = BMPattern::new("abbaaba");
        assert_eq!(p1.find_all("abbaabbaababbaaba"), vec![4, 10]);

        let p2 = BMPattern::new("aaa");
        assert_eq!(p2.find_all("aaaaa"), vec![0, 1, 2]);
    }

    #[test]
    fn bm_find_all_randomdata_works() {
        for (pat, text, result) in spm::gen_test_case() {
            assert_eq!(BMPattern::new(pat.as_str()).find_all(text.as_str()), result)
        }
    }
}
