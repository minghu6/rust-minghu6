use std::{
    cmp::min,
    collections::BTreeSet,
    iter::{self, once},
    thread,
};

use bit_vec::BitVec;
use lazy_static::lazy_static;

////////////////////////////////////////////////////////////////////////////////
//// Constant

static WN: usize = 7;

/// set amount of cpu cores used in parallel tasks
pub static USED_CPU_CORES_NUM: usize = 3;

lazy_static! {
    static ref P: Vec<usize> =
        once(0).chain(bengelloun_sieve_inf().take(20)).collect::<Vec<usize>>();

    static ref W: Vec<WStruct> = {
        debug_assert!(WN <= 10);  // memory overhead too high.

        let mut res = vec![
            WStruct { wheel: vec![], wheel_gap: vec![], prod: 1, ipm: vec![] },  // W0
            WStruct { wheel: vec![1], wheel_gap: vec![2], prod: 2, ipm: vec![] },  // W1
        ];

        // let mut bits = BitVec::from_elem(2+1, false);
        // bits.set(1, true);

        for k in 2..=WN {
            // let w0 = &res[k-1].wheel;
            let wg0 = &res[k-1].wheel_gap;
            let prod0 = &res[k-1].prod;

            // bits.grow((bits.len()-1) * (P[k] - 1), false);

            // // rolling
            // for r in 1..P[k] {
            //     let l = r * prod0;

            //     for i in w0.iter() {
            //         bits.set(l + *i, true);
            //     }
            // }

            // // remove multiple p
            // for i in w0.iter() {
            //     bits.set(i * P[k], false);
            // }

            // let w = bits.iter()
            //     .enumerate()
            //     .skip(1)
            //     .filter_map(|(i, flag)| if flag { Some(i) } else { None })
            //     .collect::<Vec<usize>>();

            // 3 times as fast as bit-vec impl
            let mut w = Vec::with_capacity(wg0.len() * (P[k] - 1));
            let mut acc = 1;

            for _ in 0..P[k] {
                for i in 0..wg0.len() {
                    if acc % P[k] != 0 {
                        w.push(acc);
                    }

                    acc += wg0[i];
                }
            }

            let prod = prod0 * P[k];

            // dbg!(&w, prod, &wg0);

            let wg = (1..w.len())
                .map(|i| w[i] - w[i - 1])
                .chain(once(prod + 1 - w[w.len() - 1]))
                .collect::<Vec<_>>();

            let ipm = BTreeSet::from_iter(
                    wg.iter().map(|x| *x / 2)
                )
                .into_iter()
                .collect::<Vec<_>>();

            res.push(WStruct { wheel: w, wheel_gap: wg, prod, ipm });
        }

        res
    };
}

////////////////////////////////////////////////////////////////////////////////
//// Structures

#[derive(Debug)]
struct WStruct {
    wheel: Vec<usize>,
    wheel_gap: Vec<usize>,
    prod: usize,
    /// difference kind of gap (ordered)
    #[allow(dead_code)]
    ipm: Vec<usize>,
}

////////////////////////////////////////////////////////////////////////////////
//// Functions

/// Eratosthenes Sieve
pub fn e_sieve(n: usize) -> Box<dyn Iterator<Item = usize>> {
    if n == 0 {
        return Box::new(iter::empty());
    }

    let mut bits = BitVec::from_elem(n + 1, true);

    for i in 2..=n.isqrt() {
        if bits[i] {
            for j in (i * i..=n).step_by(i) {
                bits.set(j, false);
            }
        }
    }

    Box::new(bits
        .into_iter()
        .enumerate()
        .skip(2)
        .filter_map(|(i, flag)| if flag { Some(i) } else { None }))
}

/// Space: O(\sqrt{n})
pub fn e_seg_sieve(n: usize) -> impl Iterator<Item = usize> {
    std::iter::from_coroutine(
        #[coroutine]
        move || {
            if n == 0 {
                return;
            }

            let delta = n.isqrt();
            let pris = e_sieve(delta).collect::<Vec<usize>>();

            for i in 0..pris.len() {
                yield pris[i];
            }

            let mut seg = BitVec::from_elem(delta + 1, true);

            // (l, l+delta]
            for l in (delta..=n).step_by(delta) {
                let actual_delta = min(delta, n - l);

                for &p in pris.iter() {
                    for i in (p - l % p..=actual_delta).step_by(p) {
                        seg.set(i, false);
                    }
                }

                for i in 1..=actual_delta {
                    if seg[i] {
                        yield l + i
                    }
                }

                seg.set_all();
            }
        },
    )
}

pub fn e_seg_sieve_p(n: usize) -> Box<dyn Iterator<Item = usize>> {
    if n == 0 {
        return Box::new(iter::empty());
    }

    let delta = n.isqrt();
    let pris = e_sieve(delta).collect::<Vec<usize>>();

    fn sieve_subtask<'a>(
        pris: &'a [usize],
        l0: usize,
        l1: usize,
        delta: usize,
    ) -> Vec<usize> {
        let mut ans = vec![];

        let mut seg = BitVec::from_elem(delta + 1, true);

        for l in (l0..=l1).step_by(delta) {
            let actual_delta = min(delta, l1 - l);

            for &p in pris.iter() {
                for i in (p - l % p..=actual_delta).step_by(p) {
                    seg.set(i, false);
                }
            }

            for i in 1..=actual_delta {
                if seg[i] {
                    ans.push(l + i);
                }
            }

            seg.set_all();
        }

        ans
    }

    let ans = thread::scope(|scope| {
        let pris_ref = &pris;

        let mut l = delta;

        let mut k_delta = (n / delta - 1) / USED_CPU_CORES_NUM;

        if l + k_delta * delta < n {
            k_delta += 1;
        }

        let mut handles = vec![];

        for _ in 0..USED_CPU_CORES_NUM {
            handles.push(scope.spawn(move || {
                sieve_subtask(pris_ref, l, min(l + k_delta * delta, n), delta)
            }));

            l += k_delta * delta;
        }

        handles
            .into_iter()
            .map(|handle| handle.join().unwrap())
            .flatten()
            .collect::<Vec<usize>>()
    });

    Box::new(pris.into_iter().chain(ans.into_iter()))
}


/// 0. p^2 >= l+delta
/// 1. pris from 0..x, no empty.
/// 2. return (l, l+delta]
fn e_seg_sieve_0<'a>(
    pris: &'a [usize],
    l: usize,
    delta: usize,
) -> impl Iterator<Item = usize> + 'a {
    debug_assert!(!pris.is_empty());

    std::iter::from_coroutine(
        #[coroutine]
        move || {
            let mut seg = BitVec::from_elem(delta + 1, true);

            for &p in pris.iter() {
                for i in (p - l % p..=delta).step_by(p) {
                    seg.set(i, false);
                }
            }

            for (i, flag) in seg.into_iter().enumerate().skip(1) {
                if flag {
                    yield l + i
                }
            }
        },
    )
}

/// 1. pris from 0..x, no empty.
/// 2. return (l, l+delta]
pub fn e_inc_sieve_reentrant(pris: &mut Vec<usize>, n: usize) {
    debug_assert!(!pris.is_empty());

    let mut p = pris.last().unwrap().clone();

    while p * p < n {
        pris.extend(
            e_seg_sieve_0(pris, p, p * (p - 1))
                .into_iter()
                .collect::<Vec<usize>>(),
        );

        p = pris.last().unwrap().clone();
    }

    let rem = e_seg_sieve_0(pris, p, n - p).collect::<Vec<usize>>();

    pris.extend(rem);
}

pub fn e_inc_sieve(n: usize) -> Box<dyn Iterator<Item = usize>> {
    if n <= 3 {
        if n == 3 {
            return Box::new([2, 3].into_iter());
        } else if n == 2 {
            return Box::new([2].into_iter());
        } else {
            return Box::new(iter::empty());
        }
    }

    let mut pris = vec![2, 3];

    e_inc_sieve_reentrant(&mut pris, n);

    Box::new(pris.into_iter())
}

#[cfg(test)]
#[deprecated = "False algorithm"]
pub fn spiral_wheel() -> impl Iterator<Item = usize> {
    std::iter::from_coroutine(
        #[coroutine]
        || {
            // linear wheel, w_1
            let mut wheel = vec![2];
            let mut l = 2;
            let mut miles = 1;
            let mut rotation = 0;
            // p_{k+1}
            let mut p1 = 3usize;

            yield 2;

            loop {
                /* Roll infinite */

                let mut acc = 1;
                let mut wheel_aux = vec![acc];

                for v in wheel[..wheel.len() - 1].iter() {
                    wheel_aux.push(acc + v);
                    acc += v;
                }

                while miles + wheel[rotation] < p1.pow(2) {
                    yield miles + wheel[rotation];

                    miles += wheel[rotation];
                    rotation = (rotation + 1) % wheel.len();

                    if miles >= 800 {
                        dbg!(&wheel_aux);
                    }
                }

                /* Expand to next level wheel */

                // NOTE: wheel would expand too large as p! grow faster than p^2
                // lim(n->infi): p!/p^2 = infi

                let mut wheel1 = Vec::with_capacity((wheel.len() - 1) * p1);
                let mut acc = 1;
                let mut rem = 0;

                for i in 0..wheel.len() * p1 {
                    if (acc + rem + wheel[i % wheel.len()]) % p1 != 0 {
                        acc += rem + wheel[i % wheel.len()];
                        wheel1.push(rem + wheel[i % wheel.len()]);
                        rem = 0;
                    } else {
                        // remove the marked point from w_k -> w_{k+1}
                        rem = wheel[i % wheel.len()];
                    }
                }

                rotation = ((miles / l) * wheel.len() + rotation + 1 - 1 - 1)
                    % ((p1 - 1) * wheel.len());
                l *= p1;

                wheel = wheel1;
                p1 = 1 + wheel[0];
            }
        },
    )
}

pub fn mairson_sieve(n: usize) -> impl Iterator<Item = usize> {
    std::iter::from_coroutine(
        #[coroutine]
        move || {
            if n == 0 {
                return;
            }

            let mut right: Vec<usize> = (1..=n).chain(once(0)).collect();
            let mut left: Vec<usize> = once(0).chain(0..=n - 1).collect();

            // lpf (least prime factor)
            let mut p = 2usize;

            // while p * p <= n {
            //     /* collected andthen remove all number which lpf is p from S. */
            //     let mut c = vec![];
            //     let mut f = p;

            //     while p * f <= n {
            //         // remove p*f from S
            //         c.push(p * f);
            //         f = forward[f];
            //     }

            //     for i in c {
            //         forward[backward[i]] = forward[i];
            //         backward[forward[i]] = backward[i];
            //     }

            //     p = forward[p];
            // }

            let mut f_max = n / 2;

            while f_max >= p {
                let mut f = f_max;

                while f >= p {
                    right[left[p * f]] = right[p * f];
                    left[right[p * f]] = left[p * f];

                    right[p * f] = n + 1; // flag it
                    f = left[f];
                }

                p = right[p];
                if right[f_max] == n + 1 {
                    f_max = left[f_max];
                }
                while f_max * p > n {
                    f_max = left[f_max];
                }
            }

            let mut i = 1;

            while right[i] != 0 {
                yield right[i];

                i = right[i];
            }
        },
    )
}

pub fn mairson_dual_sieve(n: usize) -> Box<dyn Iterator<Item = usize>> {
    if n <= 1 {
        return Box::new(iter::empty());
    }

    let mut bits = BitVec::from_elem(n + 1, true);
    let mut pris = vec![];

    for f in 2..=n / 2 {
        if bits[f] {
            pris.push(f);
        }

        for p in pris.iter() {
            if f * p > n {
                break;
            }

            bits.set(f * p, false);

            // if f % p == 0 {
            //     break;
            // }
        }
    }

    Box::new(pris.into_iter().chain(
        bits.into_iter()
            .enumerate()
            .skip(1 + n / 2)
            .flat_map(|(i, flag)| {
                if flag {
                    Some(i)
                } else {
                    None
                }
            },),
    ),)
}

/// Sublinear additive sieve by Paul Pritchard
pub fn wheel_sieve(n: usize) -> impl Iterator<Item = usize> {
    /// saving 1/3 time compared with vector (each turn create new vector).
    ///
    /// space cost about 3/5 N.
    struct CompactDoubleArrayList {
        /// 0 for nil
        tail: usize,
        /// [value, forward, backward]
        arr: Vec<Meta>,
    }

    #[derive(Default)]
    struct Meta {
        value: usize,
        left: usize,
        right: usize,
    }

    impl CompactDoubleArrayList {
        fn new() -> Self {
            let tail = 0;
            let arr = vec![Meta::default()];

            Self { tail, arr }
        }

        fn push(&mut self, v: usize) {
            self.arr[self.tail].right = self.tail + 1;

            let new_node = Meta {
                value: v,
                left: self.tail,
                right: 0,
            };

            if self.tail == self.arr.len() - 1 {
                self.arr.push(new_node); // dynamic extend for saving some memory
            } else {
                self.arr[self.tail + 1] = new_node;
            }

            self.tail += 1;
        }

        fn filtering<P: Fn(usize) -> bool>(&mut self, pred: P) {
            let mut i = self.arr[0].right;

            while i != 0 {
                if !pred(self.arr[i].value) {
                    let left = self.arr[i].left;
                    let right = self.arr[i].right;

                    self.arr[left].right = right;
                    self.arr[right].left = left;
                }

                i = self.arr[i].right;
            }
        }

        fn nth(&self, index: usize) -> usize {
            let mut i = self.arr[0].right;
            let mut c = 0;

            while i != 0 {
                if c == index {
                    return self.arr[i].value;
                }

                i = self.arr[i].right;
                c += 1;
            }

            unreachable!("{index} > {c}");
        }

        fn into_iter(self) -> impl Iterator<Item = usize> {
            std::iter::from_coroutine(
                #[coroutine]
                move || {
                    let mut i = self.arr[0].right;

                    while i != 0 {
                        yield self.arr[i].value;
                        i = self.arr[i].right;
                    }
                },
            )
        }

        fn rolling(&mut self, l: usize, n: usize) {
            let mut i = self.arr[0].right;

            debug_assert!(i != 0);

            while l + self.arr[i].value <= n {
                self.push(l + self.arr[i].value);
                i = self.arr[i].right;
            }
        }

        fn delete_multiple_p(&mut self, p: usize) {
            self.filtering(|v| v % p != 0)
        }
    }


    std::iter::from_coroutine(
        #[coroutine]
        move || {
            if n <= 1 {
                return;
            }

            let mut p = 3usize;
            // let mut w = vec![1];  // w_1
            let mut w = CompactDoubleArrayList::new();
            w.push(1);

            let mut l = 2;
            yield 2;

            while p.pow(2) <= n {
                w.rolling(l, min(p * l, n));
                w.delete_multiple_p(p);

                yield p;

                // prevent multiple overflow
                l = min(p * l, n);
                p = w.nth(1);
            }

            w.rolling(l, n);

            for v in w.into_iter().skip(1) {
                yield v;
            }
        },
    )
}

/// -> (v in wheel*, vi in wheel)
fn locate_in_wheel(w: &[usize], prod: usize, raw: usize) -> (usize, usize) {
    let prod_rem = raw % prod;
    let prod_base = raw - prod_rem;

    let (v0, vi) = match w.binary_search(&prod_rem) {
        Ok(i) => (w[i], i),
        Err(i) => (w[i], i),
    };

    (prod_base + v0, vi)
}

/// AKA SFWS (segmented fixed-wheel sieve)
pub fn fixed_wheel_seg_sieve(n: usize) -> impl Iterator<Item = usize> {
    let k = 7;
    let WStruct {
        wheel: w,
        wheel_gap: wg,
        prod,
        // ipm,
        ..
    } = &W[k];

    std::iter::from_coroutine(
        #[coroutine]
        move || {
            if n == 0 {
                return;
            }

            let delta = n.isqrt();

            let pris = e_sieve(delta).collect::<Vec<usize>>();
            let np = pris.len();

            let mut v = 1 + wg[0];
            let mut vi = 1;

            if np <= k {
                // Just rolling to n

                for i in 1..=k {
                    if P[i] > n {
                        return;
                    }

                    yield P[i];
                }

                while v <= n {
                    yield v;

                    v += wg[vi];
                    vi = (vi + 1) % wg.len();
                }

                return;
            }

            for i in 0..pris.len() {
                yield pris[i];
            }

            /* Init v0, vi */

            let v_raw = delta + 1;
            let (mut v, mut vi) = locate_in_wheel(w, *prod, v_raw);

            /* Init factors */
            // absolute value
            let mut factors = vec![];

            for i in 0..np - k {
                let p = pris[k + i];
                // (delta + p - delta % p) + 1
                let f_raw = delta / p + 1;

                factors.push(locate_in_wheel(w, *prod, f_raw));
            }

            /* Run the algorithm */

            let mut bits = BitVec::from_elem(delta + 1, true);

            for l in (delta..=n).step_by(delta) {
                let end = min(l + delta, n);

                /* sift for p_k..p_np */

                for i in 0..np - k {
                    let p = pris[k + i];
                    let (mut f, mut fi) = factors[i];
                    // let (mut c, mut j) = factors[i];

                    while p * f <= end {
                        bits.set(p * f - l, false);

                        f += wg[fi];
                        // c += pms[i][wg[j] / 2];
                        fi = (fi + 1) % wg.len();
                    }

                    factors[i] = (f, fi);
                    // factors[i] = (c, j);
                }

                /* accumulate primes */

                while v <= end {
                    if bits[v - l] {
                        yield v;
                    }

                    v += wg[vi];
                    vi = (vi + 1) % wg.len();
                }

                /* reset for next segment */

                bits.set_all();
            }
        },
    )
}

pub fn fixed_wheel_seg_sieve_p(n: usize) -> Box<dyn Iterator<Item = usize>> {
    let k = 7;
    let WStruct { wheel_gap: wg, .. } = &W[k];

    if n == 0 {
        return Box::new(std::iter::empty());
    }

    let delta = n.isqrt();

    let pris = e_seg_sieve(delta).collect::<Vec<usize>>();
    let np = pris.len();

    let mut v = 1 + wg[0]; // v = p_{k+1}
    let mut vi = 1;

    if np <= k {
        // Just rolling to n

        let mut ans = vec![];

        for i in 1..=k {
            if P[i] > n {
                return Box::new(ans.into_iter());
            }

            ans.push(P[i]);
        }

        while v <= n {
            ans.push(v);

            v += wg[vi];
            vi = (vi + 1) % wg.len();
        }

        return Box::new(ans.into_iter());
    }

    fn sieve_subtask(
        k: usize,
        pris: &[usize],
        np: usize,
        l0: usize,
        l1: usize,
        delta: usize,
    ) -> Vec<usize> {
        let WStruct {
            wheel: w,
            wheel_gap: wg,
            prod,
            // ipm,
            ..
        } = &W[k];

        let mut ans = vec![];

        /* Init v, vi */

        let v_raw: usize = l0 + 1;
        let (mut v, mut vi) = locate_in_wheel(w, *prod, v_raw);

        /* Init factors */
        // absolute value
        let mut factors = vec![];

        for i in 0..np - k {
            let p = pris[k + i];
            let f_raw = l0 / p + 1;

            factors.push(locate_in_wheel(w, *prod, f_raw));
        }

        /* Run the algorithm */

        let mut bits = BitVec::from_elem(delta + 1, true);

        for l in (l0..=l1).step_by(delta) {
            let end = min(l + delta, l1);

            /* sift for p_k..p_np */

            for i in 0..np - k {
                let p = pris[k + i];
                let (mut f, mut j) = factors[i];

                while p * f <= end {
                    bits.set(p * f - l, false);

                    f += wg[j];
                    j = (j + 1) % wg.len();
                }

                factors[i] = (f, j);
            }

            /* accumulate primes */

            while v <= end {
                if bits[v - l] {
                    ans.push(v);
                }

                v += wg[vi];
                vi = (vi + 1) % wg.len();
            }

            /* reset for next segment */

            bits.set_all();
        }

        ans
    }

    let ans = thread::scope(|scope| {
        let pris_ref = &pris;

        let np = pris
            .iter()
            .enumerate()
            .rev()
            .find(|(_, &p)| p * p <= n)
            .map(|(i, _)| i + 1)
            .unwrap_or(0);

        let mut l = delta;

        let mut k_dekta = (n / delta - 1) / USED_CPU_CORES_NUM;

        if l + k_dekta * delta < n {
            k_dekta += 1;
        }

        let mut handles = vec![];

        for _ in 0..USED_CPU_CORES_NUM {
            handles.push(scope.spawn(move || {
                sieve_subtask(
                    k,
                    pris_ref,
                    np,
                    l,
                    min(l + k_dekta * delta, n),
                    delta,
                )
            }));

            l += k_dekta * delta;
        }

        handles
            .into_iter()
            .map(|handle| handle.join().unwrap())
            .flatten()
            .collect::<Vec<usize>>()
    });

    Box::new(pris.into_iter().chain(ans.into_iter()))
}

/// odd factorization
pub fn sundaram_sieve(n: usize) -> Box<dyn Iterator<Item = usize>> {
    // i + j + 2*i*j
    // up to 2n+2

    if n <= 1 {
        return Box::new(iter::empty());
    }

    let k = (n - 1) / 2;
    let mut bits = BitVec::from_elem(k + 1, true);

    for i in 1..=k {
        for j in i..=k {
            if i + j + 2 * i * j > k {
                break;
            }

            bits.set(i + j + 2 * i * j, false);
        }
    }

    Box::new(
        once(2).chain(bits.into_iter().enumerate().skip(1).filter_map(
            |(i, flag)| if flag { Some(2 * i + 1) } else { None },
        ))
    )
}

/// odd factorization
pub fn sundaram_sieve_improved(n: usize) -> Box<dyn Iterator<Item = usize>> {
    if n <= 1 {
        return Box::new(iter::empty());
    }

    let k: usize = (n - 1) / 2;
    let mut bits = BitVec::from_elem(k + 1, true);

    for odd1 in (3..=n.isqrt()).step_by(2) {
        // for c in (odd1 * odd1..=n).step_by(2*odd1) {
        //     bits.set((c - 1) / 2, false);
        // }

        // be faster too much than code commented above,
        // It seems that change multiple to add is anti-optimize in Rust
        // however div is slow

        for odd2 in (odd1..).step_by(2) {
            if odd1 * odd2 > n {
                break;
            }
            bits.set((odd1 * odd2 - 1) / 2, false);
        }
    }

    Box::new(
        once(2).chain(bits.into_iter().enumerate().skip(1).filter_map(
            |(i, flag)| if flag { Some(2 * i + 1) } else { None },
        )),
    )
}

// /// odd factorization
// pub fn sundaram_sieve_improved_seg(n: usize) -> Box<dyn Iterator<Item = usize>> {
//     sieve_spec_case!(n; [0, 10]);

//     Box::new(std::iter::from_coroutine(
//         #[coroutine]
//         move || {
//             for p in [2, 3, 5, 7] {
//                 yield p;
//             }

//             let mut delta = n.isqrt();

//             if delta % 2 == 1 {
//                 delta += 1;
//             }

//             let k = (delta - 2) / 2;

//             let mut bits = BitVec::from_elem(k + 1, true);
//             let mut l = 8;  // for odd1 * odd2 - l is still odd number

//             let mut odd1 = 3;
//             let mut odd2 = odd1;

//             loop {
//                 if odd1 * odd1 > n {
//                     // interger division allow n is even.
//                     for i in 1..=min(k, (n-l-1) / 2) {
//                         if bits[i] {
//                             yield i * 2 + 1 + l;
//                         }
//                     }

//                     return;
//                 }

//                 if odd1 * odd1 >= l+delta {
//                     for i in 1..=k {
//                         if bits[i] {
//                             yield i * 2 + 1 + l;
//                         }
//                     }

//                     bits.set_all();
//                     l += delta;
//                 }

//                 for odd2 in (odd1..).step_by(2) {
//                     if odd1 * odd2 >= min(l+delta, n+1) {
//                         break;
//                     }

//                     // 2i + 1
//                     bits.set((odd1 * odd2 - l - 1) / 2, false);
//                 }
//             }
//     }))
// }


mod atkin {
    use std::cmp::min;

    use lazy_static::lazy_static;

    ///
    /// LLVM 和 rustc 几乎总是内联函数，对于私有函数不需要手动标记内联以便于跨 crate 使用
    ///
    pub(super) fn algs1(x: usize, y: usize) -> usize {
        4 * x * x + y * y
    }

    pub(super) fn algs2(x: usize, y: usize) -> usize {
        3 * x * x + y * y
    }

    pub(super) fn enable_algs3(x: usize, y: usize) -> bool {
        x > y
    }

    pub(super) fn algs3(x: usize, y: usize) -> usize {
        3 * x * x - y * y
    }

    lazy_static! {
        pub(super) static ref DELTA16: [usize; 16] =
            [1, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47, 49, 53, 59];

        // {1,13,17,29,37,41,49,53}
        pub(super) static ref DELTA16_ALGS1: [usize; 8] =
            [0, 3, 4, 7, 9, 10, 13, 14];

        // trim duplicated delta number campared with 1
        // {7,19,31,43}
        pub(super) static ref DELTA16_ALGS2: [usize; 4] =
            [1, 5, 8, 11];

        // {11, 23, 47, 59}
        pub(super) static ref DELTA16_ALGS3: [usize; 4] =
            [2, 6, 12, 15];

        pub(super) static ref DELTA16_GRPS: Vec<Vec<(usize, usize)>> = {
            let mut delta16_grps: Vec<Vec<(usize, usize)>> = vec![vec![]; 16];

            /* algorithm 1. 4x^2 + y^2 = n */

            for i in DELTA16_ALGS1.iter().cloned() {
                for f in 1..=15 {
                    for g in 1..=30 {
                        if algs1(f, g) < DELTA16[i] {
                            continue;
                        }

                        if (algs1(f, g) - DELTA16[i]) % 60 == 0 {
                            delta16_grps[i].push((f, g));
                        }
                    }
                }
            }

            /* algorithm 2. 3x^2 + y^2 = n */

            for i in DELTA16_ALGS2.iter().cloned() {
                for f in 1..=15 {
                    for g in 1..=30 {
                        if algs2(f, g) < DELTA16[i] {
                            continue;
                        }

                        if (algs2(f, g) - DELTA16[i]) % 60 == 0 {
                            delta16_grps[i].push((f, g));
                        }
                    }
                }
            }

            /* algorithm 3. 3x^2 - y^2 = 60k + delta */
            // FIXME: incorrect algoriuthm
            for i in DELTA16_ALGS3.iter().cloned() {
                for f in 3..=10 {
                    for g in 1..=min(30, f-1) {
                        if algs3(f, g) < DELTA16[i] {
                            continue;
                        }

                        if (algs3(f, g) - DELTA16[i]) % 60 == 0 {
                            delta16_grps[i].push((f, g));
                        }
                    }
                }
            }

            delta16_grps
        };
    }
}

///
///
/// [Binary Quadratic Factorization](http://www.ams.org/mcom/2004-73-246/S0025-5718-03-01501-1/S0025-5718-03-01501-1.pdf)
///
///
#[cfg(test)]
#[deprecated = "Too Slow"]
pub fn atkin_sieve_enum_lattice(n: usize) -> impl Iterator<Item = usize> {
    use std::collections::HashSet;

    use atkin::*;
    use common::def_stats;

    use crate::isqrt_ceil;

    def_stats!(Watch, { algs1, algs2, algs3, remove_p });

    std::iter::from_coroutine(
        #[coroutine]
        move || {
            // P[17] = 59
            for i in 1..=17 {
                if P[i] > n {
                    return;
                }

                yield P[i]
            }

            if n == 59 {
                return;
            }

            let n_k = n.div_ceil(60);
            let delta_k = n_k - 1;
            let delta = delta_k * 60;

            // let mut bits = BitVec::from_elem(delta + 1, false);
            let mut bits_k = BitVec::from_elem(delta_k * 16, false);

            let mut pri_square = HashSet::<usize>::new();

            for i in 1..=17 {
                let r = P[i] * P[i];

                for j in (r..=n).step_by(r) {
                    pri_square.insert(j);
                }
            }

            let mut l = 60;
            let mut l_k = 1;

            let mut watch = Watch::new();

            while l <= n {
                /* algorithm 1 4x^2 + y^2 = n */

                watch.algs1.s();

                for i in DELTA16_ALGS1.iter().cloned() {
                    let mut pairs = HashSet::<(usize, usize)>::new();

                    for (f, g) in DELTA16_GRPS[i].iter().cloned() {
                        let mut x = f;
                        let mut y0 = g;
                        let mut k0 = (algs1(f, g) - DELTA16[i]) / 60;

                        while k0 < l_k + delta_k {
                            k0 += 2 * x + 15;
                            x += 15;
                        }

                        while x > 15 {
                            x -= 15;
                            k0 -= 2 * x + 15;

                            while k0 < l_k {
                                k0 += y0 + 15;
                                y0 += 30;
                            }

                            let mut k = k0;
                            let mut y = y0;

                            while k < l_k + delta_k {
                                if pairs.insert((x, y)) {
                                    let bi = (k - l_k) * 16 + i;

                                    bits_k.set(bi, !bits_k.get(bi).unwrap());
                                }

                                k += y + 15;
                                y += 30;
                            }
                        }
                    }
                }

                watch.algs1.e();

                watch.algs2.s();

                /* algorithm 2 3x^2 + y^2 = n */

                for i in DELTA16_ALGS2.iter().cloned() {
                    let mut pairs = HashSet::<(usize, usize)>::new();

                    for (f, g) in DELTA16_GRPS[i].iter().cloned() {
                        let mut x = f;
                        let mut y0 = g;
                        let mut k0 =
                            ((algs2(f, g) - DELTA16[i]) / 60) as isize;

                        while k0 < 0 || (k0 as usize) < l_k + delta_k {
                            k0 += (x + 5) as isize;
                            x += 10;
                        }

                        while x > 10 {
                            x -= 10;
                            k0 -= (x + 5) as isize;

                            while k0 < 0 || (k0 as usize) < l_k {
                                k0 += (y0 + 15) as isize;
                                y0 += 30;
                            }

                            let mut k = k0;
                            let mut y = y0;

                            while (k as usize) < l_k + delta_k {
                                if pairs.insert((x, y)) {
                                    let bi = (k as usize - l_k) * 16 + i;

                                    bits_k.set(bi, !bits_k.get(bi).unwrap());
                                }

                                k += (y + 15) as isize;
                                y += 30;
                            }
                        }
                    }
                }

                watch.algs2.e();
                watch.algs3.s();

                /* algorithm 3 3x^2 - y^2 = 60k + delta */
                /* NOTE: There is a bug for origin formula-3
                enumerate lattice point */

                for i in DELTA16_ALGS3.iter().cloned() {
                    // let mut pairs = HashSet::<(usize, usize)>::new();

                    for k in l_k..l_k + delta_k {
                        let c = 60 * k + DELTA16[i];

                        for x in isqrt_ceil!((c + 1) / 3)..=n.isqrt() {
                            let y2 = 3 * x * x - c;
                            let y = (3 * x * x - c).isqrt();

                            if y * y == y2 && y < x {
                                let bi = (k as usize - l_k) * 16 + i;

                                bits_k.set(bi, !bits_k.get(bi).unwrap());
                            }
                        }
                    }
                }

                watch.algs3.e();
                watch.remove_p.s();

                /* remove p^2 */

                for k in l_k..l_k + delta_k {
                    for i in 0..16 {
                        if bits_k[(k - l_k) * 16 + i] {
                            let c = 60 * k + DELTA16[i];

                            if c > n {
                                break;
                            }

                            if !pri_square.remove(&c) {
                                yield c;

                                let r = c * c;

                                for j in (r..=n).step_by(r) {
                                    pri_square.insert(j);
                                }
                            }
                        }
                    }
                }
                watch.remove_p.e();

                /* update */

                bits_k.clear();

                l += delta;
                l_k += delta_k;
            }

            println!("Watch: {watch:#?}");
        },
    )
}

pub fn atkin_sieve_simple(n: usize) -> impl Iterator<Item = usize> {
    use atkin::*;

    std::iter::from_coroutine(
        #[coroutine]
        move || {
            if n == 0 {
                return;
            }

            let mut bits = BitVec::from_elem(n + 1, false);

            if n >= 2 {
                bits.set(2, true);
            }

            if n >= 3 {
                bits.set(3, true);
            }

            for x in 1..=n.isqrt() {
                for y in 1..=n.isqrt() {
                    let c1 = algs1(x, y);

                    // not using 2-3-5 wheel sieve for efficiency
                    if c1 <= n && (c1 % 12 == 1 || c1 % 12 == 5) {
                        bits.set(c1, !bits[c1]);
                    }

                    let c2 = algs2(x, y);

                    // 1 mod 6 => 7 mod 12
                    // trim duplicate element with algs1
                    if c2 <= n && c2 % 12 == 7 {
                        bits.set(c2, !bits[c2]);
                    }

                    if enable_algs3(x, y) {
                        let c3 = algs3(x, y);

                        if c3 <= n && c3 % 12 == 11 {
                            bits.set(c3, !bits[c3]);
                        }
                    }
                }
            }

            /* trim p^2 */

            for i in 5..=n.isqrt() {
                if bits[i] {
                    let r = i * i;

                    for j in (r..=n).step_by(r) {
                        bits.set(j, false);
                    }
                }
            }

            for i in 1..=n {
                if bits[i] {
                    yield i;
                }
            }
        },
    )
}

/// Greate Prime Factor Sieve
///
/// Space: O(n)
pub fn gpf_sieve(n: usize) -> Box<dyn Iterator<Item = usize>> {
    if n <= 1 {
        return Box::new(iter::empty());
    }

    let mut p = 2;
    let mut bits = BitVec::from_elem(n + 1, true);
    let mut factors = vec![];

    let mut i = 2;
    let mut pris = vec![];

    while p <= n / 2 {
        pris.push(p);

        factors.push(p);

        let mut f_stack = factors;
        factors = vec![];

        while let Some(f) = f_stack.pop() {
            if p * f <= n {
                bits.set(p * f, false);

                f_stack.push(p * f);
                factors.push(f);
            }
        }

        i += 1;

        while !bits[i] {
            i += 1
        }

        p = i;
    }

    Box::new(pris.into_iter().chain(
        bits.into_iter()
            .enumerate()
            .skip(n / 2 + 1)
            .filter_map(|(i, flag)| if flag { Some(i) } else { None }),
    ),)
}


////////////////////////////////////////
//// Infinite Sieve

pub fn e_sieve_inf() -> impl Iterator<Item = usize> {
    std::iter::from_coroutine(
        #[coroutine]
        move || {
            let mut pris = vec![2];

            let mut p1 = pris[0];
            let mut l0 = p1;
            let mut l1 = p1 * p1;
            let mut delta = l1 - l0;
            let mut seg = BitVec::from_elem(delta + 1, true);

            for i in 0..pris.len() {
                yield pris[i];
            }

            loop {
                for &p in pris.iter() {
                    for i in (p - l0 % p..=delta).step_by(p) {
                        seg.set(i, false);
                    }
                }

                for i in 1..=delta {
                    if seg[i] {
                        yield l0 + i;

                        pris.push(l0 + i);
                    }
                }

                l0 = l1;
                p1 = pris.last().unwrap().clone();
                l1 = p1 * p1;
                delta = l1 - l0;
                seg.grow(delta + 1 - seg.len(), true);
                seg.set_all();
            }
        },
    )
}

pub fn bengelloun_sieve_inf() -> impl Iterator<Item = usize> {
    std::iter::from_coroutine(
        #[coroutine]
        move || {
            let mut lastp = 2;
            let mut lpf = vec![0; 5]; // +1 cap for index start from 1.

            yield lastp;

            for n in 3.. {
                if n & 1 == 0 {
                    lpf[n] = 2;
                    lpf[n / 2 * 3] = 3;
                } else if lpf[n] == 0 {
                    yield n;

                    lpf[lastp] = n;
                    // lpf[n] = n;
                    lastp = n;

                    // => p_next < 2n
                    // => p1 / p0 < 2
                    // => (p1 / p0) * p_next < 4n - 2
                    lpf.resize(n * 4, 0);
                } else {
                    let p = lpf[n]; // lp0 > 2
                    let f = n / p;

                    // min(f, lpf[f]) = truly lpf[f]
                    if p < min(f, lpf[f]) {
                        let p1 = lpf[p]; // next prime after p

                        lpf[p1 * f] = p1;
                    }
                }
            }
        },
    )
}

/// *LINEAR PRIME-NUMBER SIEVES: A FAMILY TREE:* Algorithm 4.4.
pub fn gpf_sieve_inf() -> impl Iterator<Item = usize> {
    std::iter::from_coroutine(
        #[coroutine]
        move || {
            let mut lastp = 2;
            let mut sqrtp = 2; // minimal (for square) prime >= n
            let mut gpf = vec![0; 2 * 2 + 1]; // (p, f)

            yield 2;

            for n in 3.. {
                if n == sqrtp * sqrtp {
                    gpf[n] = sqrtp; // add starter
                    sqrtp = gpf[sqrtp]; // point to next prime after sqrtp
                }

                if gpf[n] == 0 {
                    yield n;

                    // gpf[n] = n;
                    // C_max < p_next < 2n => (p1/p0 or 2) * C_max < 4n
                    gpf[lastp] = n; // point to next prime
                    lastp = n;
                    gpf.resize(4 * n, 0);
                } else {
                    let p = gpf[n];
                    let f = n / p;
                    let p1 = gpf[p]; // next prime

                    gpf[p1 * f] = p1;

                    // min(f, gpf[f]) is truly gpf[f]
                    if p == min(f, gpf[f]) {
                        let mut f0 = f / p + 1;

                        while min(f0, gpf[f0]) > p {
                            f0 += 1;
                        }

                        gpf[f0 * p * p] = p;
                    }
                }
            }
        },
    )
}

///
/// ```
/// use m6_math::number::*;
///
/// assert_eq!(factorization(18), vec![2, 3, 3]);
/// assert_eq!(factorization(3), vec![3]);
/// assert_eq!(factorization(0), vec![]);
/// assert_eq!(factorization(1), vec![]);
///
/// ```
pub fn factorization(mut n: usize) -> Vec<usize> {
    let mut factors = vec![];

    if n == 0 {
        return factors;
    }

    for p in bengelloun_sieve_inf() {
        while n % p == 0 {
            n /= p;
            factors.push(p);
        }

        if n == 1 {
            break;
        }
    }

    factors
}

/// 一鱼两吃，既返回值质数列表，也返回质因数分解列表 (lpf)
/// 这种 API 风格倒也正好是 Rust 推荐的风格
pub fn mairson_dual_sieve_factorization(n: usize) -> (Vec<usize>, Vec<usize>) {
    let mut lpf = vec![0; n + 1];
    let mut pris = vec![];

    if n <= 1 {
        return (pris, lpf);
    }

    for f in 2..=n / 2 {
        if lpf[f] == 0 {
            lpf[f] = f;
            pris.push(f);
        }

        for p in pris.iter().cloned() {
            if p * f > n {
                break;
            }

            lpf[p * f] = p;

            if p == lpf[f] {
                break;
            }
        }
    }

    for i in (n / 2) + 1..=n {
        if lpf[i] == 0 {
            lpf[i] = i;
            pris.push(i);
        }
    }

    (pris, lpf)
}


#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use bag::BTreeBag;
    use common::{random_range, same};
    use derive_new::new;
    use derive_where::derive_where;

    use super::*;

    #[ignore = "for debug"]
    #[allow(deprecated)]
    #[test]
    fn debug_sieve() {
        // use bag::BTreeBag;

        // for k in 1..=7 {
        //     print!("{k}: {:#?}\n", BTreeBag::from_iter(&W[k].wheel_gap));
        // }

        let v = e_seg_sieve_p(16900).collect::<Vec<_>>();
        println!("{v:#?}");
    }

    #[test]
    fn test_limited_binary_quadratic_equation_solving() {
        use atkin::*;

        #[derive(new)]
        #[derive_where(PartialEq, Eq, PartialOrd, Ord)]
        #[derive(Clone, Copy, Debug)]
        struct T4 {
            #[derive_where(skip)]
            x: usize,
            #[derive_where(skip)]
            y: usize,
            k: usize,
            delta: usize,
        }

        /* Init set */

        let mut delta16_grps: Vec<Vec<(usize, usize)>> = vec![vec![]; 16];

        /* algorithm 1 4x^2 + y^2 = n */

        for i in DELTA16_ALGS1.iter().cloned() {
            for f in 1..=15 {
                for g in 1..=30 {
                    if algs1(f, g) < DELTA16[i] {
                        continue;
                    }

                    if (algs1(f, g) - DELTA16[i]) % 60 == 0 {
                        delta16_grps[i].push((f, g));
                    }
                }
            }
        }

        /* algorithm 2 3x^2 + y^2 = n */

        for i in DELTA16_ALGS2.iter().cloned() {
            for f in 1..=15 {
                for g in 1..=30 {
                    if algs2(f, g) < DELTA16[i] {
                        continue;
                    }

                    if (algs2(f, g) - DELTA16[i]) % 60 == 0 {
                        delta16_grps[i].push((f, g));
                    }
                }
            }
        }

        /* algorithm 3 3x^2 - y^2 = 60k + delta */

        for i in DELTA16_ALGS3.iter().cloned() {
            for f in 3..=15 {
                for g in 1..=30 {
                    if 3 * f * f <= g * g {
                        continue;
                    }

                    if algs3(f, g) < DELTA16[i] {
                        continue;
                    }

                    if (algs3(f, g) - DELTA16[i]) % 60 == 0 {
                        delta16_grps[i].push((f, g));
                    }
                }
            }
        }

        for i in 0..16 {
            print!("{}: ", DELTA16[i]);

            for (x, y) in delta16_grps[i].iter().cloned() {
                let c = if DELTA16_ALGS1.contains(&i) {
                    algs1(x, y)
                } else if DELTA16_ALGS2.contains(&i) {
                    algs2(x, y)
                } else {
                    algs3(x, y)
                };

                print!("({x}, {y}){} ", (c - DELTA16[i]) / 60)
            }

            println!();
        }

        let n: usize = 1690;
        let n_k = n / 60;
        let delta_k = (n_k - 1).isqrt();
        let delta = 60 * delta_k;

        let mut l_k = 1;
        let mut l = 60;

        let mut pairs = BTreeBag::<T4>::new();

        while l <= n {
            println!("\n[{}..{}]", l_k, l_k + delta_k);

            for i in DELTA16_ALGS1.iter().cloned() {
                for (f, g) in delta16_grps[i].iter().cloned() {
                    let mut x = f;
                    let mut y0 = g;
                    let mut k0 = (algs1(f, g) - DELTA16[i]) / 60;

                    while k0 < l_k + delta_k {
                        k0 += 2 * x + 15;
                        x += 15;
                    }

                    while x > 15 {
                        x -= 15;
                        k0 -= 2 * x + 15;

                        while k0 < l_k {
                            k0 += y0 + 15;
                            y0 += 30;
                        }

                        let mut k = k0;
                        let mut y = y0;

                        while k < l_k + delta_k {
                            assert_eq!(algs1(x, y), 60 * k + DELTA16[i]);

                            println!("({x}, {y}, {k}): {}", DELTA16[i]);

                            pairs.insert(T4::new(x, y, k, DELTA16[i]));

                            k += y + 15;
                            y += 30;
                        }
                    }
                }
            }

            for i in DELTA16_ALGS2.iter().cloned() {
                for (f, g) in delta16_grps[i].iter().cloned() {
                    let mut x = f;
                    let mut y0 = g;
                    let mut k0 = ((algs2(f, g) - DELTA16[i]) / 60) as isize;

                    while k0 < 0 || (k0 as usize) < l_k + delta_k {
                        k0 += (x + 5) as isize;
                        x += 10;
                    }

                    while x > 10 {
                        x -= 10;
                        k0 -= (x + 5) as isize;

                        while k0 < 0 || (k0 as usize) < l_k {
                            k0 += (y0 + 15) as isize;
                            y0 += 30;
                        }

                        let mut k = k0;
                        let mut y = y0;

                        while (k as usize) < l_k + delta_k {
                            assert_eq!(
                                algs2(x, y),
                                60 * k as usize + DELTA16[i]
                            );

                            println!("({x}, {y}, {k}): {}", DELTA16[i]);

                            pairs
                                .insert(T4::new(x, y, k as usize, DELTA16[i]));

                            k += (y + 15) as isize;
                            y += 30;
                        }
                    }
                }
            }

            for i in DELTA16_ALGS3.iter().cloned() {
                for (f, g) in delta16_grps[i].iter().cloned() {
                    let mut x = f;
                    let mut y0 = g;
                    let mut k0 = ((algs3(f, g) - DELTA16[i]) / 60) as isize;

                    'outter: loop {
                        while k0 >= 0 && k0 as usize >= l_k + delta_k {
                            if x <= y0 {
                                break 'outter;
                            }

                            k0 -= (y0 + 15) as isize;
                            y0 += 30;
                        }

                        let mut k: isize = k0;
                        let mut y = y0;

                        while k >= 0 && k as usize >= l_k && y < x {
                            assert_eq!(
                                algs3(x, y),
                                60 * k as usize + DELTA16[i]
                            );

                            println!("({x}, {y}, {k}): {}", DELTA16[i]);

                            pairs
                                .insert(T4::new(x, y, k as usize, DELTA16[i]));

                            k -= (y + 15) as isize;
                            y += 30;
                        }

                        k0 += (x + 5) as isize;
                        x += 10;
                    }
                }
            }

            l_k += delta_k;
            l += delta;
        }

        // println!("Find duplicate pair:");

        // for (k, n) in pairs.iter() {
        //     if n > 1 {
        //         println!("{:?}", pairs.get(k).unwrap().collect::<Vec<_>>());
        //     }
        // }
    }

    #[test]
    fn test_prime() {
        macro_rules! test_prime {
            (n=$n:expr; $($fn:ident),+ $(,)?; $($fn_inf:ident),* $(,)?) => {
                for n in [0, 1] {
                    $(assert!($fn(n).collect::<Vec<usize>>().is_empty(),
                    "{}: {n}", stringify!($fn));)+
                }

                for n in [2] {
                    $(assert_eq!($fn(n).collect::<Vec<usize>>(), vec![2],
                     "{}: {n}", stringify!($fn));)+
                }

                for n in [3, 4] {
                    $(assert_eq!($fn(n).collect::<Vec<usize>>(),
                        vec![2, 3], "{}: {n}", stringify!($fn));
                    )+
                }

                let n = $n;

                test_prime!(sub co-check| n, $($fn),+);
                test_prime!(sub check-inf| n, $($fn_inf),*);
            };
            (sub co-check | $n:expr, $standard:ident, $($name:ident),+ ) => {
                let $standard: BTreeSet<usize> = $standard($n).collect();
                $(let $name: BTreeSet<usize> = $name($n).collect();)+

                for i in 1..=$n {
                    let flag = $standard.contains(&i);

                    $(
                        assert!(
                            $name.contains(&i) == flag,
                            "{i} should be {flag} for `{}`",
                            stringify!($name)
                        );

                        assert_eq!(
                            $standard.len(),
                            $name.len(),
                            "{}: -/{} should has {} primes",
                            stringify!($name),
                            $n,
                            $standard.len()
                        );
                    )+
                }
            };
            (sub check-inf | $n:expr, $($name:ident),+ ) => {
                let e_sieve: BTreeSet<usize> = e_sieve($n).collect();
                $(let $name: BTreeSet<usize> = $name().take(e_sieve.len()).collect();)+

                for i in 1..=$n {
                    assert!(
                        same!(
                            e_sieve.contains(&i),
                            $($name.contains(&i)),+
                        ),
                        "{i}"
                    );
                }
            }
        }

        let mairson_dual_sieve_factorization =
            |n: usize| -> Box<dyn Iterator<Item = usize>> {
                Box::new(mairson_dual_sieve_factorization(n).0.into_iter())
            };

        let fixed_list = [169, 1690, 16900];
        let rand_list_meta = [
            (1, 10, 3),
            (100, 300, 5),
            (1000, 3000, 3),
            (10_000, 30_000, 3),
        ];

        let rand_list = rand_list_meta
            .into_iter()
            .map(|(s, e, iters)| (0..iters).map(move |_| random_range!(s..=e)))
            .flatten()
            .collect::<Vec<usize>>();

        for n in fixed_list.into_iter().chain(rand_list) {
            test_prime!(
                n = n
                ;
                e_sieve,
                e_seg_sieve,
                wheel_sieve,
                mairson_sieve,
                mairson_dual_sieve,
                e_inc_sieve,
                gpf_sieve,
                fixed_wheel_seg_sieve,
                sundaram_sieve,
                sundaram_sieve_improved,
                atkin_sieve_simple,
                // atkin_sieve_enum_lattice,
                e_seg_sieve_p,
                fixed_wheel_seg_sieve_p,
                mairson_dual_sieve_factorization
                ;
                e_sieve_inf,
                bengelloun_sieve_inf,
                gpf_sieve_inf
            );
        }
    }


    #[test]
    fn test_factorization() {
        let n = 16800;
        let lpf = mairson_dual_sieve_factorization(n).1;

        for _ in 0..100 {
            let v = random_range!(2..=n);

            let ans = {
                let mut v = v;
                let mut ans = vec![];

                while lpf[v] < v {
                    ans.push(lpf[v]);
                    v /= lpf[v];
                }

                ans.push(v);

                ans
            };

            assert_eq!(factorization(v), ans);
        }
    }
}
