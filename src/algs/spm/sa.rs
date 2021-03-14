// use std::cmp::Ordering;
// use std::cmp::min;

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

fn _cmp_rank(x: usize, y: usize, w: usize, old_rk: &Vec<usize>) -> bool {
    old_rk[x] == old_rk[y] && old_rk[x + w] == old_rk[y + w]
}

fn _calc_extend_capacity(patlen: usize) -> usize {
    2f32.powi((patlen as f32).log2().ceil() as i32) as usize
}
// fn _cmp_rank2( w: usize, patlen: usize, i:usize, old_rk: &Vec<usize>, sa: &Vec<usize>) -> bool {
//     i+1<patlen && sa[i]+w <patlen && sa[i+1]+w <patlen && old_rk[sa[i]] == old_rk[sa[i+1]] && old_rk[sa[i] + w] == old_rk[sa[i+1] + w]
// }

// binary lifting
pub fn suffix_array_bl<'a>(pat: &'a [u8]) -> Vec<usize> {
    // let patlen = pat.len();
    // let mut sa = vec![0; patlen];
    // let mut rk = vec![0usize; patlen];

    // for i in 0..patlen {
    //     rk[i] = pat[i] as usize;
    //     sa[i] = i;
    // }

    // // init w = 1
    // sa.sort_unstable_by(|&x, &y|{
    //     rk[x].cmp(&rk[y])
    // });
    // let old_rk = rk.clone();
    // let mut p = 0;
    // for i in 0..patlen {
    //     rk[sa[i]] = p;

    //     if i+1 < patlen && old_rk[sa[i]] != old_rk[sa[i+1]] {
    //         p += 1;
    //     }
    // }

    // let mut w = 1;
    // while w < patlen {
    //     sa.sort_unstable_by(|&x, &y| {
    //         if rk[x] == rk[y] {
    //             if x+w >= patlen {
    //                 return Ordering::Less;
    //             } else if y+w >= patlen {
    //                 return Ordering::Greater;
    //             } else {
    //                 return rk[x+w].cmp(&rk[y+w]);
    //             }
    //         } else {
    //             return rk[x].cmp(&rk[y]);
    //         }
    //     });

    //     let old_rk = rk.clone();
    //     p = 0;
    //     (0..patlen).into_iter().for_each(|i| {
    //         rk[sa[i]] = p;
    //         if !_cmp_rank2( w, patlen, i, &old_rk, &sa) { p += 1; };
    //     });

    //     w *= 2;
    // }
    // sa

    let patlen = pat.len();
    let extend_capacity = _calc_extend_capacity(patlen);
    let mut rk = vec![0usize; patlen + extend_capacity + 1];  // index start from 1； 0 代表越界无穷小
    let mut sa = vec![0; patlen + extend_capacity + 1];  // index start from 1, 编号也从1开始

    for i in 1..patlen + 1 {
        rk[i] = pat[i - 1] as usize;
        sa[i] = i;
    }

    // init w = 1
    sa[1..patlen + 1].sort_unstable_by(|&x, &y|{
        rk[x].cmp(&rk[y])
    });
    let mut old_rk = rk.clone();
    let mut p = 0;
    for i in 1..patlen + 1 {
        if old_rk[sa[i]] != old_rk[sa[i - 1]] {
            p += 1;
        }

        rk[sa[i]] = p;
    }

    let mut w = 1;
    while w < patlen {
        sa[1..patlen + 1].sort_unstable_by(|&x, &y| {
            if rk[x] == rk[y] { rk[x + w].cmp(&rk[y + w]) } else { rk[x].cmp(&rk[y]) }
        });

        old_rk = rk.clone();
        p = 0;
        (1..patlen + 1).into_iter().for_each(|i| {
            rk[sa[i]] = if _cmp_rank(sa[i], sa[i - 1], w, &old_rk) { p } else { p += 1; p };
        });

        w *= 2;
    }

    sa.into_iter()
      .skip(1)
      .take(patlen)
      .map(|x| { x - 1 })
      .collect::<Vec<usize>>()
}


pub fn suffix_array_bl_radix<'a>(pat: &'a [u8]) -> Vec<usize> {
    let patlen = pat.len();
    let extend_capacity = _calc_extend_capacity(patlen);
    let mut rk = vec![0usize; patlen + extend_capacity + 1];
    let mut sa = vec![0; patlen + extend_capacity + 1];
    for i in 1..patlen + 1 {
        rk[i] = pat[i - 1] as usize;
        sa[i] = i;
    }

    // init w = 1
    let mut cnt = vec![0; 256 + _calc_extend_capacity(256) + 1];
    let mut old_sa = sa.clone();
    for i in 1..patlen + 1 { cnt[rk[old_sa[i]]] += 1 }
    for i in 1..256 + 1 { cnt[i] += cnt[i - 1] }
    for i in (1..patlen + 1).rev() {
        sa[cnt[rk[old_sa[i]]]] = old_sa[i];
        cnt[rk[old_sa[i]]] -= 1;
    }
    let mut old_rk = rk.clone();
    let mut p = 0;
    for i in 1..patlen + 1 {
        if old_rk[sa[i]] != old_rk[sa[i - 1]] {
            p += 1;
        }

        rk[sa[i]] = p;
    }

    let mut w = 1;
    while w < patlen {
        // 双关键字的基数排序
        // 第二关键字计数排序
        cnt = vec![0; patlen + extend_capacity + 1];
        old_sa = sa.clone();
        for i in 1..patlen + 1 {
            cnt[rk[old_sa[i] + w]] += 1;
        }
        for i in 1..patlen + 1 { cnt[i] += cnt[i - 1] }
        for i in (1..patlen + 1).rev() {
            sa[cnt[rk[old_sa[i] + w]]] = old_sa[i];
            cnt[rk[old_sa[i] + w]] -= 1;
        }

        // 第一关键字排序
        cnt.fill(0);
        old_sa = sa.clone();
        for i in 1..patlen + 1 { cnt[rk[old_sa[i]]] += 1 }
        for i in 1..patlen + 1 { cnt[i] += cnt[i - 1] }
        for i in (1..patlen + 1).rev() {
            sa[cnt[rk[old_sa[i]]]] = old_sa[i];
            cnt[rk[old_sa[i]]] -= 1;
        }

        old_rk = rk.clone();
        p = 0;
        for i in 1..patlen + 1 {
            if !(old_rk[sa[i]] == old_rk[sa[i - 1]] && old_rk[sa[i] + w] == old_rk[sa[i - 1] + w]) {
                p += 1;
            }

            rk[sa[i]] = p;
        }

        w *= 2;
    }

    sa.into_iter()
    .skip(1)
    .take(patlen)
    .map(|x| { x - 1 })
    .collect::<Vec<usize>>()
}

pub fn suffix_array_bl_radix_improved<'a>(pat: &'a [u8]) -> Vec<usize> {
    let patlen = pat.len();
    let extend_capacity = _calc_extend_capacity(patlen);
    let mut rk = vec![0usize; patlen + extend_capacity + 1];
    let mut sa = vec![0; patlen + extend_capacity + 1];
    for i in 1..patlen + 1 {
        rk[i] = pat[i - 1] as usize;
        sa[i] = i;
    }

    // init w = 1
    let mut cnt = vec![0; 256 + _calc_extend_capacity(256) + 1];
    let mut old_sa = sa.clone();
    for i in 1..patlen + 1 { cnt[rk[old_sa[i]]] += 1 }
    for i in 1..256 + 1 { cnt[i] += cnt[i - 1] }
    for i in (1..patlen + 1).rev() {
        sa[cnt[rk[old_sa[i]]]] = old_sa[i];
        cnt[rk[old_sa[i]]] -= 1;
    }
    let mut old_rk = rk.clone();
    let mut p = 0;
    for i in 1..patlen + 1 {
        if old_rk[sa[i]] != old_rk[sa[i - 1]] {
            p += 1;
        }

        rk[sa[i]] = p;
    }

    let mut w = 1;
    while w < patlen {
        // 双关键字的基数排序
        // 第二关键字计数排序的实质性内容
        old_sa = sa.clone();
        for i in 1..w + 1 {
            sa[i] = patlen + i - w;
        }
        let mut k = w + 1;
        for i in 1..patlen + 1 {
            if old_sa[i] > w {
                sa[k] = old_sa[i] - w;
                k += 1;
            }
        }

        // 第一关键字排序
        cnt = vec![0; patlen + extend_capacity + 1];
        old_sa = sa.clone();
        for i in 1..patlen + 1 { cnt[rk[old_sa[i]]] += 1 }
        for i in 1..patlen + 1 { cnt[i] += cnt[i - 1] }
        for i in (1..patlen + 1).rev() {
            sa[cnt[rk[old_sa[i]]]] = old_sa[i];
            cnt[rk[old_sa[i]]] -= 1;
        }

        old_rk = rk.clone();
        p = 0;
        (1..patlen + 1).into_iter().for_each(|i| {
            rk[sa[i]] = if _cmp_rank(sa[i], sa[i - 1], w, &old_rk) { p } else { p += 1; p };
        });

        w *= 2;
    }

    sa.into_iter()
    .skip(1)
    .take(patlen)
    .map(|x| { x - 1 })
    .collect::<Vec<usize>>()
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::super::super::test::spm::gen_sa_test_case;
    #[test]
    fn ensure_compute_sa_correctly_fixeddata() {
        for (pat, res) in [("aabaaaab", vec![3, 4, 5, 0, 6, 1, 7, 2]),
                           ("abaab", vec![2, 3, 0, 4, 1]),
                           ("banana$", vec![6, 5, 3, 1, 0, 4, 2]),
                           ("ba", vec![1, 0]),
                           ("b", vec![0])].iter() {

            assert_eq!(compute_suffix_array_naive(pat.as_bytes()), res.clone());
            assert_eq!(suffix_array_bl(pat.as_bytes()), res.clone());
            assert_eq!(suffix_array_bl_radix(pat.as_bytes()), res.clone());
            assert_eq!(suffix_array_bl_radix_improved(pat.as_bytes()), res.clone());
        }
    }

    #[test]
    fn ensure_compute_sa_correctly_randomdata() {
        for (pat, res) in gen_sa_test_case() {
            assert_eq!(compute_suffix_array_naive(pat.as_bytes()), res.clone());
            assert_eq!(suffix_array_bl(pat.as_bytes()), res.clone());
            assert_eq!(suffix_array_bl_radix(pat.as_bytes()), res.clone());
            assert_eq!(suffix_array_bl_radix_improved(pat.as_bytes()), res.clone());
        }
    }
}