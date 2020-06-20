pub mod spm;



#[cfg(
    all(
        any(target_arch = "x86", target_arch = "x86_64"),
    )
)]
pub fn hardware_randvalue() -> usize {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        #[cfg(target_arch="x86")]
        use core::arch::x86::_rdrand32_step as _rdrandsize_step;

        #[cfg(target_arch="x86_64")]
        use core::arch::x86_64::_rdrand64_step as _rdrandsize_step;

        let mut rand_value = 0;
        unsafe {
            match _rdrandsize_step(&mut rand_value) {
                1 => {
                   return rand_value as usize;
                },
                _ => assert!(false),
            }
        }
    }

    0
}



#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn hardware_randvalue_works() {
        let times = 100;
        let mut result = Vec::with_capacity(times);

        for _ in 0..times {
            result.push(hardware_randvalue());
        }

        assert_ne!(result, vec![0;times]);
    }
}