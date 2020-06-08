#![allow(dead_code)]

use std::collections::{ HashMap, VecDeque };
use std::cmp::max;

pub struct BMPattern {
    pat_vec: Vec<char>,
    delta1_table: HashMap<char, usize>,
    delta2_table: Vec<usize>
}

impl BMPattern {
    pub fn new(pat: &str)-> Self {
        assert_ne!(pat.len(), 0);

        let pat_vec = pat.chars().collect();
        let delta1_table = BMPattern::build_delta1_table(&pat_vec);
        let delta2_table = BMPattern::build_delta2_table_naive(&pat_vec);

        BMPattern {
            pat_vec,
            delta1_table,
            delta2_table
        }
    }

    fn build_delta1_table(p: &Vec<char>) -> HashMap<char, usize> {
        let mut delta1_table = HashMap::new();

        for i in 0..p.len()-1 {
            delta1_table.insert(p[i].clone(), p.len()-i-1);
        }

        delta1_table
    }

    fn build_delta2_table_naive(p: &Vec<char>) -> Vec<usize> {  // refer to kmp calc_next_improved
        let mut delta2_table = vec![];

        for (i, _) in p.iter().enumerate() {
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

    // bad idea! no works...
    fn build_delta2_table_improved(p: &Vec<char>) -> Vec<usize> {  // refer to kmp calc_next_improved
        let mut rpr = vec![0isize].repeat(p.len());
        rpr[p.len()-1] = (p.len() - 1) as isize;

        if p.len() > 1 {
            for j in (-1..(p.len()-1) as isize).rev() {
                if j < 0 || p[j as usize] == p[p.len()-1] && (j == 0 || p[(j-1) as usize] != p[p.len()-2]){
                    rpr[p.len()-2] = j;
                    break;
                }
            }
        }

        for i in (1..p.len()-1).rev() {
            if rpr[i] > 0 && (p[(rpr[i]-1) as usize] == p[i] && (rpr[i]==1 || p[(rpr[i]-2) as usize] != p[i-1])){
                rpr[i-1] = rpr[i] - 1;
                println!("rpr({}): {}", i-1, rpr[i-1]);
            } else {
                let start_pos = rpr[i] - 2;
                let subpatlen = (p.len()-1 -(i-1)) as isize;

                println!("subpatlen:{}, start_pos:{}", subpatlen, start_pos);

                for j in (-subpatlen..(start_pos+1) as isize).rev() {
                    // subpat 匹配
                    if (j..j+subpatlen).zip(i..p.len()).all(|(rpr_index, subpat_index)| {
                        if rpr_index < 0 {
                            return true
                        }

                        if p[rpr_index as usize] == p[subpat_index] {
                            return true
                        }

                        false
                    }) && (j<=0 || p[(j-1) as usize] != p[i-1]) {
                        rpr[i-1] = j;
                        break
                    }
                }
            }
        }

        println!("rpr: {:?}", rpr);
        rpr.iter().map(|pos| (p.len() as isize -1 - pos) as usize).collect()
    }

    fn delta1(&self, char: char) -> usize {
        if let Some(char_index) = self.delta1_table.get(&char) {
            return char_index.clone()
        }

        self.pat_vec.len()
    }


    #[inline]
    pub fn find_all(&self, string: &str) -> Vec<usize> {
        // index: enumerate index, (0..nth).stepby(1)
        // index_base: base offset of string_vec, string(bm_i) = string_vec[bm_i-index_base]
        // bytes_pos: bytes position of char
        // bm_i: boyer moor's i
        // bm_j: boyer moor's j

        let mut result = vec![];

        let mut string_vec = VecDeque::with_capacity(self.pat_vec.len());

        let mut bm_i = (self.pat_vec.len() - 1) as isize;
        let mut bm_j: isize;
        let mut index_base = 0usize;

        for (index, (bytes_pos, char)) in string.char_indices().enumerate() {
            if index >= index_base {
                string_vec.push_back((bytes_pos, char));
            }

            if index < bm_i as usize {
                continue
            }

            bm_j = (self.pat_vec.len() - 1) as isize;
            loop {
                if bm_j < 0 {
                    let (subpat_char_index, _) = string_vec[(bm_i+1) as usize - index_base];
                    result.push(subpat_char_index);

                    bm_i += 1 + self.pat_vec.len() as isize ;

                    let new_index_base = bm_i as usize - (self.pat_vec.len() -1);
                    string_vec.drain(0..new_index_base-index_base);
                    index_base = new_index_base;

                    break
                }

                let (_, string_char) = string_vec[bm_i as usize - index_base];
                if string_char == self.pat_vec[bm_j as usize] {
                        bm_j -= 1;
                        bm_i -= 1;
                        continue
                }
                bm_i += max(self.delta1(string_char), self.delta2_table[bm_j as usize]) as isize;

                let new_index_base = bm_i as usize - (self.pat_vec.len() -1);
                string_vec.drain(0..new_index_base-index_base);
                index_base = new_index_base;

                break
            }
        }

        // for (index, (bytes_pos, char)) in string.char_indices().enumerate() {
        //     string_vec.push_back((bytes_pos, char));

        //     if index < bm_i as usize {
        //         continue
        //     }

        //     bm_j = (self.pat_vec.len() - 1) as isize;
        //     loop {
        //         if bm_j < 0 {
        //             let (subpat_char_index, string_char) = string_vec[(bm_i+1) as usize];
        //             result.push(subpat_char_index);

        //             bm_i += 1 + self.pat_vec.len() as isize ;

        //             let new_index_base = bm_i as usize - (self.pat_vec.len() -1);
        //             index_base = new_index_base;

        //             break
        //         }

        //         let (_, string_char) = string_vec[bm_i as usize];
        //         if string_char == self.pat_vec[bm_j as usize] {
        //                 bm_j -= 1;
        //                 bm_i -= 1;
        //                 continue
        //         }
        //         bm_i += max(self.delta1(string_char), self.delta2_table[bm_j as usize]) as isize;

        //         let new_index_base = bm_i as usize - (self.pat_vec.len() -1);
        //         index_base = new_index_base;

        //         break
        //     }
        // }


        result
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::super::super::test::spm;

    #[test]
    fn bm_delta2_table_built_correctly() {
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

