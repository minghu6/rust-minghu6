use std::{
    collections::HashSet,
    ffi::OsString,
    fs::DirEntry,
    path::{Path, PathBuf},
    rc::Rc,
};

use common::{ Itertools, error_code::* };


////////////////////////////////////////////////////////////////////////////////
//// Macro

/// Map inner error into ErrorCode
#[macro_export]
macro_rules! read_dir_wrapper {
    ($path: expr) => {{
        let path = $path;
        match std::fs::read_dir(path) {
            Ok(iter) => Ok(iter.map(|res| match res {
                Ok(entry) => Ok(entry),
                Err(err) => Err(ErrorCode::IterDirEntry(err)),
            })),
            Err(err) => Err(ErrorCode::ReadDir(err)),
        }
    }};
}

////////////////////////////////////////////////////////////////////////////////
//// Trait

pub trait Exclude = Fn(&Path) -> bool;

////////////////////////////////////////////////////////////////////////////////
//// Structure

#[derive(Clone)]
pub struct FindOptions<'a> {
    pub pre_exclude_opt: Option<Rc<dyn Fn(&Path) -> bool + 'a>>,
    pub post_include_ext_opt: Option<Rc<HashSet<OsString>>>,
    pub exclude_dot: bool,
    pub recursive: bool
}


pub struct SynWalk<'a> {
    stack: Vec<Result<DirEntry>>,
    opt: FindOptions<'a>,
}


/// No performance benefit just for test and verification.
pub struct ASynCollect<'a, P: AsRef<Path>> {
    startdir: P,
    opt: FindOptions<'a>,
}


////////////////////////////////////////////////////////////////////////////////
//// Implementation

impl<'a> FindOptions<'a> {
    pub fn with_pre_exclude<F: Exclude + 'a>(mut self, f: F) -> Self {
        self.pre_exclude_opt = Some(Rc::new(f));
        self
    }

    pub fn with_post_include_ext<S: AsRef<str>>(mut self, includes: &[S]) -> Self {
        let post_include_ext =
            Rc::new(HashSet::from_iter(includes.into_iter().map(|s| {
                let s = s.as_ref();
                if s.starts_with(".") {
                    OsString::from(&s[1..])
                } else {
                    OsString::from(&s[..])
                }
            })));

        self.post_include_ext_opt = Some(post_include_ext);
        self
    }

    pub fn recursive(mut self, enable: bool) -> Self {
        self.recursive = enable;
        self
    }

    pub fn verify<P: AsRef<Path>>(&self, p: P) -> bool {
        let path = p.as_ref();

        if path.is_symlink() {
            // Note: path may be both symlink and dir
            return false;
        }

        if path.is_dir() {
            if !self.recursive {
                return false;
            }

            if let Some(ref exclude) = self.pre_exclude_opt {
                if exclude(path) {
                    return false;
                }
            }
            if self.exclude_dot && is_dot_file(path) {
                return false;
            }

            true
        }
        else {
            debug_assert!(path.is_file());

            if let Some(ref post_include_opt) =
                self.post_include_ext_opt
            {
                if let Some(osstr) = path.extension() {
                    if !post_include_opt.contains(osstr) {
                        return false;
                    }
                } else {
                    return false;
                }
            }
            if self.exclude_dot && is_dot_file(&path) {
                return false;
            }

            true
        }

    }

}

impl<'a> Default for FindOptions<'a> {
    fn default() -> Self {
        Self {
            pre_exclude_opt: Default::default(),
            post_include_ext_opt: Default::default(),
            exclude_dot: true,
            recursive: true
        }
    }
}


impl<'a> SynWalk<'a> {
    pub fn pre_exclude<F: Exclude + 'a>(self, f: F) -> Self {
        Self {
            stack: self.stack,
            opt: self.opt.with_pre_exclude(f),
        }
    }

    pub fn post_include_ext<S: AsRef<str>>(self, includes: &[S]) -> Self {
        Self {
            stack: self.stack,
            opt: self.opt.with_post_include_ext(includes),
        }
    }

    pub fn recursive(self, enable: bool) -> Self {
        Self {
            stack: self.stack,
            opt: self.opt.recursive(enable),
        }
    }

    pub fn with_opt(self, opt: FindOptions<'a>) -> Self {
        Self {
            stack: self.stack,
            opt
        }
    }
}

impl<'a> Iterator for SynWalk<'a> {
    type Item = Result<DirEntry>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(res_entry) = self.stack.pop() {
            if let Ok(entry) = res_entry {
                let path = entry.path();

                if self.opt.verify(&path) {
                    if path.is_dir() {
                        match read_dir_wrapper!(path) {
                            Ok(iter) => {
                                self.stack.extend(iter);
                            }
                            Err(err) => {
                                return Some(Err(err));
                            }
                        }
                    }
                    else {
                        return Some(Ok(entry));
                    }

                }
            } else {
                // Some(Err)
                return Some(res_entry);
            }
        }
        None
    }
}


impl<'a, P: AsRef<Path>> ASynCollect<'a, P> {
    pub fn new(startdir: P) -> Self {
        Self {
            startdir,
            opt: FindOptions::default()
        }
    }

    pub fn pre_exclude<F: Exclude + 'a>(self, f: F) -> Self {
        Self {
            startdir: self.startdir,
            opt: self.opt.with_pre_exclude(f)
        }
    }

    pub fn post_include_ext<S: AsRef<str>>(self, includes: &[S]) -> Self {
        Self {
            startdir: self.startdir,
            opt: self.opt.with_post_include_ext(includes)
        }
    }

    pub async fn collect(&self) -> Result<Vec<Result<PathBuf>>> {
        let mut res = vec![];
        let mut subdirs = vec![];

        for entry_res in read_dir_wrapper!(&self.startdir)? {
            if let Ok(entry) = entry_res {
                let path = entry.path();

                if self.opt.verify(&path) {
                    if path.is_dir() {
                        subdirs.push(path);
                    }
                    else if path.is_file() {
                        res.push(Ok(path));
                    }
                    else {
                        res.push(Err(ErrorCode::IrregularFile(path)));
                    }
                }

            } else {
                res.push(Err(entry_res.unwrap_err()));
            }
        }

        let n = 4;
        let slice = subdirs.len() / n;

        let mut tasks;
        if slice > 0 {
            tasks = Vec::with_capacity(n);
            let mut remains = subdirs.len() % n;
            let mut lo = 0;

            for _ in 0..n {
                let hi = if remains > 0 {
                    remains -= 1;
                    lo + slice + 1
                } else {
                    lo + slice
                };

                tasks.push(&subdirs[lo..hi]);
                lo = hi
            }
        } else {
            tasks = Vec::with_capacity(subdirs.len());
            for i in 0..subdirs.len() {
                tasks.push(&subdirs[i..i + 1]);
            }
        }

        // stub
        // println!("res: {res:#?}");
        // println!("tasks: {tasks:#?}");

        let mut collect = vec![];
        for task in tasks.into_iter() {
            collect.push(_asyn_once_collect(
                task,
                self.opt.clone(),
            ));
        }

        for furure in collect.into_iter() {
            let subvec = furure.await?;
            res.extend(subvec);
        }

        Ok(res)
    }

}


////////////////////////////////////////////////////////////////////////////////
//// Function

pub fn is_dot_file<P: AsRef<Path>>(p: P) -> bool {
    if let Some(name) = p.as_ref().file_name() {
        name.to_string_lossy().starts_with(".")
    }
    else {
        false
    }
}


/// Almost 3 time faster than py ver. impl. with release opt.
/// on a rough bench test.
///
/// If using `print`, python would be slowed down to 10 times slower than it.
pub fn syn_walk<'a, P: AsRef<Path>>(startdir: P) -> Result<SynWalk<'a>> {
    Ok(SynWalk {
        stack: Vec::from_iter(read_dir_wrapper!(startdir)?),
        opt: FindOptions::default(),
    })
}

async fn _asyn_once_collect<'a, P: AsRef<Path>>(
    dirs: &[P],
    opt: FindOptions<'a>,
) -> Result<Vec<Result<PathBuf>>> {
    let mut res = vec![];
    for startdir in dirs.into_iter() {
        res.extend(
            SynWalk {
                stack: Vec::from_iter(read_dir_wrapper!(startdir)?),
                opt: opt.clone()
            }
            .map_ok(|x| x.path())
            .collect_vec(),
        );
    }
    Ok(res)
}


#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::syn_walk;


    #[test]
    fn test_walk() {
        let exclude = |p: &Path| {
            if p.to_str().unwrap() == "." {
                return false;
            }

            if p.file_name().is_none() {
                return true;
            }
            let fname = p.file_name().unwrap().to_str().unwrap();

            fname.starts_with(".") || fname == "target"
        };

        for p in syn_walk(".")
            .unwrap()
            .pre_exclude(exclude)
            .post_include_ext(&[".c", ".h", ".rs", ".toml"])
        {
            let path = p.unwrap();


            println!("{}", path.path().to_str().unwrap());
        }
    }
}
