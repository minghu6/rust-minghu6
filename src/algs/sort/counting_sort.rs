

// 狭义 计数排序
// O(n+w) 只适合对值域为自然数的子集进行排序
// min, max
pub fn counting_sort(origin: &mut Vec<usize>, value_scope: (usize, usize)) -> &mut Vec<usize> {
    let (low, height) = value_scope;
    let mut cnt = vec![0; height - low + 1];

    for i in 0..origin.len() {
        cnt[origin[i] - low] += 1;
    }

    for i in 1..height - low + 1 {
        cnt[i] += cnt[i - 1];  // 排名从1开始
    }

    let old_origin = origin.clone();
    for i in (0..old_origin.len()).rev() {
        origin[cnt[old_origin[i]] - 1] = old_origin[i];
        cnt[old_origin[i]] -= 1;
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
                let old_origin = origin.clone();

                counting_sort(&mut origin, (0, len));

                if !origin.is_sorted() {
                    eprintln!("old_origin: {:?}", old_origin);
                    eprintln!("sorted origin: {:?}", origin);
                }

                assert!(origin.is_sorted());
            }
        }
    }
}
