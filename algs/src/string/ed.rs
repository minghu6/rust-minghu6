//! Edit Distance (ED)

/// Levenshtein Distance
pub fn lev_d(word1: &str, word2: &str) -> usize {
    let mut word1 = word1.as_bytes();
    let mut word2 = word2.as_bytes();

    if word1.len() > word2.len() {
        (word1, word2) = (word2, word1)
    }

    let n = word1.len();
    let m = word2.len();

    let mut cache: Vec<usize> = (0..=n).collect();

    for j in 1..=m {
        let mut pre1 = cache[0];
        cache[0] = j;

        for i in 1..=n {
            let pre0 = cache[i];

            if word1[i - 1] == word2[j - 1] {
                cache[i] = pre1;
            }
            else {
                let ans_insert = 1 + pre0;
                let ans_remove = 1 + cache[i - 1];
                let ans_replace = 1 + pre1;

                cache[i] = ans_insert.min(ans_remove).min(ans_replace);
            }

            pre1 = pre0;
        }
    }

    cache.pop().unwrap()
}
