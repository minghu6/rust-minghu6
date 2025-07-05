use std::cmp::{max, min};

use crate::string::{AlphaBet, PrefixRollingHash};


pub fn sub_palindromes_to_longest_palindromes(
    subs: (&[usize], &[usize]),
) -> (usize, Vec<usize>) {
    let (d1, d2) = subs;

    assert_eq!(d1.len(), d2.len());

    if d1.is_empty() {
        return (0, Vec::with_capacity(0));
    }

    let max_odd_r = *d1.into_iter().max().unwrap();
    let max_even_r = *d2.into_iter().max().unwrap();

    if max_even_r >= max_odd_r {
        (
            max_even_r * 2,
            d2.into_iter()
                .enumerate()
                .filter(|(_, x)| **x == max_even_r)
                .map(|(i, _)| i - max_even_r)
                .collect(),
        )
    }
    else {
        (
            max_odd_r * 2 - 1,
            d1.into_iter()
                .enumerate()
                .filter(|(_, x)| **x == max_odd_r)
                .map(|(i, _)| i + 1 - max_odd_r)
                .collect(),
        )
    }
}


/// O(n^2) -> (Odd, Even)
///
/// for odd length palindrome: "aba", r=2
pub fn find_sub_palindromes_brute_force(
    chars: &[char],
) -> (Vec<usize>, Vec<usize>) {
    if chars.is_empty() {
        return (Vec::with_capacity(0), Vec::with_capacity(0));
    }

    let n = chars.len();

    // Odd Symmetry

    let mut d1 = vec![1; n];

    for i in 1..n - 1 {
        let mut matched_r = min(i + 1, n - i);

        for r in 2..=matched_r {
            if chars[i - (r - 1)] != chars[i + (r - 1)] {
                matched_r = r - 1;
                break;
            }
        }

        d1[i] = matched_r;
    }

    // Even Symmetry
    let mut d2 = vec![0; n];

    for i in 0..n - 1 {
        let mut matched_r = min(i + 1, n - 1 - i);

        for r in 1..=matched_r {
            if chars[i + 1 - r] != chars[i + r] {
                matched_r = r - 1;
                break;
            }
        }

        d2[i + 1] = matched_r;
    }

    (d1, d2)
}


/// O(nlogn)
///
pub fn find_longest_palindromes_hash_native<const N: usize>(
    chars: &[char],
    alphabet: &dyn AlphaBet,
    npows: &[[u64; N]],
) -> (usize, Vec<usize>) {
    if chars.is_empty() {
        return (0, Vec::with_capacity(0));
    }

    let p = alphabet.prime();
    let n = chars.len();

    let mut char_ranks = Vec::with_capacity(n);

    for c in chars {
        char_ranks.push(alphabet.rank(*c).unwrap());
    }

    let forward_hash =
        PrefixRollingHash::<N>::new(char_ranks.iter().cloned(), p);
    let backward_hash =
        PrefixRollingHash::<N>::new(char_ranks.iter().rev().cloned(), p);


    // Odd Symmetry

    let max_odd_r = (n - 1) / 2;

    let mut odd_r = 0;
    let mut odd_i = vec![];

    if max_odd_r > 0 {
        for k in (0..=max_odd_r.ilog2()).rev() {
            let r = odd_r + 2_usize.pow(k);

            if r > max_odd_r {
                continue;
            }

            let d = r * 2 + 1;
            let mut found = false;

            for i in 0..n - d + 1 {
                let h1 = forward_hash.query(i..i + d, npows);
                let h2 = backward_hash.query(n - (i + d)..n - i, npows);

                if h1 == h2 {
                    if !found {
                        found = true;
                        odd_r = r;
                        odd_i.clear();
                    }

                    odd_i.push(i);
                }
            }
        }
    }

    // Even Symmetry

    let max_even_r = n / 2;

    let mut even_r = 0;
    let mut even_i = vec![];

    if max_even_r > 0 {
        for k in (0..=max_even_r.ilog2()).rev() {
            let r = even_r + 2_usize.pow(k);

            if r > max_even_r {
                continue;
            }

            let d = r * 2;
            let mut found = false;

            for i in 0..n - d + 1 {
                let h1 = forward_hash.query(i..i + d, npows);
                let h2 = backward_hash.query(n - (i + d)..n - i, npows);

                if h1 == h2 {
                    if !found {
                        found = true;
                        even_r = r;
                        even_i.clear();
                    }

                    even_i.push(i);
                }
            }
        }
    }


    if odd_r >= even_r {
        if odd_r == 0 {
            return (1, (0..n).collect());
        }

        (odd_r * 2 + 1, odd_i)
    }
    else {
        (even_r * 2, even_i)
    }
}


/// O(n)
pub fn find_longest_palindromes_hash_dp<const N: usize>(
    chars: &[char],
    alphabet: &dyn AlphaBet,
    npows: &[[u64; N]],
) -> (usize, Vec<usize>) {
    if chars.is_empty() {
        return (0, Vec::with_capacity(0));
    }

    let p = alphabet.prime();
    let n = chars.len();

    let mut char_ranks = Vec::with_capacity(n);

    for c in chars {
        char_ranks.push(alphabet.rank(*c).unwrap());
    }

    let forward_hash =
        PrefixRollingHash::<N>::new(char_ranks.iter().cloned(), p);
    let backward_hash =
        PrefixRollingHash::<N>::new(char_ranks.iter().rev().cloned(), p);

    let mut r = vec![0; n];
    r[0] = 1;

    let mut max_d = 1;

    for i in 1..n {
        for d in (1..=min(i + 1, r[i - 1] + 2)).rev() {
            let h1 = forward_hash.query(i + 1 - d..=i, npows);
            let h2 = backward_hash.query(n - (i + 1)..n - (i + 1) + d, npows);

            if h1 == h2 {
                max_d = max(max_d, d);
                r[i] = d;
                break;
            }
        }
    }

    (
        max_d,
        r.into_iter()
            .enumerate()
            .filter(|(_, d)| *d == max_d)
            .map(|(i, _)| i + 1 - max_d)
            .collect(),
    )
}


/// return d1
fn find_sub_palindromes_manacher_odd(chars: &[char]) -> Vec<usize> {
    if chars.is_empty() {
        return Vec::with_capacity(0);
    }

    let n = chars.len();

    let mut d1 = vec![1; n];

    let mut pl = 0;
    let mut pr = 0;

    for i in 1..n {
        let mut r = 1;

        if i < pr {
            let j = pl + pr - i;

            r = min(d1[j], pr - i + 1);
        }

        while i + r - 1 < n - 1
            && i - (r - 1) > 0
            && chars[i + r] == chars[i - r]
        {
            r += 1;
        }

        d1[i] = r;

        if i + r - 1 > pr {
            pr = i + r - 1;
            pl = i - (r - 1);
        }
    }

    d1
}


/// return d2
fn find_sub_palindromes_manacher_even(chars: &[char]) -> Vec<usize> {
    if chars.is_empty() {
        return Vec::with_capacity(0);
    }

    let n = chars.len();

    let mut d2 = vec![0; n]; // actual value from 1..n-1

    let mut pl = 0;
    let mut pr = 0;

    for i in 1..n {
        let mut r = 0;

        if i < pr {
            let j = pl + pr - i + 1;

            r = min(d2[j], pr - i + 1);
        }

        while i + r - 1 < n - 1
            && i - r > 0
            && chars[i + r] == chars[i - r - 1]
        {
            r += 1;
        }

        d2[i] = r;

        if i + r - 1 > pr {
            pr = i + r - 1;
            pl = i - r;
        }
    }

    d2
}


pub fn find_sub_palindromes_manacher(
    chars: &[char],
) -> (Vec<usize>, Vec<usize>) {
    let d1 = find_sub_palindromes_manacher_odd(chars);
    let d2 = find_sub_palindromes_manacher_even(chars);

    (d1, d2)
}


pub fn find_sub_palindromes_manacher_unify(
    chars: &[char],
) -> (Vec<usize>, Vec<usize>) {
    if chars.is_empty() {
        return (Vec::with_capacity(0), Vec::with_capacity(0));
    }

    let n = chars.len();

    let n2 = n * 2 - 1;
    let mut chars2 = vec!['#'; n2];

    for (i, c) in chars.into_iter().enumerate() {
        chars2[i * 2] = *c;
    }

    let d21 = find_sub_palindromes_manacher_odd(&chars2);

    let mut d1 = vec![1; n];
    let mut d2 = vec![0; n];

    // i=0 is nonsense but the result is ok
    for (i, v) in d21.into_iter().enumerate() {
        if i % 2 == 0 {
            d1[i / 2] = (v + 1) / 2;
        }
        else {
            d2[(i + 1) / 2] = v / 2;
        }
    }

    (d1, d2)
}



#[cfg(test)]
mod tests {
    use super::*;
    use crate::string::{create_npows, gen_random_dna_string, DigitsLetters};

    #[allow(unused)]
    #[test]
    fn test_find_longest_palindromes_case() {
        fn verify_case(text: &str, expect: (Vec<usize>, Vec<usize>)) {
            let chars: Vec<char> = text.chars().collect();
            let alphabet = DigitsLetters;
            let npows = create_npows::<1>(alphabet.prime(), 100);

            let ans_brute_force = find_sub_palindromes_brute_force(&chars);
            let manacher_d2 = find_sub_palindromes_manacher_even(&chars);
            let manacher_unify = find_sub_palindromes_manacher_unify(&chars);

            let ans_hash_native = find_longest_palindromes_hash_native(
                &chars, &alphabet, &npows,
            );
            let ans_hash_dp =
                find_longest_palindromes_hash_dp(&chars, &alphabet, &npows);

            assert_eq!(expect.1, manacher_d2, "| {text}");
            assert_eq!(expect, manacher_unify, "| {text}");
            // assert_eq!(expect, ans_brute_force, "{text}");
            // assert_eq!(expect, ans_hash_native, "{text}");
            // assert_eq!(expect, ans_hash_dp, "{text}");
        }

        verify_case("GA", (vec![1, 1], vec![0, 0]));

    }

    #[test]
    fn test_find_longest_palindromes_random() {
        let alphabet = DigitsLetters;
        let npows = create_npows::<1>(alphabet.prime(), 1000);

        for _ in 0..50 {
            let texts = [0, 1, 2, 3, 5, 10, 20, 30, 50, 100, 200]
                .into_iter()
                .map(|size| gen_random_dna_string(size));

            for text in texts {
                let chars: Vec<char> = text.chars().collect();

                // let manacher_d1 = find_sub_palindromes_manacher_odd(&chars);
                // let manacher_d2 = find_sub_palindromes_manacher_even(&chars);

                let manacher_unify =
                    find_sub_palindromes_manacher_unify(&chars);
                let manacher = find_sub_palindromes_manacher(&chars);
                let brute_force = find_sub_palindromes_brute_force(&chars);

                let brute_force_longest =
                    sub_palindromes_to_longest_palindromes((
                        &brute_force.0,
                        &brute_force.1,
                    ));

                let hash_native_longest = find_longest_palindromes_hash_native(
                    &chars, &alphabet, &npows,
                );
                let hash_dp_longest = find_longest_palindromes_hash_dp(
                    &chars, &alphabet, &npows,
                );

                assert_eq!(brute_force, manacher, "| {text}");
                assert_eq!(brute_force, manacher_unify, "| {text}");

                assert_eq!(brute_force_longest, hash_dp_longest, "| {text}");
                assert_eq!(hash_native_longest, hash_dp_longest, "| {text}");
            }
        }
    }
}
