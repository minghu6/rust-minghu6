#[cfg(test)]
mod tests {

    use std::{
        collections::HashSet,
        hash::{Hash, Hasher},
    };

    use common::random;
    use twox_hash::XxHash64;

    pub fn test_hash_collision_randint<'a>(
        get_hasher: impl Fn() -> Box<dyn Hasher + 'a>,
    ) {
        let mut hasher = get_hasher();

        let mut one_turn = |m: u64| -> usize {
            let mut coll = HashSet::new();
            let mut cnt = 0usize;

            for _i in 0..m {
                let key = random::<usize>();
                key.hash(&mut hasher);
                let mut code = hasher.finish() % m;

                while coll.contains(&code) {
                    cnt += 1;
                    code = (code + 1) % m;
                }

                coll.insert(code);
            }

            cnt
        };

        let mut average_test = |turns: u64, m: u64| {
            let mut total = 0;
            for _ in 0..turns {
                total += one_turn(m);
            }
            println!("{}-int collision: {}", m, total as f64 / turns as f64);
        };

        average_test(10, 5);
        average_test(10, 20);
        average_test(10, 50);
        average_test(10, 100);
        average_test(10, 500);
        average_test(10, 1000);
        average_test(10, 10000);
    }

    #[test]
    fn test_hash_collision() {
        test_hash_collision_randint(|| box XxHash64::default())
    }
}
