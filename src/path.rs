use std::{
    collections::HashSet,
    ffi::OsString,
    fs::{read_dir, DirEntry},
    path::Path,
    rc::Rc,
};


////////////////////////////////////////////////////////////////////////////////
//// Trait

pub trait Exclude = Fn(&Path) -> bool;

////////////////////////////////////////////////////////////////////////////////
//// Structures

#[derive(Clone)]
pub struct FindOptions<'a> {
    pub pre_exclude_opt: Option<Rc<dyn Fn(&Path) -> bool + 'a>>,
    pub post_include_ext_opt: Option<Rc<HashSet<OsString>>>,
    pub exclude_dot: bool,
    pub recursive: bool,
}

pub struct Walk<'a> {
    stack: Vec<DirEntry>,
    opt: FindOptions<'a>,
}

////////////////////////////////////////////////////////////////////////////////
//// Implementations

impl<'a> FindOptions<'a> {
    pub fn with_pre_exclude<F: Exclude + 'a>(mut self, f: F) -> Self {
        self.pre_exclude_opt = Some(Rc::new(f));
        self
    }

    pub fn with_post_include_ext<S: AsRef<str>>(
        mut self,
        includes: &[S],
    ) -> Self {
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
        } else {
            debug_assert!(path.is_file());

            if let Some(ref post_include_opt) = self.post_include_ext_opt {
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
            recursive: true,
        }
    }
}


impl<'a> Walk<'a> {
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
            opt,
        }
    }
}

impl<'a> Iterator for Walk<'a> {
    type Item = DirEntry;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(entry) = self.stack.pop() {
            let path = entry.path();

            if self.opt.verify(&path) {
                if path.is_dir() {
                    if let Ok(read_dir) = read_dir(path) {
                        self.stack.extend(read_dir.into_iter().flatten())
                    };
                } else {
                    return Some(entry);
                }
            }
        }
        None
    }
}


////////////////////////////////////////////////////////////////////////////////
//// Functions

pub fn is_dot_file<P: AsRef<Path>>(p: P) -> bool {
    if let Some(name) = p.as_ref().file_name() {
        name.to_string_lossy().starts_with(".")
    } else {
        false
    }
}


/// Almost 3 time faster than py ver. impl. with release opt.
/// on a rough bench test.
///
/// If using `print`, python would be slowed down to 10 times slower than it.
pub fn syn_walk<'a, P: AsRef<Path>>(startdir: P) -> Walk<'a> {
    Walk {
        stack: read_dir(startdir)
            .into_iter()
            .flatten()
            .map(|result_entry| result_entry.into_iter())
            .flatten()
            .collect::<Vec<_>>(),
        opt: FindOptions::default(),
    }
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
            .pre_exclude(exclude)
            .post_include_ext(&[".c", ".h", ".rs", ".toml"])
        {
            println!("{}", p.path().to_str().unwrap());
        }
    }
}
