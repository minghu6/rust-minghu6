pub mod tree;
pub mod toposort;


use std::fmt::Debug;

use self::tree::diameter::diameter_dp;
use super::easycoll::{M1, MV};
use crate::{apush, get, set, stack, queue};



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
                    .rev()
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
                    if v == p { continue }

                    q.enq((v, u));
                }

                return Some(u);
            }

            None
        })

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

    use crate::collections::graph::{to_undirected_vec, Graph};


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
}
