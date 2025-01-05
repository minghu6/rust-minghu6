
pub mod prime_sieves;

pub use prime_sieves::*;

////////////////////////////////////////////////////////////////////////////////
//// Macros

///
/// m, n shouldb e uint
///
#[macro_export]
macro_rules! gcd_rem {
    ($m:expr, $n:expr) => { {
        let mut m = $m;
        let mut n = $n;

        if m < n {
            (m, n) = (n, m);
        }

        if n % 2 != 0 {
            while m % 2 == 0 {
                m >>= 1;
            }
        }

        while n > 0 {
            (m, n) = (n, m % n)
        }

        m
    } };
}

#[macro_export]
macro_rules! gcd_sub {
    ($m:expr, $n:expr) => {{
        let mut m = $m;
        let mut n = $n;

        if m < n {
            (m, n) = (n, m);
        }

        if n > 0 {
            while m != n {
                m -= n;

                if m < n {
                    (m, n) = (n, m);
                }
            }
        }

        m
    }};
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
    ($m:expr, $n:expr) => {{
        let m = $m;
        let n = $n;

        if m == 0 || n == 0 {
            0
        } else {
            (m * n) / gcd_rem!(m, n)
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

#[macro_export]
macro_rules! is_perfect_square {
    ($n: expr) => {{
        let n = $n;

        n.isqrt().pow(2) == n
    }};
}

#[macro_export]
macro_rules! isqrt_ceil {
    ($n: expr) => {{
        let n = $n;

        let root = n.isqrt();

        if root * root < n {
            root + 1
        } else {
            root
        }
    }};
}


#[cfg(test)]
mod tests {

    #[test]
    fn test_gcd() {
        for i in 0..=1000 {
            for j in 0..=1000 {
                let r1 = gcd_rem!(i, j);
                let r2 = gcd_sub!(i, j);

                let (r4, x, y) = ext_gcd!(i, j);
                assert_eq!(x * i + y * j, r4);

                assert_eq!(r1, r2);
                assert_eq!(r4, r2);
            }
        }
    }
}
