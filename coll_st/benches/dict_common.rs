
#[macro_export]
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

