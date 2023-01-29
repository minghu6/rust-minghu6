// ref to: Li, Zhize; Li, Jian; Huo, Hongwei (2016). *Optimal In-Place Suffix Sorting*. Proceedings of the 25th International Symposium on String Processing and Information Retrieval (SPIRE). Lecture Notes in Computer Science. 11147. Springer. pp. 268–284. arXiv:1610.08305. doi:10.1007/978-3-030-00479-8_22. ISBN:978-3-030-00478-1.
// In-place, O(n)
#![allow(dead_code)]

use std::cmp::max;
use std::cmp::Ordering;
use std::slice::from_raw_parts_mut;


const LTYPE: bool = false;
const STYPE: bool = true;
const MAX_SA_VALUE: usize = usize::MAX / 2;
const EMPTY: usize = MAX_SA_VALUE + 1;
const UNIQUE: usize = MAX_SA_VALUE + 2;
const MULTI: usize = MAX_SA_VALUE + 3;  // >= 258

#[inline]
fn lms_str_cmp<E: Ord>(l1: &[E], l2: &[E]) -> Ordering {
    for (x, y) in l1.iter().zip(l2.iter()) {
        let cmp_res = x.cmp(&y);

        if cmp_res != Ordering::Equal { return cmp_res; }
    }

    Ordering::Equal
}

#[inline]
fn pat_char_type(cur: usize, prev: usize, last_scanned_type: bool) -> bool {
    if cur < prev || cur == prev && last_scanned_type == STYPE { STYPE }
    else { LTYPE }
}

// PASS! -- from induced sort
fn rename_pat(pat: &mut [usize], sa: &mut [usize]) {
    let patlastpos = pat.len() - 1;
    // 全部刷成bucket head
    sa.fill(0);

    for i in 0..pat.len() { sa[pat[i]] += 1 }
    for i in 1..sa.len() { sa[i] += sa[i - 1] }

    for i in 0..pat.len() - 1 {
        pat[i] = sa[pat[i]] - 1;
    };
    // 将L-suffix刷成bucket head
    sa.fill(0);

    for i in 0..pat.len() { sa[pat[i]] += 1 }
    let mut last_scanned_type = STYPE;
    pat[patlastpos] = 0;
    for i in (0..pat.len() - 1).rev() {
        if pat_char_type(pat[i], pat[i + 1], last_scanned_type) == STYPE {
            last_scanned_type = STYPE;
        } else {
            pat[i] -= sa[pat[i]] - 1;
            last_scanned_type = LTYPE;
        }
    }

}

// PASS!
fn sort_lms_char(pat: &mut [usize], sa: &mut [usize]) -> usize {
    sa.fill(EMPTY);

    let mut last_scanned_type = STYPE;
    for i in (0..pat.len() - 1).rev() {
        if pat_char_type(pat[i], pat[i + 1], last_scanned_type) == STYPE {
            last_scanned_type = STYPE;
        } else {
            if last_scanned_type == STYPE {  // pat[i + 1] is LMS type
                sa[pat[i + 1]] += 1;
            }

            last_scanned_type = LTYPE;
        }
    }

    let mut lms_cnt = 0;
    last_scanned_type = STYPE;
    for i in (0..pat.len() - 1).rev() {
        if pat_char_type(pat[i], pat[i + 1], last_scanned_type) == STYPE {
            last_scanned_type = STYPE;
        } else {
            let e_i = i + 1;
            let e = pat[e_i];

            if last_scanned_type == STYPE {  // pat[i + 1] is LMS type
                lms_cnt += 1;
                if sa[e] == UNIQUE {
                    sa[e] = e_i;
                } else if sa[e] >= MULTI && sa[e - 1] == EMPTY {
                    if sa[e - 2] == EMPTY {
                        sa[e - 2] = e_i;
                        sa[e - 1] = 1;  // set counter
                    } else {  // implies that MUL = 2 and no need for counter
                        sa[e] = e_i;
                        sa[e - 1] = EMPTY;
                    }
                } else if sa[e] >= MULTI && sa[e - 1] != EMPTY {
                    let c = sa[e - 1];  // get counter

                    if sa[e - 2 - c] == EMPTY {
                        sa[e - 2 - c] = e_i;
                        sa[e - 1] += 1;  // update counter
                    } else {
                        for j in (1..c + 1).rev() {
                            sa[e - c + j] = sa[e - 2 - c + j]
                        }
                        sa[e - c] = e_i;
                        sa[e - c - 1] = EMPTY;
                    }
                } else if sa[e] < EMPTY {
                    for j in (0..e).rev() {
                        if sa[j] == EMPTY {
                            sa[j] = e_i;
                            break;
                        }
                    }
                }
            }

            last_scanned_type = LTYPE;
        }
    }

    for i in (0..pat.len()).rev() {
        if sa[i] >= MULTI {
            let c = sa[i - 1];
            for j in (1..c + 1).rev() {  // 逆序防止前面的覆盖后面的
                sa[i - c + j] = sa[i - 2 - c + j];
            }
            sa[i - c - 1] = EMPTY;
            sa[i - c] = EMPTY;
        }
    }

    lms_cnt
}

// PASS!
fn sort_lms_substr(pat: &mut [usize], sa: &mut [usize]) {
    // step 1
    induced_sort(pat, sa);

    // step 2
    let pat_last_pos = pat.len() - 1;
    let mut lms_cnt = 0;
    let mut i = pat_last_pos;
    let mut bucket_tail_ptr = pat_last_pos + 1;  // for renamed bucket ver
    let mut bucket = EMPTY;  // 可以省略，但是为了书写代码方便
    let mut num = 0;  // S type number of bucket
    while i > 0 {
        if pat[sa[i]] != bucket {  // reach new bucket
            num = 0;

            let mut l = 0;
            while pat[sa[i - l]] == pat[sa[i]] {  // 扫描桶来计算桶中S字符数量，根据定义 当l=i时循环必然终止
                let pat_i = sa[i - l];             // l < i, 即 i - l > 0, 0 <= pat_i < patlen - 1
                if pat[pat_i] < pat[pat_i + 1] {
                    let mut k = pat_i;
                    while k > 0 && pat[k - 1] == pat[pat_i] { k -= 1 }
                    num += pat_i - k + 1;
                } else {
                    break;   // bucket不含S字符，结束扫描
                }

                l += 1;
            }

            bucket_tail_ptr = i;
            bucket = pat[sa[bucket_tail_ptr]];
        }

        if num > 0
        && i > bucket_tail_ptr - num
        && sa[i] > 0
        && pat[sa[i]] < pat[sa[i] - 1]  {
            sa[pat_last_pos - lms_cnt] = sa[i];
            lms_cnt += 1;
        }

        i -= 1;
    }

    sa[pat_last_pos - lms_cnt ] = sa[i];  // i = 0
    lms_cnt += 1;
    sa[0..pat_last_pos - lms_cnt + 1].fill(EMPTY);
}

// PASS!
fn construct_pat1(pat: &mut [usize], sa: &mut [usize], lms_cnt: usize) -> bool {
    let patlen = pat.len();

    let mut prev_lms_str_len = 1;
    let mut rank = 0;
    sa[(patlen - 1) / 2] = rank;
    let mut has_duplicated_char = false;
    for i in patlen - lms_cnt + 1..patlen {  // 从警戒哨字符的下一个字符开始
        let mut j = sa[i];
        while pat[j] <= pat[j + 1] { j += 1 } // 寻找suf(sa[i])右边第一个L字符，因为排除了警戒哨这个LMS后缀，所以必然不会越界
        let mut k = j;
        while k + 1 < patlen && pat[k] >= pat[k + 1] { k += 1 }  // 找到suf(sa[i])右边第一个LMS字符
        let cur_lms_str_len = k + 1 - sa[i];
        let cmp_res = lms_str_cmp(&pat[sa[i]..sa[i] + cur_lms_str_len], &pat[sa[i - 1]..sa[i - 1] + prev_lms_str_len]);

        if  cmp_res != Ordering::Equal {
            rank += 1
        }

        if rank == sa[sa[i - 1] / 2] {
            has_duplicated_char = true;
        }
        let rank_index = sa[i] / 2;
        sa[rank_index] = rank;  // 整除

        prev_lms_str_len = cur_lms_str_len;
    }

    // move to head of sa
    let mut j = 0;
    for i in 0..patlen - lms_cnt {
        if sa[i] != EMPTY {
            sa[j] = sa[i];
            if i > j {
                sa[i] = EMPTY;
            }
            j += 1;
        }
    }

    for i in lms_cnt..patlen { sa[i] = EMPTY }

    has_duplicated_char
}

fn sort_lms_suf(pat: &mut [usize], sa: &mut [usize], lms_cnt: usize, has_duplicated_char: bool) {
    // solve T1 recursively
    let patlen = pat.len();
    let salen = sa.len();
    unsafe {
        let sa_ptr = sa.as_mut_ptr();
        let mut pat1 = from_raw_parts_mut(sa_ptr, lms_cnt);
        let mut sa1 = from_raw_parts_mut(sa_ptr.offset((patlen - lms_cnt) as isize), salen - (patlen - lms_cnt));

        if has_duplicated_char {
            _compute_suffix_array_16_1(&mut pat1, &mut sa1);
        } else {
            for i in 0..lms_cnt { sa1[pat1[i]] = i }
        }

    }

    // move SA1 to SA[0...n1-1]
    for i in 0..lms_cnt {
        sa[i] = sa[patlen- lms_cnt + i];
    }

    // put all LMS-suffixes in SA tail
    let mut last_scanned_type = STYPE;
    let mut j = 0;
    for i in (0..pat.len() - 1).rev() {
        if pat[i] < pat[i + 1] || pat[i] == pat[i + 1] && last_scanned_type == STYPE {
            last_scanned_type = STYPE;
        } else {
            if last_scanned_type == STYPE {
                sa[patlen - 1 - j] = i + 1;
                j += 1;
            }

            last_scanned_type = LTYPE;
        }
    }

    // backward map the LMS-suffixes rank
    for i in 0..lms_cnt {
        let relative_rank = sa[i];
        sa[i] = sa[patlen - lms_cnt + relative_rank];
        sa[patlen - lms_cnt + relative_rank] = EMPTY;
    }

    let mut tail = EMPTY;
    let mut rfp = EMPTY;
    for i in (1..lms_cnt).rev() { // sa[0] 保持原位
        if pat[sa[i]] != tail {
            tail = pat[sa[i]];
            rfp = tail;
        }

        sa[rfp] = sa[i];
        if rfp != i { sa[i] = EMPTY }
        rfp -= 1;
    }
}

// PASS!
fn induced_sort(pat: &mut [usize], sa: &mut [usize]) {
    let patlen = pat.len();

    // let mut suf_t = vec![STYPE; patlen];
    // let mut last_scanned_type = STYPE;
    // for i in (0..pat.len() - 1).rev() {
    //     if pat_char_type(pat[i], pat[i + 1], last_scanned_type) == STYPE {
    //         suf_t[i] = STYPE;
    //         last_scanned_type = STYPE;
    //     } else {
    //         suf_t[i] = LTYPE;
    //         last_scanned_type = LTYPE;
    //     }
    // }

    // place L-suff in SA
    // init
    let mut last_scanned_type = STYPE;
    for i in (0..patlen - 1).rev() {
        if pat_char_type(pat[i], pat[i + 1], last_scanned_type) == LTYPE {
            sa[pat[i]] += 1;  // >= EMPTY
            last_scanned_type = LTYPE;
        } else {
            last_scanned_type = STYPE;
        }
    }
    //place
    let mut i = 0;
    while i < patlen {
        if sa[i] < EMPTY && sa[i] > 0 {
            let j = sa[i] - 1;
            let mut is_ltype = false;
            // if suf_t[j] == LTYPE {
            //     is_ltype = true;
            // }
            if pat[j] > pat[j + 1] {
                is_ltype = true;
            } else if pat[j] == pat[j + 1] {  // 判断sa[i]是否是L后缀的编号
                let next_i = sa[pat[sa[i]]];
                if next_i >= MULTI {
                    is_ltype = true;
                } else if next_i < EMPTY && pat[sa[i]] + 1 < patlen {
                    if sa[pat[sa[i]] + 1] == EMPTY {
                        is_ltype = true;
                    } else if sa[pat[sa[i]] + 1] < EMPTY {
                        if pat[sa[pat[sa[i]] + 1]] == pat[sa[i]] {
                            is_ltype = true;
                        }
                    }
                }
            }

            if is_ltype {
                if sa[pat[j]] == UNIQUE {
                    sa[pat[j]] = j;
                } else if sa[pat[j]] >= MULTI && sa[pat[j] + 1] == EMPTY {
                    if sa[pat[j]] - EMPTY > 2 {
                        sa[pat[j] + 2] = j;
                        sa[pat[j] + 1] = 1;  // set counter
                    } else {
                        sa[pat[j]] = j;
                    }
                } else if sa[pat[j]] >= MULTI && sa[pat[j] + 1] != EMPTY {
                    let e = pat[j];
                    let c = sa[e + 1];
                    let lfp = e + c + 2;
                    if  c + 2 < sa[pat[j]] - EMPTY {  // 没到bucket尾部
                        sa[lfp] = j;
                        sa[e + 1] += 1;  // update counter
                    } else {
                        for k in 1..c + 1 {
                            sa[e + k - 1] = sa[e + k + 1];
                        }
                        sa[e + c] = j;
                        sa[e + c + 1] = EMPTY;
                        if i >= e + 2 && i <= e + c + 1 {
                            i -= 2;
                        }
                    }
                } else if sa[pat[j]] < EMPTY {
                    for k in pat[j]..patlen {
                        if sa[k] == EMPTY {
                            sa[k] = j;
                            break;
                        }
                    }
                }
            }
        } else if sa[i] >= MULTI {
            i += 1;
        }

        i += 1;
    }

    //remove LMS-suff form SA, 一个桶里可能有多个LMS后缀
    last_scanned_type = STYPE;
    for i in (0..pat.len() - 1).rev() {
        if pat_char_type(pat[i], pat[i + 1], last_scanned_type) == STYPE {
            last_scanned_type = STYPE;
        } else {
            if last_scanned_type == STYPE {  // pat[i + 1] is LMS type
                if sa[pat[i + 1]] <= EMPTY {
                    sa[pat[i + 1]] = UNIQUE;
                } else {
                    sa[pat[i + 1]] += 1;
                }
            }

            last_scanned_type = LTYPE;
        }
    }
    i = patlen - 1;
    while i > 0 {
        if sa[i] > EMPTY {
            let c = sa[i] - EMPTY;
            for k in 0..c {
                sa[i - k] = EMPTY;
            }
            i -= c - 1;
        }

        i -= 1;
    }
    sa[0] = pat.len() - 1;

    // place S-suff in SA
    // init
    let mut last_scanned_type = STYPE;
    for i in (0..patlen - 1).rev() {
        if pat_char_type(pat[i], pat[i + 1], last_scanned_type) == STYPE {
            if sa[pat[i]] >= EMPTY {
                sa[pat[i]] += 1;
            } else {
                sa[pat[i]] = UNIQUE;
            }
            last_scanned_type = STYPE;
        } else {
            last_scanned_type = LTYPE;
        }
    }
    i = patlen - 1;
    while i > 0 {
        if sa[i] < EMPTY && sa[i] > 0 {
            let j = sa[i] - 1;
            let mut is_stype = false;
            // if suf_t[j] == STYPE {
            //     is_stype = true;
            // }
            if pat[j] < pat[j + 1] {
                is_stype = true;
            } else if pat[j] == pat[j + 1] {  // 判断sa[i]是否是S后缀的编号
                let next_i = sa[pat[sa[i]]];
                if next_i >= MULTI {
                    is_stype = true;
                } else if next_i < EMPTY && pat[sa[i]] - 1 > 0 {
                    if sa[pat[sa[i]] - 1] == EMPTY {
                        is_stype = true;
                    } else if sa[pat[sa[i]] - 1] < EMPTY {
                        if pat[sa[pat[sa[i]] - 1]] == pat[sa[i]] {
                            is_stype = true;
                        }
                    }
                }
            }

            if is_stype {
                if sa[pat[j]] == UNIQUE {
                    sa[pat[j]] = j;
                } else if sa[pat[j]] >= MULTI && sa[pat[j] - 1] == EMPTY {
                    if sa[pat[j]] - EMPTY > 2 {
                        sa[pat[j] - 2] = j;
                        sa[pat[j] - 1] = 1;  // set counter
                    } else {
                        sa[pat[j]] = j;
                    }
                } else if sa[pat[j]] >= MULTI && sa[pat[j] - 1] != EMPTY {
                    let e = pat[j];
                    let c = sa[e - 1];
                    let num = sa[pat[j]] - EMPTY;
                    if c + 2 < num {  // 没到bucket头部
                        let rfp = e - c - 2;
                        sa[rfp] = j;
                        sa[e - 1] += 1;
                    } else {
                        for k in 1..c + 1 {
                            sa[e - k + 1] = sa[e - k - 1];
                        }
                        sa[e - c] = j;
                        sa[e - c - 1] = EMPTY;
                        if i >= e - num + 1 && i <= e - 2 {
                            i += 2;
                        }
                    }
                } else if sa[pat[j]] < EMPTY {
                    for k in (0..pat[j]).rev() {
                        if sa[k] == EMPTY {
                            sa[k] = j;
                            break;
                        }
                    }
                }
            }
        } else if sa[i] >= MULTI {
            i -= 1;
        }
        i -= 1;
    }
}

fn _compute_suffix_array_16_1(pat: &mut [usize], sa: &mut [usize]) {
    rename_pat(pat, sa);
    let lms_cnt = sort_lms_char(pat, sa);
    sort_lms_substr(pat, sa);
    let has_duplicated_char = construct_pat1(pat, sa, lms_cnt);
    sort_lms_suf(pat, sa, lms_cnt, has_duplicated_char);
    induced_sort(pat, sa);
}

pub fn suffix_array_16(pat: &[u8]) -> Vec<usize> {
    let mut pat = pat.into_iter().map(|x| *x as usize).collect::<Vec<usize>>();
    pat.push(0);
    let mut sa = vec![0; max(pat.len(), 256) * 1];
    _compute_suffix_array_16_1(&mut pat[..], &mut sa[..]);

    sa[1..pat.len()].to_vec()
}


#[cfg(test)]
mod tests {
    use super::{ *, super::* };

    #[test]
    fn ensure_compute_sa16_correctly_fixeddata() {
        for (pat, res) in [("aabaaaab", vec![3, 4, 5, 0, 6, 1, 7, 2]),
                           ("abaab", vec![2, 3, 0, 4, 1]),
                           ("banana$", vec![6, 5, 3, 1, 0, 4, 2]),
                           ("ba", vec![1, 0]),
                           ("b", vec![0])].iter() {

            assert_eq!(suffix_array_16(pat.as_bytes()), res.clone());
        }
    }

    #[test]
    fn ensure_compute_sa16_correctly_randomdata() {
        for (pat, res) in gen_sa_test_case() {
            assert_eq!(suffix_array_16(pat.as_bytes()), res.clone());
        }
    }
}
