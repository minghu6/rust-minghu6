use std::{cell::Cell, time::Duration};



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


macro_rules! anchor {
    ($name:ident) => {
        #[cfg(tprofile)]
        let $name = std::time::Instant::now();
    };
}


macro_rules! stats {
    ($label:ident, $from:ident, $end:ident) => {
        #[cfg(tprofile)]
        {
            // TPROFILE_STATS.with(|map_cell| {

            // });


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


pub(crate) use anchor;
pub(crate) use stats;


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

}
