

// 狭义 计数排序
// O(n+w) 只适合对自然数的子集进行排序
// min, max
pub fn counting_sort(origin: &mut Vec<usize>, value_scope: (usize, usize)) -> &mut Vec<usize> {
    let (low, height) = value_scope;
    let mut cnt = vec![0; height - low + 1];

    for i in 0..origin.len() {
        cnt[origin[i] - low] += 1;
    }

    let mut j = 0;
    for i in 0..(height - low + 1) {
        while cnt[i] > 0 {
            origin[j] = low + i;
            cnt[i] -= 1;
            j += 1;
        }
    }

    origin
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::prelude::*;

    #[test]
    fn test_counting_sort() {
        let mut rng = thread_rng();

        for _round in 0..100 {
            for len in 1..100 {
                let mut origin:Vec<usize> = (0..len).collect();
                origin.shuffle(&mut rng);

                counting_sort(&mut origin, (0, len));

                assert!(origin.is_sorted());
            }
        }
    }
}
