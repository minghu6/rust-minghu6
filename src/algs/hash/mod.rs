


#[cfg(test)]
mod tests {

    use crate::test::hash::test_hash_collision_randint;
    use twox_hash::XxHash64;

    #[test]
    fn test_hash_collision() {

        test_hash_collision_randint(|| box XxHash64::default())

    }

}
