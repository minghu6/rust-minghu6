//! Rabin Karp (rolling hash) algorithm

use crate::string::{
    rolling_hash, AlphaBet, ComputeRollingHashError, PrefixRollingHash,
};


pub struct RabinKarpPatten<const N: usize = 1> {
    hash: [u64; N],
    pat_len: usize, // unicode length
}

pub struct RabinKarpText<const N: usize = 1> {
    prefix_hash: PrefixRollingHash<N>,
    map: Vec<usize>,
}


impl<const N: usize> RabinKarpText<N> {
    pub fn new(
        text: &str,
        alphabet: &dyn AlphaBet,
    ) -> Result<Self, ComputeRollingHashError> {
        let chars = alphabet.char_indices(text)?;
        let prefix_hash = PrefixRollingHash::new(
            chars.iter().map(|(_, c)| *c),
            alphabet.prime(),
        );

        let map = text.char_indices().map(|(i, _)| i).collect();

        Ok(Self { prefix_hash, map })
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }
}

impl<const N: usize> RabinKarpPatten<N> {
    pub fn new(
        pat: &str,
        alphabet: &dyn AlphaBet,
    ) -> Result<Self, ComputeRollingHashError> {
        let hash = rolling_hash(pat, alphabet)?;
        let pat_len = pat.chars().count();

        Ok(Self { hash, pat_len })
    }

    /// patlen unicode
    pub fn len(&self) -> usize {
        self.pat_len
    }

    pub fn find<'a>(
        &'a self,
        text: &'a RabinKarpText<N>,
        npows: &'a [[u64; N]],
    ) -> impl Iterator<Item = usize> + 'a {
        std::iter::from_coroutine(|| {
            if text.len() < self.pat_len {
                return;
            }

            for i in 0..text.len() - self.pat_len + 1 {
                let subhash =
                    text.prefix_hash.query(i..i + self.pat_len, npows);

                if subhash == self.hash {
                    // test false positive
                    // verify string text

                    yield text.map[i]
                }
            }
        })
    }
}


#[cfg(test)]
mod tests {

    use super::RabinKarpPatten;
    use crate::string::{
        create_npows, extend_npows, gen_test_case, gen_test_case_multiple,
        rk::RabinKarpText, AlphaBet, CommonChinese,
    };


    #[test]
    fn test_rk_single() {
        fn test<const N: usize>() {
            let alphabet = CommonChinese;
            let npows = create_npows(alphabet.prime(), 100);

            for (pat, text, expect) in gen_test_case() {
                let rk = RabinKarpPatten::<N>::new(&pat, &alphabet).unwrap();

                let text_prefix_hash =
                    RabinKarpText::new(&text, &alphabet).unwrap();

                let res: Vec<usize> =
                    rk.find(&text_prefix_hash, &npows).collect();

                assert_eq!(res, expect, "pat len: {}", pat.len());
            }

            println!("pass {N}.");
        }

        test::<1>();
        test::<2>();
    }

    #[test]
    fn test_rk_multi() {
        fn test<const N: usize>() {
            let alphabet = CommonChinese;
            let p = alphabet.prime();
            let mut npows = create_npows(p, 100);

            for (pats, text, expect) in gen_test_case_multiple() {
                let text_prefix_hash =
                    RabinKarpText::<N>::new(&text, &alphabet).unwrap();

                for pat in pats {
                    let rk = RabinKarpPatten::new(&pat, &alphabet).unwrap();
                    extend_npows(p, &mut npows, rk.len());

                    let res: Vec<usize> =
                        rk.find(&text_prefix_hash, &mut npows).collect();

                    assert_eq!(
                        res,
                        *expect.get(&pat).unwrap(),
                        "pat len: {}",
                        pat.len()
                    );
                }
            }

            println!("pass {N}.");
        }

        test::<1>();
        test::<2>();
    }
}
