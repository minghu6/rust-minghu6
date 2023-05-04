use std::iter::once;

use coll::{easycoll::M1, get};

use crate::{Graph, sp};


////////////////////////////////////////////////////////////////////////////////
//// Structure


pub struct Path<'a> {
    g: &'a Graph,
    path: Vec<usize>,
}


/// Freezed path
pub struct FPath {
    weight: isize,
    path: Vec<(usize, usize, isize)>,
}


////////////////////////////////////////////////////////////////////////////////
//// Implementation

impl FPath {
    pub fn iter<'a>(
        &'a self,
    ) -> impl Iterator<Item = (usize, usize, isize)> + 'a {
        self.path.iter().cloned()
    }

    #[inline]
    pub const fn weight(&self) -> isize {
        self.weight
    }
}


impl<'a> Path<'a> {
    pub fn new(g: &'a Graph, path: &[usize]) -> Self {
        Self {
            g,
            path: path.into_iter().cloned().collect(),
        }
    }

    pub fn from_cycle(g: &'a Graph, cycle: &[usize]) -> Self {
        Self {
            g,
            path: cycle.into_iter().cloned().chain(once(cycle[0])).collect(),
        }
    }

    pub fn from_pre(g: &'a Graph, dst: usize, pre: &M1<usize, usize>) -> Self {
        let path = sp::pre_to_path!(dst, pre);

        Self { g, path }
    }

    /// Empty path weight is zero
    pub fn weight(&self) -> isize {
        self.iter().map(|x| x.2).sum::<isize>()
    }

    pub fn iter<'b>(
        &'b self,
    ) -> impl Iterator<Item = (usize, usize, isize)> + 'b {
        std::iter::from_generator(|| {
            let mut cur = self.path[0];

            for v in self.path[1..].iter().cloned() {
                yield (cur, v, get!(self.g.w => (cur, v)));
                cur = v;
            }
        })
    }

    pub fn freeze(&self) -> FPath {
        FPath {
            weight: self.weight(),
            path: self.iter().collect(),
        }
    }
}
