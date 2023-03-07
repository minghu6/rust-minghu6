use std::fmt::Write;



pub fn strshift<T: ToString>(it: T, pad: &str) -> String {
    let mut cache = String::new();

    write!(cache, "{}", it.to_string()).unwrap();

    let mut res = vec![];
    for line in cache.split('\n') {
        res.push(pad.to_string() + line);
    }

    res.join("\n")
}


pub fn normalize<T: Ord>(raw_data: &[T]) -> Vec<usize> {
    if raw_data.is_empty() {
        return vec![];
    }

    let mut res = vec![0; raw_data.len()];
    let mut taged: Vec<(usize, &T)> = raw_data
        .into_iter()
        .enumerate()
        .collect();

    taged.sort_by_key(|x| x.1);

    let mut rank = 1;
    let mut iter = taged.into_iter();
    let (i, mut prev) = iter.next().unwrap();
    res[i] = rank;

    for (i, v) in iter {
        if v != prev {
            rank += 1;
        }
        prev = v;
        res[i] = rank;
    }

    res
}


pub fn gen() -> impl FnMut() -> usize {
    let mut _inner = 0;
    move || {
        let old = _inner;
        _inner += 1;
        old
    }
}


pub fn gen_unique() -> impl FnMut() -> usize {
    let mut set = std::collections::HashSet::new();

    move || {
        let mut v = crate::random();

        while set.contains(&v) {
            v = crate::random();
        }

        set.insert(v);

        v
    }
}

////////////////////////////////////////////////////////////////////////////////
//// Declare Macro

#[macro_export]
macro_rules! ht {
    ( $head_expr:expr, $tail_expr:expr ) => {
        {
            let head = $head_expr;
            let tail = $tail_expr;

            let mut _vec = vec![head];
            _vec.extend(tail.iter().cloned());
            _vec
        }
    };
    ( $head:expr) => {
        {
            ht!($head, vec![])
        }
    };

}


macro_rules! def_coll_init {
    (seq | $name:ident, $new:expr, $push:ident) => {
        #[macro_export]
        macro_rules! $name {
            ( $$($value:expr),* ) => {{
                #[allow(unused_mut)]
                let mut _coll = $new;

                $$(
                    _coll.$push($value);
                )*

                _coll
            }};
        }
        #[allow(unused)]
        pub(crate) use $name;
    };
    (map | $name:ident, $new:expr) => {
        #[macro_export]
        macro_rules! $name {
            ( $$($k:expr => $v:expr),* $$(,)? ) => {{
                let mut _coll = $new;

                $$(
                    _coll.insert($k, $v);
                )*

                _coll
            }};
        }
        #[allow(unused)]
        pub(crate) use $name;
    };
}

def_coll_init!(seq | vecdeq, std::collections::VecDeque::new(), push_back);
def_coll_init!(seq | hashset, std::collections::HashSet::new(), insert);
def_coll_init!(map | hashmap, std::collections::HashMap::new());
def_coll_init!(map | btreemap, std::collections::BTreeMap::new());



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gen_unique() {

        let mut unique = gen_unique();

        for _ in 0..1000 {
            unique();
        }
    }
}
