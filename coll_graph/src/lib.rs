#![feature(iter_from_generator)]
#![feature(let_chains)]
#![feature(generators)]
#![feature(type_alias_impl_trait)]


pub mod mst;
pub mod sp;
pub mod toposort;
pub mod tree;
pub mod scc;
pub mod bcc;
pub mod test;
#[cfg(test)]
mod debug;


use std::{
    collections::{BTreeSet, HashMap, hash_map::Entry},
};

use coll::{
    union_find::{UnionFind, SZ},
    apush, get, getopt, set, ordered_insert,
};
use common::Itertools;

use crate::scc::scc_tarjan;



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
    pub e: Vec<Vec<usize>>,
    /// In-edges
    pub rev: Vec<Vec<usize>>,
    pub w: HashMap<(usize, usize), isize>,
}



impl Graph {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn from_directed_iter<T: IntoIterator<Item = (usize, usize, isize)>>(iter: T) -> Self {
        let mut e = Vec::new();
        let mut rev = Vec::new();
        let mut w = HashMap::new();

        for (u, v, w_) in iter {
            apush!(e => u => v);
            apush!(rev => v => u);
            set!(w => (u, v) => w_);
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
        for i in 0..self.e.len() {
            if !self.e[i].is_empty() {
                return i;
            }
        }

        unreachable!("There is no edge.")
    }

    /// O(|E|)
    pub fn vertexs(&self) -> impl Iterator<Item = usize> {
        let mut vertexs = BTreeSet::new();

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
            for (u, tos) in self.e.iter().cloned().enumerate() {
                for v in tos {
                    yield (u, v, get!(self.w => (u, v)))
                }
            }
        })
    }

    pub fn contains_edge(&self, edge: (usize, usize)) -> bool {
        // self.w 与 self.e 可能是独立修改的
        // getopt!(self.w => edge).is_some()

        let (u, v) = edge;

        if let Some(tos) = getopt!(self.e => u) {
            tos.into_iter().find(|&x| x == v).is_some()
        } else {
            false
        }
    }

    /// O(|E|) for undirected graph or directed graph
    pub fn components(&self) -> Vec<Vec<usize>> {
        if self.is_dir {
            return scc_tarjan(self);
        }

        let mut dsu = UnionFind::new(Some(SZ));
        let vertexs: Vec<usize> = self.vertexs().collect();

        for v in vertexs.iter().cloned() {
            dsu.insert(v);
        }

        for (u, v, _) in self.edges() {
            dsu.cunion(u, v);
        }

        let mut comps: HashMap<usize, Vec<usize>> = HashMap::new();

        for v in vertexs.iter().cloned() {
            let p = dsu.cfind(v);

            match comps.entry(p) {
                Entry::Occupied(mut ent) => {
                    ordered_insert!(ent.get_mut(), v);
                }
                Entry::Vacant(ent) => {
                    ent.insert(vec![v]);
                }
            };
        }

        comps
            .into_iter()
            .map(|(_p, v)| v)
            .sorted_unstable_by_key(|v| v[0])
            .collect()
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
