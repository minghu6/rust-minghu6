use std::{mem::zeroed, ops::RangeBounds};

use common::parse_range;


const M: [u64; 2] = [1_000_000_000 + 9, 1_000_000_000 + 7];


macro_rules! check_n {
    () => {
        if N == 0 || N > 2 {
            unimplemented!("N should be in [1, 2], however {N} found")
        }
    };
}


////////////////////////////////////////////////////////////////////////////////
//// Trait

pub trait AlphaBet {
    /// None for char out of scope of the alphabet
    fn rank(&self, c: char) -> Option<u64>;
    fn len(&self) -> usize;
    fn prime(&self) -> u64;
    fn char_indices(
        &self,
        s: &str,
    ) -> Result<Vec<(usize, u64)>, ComputeRollingHashError> {
        // Create chars
        let mut chars = vec![];

        // Init chars
        for (i, c) in s.char_indices() {
            chars.push((
                i,
                self.rank(c)
                    .ok_or(ComputeRollingHashError::CharOutOfScope(c))?,
            ));
        }

        Ok(chars)
    }
}


////////////////////////////////////////////////////////////////////////////////
//// Structures

pub struct DigitsLetters;

/// Common CJK + CJK Symbols and Punctuation + ASCII
pub struct CommonChinese;


/// [msb] polynomial rolling computing
#[repr(transparent)]
pub struct PrefixRollingHash<const N: usize = 1>
// where
//     [(); 64 * N]:,
{
    nprefix: [Vec<u64>; N],
}


#[derive(Debug, Clone, Copy)]
pub enum ComputeRollingHashError {
    EmptyStr,
    CharOutOfScope(char),
}


////////////////////////////////////////////////////////////////////////////////
//// Implementations


/// String Prefix (MSB) Plynomial Rolling Hash
impl<const N: usize> PrefixRollingHash<N> {
    pub fn new(mut chars: impl Iterator<Item = u64>, p: u64) -> Self {
        check_n!();

        // Create nprefix
        let mut nprefix: [Vec<u64>; N] = unsafe { zeroed() };

        if let Some(c) = chars.next() {
            for n in 0..N {
                nprefix[n] = vec![c];
            }
        }
        else {
            for n in 0..N {
                nprefix[n] = Vec::with_capacity(0);
            }
        }

        for c in chars {
            // Init nprefix
            for n in 0..N {
                nprefix[n]
                    .push((nprefix[n].last().unwrap() * p % M[n] + c) % M[n]);
            }
        }

        Self { nprefix }
    }

    /// Unicode characters length
    pub fn len(&self) -> usize {
        self.nprefix[0].len()
    }

    pub fn query<R: RangeBounds<usize>>(
        &self,
        range: R,
        npows: &[[u64; N]],
    ) -> [u64; N] {
        let (l, r) = parse_range!(range, self.len());

        let mut res = [0; N];

        if l == 0 {
            for n in 0..N {
                res[n] = self.nprefix[n][r];
            }
        }
        else {
            for n in 0..N {
                // coefficent
                let coeff = npows[r - (l - 1)][n];

                let a = self.nprefix[n][r];
                let b = self.nprefix[n][l - 1] * coeff % M[n];

                res[n] = if a < b { M[n] - (b - a) } else { a - b };
            }
        };

        res
    }
}


impl AlphaBet for DigitsLetters {
    fn rank(&self, c: char) -> Option<u64> {
        Some(match c {
            '0'..='9' => c as u64 - '0' as u64 + 1,
            'A'..='Z' => c as u64 - 'A' as u64 + 1 + 10,
            'a'..='z' => c as u64 - 'a' as u64 + 1 + 10 + 26,
            _ => return None,
        })
    }

    fn len(&self) -> usize {
        26 * 2 + 10
    }

    fn prime(&self) -> u64 {
        79 // 0x4F
    }
}


impl AlphaBet for CommonChinese {
    fn rank(&self, c: char) -> Option<u64> {
        let c = c as u64;

        Some(match c {
            0..=127 => c + 1,
            0x3000..=0x303F => c - 0x3000 + 1 + 128,
            0x4E00..=0x9FFF => c - 0x4E00 + 1 + 128 + 0x40,
            _ => return None,
        })
    }

    /// 21184
    fn len(&self) -> usize {
        (0x9FFF - 0x4E00 + 1) + (127 - 0 + 1) + (0x303F - 0x3000 + 1)
    }

    fn prime(&self) -> u64 {
        21187
    }
}


////////////////////////////////////////////////////////////////////////////////
//// Functions

// fn pow(pows: &[u64; 64], m: u64, mut i: usize) -> u64 {
//     let mut mul = 1;
//     let mut k = 1;

//     while i > 0 {
//         if i % 2 > 0 {
//             mul = (mul * pows[k]) % m;
//         }

//         i >>= 1;
//         k += 1;
//     }

//     mul
// }


pub fn create_npows<const N: usize>(p: u64, len: usize) -> Vec<[u64; N]> {
    if len == 0 {
        return Vec::with_capacity(0);
    }

    let mut npows: Vec<[u64; N]> = Vec::with_capacity(len);

    extend_npows(p, &mut npows, len);

    npows
}


pub fn extend_npows<const N: usize>(
    p: u64,
    npows: &mut Vec<[u64; N]>,
    new_len: usize,
) {
    for i in npows.len()..new_len {
        npows.push([0; N]);

        for n in 0..N {
            if i == 0 {
                npows[i][n] = 1
            }
            else {
                npows[i][n] = npows[i - 1][n] * p % M[n]
            }
        }
    }
}


pub fn rolling_hash<const N: usize>(
    s: &str,
    alphabet: &dyn AlphaBet,
) -> Result<[u64; N], ComputeRollingHashError> {
    check_n!();

    let mut nacc = [0; N];

    if s.is_empty() {
        return Ok(nacc);
    }

    let p = alphabet.prime();

    for c in s.chars() {
        let c = alphabet
            .rank(c)
            .ok_or(ComputeRollingHashError::CharOutOfScope(c))?;

        for n in 0..N {
            if nacc[n] == 0 {
                nacc[n] = c;
            }
            else {
                nacc[n] = (nacc[n] * p % M[n] + c) % M[n];
            }
        }
    }

    Ok(nacc)
}



#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn test_alphabet() {
        let alphabet = DigitsLetters;

        assert_eq!(alphabet.rank('a'), Some(37));
        assert_eq!(alphabet.rank('z'), Some(62));
        assert_eq!(alphabet.rank(','), None);
    }

    #[test]
    fn test_rolling_hash() {
        let alphabet = CommonChinese;
        let npows = create_npows(alphabet.prime(), 10);

        let text = "adabcdacd";
        let text_chars = alphabet.char_indices(text).unwrap();

        let text_prefix_hash = PrefixRollingHash::new(
            text_chars.iter().map(|(_, c)| *c),
            alphabet.prime(),
        );

        for l in 1..=text.len() {
            for i in 0..text.len() - l {
                let mut acc = 0;

                for j in i..i + l {
                    let c = text_chars[j].1;
                    let part = c * npows[l - (j - i) - 1][0] % M[0];

                    acc = (acc + part) % M[0];
                }

                assert_eq!(
                    acc,
                    text_prefix_hash.query(i..i + l, &npows)[0],
                    "{i}, {l}"
                );
            }
        }

        let pat = "abc";
        let pat_hash = rolling_hash::<1>(pat, &alphabet).unwrap();

        assert_eq!(pat_hash, text_prefix_hash.query(2..=4, &npows));
    }

    // #[test]
    // fn test_pow() {
    //     let alphabet = CommonChinese;
    //     let p = alphabet.prime();

    //     let mut npows = [[0; 64]; 1];

    //     alphabet.build_pows(M[0], &mut npows[0]);

    //     for i in 1..10000 {
    //         let mut mul = 1;

    //         for _ in 1..=i {
    //             mul = mul * p % M[0];
    //         }

    //         assert_eq!(pow(&npows[0], M[0], i), mul);
    //     }
    // }
}
