pub mod mst;
pub mod toposort;
pub mod tree;


use std::{collections::HashMap, fmt::Debug};

use self::tree::diameter::diameter_dp;
use super::{
    aux::VerifyResult,
    easycoll::{M1, MV},
};
use crate::{apush, collections::aux::VerifyError, get, queue, set, stack};



////////////////////////////////////////////////////////////////////////////////
//// Structure

/// Adjacent link formed simple (directed) connected graph
#[derive(Default)]
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

    pub fn anypoint(&self) -> usize {
        *self.e.0.keys().next().unwrap()
    }

    /// The length of the shortest path between the most distanced nodes.
    pub fn diameter(&self) -> isize {
        diameter_dp(self)
    }

    pub fn vertexs(&self) -> impl Iterator<Item = &usize> {
        self.e.0.keys()
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
        let (u, v) = edge;

        let tos = get!(self.e => u);

        tos.iter().find(|&&x| x == v).is_some()
    }


    ////////////////////////////////////////////////////////////////////////////
    /// Verify

    /// verify spanning tree
    pub fn verify_st(&self, st: &[(usize, usize)]) -> VerifyResult {
        let mut vertx = self
            .vertexs()
            .cloned()
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
            Err(VerifyError::Fail)
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
            Err(VerifyError::Fail)
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


    use itertools::Itertools;

    use crate::{collections::graph::{to_undirected_vec, Graph, mst::mst_prim}, get};

    use super::mst::mst_kruskal;


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
    fn test_mst() {
        let g = setup_g_data();

        let data =
            vec![(2, vec![(1, 4), (1, 2), (4, 6), (2, 5), (3, 5), (5, 7)])];

        for (gi, edges) in data {
            let min = edges
                .into_iter()
                .map(|x| get!(g[gi].w => x))
                .sum();

            /* verify krusal (edge) algorithm */
            let st = mst_kruskal(&g[gi]);
            assert_eq!(g[gi].verify_mst(min, &st), Ok(()));

            /* verify prim (vertex) algorithm */
            let st = mst_prim(&g[gi]);
            assert_eq!(g[gi].verify_mst(min, &st), Ok(()));
        }
    }
}
