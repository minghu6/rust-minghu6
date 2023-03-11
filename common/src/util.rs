use std::fmt::Write;


////////////////////////////////////////////////////////////////////////////////
//// Macro

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


macro_rules! def_coll_init {
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

def_coll_init!(seq | vecdeq, std::collections::VecDeque::new(), push_back);
def_coll_init!(seq | hashset, std::collections::HashSet::new(), insert);
def_coll_init!(map | hashmap, std::collections::HashMap::new());
def_coll_init!(map | btreemap, std::collections::BTreeMap::new());


////////////////////////////////////////////////////////////////////////////////
//// Function

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


pub fn gen() -> impl FnMut() -> usize {
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

    use std::ptr::{ copy, copy_nonoverlapping, read };

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
            copy(
                v1_ptr.add(v1_given),
                v1_ptr,
                cnt_rh
            );

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
            copy(
                v1_ptr,
                v1_ptr.add(v0_given),
                v1_old_len
            );

            copy_nonoverlapping(
                v0_ptr.add(cnt_lf),
                v1_ptr,
                v0_given
            );

            v0.set_len(cnt_lf);
        }
    }
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

    // #[test]
    // fn test_vec_even_up() {
    //     let mut v0 = (0..2).collect();
    //     let mut v1 = (4..7).collect();

    //     vec_even_up(&mut v0, &mut v1);

    //     assert_eq!(v0, vec![0, 1]);
    // }
}
