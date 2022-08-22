use std::path::Path;

use futures::executor::block_on;
use minghu6::{error_code::*, etc::path::ASynCollect};

pub fn count_lines_dir<P: AsRef<Path>>(_path: P) -> Result<usize> {

    Ok(0)
}


#[allow(unused)]
fn main() -> Result<()>{

    let mut cnt = 0;
    // for entry in syn_walk(".")?.post_include_ext(&[".c", ".h"]) {
    //     let p = entry?.path();
    //     cnt += 1;
    //     // println!("{}", p.as_os_str().to_string_lossy())
    // }
    for entry in block_on(
        ASynCollect::new(".")
            .post_include_ext(&[".c", ".h"])
            .collect()
        )?
    {
        let p = entry?;
        cnt += 1;
        // println!("{}", p.as_os_str().to_string_lossy())
    }
    println!("total {cnt}");
    Ok(())
}

