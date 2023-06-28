use std::cmp::min;

use crate::string::{AlphaBet, PrefixRollingHash};


/// -> (left, len)
pub fn find_the_longest_palindrome_brute_force(text: &str) -> (usize, usize) {
    assert!(!text.is_empty());

    let chars: Vec<char> = text.chars().collect();
    let n = chars.len();

    // Odd Symmetry

    let mut odd_r = 0;
    let mut odd_i = 0;


    for i in 1..n - 1 {
        let mut matched_r = min(i, n - 1 - i);

        for r in 1..=matched_r {

            if chars[i - r] != chars[i + r] {
                matched_r = r-1;

                break;
            }
        }

        if matched_r > odd_r {
            odd_r = matched_r;
            odd_i = i;
        }
    }

    // Even Symmetry

    let mut even_r = 0;
    let mut even_i = 0;

    for i in 0..n - 1 {
        let mut matched_r = min(i + 1, n - 1 - i);

        for r in 1..=matched_r {
            if chars[i + 1 - r] != chars[i + r] {
                matched_r = r-1;

                break;
            }
        }

        if matched_r > even_r {
            even_r = matched_r;
            even_i = i;
        }
    }

    if odd_r >= even_r {
        (odd_i-odd_r, odd_r * 2 + 1)
    }
    else {
        (even_i+1-even_r, even_r * 2)
    }
}


/// -> (left, len)
///
/// Note: single char is also a palindrome,
pub fn find_the_longest_palindrome_hash_native<const N: usize>(
    text: &str,
    alphabet: &dyn AlphaBet,
    npows: &[[u64; N]],
) -> (usize, usize) {
    assert!(!text.is_empty());

    let chars = alphabet.char_indices(text).unwrap();
    let p = alphabet.prime();

    let forward_hash =
        PrefixRollingHash::<N>::new(chars.iter().map(|(_, c)| *c), p);
    let backward_hash =
        PrefixRollingHash::<N>::new(chars.iter().map(|(_, c)| *c).rev(), p);

    let n = chars.len();

    // Odd Symmetry

    let max_odd_r = (n - 1) / 2;

    let mut odd_rl = 0;  // range left
    let mut odd_r = 0;

    if max_odd_r > 0 {
        for k in (0..=max_odd_r.ilog2()).rev() {
            let r = odd_r + 2_usize.pow(k);

            if r > max_odd_r {
                continue;
            }

            let d = r * 2 + 1;

            for rl in 0..n-d+1 {
                let h1 = forward_hash.query(rl..rl+d, npows);
                let h2 = backward_hash.query(n-(rl+d)..n-rl, npows);

                if h1 == h2 {
                    odd_r = r;
                    odd_rl = rl;

                    break;
                }
            }
        }
    }

    // Even Symmetry

    let max_even_r = n / 2;

    let mut even_rl = 0;
    let mut even_r = 0;

    if max_even_r > 0 {
        for k in (0..=max_even_r.ilog2()).rev() {
            let r = even_r + 2_usize.pow(k);

            if r > max_even_r {
                continue;
            }

            let d = r * 2;

            for rl in 0..n-d+1 {
                let h1 = forward_hash.query(rl..rl+d, npows);
                let h2 = backward_hash.query(n-(rl+d)..n-rl, npows);

                if h1 == h2 {
                    even_r = r;
                    even_rl = rl;

                    break;
                }
            }
        }
    }

    if odd_r >= even_r {
        (odd_rl, odd_r * 2 + 1)
    }
    else {
        (even_rl, even_r * 2)
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use crate::string::{gen_random_dna_text, DigitsLetters, create_npows};

    #[test]
    fn test_find_the_longest_palindromic_manually() {
        let text = "GCTCAAGCAC";

        let alphabet = DigitsLetters;
        let npows = create_npows::<1>(alphabet.prime(), 100);

        let ans = find_the_longest_palindrome_hash_native(text, &alphabet, &npows);
        assert_eq!(ans, (1, 3), "{text}");

        let text = "GAAAGGAATATTTCCTGTGA";
        let ans = find_the_longest_palindrome_hash_native(text, &alphabet, &npows);
        assert_eq!(ans.1, 6, "{text}");

        let text = "TCGGAATAAG";
        let ans = find_the_longest_palindrome_brute_force(text);
        assert_eq!(ans.1, 7, "{ans:?} | {text}");
        let ans = find_the_longest_palindrome_hash_native(text, &alphabet, &npows);
        assert_eq!(ans.1, 7, "{ans:?} | {text}");

        let text = "CGGGGGTTCG";
        let ans = find_the_longest_palindrome_brute_force(text);
        assert_eq!(ans.1, 5, "{ans:?} | {text}");
    }

    #[test]
    fn test_find_the_longest_palindromic_random() {

        let alphabet = DigitsLetters;
        let npows = create_npows::<1>(alphabet.prime(), 1000);

        for _ in 0..10 {
            let texts = [1, 10, 20, 30, 50, 100, 200]
            .into_iter()
            .map(|size| gen_random_dna_text(size));

            for text in texts {
                let brute_force = find_the_longest_palindrome_brute_force(&text);
                let hash = find_the_longest_palindrome_hash_native(
                    &text,
                    &alphabet,
                    &npows
                );

                assert_eq!(brute_force.1, hash.1, "{brute_force:?} / {hash:?} | {text}");
            }
        }
    }
}
