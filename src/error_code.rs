use std::{ffi::OsString, path::PathBuf};

use proc_macros::make_simple_error_rules;

make_simple_error_rules!(ValidateFailedError);

/// United Error Code
#[derive(Debug)]
pub enum ErrorCode {
    ReadDir(std::io::Error),
    IterDirEntry(std::io::Error),
    AbsolutePath(std::io::Error),
    Open(std::io::Error),

    InvalidUnicodeOSStr(OsString),
    IrregularFile(PathBuf)
}

pub type Result<T> = std::result::Result<T, ErrorCode>;
