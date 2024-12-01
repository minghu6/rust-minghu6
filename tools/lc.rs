#![feature(const_option_ext)]
#![allow(unused_imports)]

use std::{
    fs::{read, read_dir, File},
    io::{self, BufRead, BufReader, Read},
    path::{Path, PathBuf},
};

use minghu6::path::{syn_walk, FindOptions};

use clap::Parser;

// #[inline]
// pub fn count_lines_file<P: AsRef<Path>>(path: P) -> Result<usize> {
//     let mut cnt = 0


// }

const NEWLINE_CODE: u8 = b'\n';

pub struct Cnt {
    cnt: usize,
    path: PathBuf,
    files: usize
}

pub fn count_lines_dir<P: AsRef<Path>>(path: P, opt: FindOptions) -> io::Result<Cnt> {
    let mut cnt = 0;
    let mut files = 0;

    for path in syn_walk(&path).with_opt(opt) {
        let path = path.path();
        // let file =
        //     File::open(path).map_err(|err| ErrorCode::Open(err))?;
        // let lines = BufReader::new(file).lines().count();

        let bytes = read(path)?;
        let lines = bytes
        .into_iter()
        .filter(|c| *c == NEWLINE_CODE)
        .count();

        cnt += lines;
        files += 1;
    }

    Ok(Cnt {
        cnt,
        path: path.as_ref().to_owned(),
        files,
    })

}


/// count lines for depth 1, used for counting root dir
pub fn count_lines_dir_d1<P: AsRef<Path>>(path: P, opt: FindOptions) -> io::Result<Cnt> {
    let mut cnt = 0;
    let mut files = 0;

    for result_entry_res in read_dir(path.as_ref())? {
        let p = result_entry_res?.path();

        if !p.is_dir() && opt.verify(&p) {
            let bytes = read(p)?;
            let lines = bytes
            .into_iter()
            .filter(|c| *c == NEWLINE_CODE)
            .count();

            cnt += lines;
            files += 1;
        }
    }

    Ok(Cnt {
        cnt,
        path: path.as_ref().to_owned(),
        files,
    })
}


fn stats(cnts: Vec<Cnt>) {
    let total: usize = cnts
    .iter()
    .map(|cnt|cnt.cnt)
    .sum();

    let files: usize = cnts
    .iter()
    .map(|cnt|cnt.files)
    .sum();

    let max_fn_len = 40;
    let fn_padding = cnts
    .iter()
    .map(|cnt| cnt.path.as_os_str().len())
    .filter(|len| *len < max_fn_len)
    .max()
    .unwrap();

    let cnt_padding = cnts
    .iter()
    .map(|cnt| cnt.cnt)
    .max()
    .unwrap()
    .to_string()
    .len();

    println!("counting: {files}");
    println!("   total: {total}");
    println!();

    for Cnt { cnt, path, files: _ } in cnts.into_iter() {
        let percent = (cnt * 100) as f64 / total as f64;
        let percent_s = if percent < 0.01 {
            "<0.01".to_owned()
        }
        else {
            format!("{percent:05.2}")
        };

        println!("{:<fn_padding$} {} {:>cnt_padding$}", path.as_os_str().to_string_lossy(), percent_s, cnt)
    }

}


/// Example: Sort by line count: BIN -i=.h -i=.c | sed '1,2d' | sort -k2
/// About 2.5 times faster than Python version
#[derive(Parser)]
#[clap()]
struct Args {
    /// extensions specified
    #[clap(short='i')]
    include_extensions_opt: Option<Vec<String>>,

    #[clap(default_value=".")]
    target_dir: PathBuf
}


fn main() -> io::Result<()> {
    let args = Args::parse();
    let mut opt = FindOptions::default();

    if let Some(exts) = args.include_extensions_opt {
        opt = opt.with_post_include_ext(&exts[..])
    }

    let target_dir = args.target_dir;

    let mut topdirs = vec![];
    for result_entry in read_dir(&target_dir)? {
        let path = result_entry?.path();

        if path.is_dir() && opt.verify(&path) {
            topdirs.push(path);
        }
    }

    let mut cnts = Vec::with_capacity(topdirs.len());
    for p in topdirs.into_iter() {
        cnts.push(count_lines_dir(p, opt.clone())?);
    }
    let mut topcnt = count_lines_dir_d1(&target_dir, opt)?;
    topcnt.path = PathBuf::from("<.>");
    cnts.push(topcnt);

    stats(cnts);

    Ok(())
}
