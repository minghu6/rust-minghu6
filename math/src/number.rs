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
    ($m:expr,$n:expr) => {
        {
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
        }
    };
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
        }
        else {
            for i in m..=m*n {
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
        }
        else {
            (m * n) / gcd!($ty|m, n)
        }
    }};
}



#[cfg(test)]
mod tests {

    #[test]
    fn test_gcd() {
        for i in 0..=1000 {
            for j in 0..=1000 {
                let r1 = gcd!(mod| i, j);
                let r2 = gcd!(sub| i, j);
                let r3 = gcd!(i32| i, j);

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
                let r1 = lcm!(i32|i, j);
                let r2 = lcm!(brute| i, j);

                assert_eq!(r1, r2);
            }
        }
    }
}
