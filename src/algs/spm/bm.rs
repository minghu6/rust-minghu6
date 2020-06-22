#![allow(dead_code)]

use std::cmp::{ max, min };

use super::{ compute_pi, compute_k };

/// stringlen-1 + patlen <= large && stringlen-1 + large <= 2^$(usize width)
#[cfg(target_pointer_width = "64")]
const LARGE: usize = 10_000_000_000_000_000_000;

#[cfg(not(target_pointer_width = "64"))]
const LARGE: usize = 2_000_000_000;

pub struct BMPattern<'a> {
    pat_bytes: &'a [u8],
    delta1: [usize; 256],
    delta2: Vec<usize>,
    k: usize
}

impl<'a> BMPattern<'a> {
    pub fn new(pat: &'a str) -> Self {
        assert_ne!(pat.len(), 0);

        let pat_bytes = pat.as_bytes();
        let delta1 = BMPattern::build_delta1_table(&pat_bytes);
        //let delta2 = BMPattern::build_delta2_table_naive(&pat_bytes);
        //let delta2 = BMPattern::build_delta2_table_improved_knuth(&pat_bytes);
        //let delta2 = BMPattern::build_delta2_table_improved_rytter(&pat_bytes);
        //let delta2 = BMPattern::build_delta2_table_improved_blog(&pat_bytes);
        let delta2 = BMPattern::build_delta2_table_improved_minghu6(&pat_bytes);
        let k = compute_k(&pat_bytes);

        BMPattern {
            pat_bytes,
            delta1,
            delta2,
            k
        }
    }

    #[inline]
    fn delta0(&self, char: u8) -> usize {
        if char == self.pat_bytes[self.pat_bytes.len() - 1] {
            LARGE
        } else {
            self.delta1[char as usize]
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

    pub fn build_delta2_table_naive(p: &'a [u8]) -> Vec<usize> {
        let patlen = p.len();
        let lastpos = patlen - 1;
        let mut delta2 = vec![];

        for i in 0..patlen {
            let subpatlen = (lastpos - i) as isize;

            if subpatlen == 0 {
                delta2.push(0);
                break;
            }

            for j in (-subpatlen..(i + 1) as isize).rev() {
                // subpat 匹配
                if (j..j + subpatlen)
                    .zip(i + 1..patlen)
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
                    delta2.push((lastpos as isize - j) as usize);
                    break;
                }
            }
        }

        delta2
    }

    pub fn build_delta2_table_improved_knuth(p: &'a [u8]) -> Vec<usize> {
        let mut delta2 = Vec::with_capacity(p.len());
        let patlen = p.len();
        let lastpos = patlen - 1;

        for k in 0..patlen {
            delta2.push(lastpos * 2 - 1 - k);
        }

        let mut j = lastpos;
        let mut t = patlen;
        let mut f = vec![0; patlen];

        loop {
            f[j] = t;
            while t < patlen && p[j] != p[t] {
                delta2[t] = min(delta2[t], lastpos - 1 - j);
                t = f[t];
            }

            t -= 1;
            if j == 0 {
                break;
            }
            j -= 1;
        }

        for k in 0..t + 1 {
            delta2[k] = min(delta2[k], patlen + t - 1 - k);
        }

        delta2
    }

    pub fn build_delta2_table_improved_rytter(p: &'a [u8]) -> Vec<usize> {
        let mut delta2 = Vec::with_capacity(p.len());
        let patlen = p.len();
        let lastpos = patlen - 1;

        for k in 0..p.len() {
            delta2.push(2 * lastpos - k);
        }

        let mut j = lastpos;
        let mut t = patlen;
        let mut f = vec![0; patlen];

        loop {
            f[j] = t;
            while t < patlen && p[j] != p[t] {
                // println!("case A2: t:{}, j:{}, delta2[t]:{}, new_delta2[t]:{}", t, j, delta2[t], min(delta2[t], lastpos - 1 - j));
                delta2[t] = min(delta2[t], lastpos - 1 - j);
                t = f[t];
            }

            t -= 1;
            if j == 0 {
                break;
            }
            j -= 1;
        }
        //delta2[t] = min(delta2[t], lastpos - 1 + 1); // j = -1, -j = +1
        // println!("after case A2, delta2:{:?}", delta2);
        // println!("after case A2, f:{:?}", f);
        // println!("t: {}", t);

        let mut q = t + 1;
        t = patlen - q;

        let mut j1 = 1;
        let mut t1 = 0;
        let mut f1 = f;
        while j1 - 1 < t {
            f1[j1 - 1] = t1;
            while t1 > 0 && p[j1 - 1] != p[t1 - 1] {
                t1 = f1[t1 - 1];
            }

            t1 += 1;
            j1 += 1;
        }
        // println!("f1:{:?}, t:{}, q:{}", f1, t, q);

        let mut q1 = 0;
        while q < patlen {
            for k in q1..q {
                delta2[k] = min(delta2[k], lastpos + q - 1 - k);
                // println!("delta2[{}]:{}", k, delta2[k]);
            }

            // let k = q1;
            // delta2[k] = min(delta2[k], lastpos + q - 1 - k);
            // println!("delta2[{}]:{}", k, delta2[k]);

            q1 = q;
            q = q + t - f1[t - 1];
            t = f1[t - 1];
            // println!("q1:{}, q:{}, t:{}, f1:{:?}", q1, q, t, f1);
        }

        delta2[lastpos] = 0;

        delta2
    }

    pub fn build_delta2_table_improved_blog(p: &'a [u8]) -> Vec<usize> {
        if p.len() == 1 {
            return vec![0];
        }

        let mut delta2 = Vec::with_capacity(p.len());
        let patlen = p.len();
        let lastpos = patlen - 1;
        let mut suffix = vec![patlen].repeat(patlen);
        let mut f = 0;
        let mut g = lastpos;
        for i in (0..lastpos).rev() {
            if i > g && suffix[i + lastpos - f] < i - g {
                suffix[i] = suffix[i + lastpos - f];
            } else {
                if i < g {
                    g = i;
                }
                f = i;
                while p[g] == p[g + lastpos - f] {
                    if g == 0 {
                        break;
                    }
                    g -= 1;
                }
                suffix[i] = f - g
            }

            // let mut q = i as isize;
            // while q >=0 && p[q as usize] == p[lastpos + q as usize - i] {

            //     q -= 1;
            // }

            // suffix[i] = (i as isize - q) as usize;
        }

        let mut j = 0;

        for _ in 0..patlen {
            delta2.push(patlen);
        }

        for i in (0..patlen).rev() {
            if suffix[i] == i + 1 {
                while j < lastpos - i {
                    delta2[j] = lastpos - i;
                    j += 1;
                }
            }
        }

        println!("lastpos: {}, suffix:{:?}", lastpos, suffix);
        for i in 0..lastpos {
            delta2[lastpos - suffix[i]] = lastpos - i;
        }

        delta2
    }

    pub fn build_delta2_table_improved_minghu6(p: &'a [u8]) -> Vec<usize> {
        let patlen = p.len();
        let lastpos = patlen - 1;
        let mut delta2 = Vec::with_capacity(patlen);

        // delta2[j] = lastpos * 2 - j
        for i in 0..patlen {
            delta2.push(lastpos * 2 - i);
        }

        // lastpos <= delata2[j] = lastpos * 2 - j
        let pi = compute_pi(p);
        let mut i = lastpos;
        let mut last_i = lastpos; // 只是为了初始化
        while pi[i] > 0 {
            let start;
            let end;

            if i == lastpos {
                start = 0;
            } else {
                start = patlen - pi[last_i];
            }

            end = patlen - pi[i];

            for j in start..end {
                delta2[j] = lastpos * 2 - j - pi[i];
            }

            last_i = i;
            i = pi[i] - 1;
        }

        // delata2[j] < lastpos
        let mut j = lastpos;
        let mut t = patlen;
        let mut f = pi;
        loop {
            f[j] = t;
            while t < patlen && p[j] != p[t] {
                delta2[t] = min(delta2[t], lastpos - 1 - j);
                t = f[t];
            }

            t -= 1;
            if j == 0 {
                break;
            }
            j -= 1;
        }
        delta2[lastpos] = 0;

        delta2
    }

    pub fn find_all(&self, string: &'a str) -> Vec<usize> {
        let mut result = vec![];
        let string_bytes = string.as_bytes();
        let stringlen = string_bytes.len();
        let patlen = self.pat_bytes.len();
        let pat_last_pos = patlen - 1;
        let mut string_index = pat_last_pos;
        let mut pat_index;
        let l0 =  patlen - self.k;
        let mut l = 0;

        while string_index < stringlen {
            while string_index < stringlen {
                string_index += self.delta0(string_bytes[string_index]);
            }
            if string_index < LARGE {
                break;
            }

            string_index -= LARGE;
            pat_index = pat_last_pos;
            while pat_index > l && string_bytes[string_index] == self.pat_bytes[pat_index] {
                string_index -= 1;
                pat_index -= 1;
            }

            if pat_index == l && string_bytes[string_index] == self.pat_bytes[pat_index] {
                result.push(string_index - l);

                string_index += pat_last_pos - l + self.k;
                l = l0;
            } else {
                l = 0;
                string_index += max(
                    self.delta1[string_bytes[string_index] as usize],
                    self.delta2[pat_index],
                );
            }
        }

        result
    }
}

pub struct SimplifiedBMPattern<'a> {
    pat_bytes: &'a [u8],
    delta1: [usize; 256],
}

impl<'a> SimplifiedBMPattern<'a> {
    pub fn new(pat: &'a str) -> Self {
        assert_ne!(pat.len(), 0);

        let pat_bytes = pat.as_bytes();
        let delta1 = BMPattern::build_delta1_table(&pat_bytes);

        SimplifiedBMPattern { pat_bytes, delta1 }
    }

    #[inline]
    fn delta0(&self, char: u8) -> usize {
        if char == self.pat_bytes[self.pat_bytes.len() - 1] {
            LARGE
        } else {
            self.delta1[char as usize]
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

    pub fn find_all(&self, string: &'a str) -> Vec<usize> {
        let mut result = vec![];
        let string_bytes = string.as_bytes();
        let stringlen = string_bytes.len();
        let pat_last_pos = self.pat_bytes.len() - 1;
        let mut string_index = pat_last_pos;
        let mut pat_index;

        // improved version
        while string_index < stringlen {
            while string_index < stringlen {
                string_index += self.delta0(string_bytes[string_index]);
            }
            if string_index < LARGE {
                break;
            }

            string_index -= LARGE;
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
                    pat_last_pos - pat_index + 1,
                );
            }
        }

        //old version
        // while string_index < stringlen {
        //     pat_index = pat_last_pos;
        //     while pat_index > 0 && string_bytes[string_index] == self.pat_bytes[pat_index] {
        //         string_index -= 1;
        //         pat_index -= 1;
        //     }

        //     if pat_index == 0 && string_bytes[string_index] == self.pat_bytes[0] {
        //         result.push(string_index);

        //         string_index += self.pat_bytes.len();
        //     } else {
        //         string_index += max(self.delta1[string_bytes[string_index] as usize], pat_last_pos-pat_index+1);
        //     }

        // }
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

        p = BMPattern::new("abaabaabaa");
        assert_eq!(p.delta2, vec![11, 10, 9, 11, 10, 9, 11, 10, 1, 0]);

        p = BMPattern::new("aaa");
        assert_eq!(p.delta2, vec![2, 2, 0]);
    }

    #[test]
    fn bm_find_all_fixeddata_works() {
        let mut p;

        p = BMPattern::new("abbaaba");
        assert_eq!(p.find_all("abbaabbaababbaaba"), vec![4, 10]);

        p = BMPattern::new("aaa");
        assert_eq!(p.find_all("aaaaa"), vec![0, 1, 2]);

        p = BMPattern::new("a");
        assert_eq!(p.find_all("a"), vec![0]);

        p = BMPattern::new("abcd");
        assert_eq!(p.find_all("abcdabcdabcabcd"), vec![0, 4, 11]);
    }

    #[test]
    fn bm_find_all_randomdata_works() {
        for (pat, text, result) in spm::gen_test_case() {
            let r = BMPattern::new(pat.as_str()).find_all(text.as_str());
            if r != result {
                println!("pat:{}", pat);
                println!("string:{:#?}", text);
                println!("string len: {}", text.len());
                println!("match:");
                for pos in r {
                    if let Some(youpat) = text.get(pos..pos+pat.len()) {
                        println!("{} ",youpat);
                    }
                }

                println!("\n\n");
            }
            assert_eq!(BMPattern::new(pat.as_str()).find_all(text.as_str()), result)
        }
    }

    #[test]
    fn simplified_bm_find_all_fixeddata_works() {
        let p1 = SimplifiedBMPattern::new("abbaaba");
        assert_eq!(p1.find_all("abbaabbaababbaaba"), vec![4, 10]);

        let p2 = SimplifiedBMPattern::new("aaa");
        assert_eq!(p2.find_all("aaaaa"), vec![0, 1, 2]);

        let p3 = SimplifiedBMPattern::new("a");
        assert_eq!(p3.find_all("a"), vec![0]);
    }

    #[test]
    fn simplified_bm_find_all_randomdata_works() {
        for (pat, text, result) in spm::gen_test_case() {
            assert_eq!(
                SimplifiedBMPattern::new(pat.as_str()).find_all(text.as_str()),
                result
            )
        }
    }
}
