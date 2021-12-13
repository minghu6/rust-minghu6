use std::ops::AddAssign;

use itertools::Itertools;

use crate::ht;


/// devide, devide, ... devide + rem
pub fn task_split_simple(total: usize, n: usize) -> Vec<usize> {
    assert!(total >= n && n > 0);

    let (partial_size, rem) = (total / n, total % n);

    let mut res = [partial_size].repeat(n);
    *res.last_mut().unwrap() += rem;

    res
}

/// devide + 1opt, devide + 1opt, ... devide
pub fn task_split_improved(total: usize, n: usize) -> Vec<usize> {
    assert!(total >= n && n > 0);

    let (partial_size, rem) = (total / n, total % n);

    [partial_size]
    .repeat(n)
    .into_iter()
    .scan(rem, |rem, x| {
        Some(if *rem == 0 {
            x
        }
        else {
            *rem -= 1;

            x + 1
        })
    })
    .collect_vec()
}

pub fn _split<T: Clone>(plan: impl Fn(usize, usize) -> Vec<usize>, s: &[T], n: usize) -> Vec<Vec<T>> {
    let split_plan = plan(s.len(), n);

    let mut coll = vec![];
    for (start, end) in ht![0, sum_scan(&split_plan[..])].into_iter().tuple_windows() {
        coll.push(s[start..end].iter().cloned().collect_vec())
    }

    coll
}

pub fn split_simple<T: Clone>(s: &[T], n: usize) -> Vec<Vec<T>> {
    _split(task_split_simple, s, n)
}

pub fn split_improved<T: Clone>(s: &[T], n: usize) -> Vec<Vec<T>> {
    _split(task_split_improved, s, n)
}

////////////////////////////////////////////////////////////////////////////////
//// Helper Function

pub fn sum_scan<T: AddAssign + Default + Clone>(input: &[T]) -> Vec<T> {
    input.iter().scan(T::default(), |acc, x| {
        *acc += (*x).clone();

        Some(acc.clone())
    })
    .collect()
}


#[cfg(test)]
mod test {
    use super::*;


    #[test]
    fn test_task_split() {
        assert_eq!(sum_scan(&[1, 2, 3]), [1, 3, 6]);

        assert_eq!(task_split_simple(5, 2), [2, 3]);
        assert_eq!(task_split_improved(5, 2), [3, 2]);

        println!("{:?}", split_simple(&['a', 'b', 'c', 'd', 'e'], 3));
        println!("{:?}", split_improved(&['a', 'b', 'c', 'd', 'e'], 3));
    }

}
