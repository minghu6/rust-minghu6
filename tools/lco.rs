use std::ffi::OsStr;

use coll::Itertools;
use walkdir::WalkDir;




fn main() -> Result<(), std::io::Error> {

    let mut cnt = 0;
    for entry in WalkDir::new(".")
    .into_iter()
    .filter_entry(|entry| {
        let p = entry.path();
        if p.is_dir() && !p.is_symlink() {
            return true;
        }
        if p.is_file() {
            if let Some(ext) = p.extension() {
                if ext == OsStr::new("c") || ext == OsStr::new("h") {
                    return true;
                }
            }
        }
        false
    })
    .collect_vec()
    {

        let p = entry?.into_path();
        if p.is_dir() || p.is_symlink() {
            continue;
        }

        cnt += 1;
        // println!("{}", p.as_os_str().to_string_lossy())
    }

    println!("total {cnt}");
    Ok(())
}
