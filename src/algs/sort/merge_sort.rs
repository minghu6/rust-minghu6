
/**
 * Stable Sort
 */
pub fn merge_sort<T: Ord + Clone + Copy>(pat: &mut [T]) {
    let patlen = pat.len();
    let mut temp = vec![pat[0]; patlen];
    _merge_sort(pat, &mut temp[..], 0, patlen);
}

pub fn _merge_sort<T: Ord + Clone + Copy>(pat: &mut [T], temp: &mut [T], start: usize, end: usize) {
    if start + 1 == end { return }
    let mid = start + (end - start) / 2;
    _merge_sort(pat, temp, start, mid);
    _merge_sort(pat, temp, mid, end);
    let mut lp = start;  // Left pointer
    let mut rp = mid;    // Right pointer

    for i in start..end {
        if lp >= mid || rp < end && pat[rp] < pat[lp] {
            temp[i] = pat[rp];
            rp += 1;
        } else {
            temp[i] = pat[lp];
            lp += 1;
        }
    }

    pat[start..end].clone_from_slice(&temp[start..end]);
}

#[cfg(test)]
mod tests {
    use super::{ *, super::* };


    #[test]
    fn test_merge_sort() {
        for pat in gen_test_case() {
            let mut res = pat.clone();
            merge_sort(&mut res[..]);

            if !res.is_sorted() {
                eprintln!("pat: {:?}", pat);
                eprintln!("res: {:?}", res);
            }
            assert!(res.is_sorted());
        }
    }
}
