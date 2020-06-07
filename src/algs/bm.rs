use std::collections::HashMap;
use std::cmp::max;

const LARGE:usize = 1_000_000_000;  // stringlen-1+patlen < 1G

pub struct BMPattern {
    pat_vec: Vec<char>,
    delta1_table: HashMap<char, usize>,
    delta2_table: Vec<usize>,
    large: usize
}

impl BMPattern {
    fn new(pat: &str)-> Self {
        assert_ne!(pat.len(), 0);

        let pat_vec = pat.chars().collect();
        let large = LARGE;
        let delta1_table = BMPattern::build_delta1_table(&pat_vec, large);
        let delta2_table = BMPattern::build_delta2_table_naive(&pat_vec);

        BMPattern {
            pat_vec,
            delta1_table,
            delta2_table,
            large
        }
    }

    fn build_delta1_table(p: &Vec<char>, large:usize) -> HashMap<char, usize> {
        let mut delta1_table = HashMap::new();

        for (i, c) in p.iter().enumerate() {
            delta1_table.insert(c.clone(), p.len()-i-1);
        }

        delta1_table
    }

    fn build_delta2_table_naive(p: &Vec<char>) -> Vec<usize> {  // refer to kmp calc_next_improved
        let mut delta2_table = vec![];

        for (i, char) in p.iter().enumerate() {
            let subpatlen = (p.len()-1 -i) as isize;

            if subpatlen == 0 {
                delta2_table.push(0);
                break;
            }

            for j in (-subpatlen..(i+1) as isize).rev() {
                // subpat 匹配
                if (j..j+subpatlen).zip(i+1..p.len()).all(|(rpr_index, subpat_index)| {
                    if rpr_index < 0 {
                        return true
                    }

                    if p[rpr_index as usize] == p[subpat_index] {
                        return true
                    }

                    false
                }) && (j<=0 || p[(j-1) as usize] != p[i]) {
                    delta2_table.push((p.len() as isize - 1 -j) as usize);
                    break
                }
            }
        }

        delta2_table
    }

    fn delta1(&self, char: char) -> usize {
        if let Some(char_index) = self.delta1_table.get(&char) {
            return char_index.clone()
        }

        self.pat_vec.len()
    }

    pub fn find_all(&self, string: &str) -> Vec<usize> {
        let mut result = vec![];

        let mut string_vec = vec![];
        let mut bm_i = (self.pat_vec.len() - 1) as isize;
        let mut bm_j;

        for (e_i, (c_i, c)) in string.char_indices().enumerate() {
            string_vec.push((c_i, c));

            if e_i < bm_i as usize {
                continue
            }

            bm_j = (self.pat_vec.len() - 1) as isize;
            loop {
                if bm_j < 0 {
                    let (subpat_char_index, _) = string_vec[(bm_i+1) as usize];
                    result.push(subpat_char_index);
                    bm_i += (1 + self.pat_vec.len()) as isize ;
                    break
                }

                let (_, string_char) = string_vec[bm_i as usize];
                if string_char == self.pat_vec[bm_j as usize] {
                    bm_j -= 1;
                    bm_i -= 1;
                    continue
                }

                bm_i += max(self.delta1(string_char), self.delta2_table[bm_j as usize]) as isize;
                break;
            }
        }

        result
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::super::test::spm;

    #[test]
    fn bm_delta_table_built_correctly() {
        let mut p;

        p = BMPattern::new("ABCXXXABC");
        assert_eq!(p.delta2_table, vec![13, 12, 11, 10, 9, 8, 10, 9, 0]);

        p = BMPattern::new("ABYXCDEYX");
        assert_eq!(p.delta2_table, vec![16, 15, 14, 13, 12, 11, 6, 9, 0]);

        p = BMPattern::new("abbaaba");
        assert_eq!(p.delta2_table, vec![11, 10, 9, 8, 4, 2, 0]);

        p = BMPattern::new("aaa");
        assert_eq!(p.delta2_table, vec![2, 2, 0]);
    }
    #[test]
    fn bm_find_all_works() {
        let p1 = BMPattern::new("abbaaba");
        assert_eq!(p1.find_all("abbaabbaababbaaba"), vec![4, 10]);

        let p2 = BMPattern::new("aaa");
        assert_eq!(p2.find_all("aaaaa"), vec![0, 1, 2]);

    }

    #[test]
    fn bm_find_all_randomdata_woks() {
        for (pat, text, result) in spm::gen_test_case() {
            assert_eq!(BMPattern::new(pat.as_str()).find_all(text.as_str()), result)
        }
    }
}

