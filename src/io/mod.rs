
use std::io::{
    stdin,
    Error
};



fn promote_input(prom: &str) -> Result<String, Error> {
    print!("{}", prom);
    let mut buf = String::new();
    stdin().read_line(&mut buf)?;

    Ok(buf)
}
