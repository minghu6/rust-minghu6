use std::cmp::Ordering;

use clap::Parser;
use minghu6::io::promote_input;
use rand::random;
use shadow_rs::shadow;


fn guess_num(to: usize) {
    let secret = random::<usize>() % to;

    loop {
        let res = promote_input("input an uint: ").unwrap();
        if let Ok(res_num) = usize::from_str_radix(&res, 10) {
            match res_num.cmp(&secret) {
                Ordering::Less => println!("Less"),
                Ordering::Equal => {
                    println!("BingGo!");
                    break;
                }
                Ordering::Greater => println!("Greater"),
            }
        } else {
            println!("Invalid Input!")
        }
    }
}

shadow!(build);

#[derive(Debug, clap::Parser)]
#[clap(author, version=build::PKG_VERSION, about)]
struct Args {
    /// Secret Upper Bound
    #[clap(default_value_t = 100)]
    to: usize,

    /// Print Verbose Version
    #[clap(short='V')]
    verbose_version: bool
}


fn main() {
    shadow!(build);

    let args = Args::parse();

    if args.verbose_version {

        println!("{}", build::version());

        return;
    }

    let to = args.to;

    guess_num(to);
}
