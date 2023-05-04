#![feature(iter_from_generator)]
#![feature(let_chains)]
#![feature(generators)]
#![feature(type_alias_impl_trait)]


pub mod mst;
pub mod sp;
pub mod toposort;
pub mod tree;
pub mod scc;
pub mod test;
#[cfg(test)]
mod debug;


use std::{
    collections::HashSet,
    fmt::Debug,
};

use coll::{
    easycoll::{M1, MV},
    union_find::{UnionFind, SZ},
    apush, get, getopt, set,
};



////////////////////////////////////////////////////////////////////////////////
//// Structure

/// Adjacent list formed simple (directed) connected graph
///
/// Just ignore orphan vertex
#[derive(Default, Clone)]
pub struct Graph {
    /// If it's directed
    pub is_dir: bool,
    /// Out-edges
    pub e: MV<usize, usize>,
    /// In-edges
    pub rev: MV<usize, usize>,
    pub w: M1<(usize, usize), isize>,
}



impl Graph {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn from_directed_iter<T: IntoIterator<Item = (usize, usize, isize)>>(iter: T) -> Self {
        let mut e = MV::new();
        let mut rev = MV::new();
        let mut w = M1::new();

        for (u, v, _w) in iter {
            apush!(e => u => v);
            apush!(rev => v => u);
            set!(w => (u, v) => _w);
        }

        Self { is_dir: true, e, rev, w }
    }

    pub fn from_undirected_iter<T: IntoIterator<Item = (usize, usize, isize)>>(iter: T) -> Self {
        let mut g = Self::from_directed_iter(iter
            .into_iter()
            .map(|(u, v, w)| [
                (u, v, w),
                (v, u, w)
            ])
            .flatten()
        );

        g.is_dir = false;

        g
    }

    /// get Transpose
    pub fn t(&self) -> Self {
        debug_assert!(self.is_dir);

        Self::from_directed_iter(
            self.edges().map(|(u, v, w)| (v, u, w))
        )
    }

    /// Push new edge or update old edge with new weight
    pub fn insert(
        &mut self,
        edge: (usize, usize),
        w: isize,
    ) -> Option<(usize, usize, isize)> {
        let (u, v) = edge;

        let old;
        let oldw = set!(self.w => (u, v) => w);

        if self.contains_edge(edge) {
            old = Some((u, v, oldw.unwrap()));
        } else {
            old = None;
        }

        apush!(self.e => u => v);

        old
    }

    pub fn anypoint(&self) -> usize {
        *self.e.0.keys().next().unwrap()
    }

    /// O(|E|)
    pub fn vertexs(&self) -> impl Iterator<Item = usize> {
        let mut vertexs = HashSet::new();

        for (u, v, _w) in self.edges() {
            vertexs.insert(u);
            vertexs.insert(v);
        }

        vertexs.into_iter()
    }

    pub fn edges<'a>(
        &'a self,
    ) -> impl Iterator<Item = (usize, usize, isize)> + 'a {
        std::iter::from_generator(|| {
            for (u, tos) in self.e.0.iter() {
                for v in tos {
                    yield (*u, *v, get!(self.w => (*u, *v)))
                }
            }
        })
    }

    pub fn contains_edge(&self, edge: (usize, usize)) -> bool {
        // self.w 与 self.e 可能是独立修改的
        // getopt!(self.w => edge).is_some()

        let (u, v) = edge;

        let tos = getopt!(self.e => u);

        if let Some(tos) = tos {
            tos.into_iter().find(|&x| x == v).is_some()
        } else {
            false
        }
    }

    /// O(|E|) for undirected graph
    pub fn components(&self) -> Vec<Vec<usize>> {
        if self.is_dir {
            unimplemented!()
        }

        let mut dsu = UnionFind::new(Some(SZ));
        let vertexs: Vec<usize> = self.vertexs().collect();

        for v in vertexs.iter().cloned() {
            dsu.insert(v);
        }

        for (u, v, _) in self.edges() {
            dsu.cunion(u, v);
        }

        let mut comps = MV::new();

        for v in vertexs.iter().cloned() {
            let p = dsu.cfind(v);
            apush!(comps => p => v);
        }

        comps.0.into_iter().map(|(_k, vs)| vs).collect()
    }

    ////////////////////////////////////////////////////////////////////////////
    /// Introspection

    pub fn is_connected(&self) -> bool {
        let comps = self.components();

        debug_assert!(comps.len() > 0);

        comps.len() == 1
    }

    /// 1-10 (0 means sp)
    pub fn sparisity(&self) -> usize {
        let n = self.vertexs().count();
        let m = self.edges().count();

        if self.is_dir {
            let max = n * (n - 1);
            let min = n - 1;
            let peace = (max - min) / 10;

            (m - min) / peace
        } else {
            let max = n * (n - 1) / 2;
            let min = n - 1;
            let peace = (max - min) / 10;

            (m / 2 - min) / peace
        }
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

impl Debug for Graph {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            writeln!(f, "edge: {:#?}", self.e)?;
            writeln!(f, "weight: {:#?}", self.w)?;
        } else {
            writeln!(f, "edge: {:?}", self.e)?;
            writeln!(f, "weight: {:?}", self.w)?;
        }

        Ok(())
    }
}



#[cfg(test)]
pub(crate) mod tests {

    use crate::{
         Graph,
    };


    pub(crate) fn setup_ud_g_data() -> Vec<Graph> {
        // u->v, w
        let data = vec![
            // no0
            //      1
            //    /   \
            //   2    6
            //  / \   |
            // 5  4   3
            //    |
            //    7
            vec![
                (6, 3, 1),
                (1, 2, 1),
                (1, 6, 1),
                (2, 5, 1),
                (2, 4, 1),
                (4, 7, 1),
            ],
            /*
            no1

            1
            |
            2
            |
            4
            |
            3
            */
            vec![(1, 2, 1), (2, 4, 1), (4, 3, 1)],
            // no-2
            vec![
                (1, 2, 7),
                (1, 4, 5),
                (4, 2, 9),
                (2, 3, 8),
                (2, 5, 7),
                (3, 5, 5),
                (4, 5, 15),
                (4, 6, 6),
                (6, 5, 8),
                (6, 7, 11),
                (5, 7, 9),
            ],
        ];

        data.into_iter()
            .map(|x| Graph::from_undirected_iter(x))
            .collect::<Vec<Graph>>()
    }
}
