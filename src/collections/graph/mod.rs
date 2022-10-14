pub mod tree;


use self::tree::diameter::diameter_dp;

use super::easycoll::{M1, MV};
use crate::{set, apush};



////////////////////////////////////////////////////////////////////////////////
//// Structure

/// Adjacent link formed simple (directed) connected graph
#[derive(Default, Debug)]
pub struct Graph {
    pub e: MV<usize, usize>,
    pub w: M1<(usize, usize), isize>,
}


////////////////////////////////////////////////////////////////////////////////
//// Implementation

impl FromIterator<(usize, usize, isize)> for Graph {
    fn from_iter<T: IntoIterator<Item = (usize, usize, isize)>>(
        iter: T,
    ) -> Self {
        let mut e = MV::new();
        let mut w = M1::new();

        for (u, v, _w) in iter {
            apush!(e => u => v);
            set!(w => (u, v) => _w);
        }

        Self { e, w }
    }
}


impl Graph {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn anypoint(&self) -> usize {
        *self.e.0.keys().next().unwrap()
    }

    /// The length of the shortest path between the most distanced nodes.
    pub fn diameter(&self) -> isize {
        diameter_dp(self)
    }

    pub fn vertexs(&self) -> impl Iterator<Item=&usize> {
        self.e.0.keys()
    }
}



////////////////////////////////////////////////////////////////////////////////
//// Function

pub fn to_undirected_vec<T: IntoIterator<Item = (usize, usize, isize)>>(
    iter: T,
) -> Vec<(usize, usize, isize)> {
    let mut res = vec![];

    for (u, v, w) in iter {
        res.push((u, v, w));
        res.push((v, u, w));
    }

    res
}
