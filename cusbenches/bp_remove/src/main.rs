
use minghu6::collections::bt2::bpt::*;
use rand::{prelude::SliceRandom, thread_rng};
use lazy_static::lazy_static;


macro_rules! gen_data {
    ($get_one: ident, $group: expr, $num: expr) => {{
        let group = $group;
        let num = $num;

        let mut keys = std::collections::HashSet::new();
        let mut elems = vec![];

        for _ in 0..num {
            let mut k = $get_one();
            let mut j = 0;

            while j < group {
                k += 1;
                if keys.contains(&k) {
                    continue;
                }

                keys.insert(k);
                elems.push((k, k + 1000));

                j += 1;
            }
        }

        elems
    }};
}

lazy_static! {
    static ref INSERT_DATA: Vec<(u64, u64)> = {
        let get_one = || rand::random::<u64>();

        gen_data!(get_one, 50, 2_000)
    };
    static ref KEYS: Vec<u64> = {
        let mut keys = vec![];

        for (k, _) in INSERT_DATA.iter() {
            keys.push(k.clone());
        }

        keys.shuffle(&mut thread_rng());

        keys
    };
}


fn main() {


}
