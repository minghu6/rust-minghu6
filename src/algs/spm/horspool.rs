#![allow(dead_code)]

use std::collections::{ HashMap, VecDeque };


pub struct HorsPoolPattern {
    pat_vec: Vec<char>,
    bc_table: HashMap<char, usize>
}

impl HorsPoolPattern {
    pub fn new(pat: &str)-> Self {
        assert_ne!(pat.len(), 0);

        let pat_vec = pat.chars().collect();
        let bc_table = HorsPoolPattern::build_bc_table(&pat_vec);

        HorsPoolPattern {
            pat_vec,
            bc_table
        }
    }

    fn build_bc_table(p: &Vec<char>) -> HashMap<char, usize> {
        let mut bc_table = HashMap::new();

        for i in 0..p.len()-1 {
            bc_table.insert(p[i].clone(), p.len()-i-1);
        }

        bc_table
    }

    fn bc(&self, char: &char) -> usize {
        match self.bc_table.get(char) {
            Some(offset) => offset.clone(),
            None => self.pat_vec.len()
        }
    }

    pub fn find_all(&self, string: &str) -> Vec<usize> {
        let mut result = vec![];
        let mut hp_i = (self.pat_vec.len() - 1) as isize;
        let mut pat_i:isize;
        let mut index_base = 0usize;
        let mut string_vec = VecDeque::with_capacity(self.pat_vec.len());

        let patlen = self.pat_vec.len();
        let update_base = |string_vec: &mut VecDeque<(usize, char)>, index_base, hp_i|{
            let new_index_base = hp_i - (patlen -1);
            string_vec.drain(0..new_index_base-index_base);
            new_index_base
        };

        for (index, (bytes_pos, char)) in string.char_indices().enumerate() {
            if index >= index_base {
                string_vec.push_back((bytes_pos, char));
            }

            if index < hp_i as usize {
                continue
            }

            pat_i = (self.pat_vec.len() - 1) as isize;
            loop {
                if pat_i < 0 {
                    let (subpat_char_index, _) = string_vec[(hp_i+1) as usize - index_base];
                    result.push(subpat_char_index);

                    hp_i += 1 + self.pat_vec.len() as isize ;
                    index_base = update_base(&mut string_vec, index_base, hp_i as usize);

                    break
                }

                let (_, string_char) = string_vec[hp_i as usize - index_base];
                if string_char == self.pat_vec[pat_i as usize] {
                        pat_i -= 1;
                        hp_i -= 1;

                        continue
                }

                let last_char_pos = hp_i as usize + (self.pat_vec.len() as isize -1 - pat_i) as usize;
                let (_, last_char) = string_vec[last_char_pos - index_base];
                hp_i = (last_char_pos + self.bc(&last_char)) as isize;
                index_base = update_base(&mut string_vec, index_base, hp_i as usize);

                break
            }
        }

        result
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use super::super::super::super::test::spm;
    #[test]
    fn horspool_find_all_fixeddata_works() {
        let p1 = HorsPoolPattern::new("abbaaba");
        assert_eq!(p1.find_all("abbaabbaababbaaba"), vec![4, 10]);

        let p2 = HorsPoolPattern::new("aaa");
        assert_eq!(p2.find_all("aaaaa"), vec![0, 1, 2]);

    }

    #[test]
    fn horspool_find_all_randomdata_works() {
        for (pat, text, result) in spm::gen_test_case() {
            assert_eq!(HorsPoolPattern::new(pat.as_str()).find_all(text.as_str()), result)
        }
    }
}