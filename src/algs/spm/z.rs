
pub fn compute_z_naive<'a>(pat: &'a [u8]) -> Vec<usize> {
    let patlen = pat.len();
    let mut vec_z = vec![0; patlen];

    for i in 1..patlen {
        while i + vec_z[i] < patlen && pat[vec_z[i]] == pat[i+vec_z[i]] {
            vec_z[i] += 1;
        }
    }

    vec_z
}


#[cfg(test)]
mod tests {
    use super::super::super::super::test::spm;
    use super::*;

    #[test]
    fn get_correct_z_function_static_check() {
        let mut pat = "aaaaa";
        assert_eq!(compute_z_naive(pat.as_bytes()), vec![0, 4, 3, 2, 1]);

        pat = "aaabaab";
        assert_eq!(compute_z_naive(pat.as_bytes()), vec![0, 2, 1, 0, 2, 1, 0]);

        pat = "abacaba";
        assert_eq!(compute_z_naive(pat.as_bytes()), vec![0, 0, 1, 0, 3, 0, 1]);
    }

    #[test]
    fn get_correct_z_function_dyn_check() {

    }
}