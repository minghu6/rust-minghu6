use std::{cell::Cell, fmt::Debug, time::{Duration, Instant}};



#[macro_export]
macro_rules! elapsed {
    ($($exprs:tt)*) => {
        #[allow(dead_code)]
        {
            use std::time::Instant;

            let now = Instant::now();

            $($exprs)*

            now.elapsed()
        }
    };
}


#[macro_export]
macro_rules! bench {
    ($label:expr, $($exprs:tt)*) => {
        {
            use num_format::{ Locale, ToFormattedString };

            use std::time::Duration;

            let label = $label;
            print!("{: <35}", label);

            let durs: Vec<Duration> =
            (0..3)
            .into_iter()
            .map(|_| {

                elapsed! { $($exprs)* }

            })
            .collect();

            let dur_max = durs.iter().max().unwrap().clone();
            let dur_min = durs.iter().min().unwrap().clone();

            let interval = (dur_max - dur_min);
            let ave: Duration = durs.iter().sum();

            println!(
                "{} us/iter  (+/- {})",
                ave.as_micros().to_formatted_string(&Locale::en),
                interval.as_micros().to_formatted_string(&Locale::en)
            );
        }
    };
}


#[macro_export]
macro_rules! anchor {
    ($name:ident) => {
        #[cfg(tprofile)]
        let $name = std::time::Instant::now();
    };
}


#[macro_export]
macro_rules! stats {
    ($label:ident, $from:ident, $end:ident) => {
        #[cfg(tprofile)]
        {
            let d = $end.duration_since($from);
            let key = stringify!($label);

            let mut map = TPROFILE_STATS.take();

            // $crate::set!(map => key =>
            //     $crate::get!(map => key => std::time::Duration::ZERO) + d
            // );

            map.insert(
                key,
                map.get(&key).cloned().unwrap_or(std::time::Duration::ZERO) + d
            );

            TPROFILE_STATS.set(map);
        }
    };
}

#[macro_export]
macro_rules! def_stats {
    ($struct_name:ident, { $($field:ident),+ }) => {
        use $crate::timeit::Watch_;

        #[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
        struct $struct_name {
            $($field: Watch_),+
        }

        impl $struct_name {
            fn new() -> Self {
                Self {
                    $($field: Watch_::new()),+
                }
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Watch_ {
    total: Duration,
    now_: Instant
}

impl Watch_ {
    pub fn new() -> Self {
        Self { total: Duration::ZERO, now_: Instant::now() }
    }

    pub fn s(&mut self) {
        self.now_ = Instant::now();
    }

    pub fn e(&mut self) {
        self.total += self.now_.elapsed();
    }
}

impl Debug for Watch_ {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            f.write_fmt(format_args!("{:#?}", self.total))
        }
        else {
            f.write_fmt(format_args!("{:?}", self.total))
        }
    }
}


thread_local! {
    pub static TPROFILE_STATS: Cell<std::collections::HashMap<&'static str, Duration>>
        = Cell::new(std::collections::HashMap::new());
}




#[cfg(test)]
mod tests {

    #[test]
    fn test_elapsed() {

        let now = elapsed! {

            let mut vec = vec![];

            for i in 0..1000000 {
                vec.push(i);
            }

        };


        println!("now: {} ms", now.as_millis())
    }

    #[test]
    fn test_def_stats() {

        def_stats!(Watch, { a, b, c });

        let mut watch = Watch::new();

        watch.a.s();

        for i in 0..1000 {
            std::hint::black_box(i * i);
        }

        watch.a.e();

        println!("{watch:#?}");
    }
}
