
use std::io::{
    stdin,
    Error
};

use crate::etc::TrimInPlace;


#[macro_export]
macro_rules! print_flush {
    ( $($t:tt)* ) => {
        {
            use std::io::Write;
            use std::io::stdout;

            let mut out = stdout();
            write!(out, $($t)* ).unwrap();
            out.flush().unwrap();
        }
    }
}


pub fn promote_input(prom: &str) -> Result<String, Error> {
    print_flush!("{}", prom);

    let mut buf = String::new();
    stdin().read_line(&mut buf)?;

    buf.trim_in_place();
    Ok(buf)
}
