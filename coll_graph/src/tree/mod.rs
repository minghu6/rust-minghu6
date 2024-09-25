//! Tree like graph
//!

use std::collections::HashMap;

use common::{hashset, Itertools};

use coll::{contains, get, set, hashmap};

use super::Graph;

pub mod diameter;
pub mod lca;
pub mod hpd;


////////////////////////////////////////////////////////////////////////////////
//// Structures (Enumration)

/// 从起点到终点再到起点的简单路径，每条路径访问两次（一来一回）
///
/// Kind 1 in-out
pub struct EulerSeq1 {
    /// vertex-timestamp
    ein: HashMap<usize, usize>,
    eout: HashMap<usize, usize>,
}


/// Kind 2 record every node
pub struct EulerSeq2 {
    seq: Vec<usize>,
}


#[derive(Debug, Clone, Copy)]
pub enum Center {
    One(usize),
    AdjTwo(usize, usize),
}

////////////////////////////////////////////////////////////////////////////////
//// Implementations

impl PartialEq<Self> for Center {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::One(l0), Self::One(r0)) => l0 == r0,
            (Self::AdjTwo(l0, l1), Self::AdjTwo(r0, r1)) => {
                (hashset! {l0, l1}) == hashset! {r0, r1}
            }
            _ => false,
        }
    }
}


impl EulerSeq1 {
    pub fn new(g: &Graph, start: usize) -> Self {
        let mut stack = vec![(start, false)];
        let mut ein = HashMap::new();
        let mut eout = HashMap::new();
        let mut timestamp = 0;

        while let Some((u, is_out)) = stack.pop() {
            if is_out {
                set!(eout => u => timestamp);
                timestamp += 1;
                continue;
            }

            set!(ein => u => timestamp);
            timestamp += 1;

            stack.push((u, true));
            stack.extend(
                get!(g.e => u => vec![])
                    .into_iter()
                    .filter(|v| !contains!(ein => v))
                    .map(|v| (v, false))
                    .rev(), // keep dfs order
            );
            // for v in get!(g.e => u => vec![]) {
            //     if !contains!(ein => v) {
            //         stack.push((v, false));
            //     }
            // }
        }

        Self { ein, eout }
    }


    pub fn as_seq<'a>(&'a self) -> impl Iterator<Item = usize> + 'a {
        self.ein
            .iter()
            .chain(self.eout.iter())
            .sorted_by_key(|(_, t)| *t)
            .map(|(x, _t)| *x)
    }

    /// Check if x is parent of y
    pub fn is_paren(&self, x: usize, y: usize) -> bool {
        get!(self.ein => x) <= get!(self.ein => y)
            && get!(self.eout => y) >= get!(self.eout => y)
    }
}



impl EulerSeq2 {
    pub fn new(g: &Graph, start: usize) -> Self {
        fn dfs(
            g: &Graph,
            u: usize,
            p: usize,
            mut seq: Vec<usize>,
        ) -> Vec<usize> {
            for v in get!(g.e => u => vec![]) {
                if v != p {
                    seq.push(u);
                    seq = dfs(g, v, u, seq);
                }
            }

            seq.push(u);

            seq
        }

        Self {
            seq: dfs(g, start, start, vec![]),
        }
    }

    pub fn as_seq<'a>(&'a self) -> impl Iterator<Item = usize> + 'a {
        self.seq.iter().cloned()
    }
}




////////////////////////////////////////////////////////////////////////////////
//// Functions



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
                get!(g.e => v)
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
pub fn distance(g: &Graph, u: usize) -> HashMap<usize, isize> {
    let mut d = hashmap! {};
    let mut q = vec![(u, 0)];
    let mut visited = hashset! {u};

    set!(d => u => 0);

    while let Some((u, tot)) = q.pop() {
        for v in get!(g.e => u) {
            if !visited.contains(&v) {
                visited.insert(v);

                let newtot = tot + get!(g.w => u, v);

                set!(d => v => newtot);
                q.push((v, newtot));
            }
        }
    }

    d
}


/// 某个点（或某两个邻接的点）使得最大子树的节点数相等
pub fn center(g: &Graph) -> Center {
    fn subsize(g: &Graph, u: usize, p: usize) -> usize {
        let mut cnt = 1;

        for v in get!(g.e => u) {
            if v != p {
                cnt += subsize(g, v, u);
            }
        }

        cnt
    }

    fn max_subsize(g: &Graph, u: usize) -> usize {
        get!(g.e => u => vec![])
            .into_iter()
            .map(|v| {  // 最后一颗子树结点数可以利用 n - acc 来直接求出
                let n = subsize(g, v, u);
                // println!("{u} -> {v} : {n}");
                n
            })
            .max()
            .unwrap_or(0)
    }

    let mut min = g.vertexs().count();
    let mut min_v = vec![];

    for u in g.vertexs() {
        // println!("{u}");
        let w = max_subsize(g, u);

        if w < min {
            min = w;
            min_v = vec![u];
        } else if w == min {
            min_v.push(u);
        }
    }

    if min_v.len() == 1 {
        Center::One(min_v[0])
    } else if min_v.len() == 2 {
        Center::AdjTwo(min_v[0], min_v[1])
    } else {
        unreachable!("min: {min}, min_v: {min_v:?}");
    }
}




#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use common::Itertools;

    use crate::{
        tree::{
            center, diameter::*, furthest_vertex_no_w, lca::LCATarjan,
            lca::LCADP, Center, EulerSeq1, EulerSeq2, hpd::HPD,
        },
        Graph,
    };


    fn setup_ud_tree_data() -> Vec<Graph> {
        // u->v, w
        let data = vec![
            // 0
            /*
                1---2--5
                 \   \
                 6--3 4--7
             */
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
            // 5
            vec![
                (1, 2, 1),
                (2, 3, 1),
                (2, 4, 1),
                (2, 5, 1),
                (1, 6, 1),
                (6, 7, 1),
                (6, 8, 1),
            ],
            // 6
            /*
                      1
                    /   \
                   2    3
                  / \  / \
                 4  5 6  7
            */
            vec![
                (1, 2, 1),
                (1, 3, 1),
                (2, 4, 1),
                (2, 5, 1),
                (3, 6, 1),
                (3, 7, 1),
            ],
            //  7
            /*
                   2 -- 3
                  / \  / \
                 4  5 6  7
            */
            vec![
                (2, 3, 1),
                (2, 4, 1),
                (2, 5, 1),
                (3, 6, 1),
                (3, 7, 1),
            ],
        ];

        data.into_iter()
            .map(|x| Graph::from_undirected_iter(x))
            .collect::<Vec<Graph>>()
    }

    #[test]
    fn test_furthest_vertex_no_w() {
        let graphs = setup_ud_tree_data();

        // (gindex, startv, res)
        let data = vec![(0usize, 1usize, vec![7]), (0usize, 5, vec![3])];

        for (ginx, start, res) in data.into_iter() {
            let (_d, vs) = furthest_vertex_no_w(&graphs[ginx], start);

            let vs: HashSet<usize> = HashSet::from_iter(vs);
            let res = HashSet::from_iter(res);

            // let _ = graphs[ginx].render_as_file("out.dot");

            assert_eq!(vs, res, "INPUT: g{ginx}, {start}");
        }
    }

    #[test]
    fn test_diameter() {
        /* setup data */
        let graphs = setup_ud_tree_data();

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
        let graphs = setup_ud_tree_data();

        let data = vec![(
            4,
            1,
            vec![(13, 12, 1), (12, 4, 4), (13, 11, 6), (9, 12, 4)],
        )];

        for (gin, start, qd) in data {
            let g = &graphs[gin];

            let lcabl = LCADP::new(g, start);
            let lca_tarjan = LCATarjan::new(g, start);

            for (p, q, res) in qd.clone() {
                assert_eq!(lcabl.query(p, q), res);
            }

            let q: Vec<(usize, usize)> =
                qd.clone().into_iter().map(|(u, v, _res)| (u, v)).collect();

            let res: Vec<usize> =
                qd.iter().map(|(_, _, res)| *res).collect();

            assert_eq!(lca_tarjan.queries(&q), res);

            let hpd = HPD::new(g, Some(start));

            for (x, y, res) in qd {
                assert_eq!(hpd.lca(x, y), res);
            }

        }
    }

    #[test]
    fn test_euler_seq() {
        let graphs = setup_ud_tree_data();

        let data =
            vec![(5, 1, vec![1, 2, 3, 3, 4, 4, 5, 5, 2, 6, 7, 7, 8, 8, 6, 1])];

        for (gi, start, seq) in data {
            let eulerseq1 = EulerSeq1::new(&graphs[gi], start);

            assert_eq!(eulerseq1.as_seq().collect_vec(), seq,);
        }

        let data =
            vec![(5, 1, vec![1, 2, 3, 2, 4, 2, 5, 2, 1, 6, 7, 6, 8, 6, 1])];

        for (gi, start, seq) in data {
            let eulerseq2 = EulerSeq2::new(&graphs[gi], start);

            assert_eq!(eulerseq2.as_seq().collect_vec(), seq,);
        }
    }

    #[test]
    fn test_center() {
        let g = setup_ud_tree_data();

        let data = vec![(6, Center::One(1)), (7, Center::AdjTwo(2, 3))];

        for (gi, res) in data {
            assert_eq!(center(&g[gi]), res,);
        }
    }
}
