//! Tree like graph
//!

use maplit::hashset;

use super::Graph;
use crate::{collections::easycoll::M1, get, m1, set};

pub mod diameter;
pub mod lca;


////////////////////////////////////////////////////////////////////////////////
//// Function



/// Include itself no weight property,
pub fn furthest_vertex_no_w(g: &Graph, u: usize) -> (isize, Vec<usize>) {
    let mut prev_q = vec![u];
    let mut cur_q = get!(g.e => u);
    let mut is_visited = hashset! { u };
    let mut d = 0;

    while !cur_q.is_empty() {
        prev_q = cur_q.clone();
        let mut next_q = vec![];

        for v in cur_q.into_iter() {
            next_q.extend(
                get!(g.e => u)
                    .into_iter()
                    .filter(|x| !is_visited.contains(x)),
            );

            is_visited.insert(v);
        }

        cur_q = next_q;
        d += 1;
    }

    (d, prev_q)
}


/// get further vertex from dfs (can also bfs, but dfs is more easy)
pub fn distance(g: &Graph, u: usize) -> M1<usize, isize> {
    let mut d = m1! {};
    let mut q = vec![(u, 0)];
    let mut visited = hashset! {u};

    set!(d => u => 0);

    while let Some((u, tot)) = q.pop() {
        for v in get!(g.e => u) {
            if visited.contains(&v) {
                continue;
            }
            visited.insert(v);
            let newtot = tot + get!(g.w => u, v);
            set!(d => u => newtot);

            q.push((v, newtot));
        }
    }

    d
}




#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use crate::collections::graph::{
        to_undirected_vec,
        tree::{diameter::*, furthest_vertex_no_w, lca::LCABL},
        Graph,
    };

    use super::lca::LCATarjan;


    fn setup_tree_data() -> Vec<Graph> {
        // u->v, w
        let data = vec![
            // 0
            vec![
                (1usize, 6usize, 1isize),
                (6, 3, 1),
                (1, 2, 1),
                (2, 5, 1),
                (2, 4, 1),
                (4, 7, 1),
            ],
            // 1
            vec![(1, 2, 1), (2, 4, 1), (4, 3, 1)],
            // 2
            vec![
                (1usize, 6usize, 10isize),
                (6, 3, 60),
                (1, 2, 15),
                (2, 5, 100),
                (2, 4, 39),
                (4, 7, 2),
            ],
            // 3
            vec![(1, 2, 1), (2, 4, -4), (4, 3, 2)],
            // 4
            vec![
                (1, 2, 1),
                (2, 5, 1),
                (1, 3, 1),
                (3, 6, 1),
                (6, 10, 1),
                (10, 13, 1),
                (6, 11, 1),
                (1, 4, 1),
                (4, 7, 1),
                (4, 8, 1),
                (8, 12, 1),
                (4, 9, 1),
            ],
        ];

        data.into_iter()
            .map(|x| Graph::from_iter(to_undirected_vec(x)))
            .collect::<Vec<Graph>>()
    }

    #[test]
    fn test_furthest_vertex_no_w() {
        let graphs = setup_tree_data();

        // (gindex, startv, res)
        let data = vec![(0usize, 1usize, vec![7]), (0usize, 5, vec![3])];

        for (ginx, start, res) in data.into_iter() {
            let (_d, vs) = furthest_vertex_no_w(&graphs[ginx], start);

            let vs: HashSet<usize> = HashSet::from_iter(vs);
            let res = HashSet::from_iter(res);

            assert_eq!(vs, res, "INPUT: g{ginx}, {start}");
        }
    }

    #[test]
    fn test_diameter() {
        /* setup data */
        let graphs = setup_tree_data();

        let data = vec![(0, 5), (1, 3)];

        for (ginx, res) in data.into_iter() {
            assert_eq!(diameter_2bfs_no_w(&graphs[ginx]), res);
            assert_eq!(diameter_2dfs(&graphs[ginx]), res);
            assert_eq!(diameter_dp(&graphs[ginx]), res);
        }

        let data = vec![(2, 185)];

        for (ginx, res) in data.into_iter() {
            assert_eq!(diameter_2dfs(&graphs[ginx]), res);
            assert_eq!(diameter_dp(&graphs[ginx]), res);
        }

        // negative edge weight
        let data = vec![(3, 2)];
        for (ginx, res) in data.into_iter() {
            assert_eq!(diameter_dp(&graphs[ginx]), res);
        }
    }

    #[test]
    fn test_lca() {
        /* setup data */
        let graphs = setup_tree_data();

        let data = vec![(
            4,
            1,
            vec![(13, 12, 1), (12, 4, 4), (13, 11, 6), (9, 12, 4)],
            // vec![(13, 12, 1),],

        )];

        for (gin, start, qd) in data {
            let g = &graphs[gin];

            let lcabl = LCABL::new(g, start);
            let lca_tarjan = LCATarjan::new(g, start);

            for (p, q, res) in qd.clone() {
                assert_eq!(lcabl.query(p, q), res);
            }

            let q: Vec<(usize, usize)> = qd
                .clone()
                .into_iter()
                .map(|(u, v, _res)| (u, v))
                .collect();

            let res: Vec<usize> = qd
                .into_iter()
                .map(|(_, _, res)| res)
                .collect();

            assert_eq!(lca_tarjan.queries(&q), res);
        }
    }
}
