use std::{collections::HashMap, fmt::Debug, hash::Hash};

pub use MergeBy::*;

use crate::{get, set};


////////////////////////////////////////////////////////////////////////////////
//// Structure

/// Disjoint Set / Merge Find Set, merge by nodes size
///
/// T should be cheap to clone for example integer
#[derive(Debug, Clone)]
pub struct UnionFind<T> {
    paren: HashMap<T, T>,
    data: DSUData<T>,
    merge_by: Option<MergeBy>,
}


#[derive(Debug, Clone)]
enum DSUData<T> {
    SZ(HashMap<T, usize>),
    Empty,
}


#[derive(Debug, Clone)]
pub enum MergeBy {
    SZ,
}


////////////////////////////////////////////////////////////////////////////////
//// Implementation

impl<T> DSUData<T> {
    #[allow(unused)]
    fn as_sz(&self) -> &HashMap<T, usize> {
        match self {
            Self::SZ(sz) => sz,
            _ => unreachable!(),
        }
    }

    fn as_sz_mut(&mut self) -> &mut HashMap<T, usize> {
        match self {
            Self::SZ(sz) => sz,
            _ => unreachable!(),
        }
    }
}



impl<T> UnionFind<T> {
    pub fn new(merge_by: Option<MergeBy>) -> Self {
        let paren = HashMap::new();

        if let Some(by) = merge_by {
            match by {
                MergeBy::SZ => Self {
                    paren,
                    merge_by: Some(by),
                    data: DSUData::SZ(HashMap::new()),
                },
            }
        } else {
            Self {
                paren,
                merge_by: None,
                data: DSUData::Empty,
            }
        }
    }
}


impl<T> UnionFind<T>
where
    T: Clone + Hash + Eq + PartialEq + Debug,
{
    pub fn insert(&mut self, node: T) -> Option<T> {
        // set!(self.sz => node.clone() => 1);  // use get with default as instead
        set!(self.paren => node.clone() => node)
    }

    /// Pure find without path compressed
    pub fn pfind(&self, q: T) -> T {
        let r = get!(self.paren => q);

        if q == r {
            r
        } else {
            self.pfind(r)
        }
    }

    /// Find with path compression
    pub fn cfind(&mut self, q: T) -> T {
        let mut r = get!(self.paren => q);

        if q == r {
            r
        } else {
            r = self.cfind(r.clone());
            set!(self.paren => q => r.clone());
            r
        }
    }

    /// Link base merge startegy feeded when initialization
    ///
    /// default: y is linked to x
    pub fn punion(&mut self, x: T, y: T) {
        let mut x = self.pfind(x);
        let mut y = self.pfind(y);

        if x == y {
            return;
        }

        if let Some(ref by) = self.merge_by {
            match by {
                MergeBy::SZ => {
                    let sz = self.data.as_sz_mut();

                    let xsz = get!(*sz => x => 1);
                    let ysz = get!(*sz => y => 1);

                    // let x to be the root
                    if ysz < xsz {
                        (x, y) = (y, x);
                    }

                    set!(self.paren => y => x.clone());
                    set!(sz => x => xsz + ysz);
                }
            }
        } else {
            set!(self.paren => y => x.clone());
        }
    }

    /// Union same with punion except extra path compression
    pub fn cunion(&mut self, x: T, y: T) {
        let mut x = self.cfind(x);
        let mut y = self.cfind(y);

        if x == y {
            return;
        }

        if let Some(ref by) = self.merge_by {
            match by {
                MergeBy::SZ => {
                    let sz = self.data.as_sz_mut();

                    let xsz = get!(sz => x => 1);
                    let ysz = get!(sz => y => 1);

                    // let x to be the root
                    if ysz < xsz {
                        (x, y) = (y, x);
                    }

                    set!(self.paren => y => x.clone());
                    set!(sz => x => xsz + ysz);
                }
            }
        } else {
            set!(self.paren => y => x.clone());
        }
    }
}
