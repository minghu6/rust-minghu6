

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

    // ($($exprs:tt)*) => {
    //     {
    //         bench!(5, $($exprs)*);
    //     }
    // };

}


pub fn duration_collector() {

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
