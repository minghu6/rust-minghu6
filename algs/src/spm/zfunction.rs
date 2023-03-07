use std::cmp::min;

pub fn compute_z_function_naive<'a>(pat: &'a [u8]) -> Vec<usize> {
    let patlen = pat.len();
    let mut vec_z = vec![0; patlen];

    for i in 1..patlen {
        while i + vec_z[i] < patlen && pat[vec_z[i]] == pat[i+vec_z[i]] {
            vec_z[i] += 1;
        }
    }

    vec_z
}

pub fn compute_z_function_improved<'a>(pat: &'a[u8]) -> Vec<usize> {
    let patlen = pat.len();
    let mut vec_z = vec![0; patlen];

    let mut l = 0;
    let mut r = 0;
    for i in 1..patlen {
        if i <= r {
            vec_z[i] = min(r-i+1, vec_z[i-l]);
        }
        while i + vec_z[i] < patlen && pat[vec_z[i]] == pat[i+vec_z[i]] {
            vec_z[i] += 1;
        }
        if i + vec_z[i] - 1 > r {
            l = i;
            r = i + vec_z[i] - 1;
        }
    }

    vec_z
}

pub fn compute_z_function<'a>(pat: &'a[u8]) -> Vec<usize> {
    compute_z_function_improved(pat)
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_correct_z_function_static_check() {
        let mut pat = "aaaaa";
        let mut res = vec![0, 4, 3, 2, 1];
        assert_eq!(compute_z_function_naive(pat.as_bytes()), res);
        assert_eq!(compute_z_function_improved(pat.as_bytes()), res);

        pat = "aaabaab";
        res = vec![0, 2, 1, 0, 2, 1, 0];
        assert_eq!(compute_z_function_naive(pat.as_bytes()), res);
        assert_eq!(compute_z_function_improved(pat.as_bytes()), res);

        pat = "abacaba";
        res = vec![0, 0, 1, 0, 3, 0, 1];
        assert_eq!(compute_z_function_naive(pat.as_bytes()), res);
        assert_eq!(compute_z_function_improved(pat.as_bytes()), res);

        pat = "aaaaba";
        res = vec![0, 3, 2, 1, 0, 1];
        assert_eq!(compute_z_function_naive(pat.as_bytes()), res);
        assert_eq!(compute_z_function_improved(pat.as_bytes()), res);
    }

    #[test]
    fn get_correct_z_function_dyn_check() {

    }
}