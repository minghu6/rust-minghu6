use std::{fmt::Write, ops::AddAssign};

use itertools::Itertools;

////////////////////////////////////////////////////////////////////////////////
//// Macros

#[macro_export]
macro_rules! ht {
    ( $head_expr:expr, $tail_expr:expr ) => {{
        let head = $head_expr;
        let tail = $tail_expr;

        let mut _vec = vec![head];
        _vec.extend(tail.iter().cloned());
        _vec
    }};
    ( $head:expr) => {{
        ht!($head, vec![])
    }};
}


macro_rules! def_coll_macro {
    (seq | $name:ident, $new:expr, $push:ident) => {
        #[macro_export]
        macro_rules! $name {
            ( $$($value:expr),* ) => {{
                #[allow(unused_mut)]
                let mut _coll = $new;

                $$(
                    _coll.$push($value);
                )*

                _coll
            }};
        }
        #[allow(unused)]
        pub(crate) use $name;
    };
    (map | $name:ident, $new:expr) => {
        #[macro_export]
        macro_rules! $name {
            ( $$($k:expr => $v:expr),* $$(,)? ) => {{
                let mut _coll = $new;

                $$(
                    _coll.insert($k, $v);
                )*

                _coll
            }};
        }
        #[allow(unused)]
        pub(crate) use $name;
    };
}

def_coll_macro!(seq | vecdeq, std::collections::VecDeque::new(), push_back);
def_coll_macro!(seq | hashset, std::collections::HashSet::new(), insert);
def_coll_macro!(map | hashmap, std::collections::HashMap::new());
def_coll_macro!(map | btreemap, std::collections::BTreeMap::new());


#[macro_export]
macro_rules! parse_range {
    ($range:expr, $len:expr) => {{
        use std::ops::Bound::*;

        let range = $range;
        let len = $len;

        debug_assert!(len > 0, "range len should be greater than 0");

        let l;
        let mut r;

        match range.start_bound() {
            Included(v) => l = *v,
            Excluded(v) => l = *v + 1,
            Unbounded => l = 0,
        }

        match range.end_bound() {
            Included(v) => r = *v,
            Excluded(v) => {
                assert!(*v > 0, "range upper is invalid (=0)");
                r = *v - 1
            }
            Unbounded => r = len - 1,
        }

        if r > len - 1 {
            r = len - 1
        }

        (l, r)
    }};
}


#[macro_export]
macro_rules! ordered_insert {
    ($vec:expr, $x:expr) => {{
        let vec = $vec;
        let x = $x;

        match vec.binary_search(&x) {
            Ok(oldidx) => Some(std::mem::replace(&mut vec[oldidx], x)),
            Err(inseridx) => {
                vec.insert(inseridx, x);
                None
            }
        }
    }};
    ($vec:expr, $x:expr, $f:expr) => {{
        let vec = $vec;
        let x = $x;
        let f = $f;

        match vec.binary_search_by_key(&f(&x), f) {
            Ok(oldidx) => Some(std::mem::replace(&mut vec[oldidx], x)),
            Err(inseridx) => {
                vec.insert(inseridx, x);
                None
            }
        }
    }};
}


#[macro_export]
macro_rules! min {
    ($($val:expr),+) => {
        {
            [$($val),+].into_iter().min().unwrap()
        }
    }
}


#[macro_export]
macro_rules! max {
    ($($val:expr),+) => {
        {
            [$($val),+].into_iter().max().unwrap()
        }
    }
}


#[macro_export]
macro_rules! same {
    ($($val:expr),+ ,) => {
        same!($($val),+)
    };
    ($($val:expr),+) => {
        {
            let _arr = [$($val),+];
            _arr.iter().min().unwrap() == _arr.iter().max().unwrap()
        }
    }
}


/// Make simple (integer / &str) monomorphism struct
#[macro_export]
macro_rules! def_monomorphism_struct {
    ($name:ident, str, { $($field:ident),+ }) => {
        #[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Hash)]
        struct $name<'a> {
            $($field: &'a str),+
        }

        impl<'a> $name<'a> {
            fn new($($field: &'a str),+) -> Self {
                Self {
                    $($field),+
                }
            }
        }

        impl<'a> Copy for $name<'a> {}
    };
    ($name:ident, $ty:ty, { $($field:ident),+ }) => {
        #[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Hash)]
        struct $name {
            $($field: $ty),+
        }

        impl $name {
            fn new($($field: $ty),+) -> Self {
                Self {
                    $($field),+
                }
            }
        }

        impl Copy for $name {}
    };
}

////////////////////////////////////////////////////////////////////////////////
//// Functions

pub fn strshift<T: ToString>(it: T, pad: &str) -> String {
    let mut cache = String::new();

    write!(cache, "{}", it.to_string()).unwrap();

    let mut res = vec![];
    for line in cache.split('\n') {
        res.push(pad.to_string() + line);
    }

    res.join("\n")
}


pub fn normalize<T: Ord>(raw_data: &[T]) -> Vec<usize> {
    if raw_data.is_empty() {
        return vec![];
    }

    let mut res = vec![0; raw_data.len()];
    let mut taged: Vec<(usize, &T)> =
        raw_data.into_iter().enumerate().collect();

    taged.sort_by_key(|x| x.1);

    let mut rank = 1;
    let mut iter = taged.into_iter();
    let (i, mut prev) = iter.next().unwrap();
    res[i] = rank;

    for (i, v) in iter {
        if v != prev {
            rank += 1;
        }
        prev = v;
        res[i] = rank;
    }

    res
}


pub fn generate() -> impl FnMut() -> usize {
    let mut _inner = 0;
    move || {
        let old = _inner;
        _inner += 1;
        old
    }
}


pub fn gen_unique() -> impl FnMut() -> usize {
    let mut set = std::collections::HashSet::new();

    move || {
        let mut v = crate::random();

        while set.contains(&v) {
            v = crate::random();
        }

        set.insert(v);

        v
    }
}


/// Even up two vector using clone
///
/// (This implementation is inspired by Vec::remove)
///
/// ```
/// use m6_common::vec_even_up;
///
/// /* case-0 left max for pop is easy (odd) */
///
/// let mut v0 = (0..2).collect();
/// let mut v1 = (4..7).collect();
/// vec_even_up(&mut v0, &mut v1);
/// assert_eq!(v0, vec![0, 1, 4]);
/// assert_eq!(v1, vec![5, 6]);
///
/// /* case-1 the left is samller (even) */
///
/// let mut v0 = (0..2).collect();
/// let mut v1 = (4..8).collect();
/// vec_even_up(&mut v0, &mut v1);
/// assert_eq!(v0, vec![0, 1, 4]);
/// assert_eq!(v1, vec![5, 6, 7]);
///
/// /* case-2 the left is larger (even) */
///
/// let mut v0 = (0..4).collect();
/// let mut v1 = (4..6).collect();
/// vec_even_up(&mut v0, &mut v1);
/// assert_eq!(v0, vec![0, 1, 2]);
/// assert_eq!(v1, vec![3, 4, 5]);
///
///
/// /* case-3-0 the left is larger (given more than old_len) */
///
/// let mut v0 = (0..7).collect();
/// let mut v1 = (7..9).collect();
/// vec_even_up(&mut v0, &mut v1);
/// assert_eq!(v0, vec![0, 1, 2, 3, 4,]);
/// assert_eq!(v1, vec![5, 6, 7, 8]);
///
/// ```
///
///
pub fn vec_even_up<T>(v0: &mut Vec<T>, v1: &mut Vec<T>) {
    let v0_old_len = v0.len();
    let v1_old_len = v1.len();
    let tnt = v0_old_len + v1_old_len;

    let cnt_lf = tnt.div_ceil(2);
    let cnt_rh = tnt - cnt_lf;

    /* Check if it's balanced? */
    if v0_old_len == cnt_lf {
        debug_assert_eq!(v1.len(), cnt_rh);
        return;
    }

    debug_assert!(v0_old_len > 0 || v1_old_len > 0);

    use std::ptr::{copy, copy_nonoverlapping, read};

    // V0 is smaller
    if v0_old_len < cnt_lf {
        // let v0_get = cnt_lf - v0_old_len;
        let v1_given = v1_old_len - cnt_rh;

        let v1_ptr = v1.as_mut_ptr();

        let mut i = 0;

        v0.resize_with(cnt_lf, move || unsafe {
            let v = read(v1_ptr.add(i));
            i += 1;
            v
        });

        unsafe {
            copy(v1_ptr.add(v1_given), v1_ptr, cnt_rh);

            v1.set_len(cnt_rh);
        }
    }
    // V0 is larger
    else {
        let v0_given = v0_old_len - cnt_lf;

        let v0_ptr = v0.as_mut_ptr();

        v1.resize_with(cnt_rh, move || unsafe {
            // Fill with dirty data
            // read(v0_ptr)
            std::mem::zeroed()
        });

        // get ptr after resize to avoid realloc issue
        let v1_ptr = v1.as_mut_ptr();

        unsafe {
            copy(v1_ptr, v1_ptr.add(v0_given), v1_old_len);

            copy_nonoverlapping(v0_ptr.add(cnt_lf), v1_ptr, v0_given);

            v0.set_len(cnt_lf);
        }
    }
}


pub fn vec_even_up_1<T>(v0: &mut Vec<T>, v1: &mut Vec<T>) {
    let v0_old_len = v0.len();
    let v1_old_len = v1.len();

    if v0_old_len == v1_old_len {
        return;
    }

    if v0_old_len < v1_old_len {
        v0.push(v1.remove(0));
    } else {
        v1.insert(0, v0.pop().unwrap());
    }
}



////////////////////////////////////////////////////////////////////////////////
//// Functions - multitask split

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
            } else {
                *rem -= 1;

                x + 1
            })
        })
        .collect_vec()
}

fn split_<T: Clone>(
    plan: impl Fn(usize, usize) -> Vec<usize>,
    s: &[T],
    n: usize,
) -> Vec<Vec<T>> {
    let split_plan = plan(s.len(), n);

    let mut coll = vec![];
    for (start, end) in ht![0, sum_scan(&split_plan[..])]
        .into_iter()
        .tuple_windows()
    {
        coll.push(s[start..end].iter().cloned().collect_vec())
    }

    coll
}

pub fn split_simple<T: Clone>(s: &[T], n: usize) -> Vec<Vec<T>> {
    split_(task_split_simple, s, n)
}

pub fn split_improved<T: Clone>(s: &[T], n: usize) -> Vec<Vec<T>> {
    split_(task_split_improved, s, n)
}

////////////////////////////////////////////////////////////////////////////////
//// Functions - itertools

///
/// ```
/// use m6_algs::math::sum_scan;
///
/// assert_eq!(sum_scan(&[0, 1, 2, 3]), vec![0, 1, 3, 6])
/// ```
///
pub fn sum_scan<T: AddAssign + Default + Clone>(input: &[T]) -> Vec<T> {
    input
        .iter()
        .scan(T::default(), |acc, x| {
            *acc += (*x).clone();

            Some(acc.clone())
        })
        .collect()
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gen_unique() {
        let mut unique = gen_unique();

        for _ in 0..1000 {
            unique();
        }
    }

    #[test]
    fn test_task_split() {
        assert_eq!(sum_scan(&[1, 2, 3]), [1, 3, 6]);

        assert_eq!(task_split_simple(5, 2), [2, 3]);
        assert_eq!(task_split_improved(5, 2), [3, 2]);

        println!("{:?}", split_simple(&['a', 'b', 'c', 'd', 'e'], 3));
        println!("{:?}", split_improved(&['a', 'b', 'c', 'd', 'e'], 3));
    }

    #[test]
    fn test_make_monomorphism_struct() {
        def_monomorphism_struct!(AStruct, u64, { a, b, c });

        let a = AStruct::new(1, 2, 3);

        assert_eq!(a, AStruct::new(1, 2, 3));

        def_monomorphism_struct!(BStruct, str, { d, e });

        let b = BStruct::new("abc", "def");

        assert_eq!(b, BStruct::new("abc", "def"));
    }
}
