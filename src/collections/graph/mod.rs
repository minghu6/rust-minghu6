pub mod tree;


use super::easycoll::{M1, MV};
use crate::{get, set};



////////////////////////////////////////////////////////////////////////////////
//// Structure

/// Adjacent link formed simple (directed) graph
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
            let mut heads = get!(e => u => vec![]);
            heads.push(v);
            set!(e => u => heads);

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

    // pub fn get(&self, idx: &usize) -> Option<&Vec<usize>> {
    //     self.e.get(idx)
    // }

    // pub fn getw(&self, idx: (usize, usize)) -> Option<&isize> {
    //     self.w.get(&idx)
    // }


    /// The length of the shortest path between the most distanced nodes.
    pub fn diameter(&self) -> usize {
        todo!()
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
