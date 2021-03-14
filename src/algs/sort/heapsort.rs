

fn sift_down<T: Ord + Copy>(arr: &mut [T], start: usize, end: usize) {
    let mut root = start;

    while 2 * root + 1 < end {
        let child = 2 * root + 1;
        let mut swap = root;

        if arr[swap] < arr[child] { swap = child }
        if child + 1 < end && arr[swap] < arr[child + 1] { swap = child + 1 }

        if swap == root { return }

        (arr[root], arr[swap]) = (arr[swap], arr[root]);
        root = swap;
    }
}

fn heapify<T: Ord + Copy>(arr: &mut [T]) {
    // start from last node's parent node
    let start = (arr.len() - 1 - 1) / 2;
    for i in (0..start + 1).rev() { sift_down(arr, i, arr.len()) }
}

pub fn heapsort<T: Ord + Copy>(arr: &mut [T]) {
    if arr.len() < 2 { return }
    heapify(arr);
    for i in (0..arr.len()).rev() {
        (arr[0], arr[i]) = (arr[i], arr[0]);
        sift_down(arr, 0, i);
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use super::super::super::super::test::sort::gen_test_case;

    #[test]
    fn test_heapsort_sort() {
        for pat in gen_test_case() {
            let mut res = pat.clone();
            heapsort(&mut res[..]);

            if !res.is_sorted() {
                eprintln!("pat: {:?}", pat);
                eprintln!("res: {:?}", res);
            }
            assert!(res.is_sorted());
        }
    }
}