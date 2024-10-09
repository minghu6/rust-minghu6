

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    #[test]
    fn test_int() {
        let a0 = -12i8;

        println!("{:08b}", (a0 as u8 -1) ^ u8::MAX);
        println!("{:08b}", 0b1000_0000u8 as i8);
    }
}
