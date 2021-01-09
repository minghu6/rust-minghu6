use std::cmp::Ordering;
use std::cmp::min;

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
            p += 1;
        }

        w *= 2;
    }

    sa
}


pub fn compute_suffix_array_doubling_radix<'a>(pat: &'a [u8]) -> Vec<usize> {
    let patlen = pat.len();
    let mut sa = vec![0; patlen];
    let mut rk = vec![0; patlen];

    for i in 0..patlen {
        rk[i] = pat[i] as usize;
        sa[i] = i;
    }

    // init w = 1
    sa.sort_unstable_by(|&x, &y|{
        rk[x].cmp(&rk[y])
    });
    let mut old_rk = rk.clone();
    let mut p = 0;
    for i in 0..patlen {
        rk[sa[i]] = p;

        if i + 1 < patlen && old_rk[sa[i]] != old_rk[sa[i + 1]] {
            p += 1;
        }
    }

    let mut w = 1;
    while w < patlen {
        // 双关键字的基数排序
        // 第二关键字计数排序
        let mut cnt = vec![0; patlen];
        let mut old_sa = sa.clone();
        for i in 0..patlen {
            if old_sa[i] < patlen - w {
                cnt[rk[old_sa[i] + w]] += 1;
            }
        }
        for i in 1..patlen { cnt[i] += cnt[i - 1] }
        for i in 0..w {
            sa[i] = patlen + i - w;
        }
        for i in (0..patlen).rev() {
            if old_sa[i] < patlen - w {
                sa[cnt[rk[old_sa[i] + w]] - 1 + w] = old_sa[i];
                cnt[rk[old_sa[i] + w]] -= 1;
            }
        }

        // 第一关键字排序
        cnt.fill(0);
        old_sa = sa.clone();
        for i in 0..patlen { cnt[rk[old_sa[i]]] += 1 }
        for i in 1..patlen { cnt[i] += cnt[i - 1] }
        for i in (0..patlen).rev() {
            sa[cnt[rk[old_sa[i]]] - 1] = old_sa[i];
            cnt[rk[old_sa[i]]] -= 1;
        }

        old_rk = rk.clone();
        p = 0;
        for i in 0..patlen {
            rk[sa[i]] = p;

            if i+1 < patlen && old_rk[sa[i]] == old_rk[sa[i+1]]
            && sa[i] + w < patlen
            && sa[i+1] + w < patlen
            && old_rk[sa[i] + w] == old_rk[sa[i+1] + w] {
                continue
            }
            p += 1;
        }

        w *= 2;
    }
    sa
}

pub fn compute_suffix_array_doubling_radix_improved<'a>(pat: &'a [u8]) -> Vec<usize> {
    let patlen = pat.len();
    let mut sa = vec![0; patlen];
    let mut rk = vec![0; patlen];

    for i in 0..patlen {
        rk[i] = pat[i] as usize;
        sa[i] = i;
    }

    // init w = 1
    sa.sort_unstable_by(|&x, &y|{
        rk[x].cmp(&rk[y])
    });
    let mut old_rk = rk.clone();
    let mut p = 0;
    for i in 0..patlen {
        rk[sa[i]] = p;

        if i + 1 < patlen && old_rk[sa[i]] != old_rk[sa[i + 1]] {
            p += 1;
        }
    }
    println!("sa: {:?}", sa);
    let mut w = 1;
    while w < patlen {
        // 双关键字的基数排序
        // 第二关键字计数排序的实质性内容
        let mut cnt;
        let mut old_sa = sa.clone();
        for i in 0..w {
            sa[i] = patlen + i - w;
        }
        let mut k = w;
        for i in 0..patlen {
            if old_sa[i] >= w && k < patlen {
                sa[k] = old_sa[i] - w;
                k += 1;
            }
        }

        // 第一关键字排序
        cnt = vec![0; patlen];
        old_sa = sa.clone();
        let m = min(p + 1, patlen);
        for i in 0..patlen { cnt[rk[old_sa[i]]] += 1 }
        for i in 1..m { cnt[i] += cnt[i - 1] }
        for i in (0..patlen).rev() {
            sa[cnt[rk[old_sa[i]]] - 1] = old_sa[i];
            cnt[rk[old_sa[i]]] -= 1;
        }

        old_rk = rk.clone();
        p = 0;
        for i in 0..patlen {
            rk[sa[i]] = p;

            if i+1 < patlen && old_rk[sa[i]] == old_rk[sa[i+1]]
            && sa[i] + w < patlen
            && sa[i+1] + w < patlen
            && old_rk[sa[i] + w] == old_rk[sa[i+1] + w] {
                continue
            }
            p += 1;
        }

        w *= 2;
    }

    sa
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ensure_compute_sa_correctly_fixeddata() {
        for (pat, res) in [("aabaaaab".as_bytes(), vec![3, 4, 5, 0, 6, 1, 7, 2]),
                           ("abaab".as_bytes(), vec![2, 3, 0, 4, 1]),
                           ("banana$".as_bytes(), vec![6, 5, 3, 1, 0, 4, 2]),
                           ("ba".as_bytes(), vec![1, 0]),
                           ("b".as_bytes(), vec![0])].iter() {

            assert_eq!(compute_suffix_array_naive(pat), res.clone());
            assert_eq!(compute_suffix_array_doubling(pat), res.clone());
            assert_eq!(compute_suffix_array_doubling_radix(pat), res.clone());
            assert_eq!(compute_suffix_array_doubling_radix_improved(pat), res.clone());
        }
    }
}