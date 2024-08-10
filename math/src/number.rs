////////////////////////////////////////////////////////////////////////////////
//// Macro

use bit_vec::BitVec;

/// replacement for
///
/// #![feature(int_log)]
///
/// a.ilog2()
#[macro_export]
macro_rules! ilog2 {
    ($x:expr) => {{
        let mut x = $x;
        assert!(x > 0);

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

////////////////////////////////////////////////////////////////////////////////
//// Function

pub fn e_sive(n: usize) -> ESive {
    let mut bits = BitVec::from_elem(n + 1, true);

    bits.set(0, false);
    bits.set(1, false);

    for i in 2..=n.isqrt() {
        if bits[i] {
            for j in (i * i..=n).step_by(i) {
                bits.set(j, false);
            }
        }
    }

    ESive { bits }
}

/// Space: O(\sqrt{n})
pub fn e_seg_sieve(n: usize) -> impl Iterator<Item = usize> {
    std::iter::from_coroutine(#[coroutine] move || {
        let delta = n.isqrt();

        let p = e_sive(delta).bits;

        for i in 1..=delta {
            if p[i] {
                yield i;
            }
        }

        let mut seg = BitVec::from_elem(delta+1, true);
        let mut l = delta;

        loop {
            for i in 1..=delta {
                if p[i] {
                    for j in (i - l % i..=delta).step_by(i) {
                        seg.set(j, false);
                    }
                }
            }

            for i in 1..=delta {
                if seg[i] {
                    if l + i > n {
                        return;
                    }

                    yield l + i;
                }
            }

            seg.set_all();
            l += delta;
        }
    })
}

pub fn wheel_prime() -> impl Iterator<Item = u64> + {
    std::iter::from_coroutine(#[coroutine] || {
        yield 2;
        yield 3;
        yield 5;
        yield 7;

        let mut ptr = 7u64;
        let mut po = 6u64;  // primorial
        let mut pnth = 3;
        let mut pris = vec![2, 3, 5, 7];
        let mut w_inc = vec![4, 2];

        // valid upper bound
        let mut bound = po.pow(2);


        while let Some(ptr_nxt) = ptr.checked_add(w_inc[(pnth+1) % w_inc.len()]) {
            // shift geers
            if ptr_nxt >= bound {
                po *= pris[pnth+1];
                bound = (po * pris[w_inc.len()]).pow(2);

                w_inc[0] = pris[w_inc.len()] - 1;
                w_inc.push(pris[w_inc.len()+1] - pris[w_inc.len()]);

                continue;
            }

            pnth += 1;
            ptr = ptr_nxt;

            if po < 1 << 32 {
                pris.push(ptr);
            }

            yield ptr;
        }

    })
}

////////////////////////////////////////////////////////////////////////////////
//// Structure

pub struct ESive {
    /// true for prime; false for composite
    bits: BitVec,
}


////////////////////////////////////////////////////////////////////////////////
//// Implementation


impl ESive {
    pub fn is_prime(&self, n: usize) -> bool {
        assert!(n < self.bits.len());

        self.bits[n]
    }

    pub fn iter(&self) -> impl Iterator<Item = usize> + '_ {
        std::iter::from_coroutine(#[coroutine] || {
            for (i, flag) in self.bits.iter().enumerate() {
                if flag {
                    yield i;
                }
            }
        })
    }

    pub fn into_iter(self) -> impl Iterator<Item = usize> {
        std::iter::from_coroutine(#[coroutine] || {
            for (i, flag) in self.bits.into_iter().enumerate() {
                if flag {
                    yield i;
                }
            }
        })
    }
}



#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::e_seg_sieve;
    use crate::number::{e_sive, wheel_prime};

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

    #[test]
    fn test_prime() {
        /* co-test */

        let n = 1690;

        let e_sive = e_sive(n);
        let e_seg_sive: HashSet<usize> = e_seg_sieve(n).collect();

        for i in 1..=n {
            assert_eq!(is_prime!(i; usize), e_sive.is_prime(i), "{i}");
            assert_eq!(is_prime!(i; usize), e_seg_sive.contains(&i), "{i}");
        }

        for (p0, v0) in e_sive.iter().zip(wheel_prime()) {
            assert_eq!(p0 as u64, v0);
        }
    }
}
