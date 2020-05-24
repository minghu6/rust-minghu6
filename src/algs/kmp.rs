#![allow(dead_code)]

use std::ops::{ Generator };
use std::rc::Rc;

pub struct KMPPattern {
    t: Vec<char>,
    t_str_len: usize,
    next: Vec<isize>
}

#[derive(Copy, Clone)]
pub enum ComputeNext {
    Naive, Improved
}

impl<'a> KMPPattern {
    pub fn new(t:&'a str, gen_algs: ComputeNext) -> Self {
        let t_vec = t.chars().collect();

        let next;
        match gen_algs {
            ComputeNext::Naive => next = Self::calc_next_naive(&t_vec),
            ComputeNext::Improved => next = Self::calc_next_improved(&t_vec)
        }

        KMPPattern{
            t:t_vec,
            t_str_len: t.len(),
            next
        }
    }

    pub fn find_all(&self, text:&'a str) -> Vec<usize> {
        let mut result = vec![];
        let mut pattern_index = 0isize;

        for (i, c) in text.char_indices() {
            loop {
                if self.t[pattern_index as usize] == c {
                    if pattern_index == (self.t.len() - 1) as isize {  // matched !
                        result.push(i + c.len_utf8() - self.t_str_len);
                        pattern_index = self.next[pattern_index as usize];
                        // println!("hi, p_i{}", pattern_index);
                    } else {
                        pattern_index += 1;
                        break;
                    }
                } else if pattern_index == -1  {
                    break
                } else {
                    pattern_index = self.next[pattern_index as usize];
                }
            }
        }

        result
    }

    pub fn find(&self, text:&str) -> impl Generator<Yield = usize, Return = ()> {
        let text = Box::new(String::from(text));
        let next = self.next.clone();
        let t = self.t.clone();
        let t_str_len = self.t_str_len;

        move || {
            let mut pattern_index = 0;

            for (i, c) in text.char_indices() {
                loop {
                    if t[pattern_index as usize] == c {
                        if pattern_index == (t.len() - 1) as isize {
                            yield (i + c.len_utf8() - t_str_len);
                            pattern_index = next[pattern_index as usize];
                        } else {
                            pattern_index += 1;
                            break;
                        }
                    } else if pattern_index == -1  {
                        break
                    } else {
                        pattern_index = next[pattern_index as usize];
                    }
                }
            }
        }
    }

    fn calc_next_naive(p: &Vec<char>) -> Vec<isize> {
        let mut next = vec![0; p.len()];
        next[0] = -1;

        for c in 0..p.len()-1 { // next vector, >> 1 from partial match table
            // try possible common str from long to short
            let mut max_len = c;
            while max_len > 0 && (0..max_len).zip(c+1-max_len..c+1).any(|(i, j)| p[i] != p[j]) {
                max_len -= 1;
            }

            next[c+1] = max_len as isize
        }

        next
    }

    fn calc_next_improved(p: &Vec<char>) -> Vec<isize> {
        let mut next = vec![0isize; p.len()];
        next[0] = -1;
        next[1] = 0;

        for c in 1..p.len()-1 {
            let mut max_len = next[c];

            while max_len > 0 && p[max_len as usize] != p[c] {
                max_len = next[max_len as usize];
            }

            if p[max_len as usize] == p[c] {
                max_len += 1;
            }

            next[c+1] = max_len as isize
        }

        next
    }
}


#[cfg(test)]
mod tests {
    use std::pin::Pin;
    use std::ops::{ Generator, GeneratorState };

    use super::*;

    #[test]
    fn calc_right_next() {
        for flag in [ComputeNext::Naive, ComputeNext::Improved].iter() {
            assert_eq!(KMPPattern::new("abababzabababa", *flag).next, 
                        vec![-1, 0, 0, 1, 2, 3, 4, 0, 1, 2, 3, 4, 5, 6]);

            assert_eq!(KMPPattern::new("aaaaa", *flag).next, 
                        vec![-1, 0, 1, 2, 3]);

            assert_eq!(KMPPattern::new("aaabaab", *flag).next, 
                    vec![-1, 0, 1, 2, 0, 1, 2]);

            assert_eq!(KMPPattern::new("hi哦啦，hi哦啦", *flag).next, 
                    vec![-1, 0, 0, 0, 0, 0, 1, 2, 3]);
            }
    }

    #[test]
    fn find_all_works() {
        let p1 = KMPPattern::new("abbaaba", ComputeNext::Naive);
        assert_eq!(p1.find_all("abbaabbaababbaaba"), vec![4, 10]);

        let p2 = KMPPattern::new("aaa", ComputeNext::Naive);
        assert_eq!(p2.find_all("aaaaa"), vec![0, 1, 2]);

    }

    #[test]
    fn find_works() {
        let test = |pattern, text, v| {
            let p = KMPPattern::new(pattern, ComputeNext::Naive);    
            let mut gen_p = p.find(text);
            let mut myv = vec![];
            loop {
                match Pin::new(&mut gen_p).resume(()) {
                    GeneratorState::Yielded(j) => { println!("{}", j); myv.push(j)},
                    GeneratorState::Complete(_) => { println!("finished."); break }
                }
            }
    
            assert_eq!(myv, v)
        };

        test("abbaaba", "abbaabbaababbaaba", vec![4, 10]);
        test("aaa", "aaaaa", vec![0, 1, 2]);
    }
}