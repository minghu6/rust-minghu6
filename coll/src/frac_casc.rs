use std::{cmp::Ordering::*, borrow::Borrow};


macro_rules! unpack_result {
    ($res:expr) => {
        match $res {
            Ok(x) => x,
            Err(x) => x
        }
    };
}


/// l[i], idx of li[i], idx of m[i+1]
type METype<T> = (T, Result<usize, usize>, usize);


/// accelerate multiple binary search with memory saving
pub struct FractionalCascading<'a, T> {
    m: Vec<Vec<METype<T>>>,
    l: &'a [&'a [T]]
}


impl<'a, T: Ord> FractionalCascading<'a, T> {
    pub fn new(l: &'a [&[T]]) -> Self
    where
        T: Clone
    {
        assert!(l.iter().all(|i| i.is_sorted()));
        assert!(l.len() > 0);

        debug_assert!(l.iter().all(|i| !i.is_empty() && i.is_sorted()));

        let m = Self::build_m(l);

        Self { m, l }
    }

    pub fn find<Q: Borrow<T>>(&self, k: &Q) -> Vec<Result<usize, usize>> {
        let mut res = vec![Err(0); self.m.len()];

        let mut j;

        // 1. assign res[0]
        //
        // 2. m_idx
        //
        match self.m[0].binary_search_by_key(
            &k.borrow(),
            |x| &x.0
        )
        {
            Ok(idx) => {
                res[0] = self.m[0][idx].1;
                j = self.m[0][idx].2;
            },
            Err(idx) => {
                if idx == self.m[0].len() {
                    res[0] = Err(self.l[0].len());
                    j = self.m[0][idx - 1].2;
                }
                else {
                    res[0] = self.m[0][idx].1.and_then(|x| Err(x));
                    j = self.m[0][idx].2;
                }
            },
        }

        for i in 1..self.m.len() {
            let elected;

            if j == 0 {
                elected = j;
            }
            else if j < self.m[i].len() {
                let cand_a = j - 1;
                let cand_b = j;

                if k.borrow() <= &self.m[i][cand_a].0 {
                    elected = cand_a;
                }
                else {
                    elected = cand_b;
                }
            }
            else {
                elected = j - 1;
            }

            let dup = &self.m[i][elected].0;

            match k.borrow().cmp(&self.m[i][elected].0) {
                Less => {
                    let mut idx = unpack_result!(self.m[i][elected].1);

                    // go back (the worst case down to O(nk))
                    while idx > 0 && &self.l[i][idx - 1] == dup {
                        idx -= 1;
                    }

                    res[i] = Err(idx);
                },
                Equal => {
                    res[i] = self.m[i][elected].1;
                },
                Greater => {
                    let mut idx = unpack_result!(self.m[i][elected].1);

                    // go forward (the worst case down to O(nk))
                    while idx < self.l[i].len() - 1 && &self.l[i][idx + 1] == dup {
                        idx += 1;
                    }

                    if idx == self.l[i].len() - 1 && dup == &self.l[i][idx] {
                        idx += 1;
                    }

                    res[i] = Err(idx);
                },
            }

            j = self.m[i][elected].2;
        }

        res
    }

    /// Ignore duplicated case
    pub fn quick_find<Q: Borrow<T>>(&self, k: &Q) -> Vec<Result<usize, usize>> {
        let mut res = vec![Err(0); self.m.len()];

        let mut j;

        // 1. assign res[0]
        //
        // 2. m_idx
        //
        match self.m[0].binary_search_by_key(
            &k.borrow(),
            |x| &x.0
        )
        {
            Ok(idx) => {
                res[0] = self.m[0][idx].1;
                j = self.m[0][idx].2;
            },
            Err(idx) => {
                if idx == self.m[0].len() {
                    res[0] = Err(self.l[0].len());
                    j = self.m[0][idx - 1].2;
                }
                else {
                    res[0] = self.m[0][idx].1.and_then(|x| Err(x));
                    j = self.m[0][idx].2;
                }
            },
        }

        for i in 1..self.m.len() {
            let elected;
            let m1 = &self.m[i];

            if j == 0 {
                elected = j;
            }
            else if j < m1.len() {
                let cand_a = j - 1;
                let cand_b = j;

                if k.borrow() <= &m1[cand_a].0 {
                    elected = cand_a;
                }
                else {
                    elected = cand_b;
                }
            }
            else {
                elected = j - 1;
            }

            match k.borrow().cmp(&m1[elected].0) {
                Less => {
                    res[i] = m1[elected].1.and_then(|x| Err(x));
                },
                Equal => {
                    res[i] = m1[elected].1;
                },
                Greater => {
                    res[i] = m1[elected].1.and_then(|x| Err(x + 1));
                },
            }

            j = m1[elected].2;
        }

        res
    }

    fn build_m(l: &[&[T]]) -> Vec<Vec<METype<T>>>
    where
        T: Clone
    {
        let k = l.len();
        let mut m = vec![vec![]; k];


        /* Init m_k*/

        let l_k = l[k-1];
        let m_k = &mut m[k - 1];

        for (i, v) in l_k.iter().cloned().enumerate() {
            m_k.push((v, Ok(i), 0));
        }

        /* on guard */

        if k == 1 {
            return m;
        }

        for m_i in (0..=k-2).rev() {
            let l1 = l[m_i];
            let m2 = &m[m_i+1];
            let mut m1 = Vec::with_capacity(l1.len() + m2.len());

            /* merge two sorted vec */

            let mut i = 0;
            let mut j = 1;

            macro_rules! nxt_j {
                ($j:ident) => {
                    if $j + 2 == m2.len() {
                        $j + 1
                    }
                    else {
                        $j + 2
                    }
                };
            }

            while i < l1.len() && j < m2.len() {
                match l1[i].cmp(&m2[j].0) {
                    Less => {
                        m1.push((
                            l1[i].clone(),
                            Ok(i),
                            j
                        ));

                        i += 1;
                    },
                    Equal => {
                        m1.push((
                            l1[i].clone(),
                            Ok(i),
                            j
                        ));

                        /* skip dup */

                        let dup = &l1[i];

                        while i < l1.len() && &l1[i] == dup {
                            i += 1;
                        }

                        while j < m2.len() && &m2[j].0 == dup {
                            j = nxt_j!(j);
                        }

                    },
                    Greater => {
                        m1.push((
                            m2[j].0.clone(),
                            Err(i),
                            j
                        ));

                        j = nxt_j!(j);
                    },
                }
            }

            while i < l1.len() {
                m1.push((
                    l1[i].clone(),
                    Ok(i),
                    m2.len()
                ));

                i += 1;
            }

            while j < m2.len() {
                m1.push((
                    m2[j].0.clone(),
                    Err(l1.len()),
                    j,
                ));

                j = nxt_j!(j);
            }

            m[m_i] = m1;
        }

        m
    }
}



#[cfg(test)]
mod tests {

    use std::ops::Range;

    use common::random_range;

    use super::*;

    const V_RANGE: Range<usize> = 0..10_0;

    macro_rules! gen_ordered_arr {
        ($cap:expr, $v_range:expr) => {
            {
                let cap = $cap;
                let v_range = $v_range;

                let mut arr = Vec::with_capacity(cap);

                for _ in 0..cap {
                    arr.push(random_range!(v_range.clone()));
                }

                arr.sort();

                arr
            }
        };
    }

    macro_rules! gen_karr {
        // same length
        (equal-unique| k=$k:expr, cap=$cap:expr) => {{
            let mut arr = vec![];
            let k = $k;
            let cap = $cap;

            for _ in 0..k {
                let mut sub = gen_ordered_arr!(cap, V_RANGE);
                sub.dedup();

                arr.push(sub);
            }

            arr
        }};
        (equal| k=$k:expr, cap=$cap:expr) => {{
            let mut arr = vec![];
            let k = $k;
            let cap = $cap;

            for _ in 0..k {
                arr.push(gen_ordered_arr!(cap, V_RANGE));
            }

            arr
        }};
        (bisect_down| cap=$cap:expr) => {{
            let mut arr = vec![];
            let mut cap = $cap;

            while cap > 0 {
                arr.push(gen_ordered_arr!(cap, V_RANGE));

                cap /= 2;
            }

            arr
        }};
    }

    macro_rules! sample_as_ref {
        ($sample:expr) => {
            {
                let mut sample2 = vec![];

                for sub in $sample.iter() {
                    sample2.push(&sub[..]);
                }

                sample2
            }
        };
    }

    macro_rules! assert_k_binary_search_eq {
        (res=$res:expr, expect=$expect:expr, $sample:expr) => {
            {
                let res = $res;
                let expect = $expect;
                let sample = &$sample;

                for (i, (u, v)) in res.iter().zip(expect.iter()).enumerate() {
                    if let Ok(u_idx) = *u && let Ok(v_idx) = *v {
                        if u_idx == v_idx || sample[i][u_idx] == sample[i][v_idx] {
                            // pass ok
                        }
                        else {
                            panic!("\n/{u:?} | {v:?}/\n{expect:?} / {res:?}");
                        }
                    }
                    else {
                        assert_eq!(u, v, "\n\n   res: {res:?}\nexpect: {expect:?}\n\n");
                    }
                }
            }
        };
    }


    #[test]
    fn test_fractional_cascading_dup_rand() {
        for _ in 0..100 {
            let sample = gen_karr!(equal|k=100, cap=100);
            // println!("sample: {sample:#?}");

            let sample_ref = sample_as_ref!(sample);

            let fc = FractionalCascading::new(&sample_ref);

            for _ in 0..100 {
                let q = random_range!(V_RANGE);
                // println!("q: {q}");

                let mut expect = vec![];

                for sub in sample.iter() {
                    expect.push(sub.binary_search(&q));
                }

                let res = fc.find(&q);

                // assert_eq!(res, expect);
                assert_k_binary_search_eq!(res=res, expect=expect, sample_ref);
            }
        }

    }

    #[test]
    fn test_fractional_cascading_unique_rand() {
        for _ in 0..100 {
            let sample = gen_karr!(equal-unique|k=100, cap=100);
            // println!("sample: {sample:#?}");

            let sample_ref = sample_as_ref!(sample);

            let fc = FractionalCascading::new(&sample_ref);

            for _ in 0..100 {
                let q = random_range!(V_RANGE);
                // println!("q: {q}");

                let mut expect = vec![];

                for sub in sample.iter() {
                    expect.push(sub.binary_search(&q));
                }

                let res = fc.quick_find(&q);

                assert_k_binary_search_eq!(res=res, expect=expect, sample_ref);
            }
        }

    }

    #[test]
    fn test_fractional_cascading_case1() {
        let sample = vec![
            vec![13, 38, 60],
            vec![0, 74, 76],
            vec![34, 40, 57],
        ];

        let sample_ref = sample_as_ref!(sample);

        let fc = FractionalCascading::new(&sample_ref);

        // println!("m: {:?}", fc.m);

        // let res = fc.find_with_dup(&57);
        let res = fc.quick_find(&58);

        println!("res: {res:?}");
    }

    #[test]
    fn check_fractional_cascading_time_complex() {
        let raw = |k: usize, n: usize| {
            k * n.ilog2() as usize
        };

        let fc = |k: usize, n: usize| {
            (n * k).ilog2() as usize + k
        };

        let mut raw_stats = vec![];
        let mut fc_stats = vec![];

        for k in 1..=1000 {
            for n in 1..=1000 {
                let raw_v = raw(k, n);
                let fc_v = fc(k, n);

                if raw_v > fc_v {
                    raw_stats.push((raw_v, fc_v, k, n));
                }
                else if raw_v < fc_v {
                    fc_stats.push((raw_v, fc_v, k, n));
                }
            }
        }

        raw_stats.sort_by_cached_key(|x| x.0 - x.1);
        fc_stats.sort_by_cached_key(|x| x.1 - x.0);

        println!("raw_stats gt top10: {:#?}", &raw_stats[raw_stats.len() - 10..]);
        println!("fc_stats gt top10: {:#?}", &fc_stats[fc_stats.len() - 10..]);


    }

}

