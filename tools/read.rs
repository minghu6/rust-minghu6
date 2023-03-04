#![feature(const_option_ext)]
#![allow(unused_imports)]

use std::{
    fs::{File, read},
    io::{BufRead, BufReader, Read},
    path::{Path, PathBuf},
};

use clap::Parser;

use minghu6::error_code::*;

#[derive(Parser)]
#[clap()]
struct Args {
    /// extensions specified
    #[clap(short='i')]
    include_extensions_opt: Option<Vec<String>>,

    #[clap(default_value=".")]
    target_dir: PathBuf
}



fn main() -> Result<()> {
    let _args = Args::parse();


    Ok(())
}
