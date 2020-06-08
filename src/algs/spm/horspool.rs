#![allow(dead_code)]

pub struct HorspoolPattern<'a> {
    pat_bytes: &'a [u8],
    bm_bc: [usize; 256],
}

impl<'a> HorspoolPattern<'a> {
    pub fn new(pat: &'a str) -> Self {
        assert_ne!(pat.len(), 0);

        let pat_bytes = pat.as_bytes();
        let bm_bc = HorspoolPattern::build_bm_bc(pat_bytes);

        HorspoolPattern { pat_bytes, bm_bc }
    }

    fn build_bm_bc(p: &'a [u8]) -> [usize; 256] {
        let mut bm_bc_table = [p.len(); 256];

        for i in 0..p.len() - 1 {
            bm_bc_table[p[i] as usize] = p.len() - i - 1;
        }

        bm_bc_table
    }

    pub fn find_all(&self, string: &str) -> Vec<usize> {
        let mut result = vec![];
        let string_bytes = string.as_bytes();
        let mut string_index = self.pat_bytes.len() - 1;
        let mut pat_index = self.pat_bytes.len() - 1;

        while string_index < string_bytes.len() {
            if string_bytes[string_index] == self.pat_bytes[pat_index] {
                if pat_index == 0 {
                    result.push(string_index);

                    string_index += self.pat_bytes.len();
                    pat_index = self.pat_bytes.len() - 1;

                    continue;
                }


                pat_index -= 1;
                string_index -= 1;

                continue;
            }

            let last_char_pos =
                string_index + self.pat_bytes.len() - 1 - pat_index;
            // println!(
            //     "string_index:{}, last_char_pos:{}, last_char:{}, bm_bc:{}",
            //     string_index,
            //     last_char_pos,
            //     string_bytes[last_char_pos],
            //     self.bm_bc[string_bytes[last_char_pos] as usize]
            // );

            string_index = last_char_pos + self.bm_bc[string_bytes[last_char_pos] as usize];
            pat_index = self.pat_bytes.len() - 1;
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
        let p1 = HorspoolPattern::new("abbaaba");
        assert_eq!(p1.find_all("abbaabbaababbaaba"), vec![4, 10]);

        let p2 = HorspoolPattern::new("aaa");
        assert_eq!(p2.find_all("aaaaa"), vec![0, 1, 2]);

        let p3 = HorspoolPattern::new("b");
        assert_eq!(p3.find_all("aaaaa"), vec![]);
    }

    #[test]
    fn horspool_find_all_randomdata_works() {
        for (pat, text, result) in spm::gen_test_case() {
            assert_eq!(
                HorspoolPattern::new(pat.as_str()).find_all(text.as_str()),
                result
            )
        }
    }
}
