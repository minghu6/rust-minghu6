#![allow(dead_code)]

use std::{cmp::max, collections::HashMap, fmt::Debug, hash::Hash};

use super::compute_pi;


const LARGE: usize = usize::MAX / 2;


////////////////////////////////////////////////////////////////////////////////
//// Traits

pub trait GetDelta1 {
    type Item;

    fn get_delta1(&self, char: &Self::Item) -> usize;
}

pub trait BMMatch: Sized {
    fn build_delta1_table<'a>(
        pat: &'a [Self],
    ) -> Box<dyn GetDelta1<Item = Self> + 'a>
    where
        Self: Eq + Hash,
    {
        let patlen = pat.len();
        let lastpos = patlen - 1;

        let mut map = HashMap::new();

        for (i, v) in pat.iter().enumerate() {
            map.insert(v, lastpos - i);
        }

        Box::new(HashMapWrapper { patlen, map })
    }
}


////////////////////////////////////////////////////////////////////////////////
//// Structures

struct HashMapWrapper<'a, T> {
    patlen: usize,
    map: HashMap<&'a T, usize>,
}


/// Slice based pattern match
pub struct BMPattern<'a, M> {
    pat: &'a [M],
    _delta1: Box<dyn GetDelta1<Item = M> + 'a>,
    delta2: Vec<usize>,
    k: usize,
}


pub struct SimplifiedBMPattern<'a> {
    pat_bytes: &'a [u8],
    delta1: [usize; 256],
}


pub struct HorspoolPattern<'a, M> {
    pat: &'a [M],
    _delta1: Box<dyn GetDelta1<Item = M> + 'a>,
}


pub struct SundayPattern<'a, M> {
    pat: &'a [M],
    _delta1: Box<dyn GetDelta1<Item = M> + 'a>,
}


pub struct B5STimePattern<'a, M> {
    pat: &'a [M],
    _delta1: Box<dyn GetDelta1<Item = M> + 'a>,
}


pub struct B5SSpacePattern<'a, M> {
    pat: &'a [M],
    patalphabet: SimpleBloomFilter,
    /// patpatlastpos delta1
    skip: usize,
}


/// patlen: 5   fp_rate: 0.03
///
/// patlen: 10  fp_rate: 0.07
///
/// patlen: 20  fp_rate: 0.14
///
/// patlen: 50  fp_rate: 0.32
///
/// patlen: 100 fp_rate: 0.54
///
/// patlen: 200 fp_rate: 0.79
///
/// patlen: 300 fp_rate: 0.90
#[repr(transparent)]
pub struct SimpleBloomFilter {
    data: u64,
}

////////////////////////////////////////////////////////////////////////////////
//// Implementations

impl SimpleBloomFilter {
    pub fn new() -> Self {
        SimpleBloomFilter { data: 0 }
    }

    #[inline]
    pub fn insert(&mut self, elem: &u8) {
        (self.data) |= 1u64 << (elem & 63);
    }

    #[inline]
    pub fn contains(&self, elem: &u8) -> bool {
        (self.data & (1u64 << (elem & 63))) != 0
    }
}


impl GetDelta1 for [usize; 256] {
    type Item = u8;

    fn get_delta1(&self, char: &Self::Item) -> usize {
        self[*char as usize] as _
    }
}


impl<'a, T: Eq + Hash> GetDelta1 for HashMapWrapper<'a, T> {
    type Item = T;

    fn get_delta1(&self, char: &Self::Item) -> usize {
        self.map.get(char).unwrap_or(&self.patlen).clone()
    }
}


impl BMMatch for u8 {
    fn build_delta1_table<'a>(
        pat: &'a [Self],
    ) -> Box<dyn GetDelta1<Item = Self> + 'a> {
        let mut delta1 = [pat.len(); 256];
        let lastpos = pat.len() - 1;

        for i in 0..lastpos {
            delta1[pat[i] as usize] = lastpos - i;
        }

        Box::new(delta1)
    }
}


impl<'a, T: AsRef<str>> From<&'a T> for BMPattern<'a, u8> {
    fn from(pat: &'a T) -> Self {
        pat.as_ref().into()
    }
}


impl<'a> From<&'a str> for BMPattern<'a, u8> {
    fn from(pat: &'a str) -> Self {
        pat.as_bytes().into()
    }
}


impl<'a, M> From<&'a [M]> for BMPattern<'a, M>
where
    M: BMMatch + Eq + Hash,
{
    fn from(pat: &'a [M]) -> Self {
        let _delta1 = M::build_delta1_table(pat);
        let (delta2, k) = build_delta2_table_improved_minghu6(pat);

        Self {
            pat,
            _delta1,
            delta2,
            k,
        }
    }
}


impl<'a, M: BMMatch + Eq> BMPattern<'a, M> {
    fn delta0(&self, char: &M) -> usize {
        if char == self.pat.last().unwrap() {
            LARGE
        }
        else {
            self.delta1(char)
        }
    }

    fn delta1(&self, char: &M) -> usize {
        self._delta1.get_delta1(char)
    }

    fn find(&self, string: &[M]) -> Option<usize> {
        if self.pat.is_empty() {
            return Some(0);
        }

        if string.is_empty() {
            return None;
        }

        let pat = self.pat;
        let patlastpos = pat.len() - 1;
        let stringlastpos = string.len() - 1;

        let mut i = patlastpos;

        if patlastpos == 0 {
            return string.iter().enumerate().find_map(|(i, v)| {
                if *v == self.pat[0] {
                    Some(i)
                }
                else {
                    None
                }
            });
        }

        if i > stringlastpos {
            return None;
        }

        loop {
            while i <= stringlastpos {
                i += self.delta0(&string[i]);
            }

            if i < LARGE {
                return None;
            }

            i -= LARGE + 1;

            let mut j = patlastpos - 1;

            loop {
                if string[i] == pat[j] {
                    if j == 0 {
                        return Some(i);
                    }

                    j -= 1;
                    i -= 1;

                    continue;
                }

                break;
            }

            i += max(self.delta1(&string[i]), self.delta2[j]);
        }
    }

    /// Find overlapping
    fn find_all(&'a self, string: &'a [M]) -> impl Iterator<Item = usize> + 'a
    where
        M: Debug,
    {
        std::iter::from_coroutine(
            #[coroutine]
            move || {
                let patlen = self.pat.len();
                let k = self.k;

                let mut suffix = string;
                let mut base = 0;

                while let Some(mut i) = self.find(suffix) {
                    yield i + base;

                    loop {
                        if suffix.len() < i + patlen + k {
                            return;
                        }

                        if suffix[i + patlen..i + patlen + k]
                            == self.pat[patlen - k..]
                        {
                            i += k;

                            yield i + base;
                        }
                        else {
                            break;
                        }
                    }

                    let shift = i + 1;

                    base += shift;

                    suffix = &suffix[shift..];
                }
            },
        )
    }
}


impl<'a, M> super::Find<'a, M> for BMPattern<'a, M>
where
    M: BMMatch + Eq,
{
    fn len(&self) -> usize {
        self.pat.len()
    }

    fn find(&self, string: &[M]) -> Option<usize> {
        self.find(string)
    }

    /// Find overlapping
    fn find_all(&'a self, string: &'a [M]) -> impl Iterator<Item = usize> + 'a
    where
        M: Debug,
    {
        self.find_all(string)
    }
}


impl<'a> SimplifiedBMPattern<'a> {
    pub fn new(pat: &'a str) -> Self {
        assert_ne!(pat.len(), 0);

        let pat_bytes = pat.as_bytes();
        let delta1 = Self::build_delta1_table(&pat_bytes);

        SimplifiedBMPattern { pat_bytes, delta1 }
    }

    fn delta0(&self, char: u8) -> usize {
        if char == self.pat_bytes[self.pat_bytes.len() - 1] {
            LARGE
        }
        else {
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
        let patlen = self.pat_bytes.len();
        let pat_last_pos = patlen - 1;
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
            while pat_index > 0
                && string_bytes[string_index] == self.pat_bytes[pat_index]
            {
                string_index -= 1;
                pat_index -= 1;
            }

            if pat_index == 0
                && string_bytes[string_index] == self.pat_bytes[0]
            {
                result.push(string_index);

                string_index += patlen;
            }
            else {
                string_index += max(
                    self.delta1[string_bytes[string_index] as usize],
                    pat_last_pos - pat_index + 1,
                );
            }
        }

        result
    }
}


impl<'a, T: AsRef<str>> From<&'a T> for HorspoolPattern<'a, u8> {
    fn from(pat: &'a T) -> Self {
        pat.as_ref().as_bytes().into()
    }
}


impl<'a> From<&'a str> for HorspoolPattern<'a, u8> {
    fn from(pat: &'a str) -> Self {
        pat.as_bytes().into()
    }
}


impl<'a, M> From<&'a [M]> for HorspoolPattern<'a, M>
where
    M: BMMatch + Eq + Hash,
{
    fn from(pat: &'a [M]) -> Self {
        let _delta1 = M::build_delta1_table(pat);

        Self { pat, _delta1 }
    }
}


impl<'a, M: Eq> HorspoolPattern<'a, M> {
    fn delta1(&self, char: &M) -> usize {
        self._delta1.get_delta1(char)
    }

    pub fn find(&self, string: &'a [M]) -> Option<usize> {
        if self.pat.is_empty() {
            return Some(0);
        }

        let stringlen = string.len();
        let patlastpos = self.pat.len() - 1;

        let mut i = patlastpos;

        while i < stringlen {
            if &string[i - patlastpos..=i] == self.pat {
                return Some(i - patlastpos);
            }

            i += self.delta1(&string[i]);
        }

        None
    }
}


impl<'a, M: Eq> super::Find<'a, M> for HorspoolPattern<'a, M> {
    fn len(&self) -> usize {
        self.pat.len()
    }

    fn find(&self, string: &[M]) -> Option<usize> {
        self.find(string)
    }
}


impl<'a, T: AsRef<str>> From<&'a T> for SundayPattern<'a, u8> {
    fn from(pat: &'a T) -> Self {
        pat.as_ref().as_bytes().into()
    }
}


impl<'a> From<&'a str> for SundayPattern<'a, u8> {
    fn from(pat: &'a str) -> Self {
        pat.as_bytes().into()
    }
}


impl<'a, M> From<&'a [M]> for SundayPattern<'a, M>
where
    M: BMMatch + Eq + Hash,
{
    fn from(pat: &'a [M]) -> Self {
        let _delta1 = M::build_delta1_table(pat);

        Self { pat, _delta1 }
    }
}

impl<'a, M: Eq> SundayPattern<'a, M> {
    fn delta1(&self, char: &M) -> usize {
        if char == self.pat.last().unwrap() {
            1
        }
        else {
            self._delta1.get_delta1(char) + 1
        }
    }

    pub fn find(&self, string: &'a [M]) -> Option<usize> {
        if self.pat.is_empty() {
            return Some(0);
        }

        let stringlen = string.len();
        let patlastpos = self.pat.len() - 1;

        let mut i = patlastpos;

        while i < stringlen {
            if &string[i - patlastpos..=i] == self.pat {
                return Some(i - patlastpos);
            }

            if i + 1 == stringlen {
                break;
            }

            i += self.delta1(&string[i + 1]);
        }

        None
    }
}


impl<'a, M: Eq> super::Find<'a, M> for SundayPattern<'a, M> {
    fn len(&self) -> usize {
        self.pat.len()
    }

    fn find(&self, string: &[M]) -> Option<usize> {
        self.find(string)
    }
}


impl<'a, T: AsRef<str>> From<&'a T> for B5STimePattern<'a, u8> {
    fn from(pat: &'a T) -> Self {
        pat.as_ref().as_bytes().into()
    }
}


impl<'a> From<&'a str> for B5STimePattern<'a, u8> {
    fn from(pat: &'a str) -> Self {
        pat.as_bytes().into()
    }
}


impl<'a, M> From<&'a [M]> for B5STimePattern<'a, M>
where
    M: BMMatch + Eq + Hash,
{
    fn from(pat: &'a [M]) -> Self {
        let _delta1 = M::build_delta1_table(pat);

        Self { pat, _delta1 }
    }
}


impl<'a, M: Eq> B5STimePattern<'a, M> {
    fn delta1(&self, char: &M) -> usize {
        if char == self.pat.last().unwrap() {
            0
        }
        else {
            self._delta1.get_delta1(char)
        }
    }

    pub fn find(&self, string: &'a [M]) -> Option<usize> {
        if self.pat.is_empty() {
            return Some(0);
        }

        let stringlen = string.len();
        let patlen = self.pat.len();
        let patlastpos = patlen - 1;

        let mut i = patlastpos;

        while i < stringlen {
            if &string[i - patlastpos..=i] == self.pat {
                return Some(i - patlastpos);
            }

            if i + 1 == stringlen {
                break;
            }

            if self.delta1(&string[i + 1]) == patlen {
                // sunday
                i += patlen + 1;
            }
            else {
                // horspool
                i += max(1, self.delta1(&string[i]));
            }
        }

        None
    }
}


impl<'a, M: Eq> super::Find<'a, M> for B5STimePattern<'a, M> {
    fn len(&self) -> usize {
        self.pat.len()
    }

    fn find(&self, string: &[M]) -> Option<usize> {
        self.find(string)
    }
}


impl<'a, T: AsRef<str>> From<&'a T> for B5SSpacePattern<'a, u8> {
    fn from(pat: &'a T) -> Self {
        pat.as_ref().as_bytes().into()
    }
}


impl<'a> From<&'a str> for B5SSpacePattern<'a, u8> {
    fn from(pat: &'a str) -> Self {
        pat.as_bytes().into()
    }
}


impl<'a> From<&'a [u8]> for B5SSpacePattern<'a, u8> {
    fn from(pat: &'a [u8]) -> Self {
        let (patalphabet, skip) = B5SSpacePattern::build(pat);

        Self { pat, patalphabet, skip }
    }
}


impl<'a> B5SSpacePattern<'a, u8> {
    fn build(pat: &'a [u8]) -> (SimpleBloomFilter, usize) {
        let mut patalphabet = SimpleBloomFilter::new();
        //let mut alphabet = FastBloomFilter::with_rate(p.len(), 0.15);
        let lastpos = pat.len() - 1;
        let mut skip = pat.len();

        for i in 0..pat.len() - 1 {
            patalphabet.insert(&pat[i]);

            if pat[i] == pat[lastpos] {
                skip = lastpos - i;
            }
        }

        patalphabet.insert(&pat[lastpos]);

        (patalphabet, skip)
    }

    pub fn find(&self, string: &'a [u8]) -> Option<usize> {
        if self.pat.is_empty() {
            return Some(0);
        }

        let stringlen = string.len();
        let patlen = self.pat.len();
        let patlastpos = patlen - 1;

        let mut i = patlastpos;

        while i < stringlen {
            if string[i] == self.pat[patlastpos] {
                if string[i-patlastpos..i] == self.pat[..patlastpos] {
                    return Some(i - patlastpos);
                }

                if i + 1 == stringlen {
                    break;
                }

                if !self.patalphabet.contains(&string[i + 1]) {
                    // sunday
                    i += patlen + 1;
                }
                else {
                    // horspool
                    i += self.skip;
                }
            }
            else {
                if i + 1 == stringlen {
                    break;
                }

                if !self.patalphabet.contains(&string[i + 1]) {
                    // sunday
                    i += patlen + 1;
                }
                else {
                    i += 1;
                }
            }
        }

        None
    }
}


impl<'a> super::Find<'a, u8> for B5SSpacePattern<'a, u8> {
    fn len(&self) -> usize {
        self.pat.len()
    }

    fn find(&self, string: &[u8]) -> Option<usize> {
        self.find(string)
    }
}


////////////////////////////////////////////////////////////////////////////////
//// Functions


pub fn build_delta2_table_naive(p: &[impl Eq]) -> Vec<usize> {
    let patlen = p.len();
    let lastpos = patlen - 1;
    let mut delta2 = vec![];

    for j in 0..patlen {
        let subpatlen = (lastpos - j) as isize;

        if subpatlen == 0 {
            delta2.push(0);
            break;
        }

        for i in (-subpatlen..=j as isize).rev() {
            // subpat 匹配
            if (i..i + subpatlen).zip(j + 1..patlen).all(
                |(rpr_index, subpat_index)| {
                    if rpr_index < 0 {
                        return true;
                    }

                    if p[rpr_index as usize] == p[subpat_index] {
                        return true;
                    }

                    false
                },
            ) && (i <= 0 || p[(i - 1) as usize] != p[j])
            {
                delta2.push((lastpos as isize - i) as usize);
                break;
            }
        }
    }

    delta2
}


pub fn build_delta2_table_improved_minghu6(
    pat: &[impl Eq],
) -> (Vec<usize>, usize) {
    if pat.is_empty() {
        return (vec![], 0);
    }

    let patlen = pat.len();
    let lastpos = patlen - 1;
    let mut delta2 = Vec::with_capacity(patlen);

    /* case-1: delta2[j] = 2 * lastpos - j */

    for i in 0..patlen {
        delta2.push(lastpos * 2 - i);
    }

    /* case-2: lastpos <= delata2[j] < 2 * lastpos - j */

    let pi = compute_pi(pat);
    let mut prefixlen = pi[lastpos];
    let mut i = 0;

    while prefixlen > 0 {
        for j in i..(patlen - prefixlen) {
            delta2[j] = 2 * lastpos - j - prefixlen;
        }

        i = patlen - prefixlen;
        prefixlen = pi[prefixlen - 1];
    }

    /* case-3: delata2[j] < lastpos */

    let k = patlen - pi[lastpos];
    let mut suffix = pi;
    suffix[lastpos] = patlen;

    let mut j = lastpos;

    for i in (0..patlen - 1).rev() {
        suffix[i] = j;

        while j < patlen && pat[i] != pat[j] {
            if delta2[j] > lastpos - i {
                delta2[j] = lastpos - i
            }

            j = suffix[j];
        }

        j -= 1;
    }

    delta2[lastpos] = 0;

    (delta2, k)
}


#[cfg(test)]
mod tests {
    use super::{
        super::{gen_test_case, FindStr},
        *,
    };


    #[test]
    fn bm_delta2_table_built_correctly() {
        let mut p;

        p = BMPattern::from("ABCXXXABC");
        assert_eq!(p.delta2, vec![13, 12, 11, 10, 9, 8, 10, 9, 0]);

        p = BMPattern::from("ABYXCDEYX");
        assert_eq!(p.delta2, vec![16, 15, 14, 13, 12, 11, 7, 9, 0]);

        p = BMPattern::from("abbaaba");
        assert_eq!(p.delta2, vec![11, 10, 9, 8, 5, 3, 0]);

        p = BMPattern::from("abaabaabaa");
        assert_eq!(p.delta2, vec![11, 10, 9, 11, 10, 9, 11, 10, 2, 0]);

        p = BMPattern::from("aaa");
        assert_eq!(p.delta2, vec![2, 2, 0]);
    }

    #[test]
    fn bm_find_all_fixeddata_works() {
        let mut p;

        p = BMPattern::from("abbaaba");
        assert_eq!(
            FindStr::find_all(&p, "abbaabbaababbaaba").collect::<Vec<_>>(),
            vec![4, 10]
        );

        p = BMPattern::from("aaa");
        assert_eq!(
            FindStr::find_all(&p, "aaaaa").collect::<Vec<_>>(),
            vec![0, 1, 2]
        );

        p = BMPattern::from("a");
        assert_eq!(FindStr::find_all(&p, "a").collect::<Vec<_>>(), vec![0]);

        p = BMPattern::from("abcd");
        assert_eq!(
            FindStr::find_all(&p, "abcdabcdabcabcd").collect::<Vec<_>>(),
            vec![0, 4, 11]
        );

        p = BMPattern::from("aaa");
        assert_eq!(
            FindStr::find_all(&p, "aaabbaaa").collect::<Vec<_>>(),
            vec![0, 5]
        );
    }

    #[test]
    fn bm_find_all_randomdata_works() {
        for (pat, string, result) in gen_test_case() {
            let r = FindStr::find_all(&BMPattern::from(&pat), &string)
                .collect::<Vec<_>>();
            if r != result {
                println!("pat:{}", pat);
                println!("string:{:#?}", string);
                println!("string len: {}", string.len());
                println!("match:");
                for pos in r {
                    if let Some(youpat) = string.get(pos..pos + pat.len()) {
                        println!("{}, ", youpat);
                    }
                }

                println!("\n\n");
            }
            assert_eq!(
                FindStr::find_all(&BMPattern::from(&pat), &string)
                    .collect::<Vec<_>>(),
                result
            )
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
        for (pat, text, result) in gen_test_case() {
            assert_eq!(
                SimplifiedBMPattern::new(pat.as_str()).find_all(text.as_str()),
                result
            )
        }
    }

    #[test]
    fn horspool_find_all_fixeddata_works() {
        let p1 = HorspoolPattern::from("abbaaba");
        assert_eq!(
            p1.find_all("abbaabbaababbaaba").collect::<Vec<_>>(),
            vec![4, 10]
        );

        let p2 = HorspoolPattern::from("aaa");
        assert_eq!(p2.find_all("aaaaa").collect::<Vec<_>>(), vec![0, 1, 2]);

        let p3 = HorspoolPattern::from("b");
        assert!(p3.find_all("aaaaa").collect::<Vec<_>>().is_empty());
    }

    #[test]
    fn horspool_find_all_randomdata_works() {
        for (pat, string, result) in gen_test_case() {
            assert_eq!(
                HorspoolPattern::from(&pat)
                    .find_all(&string)
                    .collect::<Vec<_>>(),
                result
            )
        }
    }

    #[test]
    fn sunday_find_all_fixeddata_works() {
        let p1 = SundayPattern::from("abbaaba");
        assert_eq!(
            p1.find_all("abbaabbaababbaaba").collect::<Vec<_>>(),
            vec![4, 10]
        );

        let p2 = SundayPattern::from("aaa");
        assert_eq!(p2.find_all("aaaaa").collect::<Vec<_>>(), vec![0, 1, 2]);

        let p3 = SundayPattern::from("b");
        assert!(p3.find_all("aaaaa").collect::<Vec<_>>().is_empty());
    }

    #[test]
    fn sunday_find_all_randomdata_works() {
        for (pat, text, result) in gen_test_case() {
            assert_eq!(
                SundayPattern::from(&pat)
                    .find_all(&text)
                    .collect::<Vec<_>>(),
                result
            )
        }
    }

    #[test]
    fn b5s_time_find_all_fixeddata_works() {
        let mut p;

        p = B5STimePattern::from("abbaaba");
        assert_eq!(
            p.find_all("abbaabbaababbaaba").collect::<Vec<_>>(),
            vec![4, 10]
        );

        p = B5STimePattern::from("aaa");
        assert_eq!(p.find_all("aaaaa").collect::<Vec<_>>(), vec![0, 1, 2]);

        p = B5STimePattern::from("b");
        assert!(p.find_all("aaaaa").collect::<Vec<_>>().is_empty());

        p = B5STimePattern::from("a");
        assert_eq!(p.find_all("a").collect::<Vec<_>>(), vec![0]);

        p = B5STimePattern::from("abcd");
        assert_eq!(
            p.find_all("abcdabcdabcabcd").collect::<Vec<_>>(),
            vec![0, 4, 11]
        );
    }

    #[test]
    fn b5s_time_find_all_randomdata_works() {
        for (pat, text, result) in gen_test_case() {
            assert_eq!(
                B5STimePattern::from(&pat)
                    .find_all(&text)
                    .collect::<Vec<_>>(),
                result
            )
        }
    }

    #[test]
    fn b5s_space_find_all_fixeddata_works() {
        let p1 = B5SSpacePattern::from("abbaaba");
        assert_eq!(
            p1.find_all("abbaabbaababbaaba").collect::<Vec<_>>(),
            vec![4, 10]
        );

        let p2 = B5SSpacePattern::from("aaa");
        assert_eq!(p2.find_all("aaaaa").collect::<Vec<_>>(), vec![0, 1, 2]);

        let p3 = B5SSpacePattern::from("b");
        assert!(p3.find_all("aaaaa").collect::<Vec<_>>().is_empty());
    }

    #[test]
    fn b5s_space_find_all_randomdata_works() {
        for (pat, string, result) in gen_test_case() {
            assert_eq!(
                B5SSpacePattern::from(&pat)
                    .find_all(&string)
                    .collect::<Vec<_>>(),
                result
            )
        }
    }
}
