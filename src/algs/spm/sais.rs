const LTYPE: bool = false;
const STYPE: bool = true;

// temporary utils for stable version to patch `fill` method
// #[inline]
// fn fill<T: Copy>(pat: &mut [T], value: T) {
//     if let Some((last, elems)) = pat.split_last_mut() {
//         for el in elems {
//             el.clone_from(&value);
//         }

//         *last = value
//     }
// }

#[inline]
fn reset_sbucket(sbucket: &mut Vec<usize>, prefix_sum: &Vec<usize>) {
    for i in 1..prefix_sum.len() {
        sbucket[i] = prefix_sum[i] - 1;
    }
    sbucket[0] = 0;
}

#[inline]
fn reset_lbucket(lbucket: &mut Vec<usize>, prefix_sum: &Vec<usize>) {
    for i in 1..prefix_sum.len() {
        lbucket[i] = prefix_sum[i - 1];
    }
    lbucket[0] = 0;
}

#[inline]
fn is_lms_char(suf_t: &Vec<bool>, x: usize) -> bool {
    x >= 1 && suf_t[x - 1] == LTYPE && suf_t[x] == STYPE
}

#[inline]
fn lmssubstr_eq(pat: &[usize], x: usize, y: usize, suf_t: &Vec<bool>) -> bool {
    let mut x = x;
    let mut y = y;
    loop {
        if pat[x] != pat[y] { return false }
        x += 1;
        y += 1;

        if is_lms_char(suf_t, x) {
            if is_lms_char(suf_t, y) {
                return pat[x] == pat[y] && suf_t[x] == suf_t[y];
            }
            return false;
        }
        if is_lms_char(suf_t, y) {
            if is_lms_char(suf_t, x) {
                return pat[x] == pat[y] && suf_t[x] == suf_t[y];
            }
            return false;
        }
    }
}

fn induced_sort(pat: &[usize],
                sa: &mut [usize],
                suf_t: &Vec<bool>,
                lbucket: &mut Vec<usize>,
                sbucket: &mut Vec<usize>,
                prefix_sum: &Vec<usize>
            ) {

    for i in 0..pat.len() {
        if sa[i] > 0 && suf_t[sa[i] - 1] == LTYPE {
            let lfp = lbucket[pat[sa[i] - 1]];
            sa[lfp] = sa[i] - 1;
            lbucket[pat[sa[i] - 1]] += 1;
        }
    }
    reset_lbucket(lbucket, prefix_sum);
    for i in (0..pat.len()).rev() {
        if sa[i] > 0 && suf_t[sa[i] - 1] == STYPE {
            let rfp = sbucket[pat[sa[i] - 1]];
            sa[rfp] = sa[i] - 1;
            sbucket[pat[sa[i] - 1]] -= 1;
        }
    }
    reset_sbucket(sbucket, prefix_sum);
}

// alphabet start from 1; 0 is used for sentinel.
fn _suffix_array_sais(pat: &[usize], sa: &mut [usize], alphabet: usize) {
    let patlen = pat.len();
    let patlastpos = patlen - 1;
    let mut suf_t = vec![false; patlen];  // 后缀类型数组
    let mut lms_pos = vec![0; patlen];  // LMS 子串位置
    let mut lmsname = vec![0; patlen];  // 按照pat上顺序放置的重命名后的LMS子串
    let mut prefix_sum = vec![0; alphabet];  // 只是用作计数排序里的前缀和数组
    let mut lbucket = vec![0; alphabet];  // 记录每个桶的桶头位置
    let mut sbucket = vec![0; alphabet];  // 记录每个桶的桶尾位置

    // 构建辅助数组
    for i in 0..patlen { prefix_sum[pat[i]] += 1 }
    for i in 1..alphabet {
        prefix_sum[i] += prefix_sum[i - 1];
        lbucket[i] = prefix_sum[i - 1];
        sbucket[i] = prefix_sum[i] - 1;
    }
    suf_t[patlastpos] = STYPE;
    for i in (0..patlen - 1).rev() {
        if pat[i] < pat[i + 1] {
            suf_t[i]= STYPE
        } else if pat[i] > pat[i + 1] {
            suf_t[i] = LTYPE
        } else {
            suf_t[i] = suf_t[i + 1];
        }
    }

    // 寻找LMS字符
    let mut lms_cnt = 0;
    for i in 1..patlen {
        if is_lms_char(&suf_t, i) {
            lms_pos[lms_cnt] = i;
            lms_cnt += 1;
        }
    }
    sa.fill(0);

    // 对LMS前缀排序
    //place_lms_char(pat, sa, &mut sbucket, &prefix_sum, &lms_pos, &lms_cnt);
    for i in (0..lms_cnt - 1).rev() {  // 把LMS字符放入对应的桶中, 插入顺序不重要，这里保持顺序插入
        let lms_i = sbucket[pat[lms_pos[i]]];
        sa[lms_i] = lms_pos[i];
        sbucket[pat[lms_pos[i]]] -= 1;
    }
    sa[0] = pat.len() - 1;
    reset_sbucket(&mut sbucket, &prefix_sum);
    induced_sort(pat, sa, &suf_t, &mut lbucket, &mut sbucket, &prefix_sum);  // sort LMS prefix

    // 从排好序的LMS前缀中按顺序提出LMS子串，并按照相对排名构建新的缩减的问题：pat1的后缀数组求解
    // rename sortd LMS sub string, place into lmsname as pat order
    let mut rank = 1;
    let mut last_lms_i = 0;
    let mut has_duplicated_char = false;
    for i in 1..patlen {
        let e_i = sa[i];
        if is_lms_char(&suf_t, e_i) {  // LMS index >= 1
            if last_lms_i >= 1 && !lmssubstr_eq(pat, e_i, last_lms_i, &suf_t) { rank += 1 }
            if last_lms_i >= 1 && rank == lmsname[last_lms_i] { has_duplicated_char = true }
            last_lms_i = e_i;
            lmsname[e_i] = rank;
        }
    }

    let mut pat1 = vec![0; lms_cnt];
    let mut sa1 = vec![0; lms_cnt];
    let mut j = 0;
    for i in 0..patlen - 1 {
        if lmsname[i] > 0 {
            pat1[j] = lmsname[i];
            j += 1;
        }
    }
    pat1[j] = 0;
    sa.fill(0);

    if has_duplicated_char {
        _suffix_array_sais(&pat1[..], &mut sa1[..], rank + 1);
    } else {
        for i in 0..lms_cnt { sa1[pat1[i]] = i }
    }

    // place LMS suffix
    // 必须按照逆序插入
    for i in (1..lms_cnt).rev() {
        let rfp = sbucket[pat[lms_pos[sa1[i]]]];  // rfp:  rightmost free pointer
        sa[rfp] = lms_pos[sa1[i]];
        sbucket[pat[lms_pos[sa1[i]]]] -= 1;
    }
    sa[0] = pat.len() - 1;
    reset_sbucket(&mut sbucket, &prefix_sum);
    induced_sort(pat, sa, &suf_t, &mut lbucket, &mut sbucket, &prefix_sum);
}
// 最常见的求解字节串的后缀数组
pub fn suffix_array_sais(pat: &[u8]) -> Vec<usize> {
    let mut pat = pat.into_iter().map(|x| *x as usize).collect::<Vec<usize>>();
    pat.push(0);

    let mut sa = vec![0; pat.len()];
    _suffix_array_sais(&pat[..], &mut sa[..], 256);
    sa[1..].to_vec()
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

            assert_eq!(suffix_array_sais(pat.as_bytes()), res.clone());
        }
    }

    #[test]
    fn ensure_compute_sa_correctly_randomdata() {
        for (pat, res) in gen_sa_test_case() {
            assert_eq!(suffix_array_sais(pat.as_bytes()), res.clone());
        }
    }
}