use std::cmp::Ordering;

// index start from 0


pub fn compute_suffix_array_naive<'a>(pat: &'a [u8]) -> Vec<usize> {
    let patlen = pat.len();
    let mut sa = vec![0; patlen];
    let mut sa_s = vec![];

    for i in 0..patlen {
        sa_s.push(&pat[i..patlen]);
    }

    sa_s.sort();

    for i in 0..patlen {
        sa[i] = patlen - sa_s[i].len();
    }

    sa
}


pub fn compute_suffix_array_doubling<'a>(pat: &'a [u8]) -> Vec<usize> {
    let patlen = pat.len();
    let mut sa = vec![0; patlen];
    let mut rk = vec![0; patlen];

    for i in 0..patlen {
        rk[i] = pat[i];
        sa[i] = i;
    }

    // init w = 1
    sa.sort_unstable_by(|&x, &y|{
        rk[x].cmp(&rk[y])
    });
    let old_rk = rk.clone();
    let mut p = 0;
    for i in 0..patlen {
        rk[sa[i]] = p;

        if i+1 < patlen && old_rk[sa[i]] != old_rk[sa[i+1]] {
            p += 1;
        }
    }

    let mut w = 1;
    while w < patlen {
        sa.sort_unstable_by(|&x, &y| {
            if rk[x] == rk[y] {
                if x+w >= patlen {
                    return Ordering::Less;
                } else if y+w >= patlen {
                    return Ordering::Greater;
                } else {
                    return rk[x+w].cmp(&rk[y+w]);
                }
            } else {
                return rk[x].cmp(&rk[y]);
            }
        });

        let old_rk = rk.clone();
        let mut p = 0;
        for i in 0..patlen {
            rk[sa[i]] = p;

            if i+1 < patlen && old_rk[sa[i]] == old_rk[sa[i+1]]
            && sa[i] + w < patlen
            && sa[i+1] + w < patlen
            && old_rk[sa[i] + w] == old_rk[sa[i+1] + w] {
                continue
            }
            println!(" i:{}", i);
            p += 1;
        }

        println!("sa: {:?}", sa);
        println!("rk-0: {:?}", old_rk);
        println!("rk-1: {:?}", rk);
        println!("");

        w *= 2;
    }

    sa
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ensure_compute_sa_correctly_fixeddata() {
        let mut pat = "aabaaaab".as_bytes();
        let mut res = vec![3, 4, 5, 0, 6, 1, 7, 2];

        assert_eq!(compute_suffix_array_naive(pat), res);
        assert_eq!(compute_suffix_array_doubling(pat), res);

        pat = "abaab".as_bytes();
        res = vec![2, 3, 0, 4, 1];

        assert_eq!(compute_suffix_array_naive(pat), res);
        assert_eq!(compute_suffix_array_doubling(pat), res);

        pat = "banana$".as_bytes();
        res = vec![6, 5, 3, 1, 0, 4, 2];

        assert_eq!(compute_suffix_array_naive(pat), res);
        assert_eq!(compute_suffix_array_doubling(pat), res);

        pat = "ba".as_bytes();
        res = vec![1, 0];

        assert_eq!(compute_suffix_array_naive(pat), res);
        assert_eq!(compute_suffix_array_doubling(pat), res);

        pat = "b".as_bytes();
        res = vec![0];

        assert_eq!(compute_suffix_array_naive(pat), res);
        assert_eq!(compute_suffix_array_doubling(pat), res);
    }
}