use std::{cmp::min, iter::once, pin::Pin, rc::Rc};

use bit_vec::BitVec;


////////////////////////////////////////////////////////////////////////////////
//// Macro

/// replacement for
///
/// #![feature(int_log)]
///
/// a.ilog2()
#[macro_export]
macro_rules! ilog2 {
    ($x:expr) => {{
        let mut x = $x;
        debug_assert!(x > 0);

        let mut i = 0;

        while x > 0 {
            x >>= 1;
            i += 1;
        }

        i
    }};
}


///
/// m, n shouldb e uint
///
#[macro_export]
macro_rules! gcd {
    (mod| $m:expr, $n:expr) => {{
        let mut m = $m;
        let mut n = $n;

        while n > 0 {
            (m, n) = (n, m % n);
        }

        m
    }};
    (sub| $m:expr, $n:expr) => {{
        use std::cmp::Ordering::*;

        let mut m = $m;
        let mut n = $n;

        if m < n {
            (m, n) = (n, m);
        }

        if n != 0 {
            loop {
                match m.cmp(&n) {
                    Less => n -= m,
                    Greater => m -= n,
                    Equal => break
                }
            }
        }

        m
    }};
    (brute| $m:expr, $n:expr) => {{
        use std::cmp::min;

        let mut m = $m;
        let n = $n;

        let smaller = min(m, n);

        for i in (1..=smaller).rev() {
            if m % i == 0 && n % i == 0 {
                m = i;
                break;
            }
        }

        m
    }};
    ($ty:ty| $m:expr, $n:expr) => {{
        let mut m: $ty = $m;
        let mut n: $ty = $n;

        if m < n {
            (m, n) = (n, m);
        }

        if n == 0 {
            m
        }
        else {
            // big int
            // use std::mem::size_of;
            // if size_of::<$ty>() * 8 - m.leading_zeros() as usize > 31 {
            //     if size_of::<$ty>() * 8 - (m/n).leading_zeros() as usize > 15
            //         && n % 2 != 0
            //     {
            //         while m % 2 == 0 {
            //             m >>= 1;
            //         }
            //     }

            //     gcd!(mod|m, n)
            // }
            // else {
            //     gcd!(mod|m, n)
            // }

            if n % 2 != 0 {
                while m % 2 == 0 {
                    m >>= 1;
                }
            }

            gcd!(mod|m, n)
        }
    }};
    ($m:expr, $n:expr) => {
        gcd!(mod|$m, $n)
    };
}


#[macro_export]
macro_rules! ext_gcd {
    ($m:expr,$n:expr) => {{
        let (mut m1, mut n1) = ($m, $n);
        let (mut x, mut y) = (1, 0);
        let (mut x1, mut y1) = (0, 1);

        while n1 > 0 {
            let q = m1 / n1;

            (x, x1) = (x1, x - q * x1);
            (y, y1) = (y1, y - q * y1);
            (m1, n1) = (n1, m1 - q * n1);
        }

        (m1, x, y)
    }};
}


/// define any lcm with 0 is 0
#[macro_export]
macro_rules! lcm {
    (brute|$m:expr, $n:expr) => {{
        let mut m = $m;
        let mut n = $n;

        if m < n {
            (m, n) = (n, m);
        }

        if n == 0 {
            0
        } else {
            for i in m..=m * n {
                if i % m == 0 && i % n == 0 {
                    m = i;
                    break;
                }
            }

            m
        }
    }};
    ($ty:ty|$m:expr, $n:expr) => {{
        let m = $m;
        let n = $n;

        if m == 0 || n == 0 {
            0
        } else {
            (m * n) / gcd!($ty | m, n)
        }
    }};
}

#[macro_export]
macro_rules! is_prime {
    ($n:expr; $ty:ty) => {{
        let n = $n;

        'ret: loop {
            if n <= 1 {
                break false;
            }

            if n == 2 || n == 3 {
                break true;
            }

            if n % 2 == 0 || n % 3 == 0 {
                break false;
            }

            for i in (5..=<$ty>::isqrt(n)).step_by(6) {
                if n % i == 0 || n % (i + 2) == 0 {
                    break 'ret false;
                }
            }

            break true;
        }
    }};
}

macro_rules! e_sieve_spec_case {
    ($n: ident) => {
        let n = $n;

        if n < 2 {
            return
        }
        else if n == 2 {
            return std::iter::from_coroutine(
                #[coroutine]
                move || {

                })
            return
        }
        else if n == 3 {
            yield 3;
            return
        }
    };
}

////////////////////////////////////////////////////////////////////////////////
//// Function

pub fn e_sieve(n: usize) -> Box<dyn Iterator<Item = usize>> {
    if n < 2 {
        return Box::new(std::iter::empty());
    }
    else if n == 2 {
        return Box::new(std::iter::once(2));
    }

    let mut bits = BitVec::from_elem(n + 1, true);

    for i in 2..=n.isqrt() {
        if bits[i] {
            for j in (i*i..=n).step_by(i) {
                bits.set(j, false);
            }
        }
    }

    Box::new(bits.into_iter().enumerate().skip(2).flat_map(|(i, flag)| if flag { Some(i) } else { None }))
}

/// Space: O(\sqrt{n})
pub fn e_seg_sieve(n: usize) -> Box<dyn Iterator<Item = usize>> {
    if n < 2 {
        return Box::new(std::iter::empty());
    }
    else if n == 2 {
        return Box::new(std::iter::once(2));
    }
    else if n == 3 {
        return Box::new([2, 3].into_iter());
    }

    Box::new(std::iter::from_coroutine(
        #[coroutine]
        move || {
            let delta = n.isqrt();

            let pris = Rc::new(e_sieve(delta).collect::<Vec<usize>>());

            for i in 0..pris.len() {
                yield pris.as_ref()[i];
            }

            let mut l = delta;

            while l + 1 <= n {
                for p in e_sieve_reentrant(pris.clone(), l, min(delta, n-l)) {
                    yield p;
                }

                l += delta;
            }
        },
    ))
}

/// 1. pris from 0..x, no empty.
/// 2. return (l, l+delta]
fn e_sieve_reentrant(
    pris: Rc<Vec<usize>>,
    l: usize,
    delta: usize,
) -> impl Iterator<Item = usize> {
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


#[deprecated = "This is a failed design demo"]
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

            let mut forward: Vec<usize> = (1..=n).chain(once(0)).collect();
            let mut backward: Vec<usize> = once(0).chain(0..=n - 1).collect();

            // lpf (least prime factor)
            let mut p = 2usize;

            while p.pow(2) <= n {
                /* collected andthen remove all number which lpf is p from S. */

                let mut c = vec![];
                let mut f = p;

                while p * f <= n {
                    // remove p*f from S
                    c.push(p * f);
                    f = forward[f];
                }

                for i in c {
                    forward[backward[i]] = forward[i];
                    backward[forward[i]] = backward[i];
                }

                p = forward[p];
            }

            let mut i = 1;

            while forward[i] != 0 {
                yield forward[i];

                i = forward[i];
            }
        },
    )
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
        arr: Vec<[usize; 3]>,
    }

    impl CompactDoubleArrayList {
        fn new() -> Self {
            let tail = 0;
            let list = vec![[0; 3]];

            Self { tail, arr: list }
        }

        fn push(&mut self, v: usize) {
            self.arr[self.tail][1] = self.tail + 1;

            let new_node = [v, 0, self.tail];

            if self.tail == self.arr.len() - 1 {
                self.arr.push(new_node);
            } else {
                self.arr[self.tail + 1] = new_node;
            }

            self.tail += 1;
        }

        fn filter<P: Fn(usize) -> bool>(&mut self, pred: P) {
            let mut i = self.arr[0][1];

            while i != 0 {
                if !pred(self.arr[i][0]) {
                    let prev = self.arr[i][2];
                    let next = self.arr[i][1];

                    self.arr[prev][1] = next;
                    self.arr[next][2] = prev;
                }

                i = self.arr[i][1];
            }
        }

        fn nth(&self, index: usize) -> usize {
            let mut i = self.arr[0][1];
            let mut c = 0;

            while i != 0 {
                if c == index {
                    return self.arr[i][0];
                }

                i = self.arr[i][1];
                c += 1;
            }

            unreachable!("{index} > {c}");
        }

        fn into_iter(self) -> impl Iterator<Item = usize> {
            std::iter::from_coroutine(
                #[coroutine]
                move || {
                    let mut i = self.arr[0][1];

                    while i != 0 {
                        yield self.arr[i][0];
                        i = self.arr[i][1];
                    }
                },
            )
        }

        fn rolling(&mut self, l: usize, n: usize) {
            let mut i = self.arr[0][1];

            debug_assert!(i != 0);

            while l + self.arr[i][0] <= n {
                self.push(l + self.arr[i][0]);
                i = self.arr[i][1];
            }
        }

        fn delete_multiple_p(&mut self, p: usize) {
            self.filter(|v| v % p != 0)
        }
    }


    std::iter::from_coroutine(
        #[coroutine]
        move || {
            if n < 2 {
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

                l = min(p * l, n);
                p = w.nth(1);
            }

            w.rolling(l, n);

            dbg!(&w.arr.len());

            for v in w.into_iter().skip(1) {
                yield v;
            }
        },
    )
}


/// AKA SFWS (segmented fixed-wheel sieve)
pub fn wheel_seg_sieve(_n: usize) -> impl Iterator<Item = usize> {
    std::iter::from_coroutine(
        #[coroutine]
        move || {},
    )
}



#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use common::same;

    use super::*;

    #[test]
    fn test_gcd() {
        for i in 0..=1000 {
            for j in 0..=1000 {
                let r1 = gcd!(mod| i, j);
                let r2 = gcd!(sub | i, j);
                let r3 = gcd!(i32 | i, j);

                let (r4, x, y) = ext_gcd!(i, j);
                assert_eq!(x * i + y * j, r4);

                assert_eq!(r1, r2);
                assert_eq!(r2, r3);
                assert_eq!(r4, r3);

                // brute too slow
                // assert_eq!(r1, gcd!(brute|i, j), "for gcd({i},{j})");
            }
        }
    }

    #[test]
    fn test_lcm() {
        for i in 0..=100 {
            for j in 0..=100 {
                let r1 = lcm!(i32 | i, j);
                let r2 = lcm!(brute | i, j);

                assert_eq!(r1, r2);
            }
        }
    }

    #[ignore = "for debug"]
    #[allow(deprecated)]
    #[test]
    fn test_wheel_prime() {
        let n: usize = 10_0;

        for p in spiral_wheel() {
            if p > n {
                break;
            }

            println!("{p}");
        }
    }

    #[test]
    fn debug() {
        let n = 40;

        for v in wheel_sieve(10_0000) {}
    }

    #[test]
    fn test_prime() {
        /* co-test */

        for n in [0, 1] {
            assert!(e_sieve(n).collect::<Vec<usize>>().is_empty());
            assert!(e_seg_sieve(n).collect::<Vec<usize>>().is_empty());
            assert!(wheel_sieve(n).collect::<Vec<usize>>().is_empty());
            assert!(mairson_sieve(n).collect::<Vec<usize>>().is_empty());
        }

        for n in [2] {
            assert_eq!(e_sieve(n).collect::<Vec<usize>>(), vec![2]);
            assert_eq!(e_seg_sieve(n).collect::<Vec<usize>>(), vec![2]);
            assert_eq!(wheel_sieve(n).collect::<Vec<usize>>(), vec![2]);
            assert_eq!(mairson_sieve(n).collect::<Vec<usize>>(), vec![2]);
        }

        for n in [3, 4] {
            assert_eq!(e_sieve(n).collect::<Vec<usize>>(), vec![2, 3]);
            assert_eq!(e_seg_sieve(n).collect::<Vec<usize>>(), vec![2, 3]);
            assert_eq!(wheel_sieve(n).collect::<Vec<usize>>(), vec![2, 3]);
            assert_eq!(mairson_sieve(n).collect::<Vec<usize>>(), vec![2, 3]);
        }

        let n = 1690;

        let e_sieve: BTreeSet<usize> = e_sieve(n).collect();
        let e_seg_sive: BTreeSet<usize> = e_seg_sieve(n).collect();
        let wheel_sieve: BTreeSet<usize> = wheel_sieve(n).collect();
        let mairson_sieve: BTreeSet<usize> = mairson_sieve(n).collect();

        for i in 1..=n {
            assert!(
                same!(
                    is_prime!(i; usize),
                    e_sieve.contains(&i),
                    e_seg_sive.contains(&i),
                    wheel_sieve.contains(&i),
                    mairson_sieve.contains(&i)
                ),
                "{i}"
            );
        }
    }
}
