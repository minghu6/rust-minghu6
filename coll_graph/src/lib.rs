#![feature(iter_from_generator)]
#![feature(let_chains)]
#![feature(generators)]
#![feature(type_alias_impl_trait)]


pub mod mst;
pub mod sp;
pub mod toposort;
pub mod tree;
pub mod scc;
mod test;
mod debug;

use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
    iter::once,
};

use coll::{
    aux::VerifyResult,
    easycoll::{M1, MV},
    union_find::{UnionFind, SZ},
    apush, aux::VerifyError, get, getopt, queue, set, stack,
};

use self::tree::diameter::diameter_dp;

pub use test::*;


////////////////////////////////////////////////////////////////////////////////
//// Structure

/// Adjacent link formed simple (directed) connected graph
#[derive(Default, Clone)]
pub struct Graph {
    /// If it's directed
    pub dir: bool,
    /// Out-edges
    pub e: MV<usize, usize>,
    /// In-edges
    pub rev: MV<usize, usize>,
    pub w: M1<(usize, usize), isize>,
}


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



impl FromIterator<(usize, usize, isize)> for Graph {
    fn from_iter<T: IntoIterator<Item = (usize, usize, isize)>>(
        iter: T,
    ) -> Self {
        let mut e = MV::new();
        let mut rev = MV::new();
        let mut w = M1::new();

        for (u, v, _w) in iter {
            apush!(e => u => v);
            apush!(rev => v => u);
            set!(w => (u, v) => _w);
        }

        Self { dir: true, e, rev, w }
    }
}


impl IntoIterator for Graph {
    type Item = (usize, usize, isize);
    type IntoIter = impl Iterator<Item = Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        let w = self.w;

        self.e
            .0
            .into_iter()
            .map(|(u, vs)| vs.into_iter().map(move |v| (u, v)))
            .flatten()
            .map(move |(u, v)| (u, v, get!(w => (u, v))))
    }
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


impl Graph {
    pub fn new() -> Self {
        Default::default()
    }

    /// get Transpose
    pub fn t(&self) -> Self {
        Self::from_iter(
            self.edges().map(|(u, v, w)| (v, u, w))
        )
    }

    pub fn insert_edge(
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

    /// The length of the shortest path between the most distanced nodes.
    pub fn diameter(&self) -> isize {
        diameter_dp(self)
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
        let mut edges = self.e.0.iter();
        let mut subedges = vec![];

        std::iter::from_fn(move || loop {
            if let Some((u, v)) = subedges.pop() {
                return Some((u, v, get!(self.w => (u, v))));
            } else {
                if let Some((from, tos)) = edges.next() {
                    subedges = tos
                        .into_iter()
                        .cloned()
                        .map(|v| (*from, v))
                        .rev()
                        .collect::<Vec<(usize, usize)>>();
                } else {
                    return None;
                }
            }
        })
    }

    /// Preorder
    pub fn dfs<'a>(
        &'a self,
        start: Option<usize>,
    ) -> impl Iterator<Item = usize> + 'a {
        let start = start.unwrap_or(self.anypoint());

        let mut stack = stack![(start, start)];

        std::iter::from_fn(move || {
            while let Some((u, p)) = stack.pop() {
                stack.extend(
                    get!(self.e => u)
                        .into_iter()
                        .filter(|v| *v != p)
                        .map(|v| (v, u))
                        .rev(),
                );

                return Some(u);
            }

            None
        })
    }

    pub fn bfs<'a>(
        &'a self,
        start: Option<usize>,
    ) -> impl Iterator<Item = usize> + 'a {
        let start = start.unwrap_or(self.anypoint());

        let mut q = queue![(start, start)];
        // let mut visited = HashSet::new();

        std::iter::from_fn(move || {
            while let Some((u, p)) = q.deq() {
                for v in get!(self.e => u) {
                    if v == p {
                        continue;
                    }

                    q.enq((v, u));
                }

                return Some(u);
            }

            None
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
        if self.dir {
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
    /// Verify

    /// verify spanning tree
    pub fn verify_st(&self, st: &[(usize, usize)]) -> VerifyResult {
        let mut vertx = self
            .vertexs()
            .map(|v| (v, ()))
            .collect::<HashMap<usize, ()>>();

        for (u, v) in st {
            if !self.contains_edge((*u, *v)) {
                return Err(VerifyError::Inv(format!("No edge {u}->{v}")));
            }

            vertx.remove(u);
            vertx.remove(v);
        }

        if vertx.is_empty() {
            Ok(())
        } else {
            Err(VerifyError::Fail(format!("remains vertex: {vertx:?}",)))
        }
    }


    /// verify minimal spanning tree
    pub fn verify_mst(
        &self,
        min: isize,
        st: &[(usize, usize)],
    ) -> VerifyResult {
        self.verify_st(st)?;

        let tot: isize = st.into_iter().map(|x| get!(self.w => x)).sum();

        if tot == min {
            Ok(())
        } else if tot < min {
            Err(VerifyError::Inv(format!("cur_tot: {tot} < min: {min}")))
        } else {
            Err(VerifyError::Fail(format!("too big")))
        }
    }

    pub fn verify_path(
        &self,
        src: usize,
        dst: usize,
        path: &[usize],
    ) -> VerifyResult {
        let mut u = src;

        for v in path {
            if self.contains_edge((u, *v)) {
                u = *v;
            } else {
                return Err(VerifyError::Inv(format!(
                    "No edge {u}-{v} for {path:?} src: {src} dst: {dst}"
                )));
            }
        }

        if u == dst {
            Ok(())
        } else {
            Err(VerifyError::Inv(format!("Not end in {dst}, found {u}")))
        }
    }

    pub fn verify_cycle(&self, cycle: &[usize]) -> VerifyResult {
        if cycle.len() == 0 {
            return Err(VerifyError::Inv(format!("Empty negative cycle")));
        }
        if cycle.len() == 1 {
            return Err(VerifyError::Inv(format!("Self loop")));
        }
        if cycle.len() == 2 && !self.dir {
            return Err(VerifyError::Fail(format!("Single edge cycle for undirected graph")));
        }

        let src = *cycle.first().unwrap();
        let dst = *cycle.last().unwrap();

        self.verify_path(src, dst, &cycle[1..])?;

        if self.contains_edge((dst, src)) {
            Ok(())
        } else {
            Err(VerifyError::Fail(format!("{dst} can't goback to {src}")))
        }
    }

    pub fn verify_negative_cycle(
        &self,
        cycle: &[usize],
    ) -> VerifyResult {
        self.verify_cycle(cycle)?;

        /* verify negative weights sumeration */

        let sumw: isize = cycle
            .iter()
            .cloned()
            .zip(cycle[1..].iter().cloned().chain(once(cycle[0])))
            .map(|(u, v)| get!(self.w => (u, v)))
            .sum();

        if sumw < 0 {
            Ok(())
        } else {
            Err(VerifyError::Fail(format!("weight sum: {sumw} >= 0, for cycle {cycle:?}")))
        }
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

        if self.dir {
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



#[cfg(test)]
mod tests {

    use common::Itertools;
    use coll::get;
    use crate::{
        mst::{mst_boruvka, mst_kruskal, mst_prim},
        to_undirected_vec, Graph,
    };


    fn setup_g_data() -> Vec<Graph> {
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
            .map(|x| Graph::from_iter(to_undirected_vec(x)))
            .collect::<Vec<Graph>>()
    }

    #[test]
    fn test_g_dfs() {
        let g = setup_g_data();

        let data = vec![(0, 1, vec![1, 2, 5, 4, 7, 6, 3])];

        for (gi, start, seq) in data {
            assert_eq!(g[gi].dfs(Some(start)).collect_vec(), seq);
        }
    }

    #[test]
    fn test_g_bfs() {
        let g = setup_g_data();

        let data = vec![(0, 1, vec![1, 2, 6, 5, 4, 3, 7])];

        for (gi, start, seq) in data {
            assert_eq!(g[gi].bfs(Some(start)).collect_vec(), seq);
        }
    }

    #[test]
    fn test_mst_fixed() {
        let g = setup_g_data();

        let data =
            vec![(2, vec![(1, 4), (1, 2), (4, 6), (2, 5), (3, 5), (5, 7)])];

        for (gi, edges) in data {
            let min = edges.into_iter().map(|x| get!(g[gi].w => x)).sum();

            /* verify krusal (edge) algorithm */
            let st = mst_kruskal(&g[gi]);
            assert_eq!(g[gi].verify_mst(min, &st), Ok(()));

            /* verify prim (vertex) algorithm */
            let st = mst_prim(&g[gi]);
            assert_eq!(g[gi].verify_mst(min, &st), Ok(()));

            /* verify boruvka algorithm */
            let st = mst_boruvka(&g[gi]);
            assert_eq!(g[gi].verify_mst(min, &st), Ok(()));
        }
    }
}
