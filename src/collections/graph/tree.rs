//! Tree like graph
//!

use std::{cmp::max, collections::HashMap};

use maplit::{hashmap, hashset};

use super::Graph;
use crate::{
    algs::math::uintlog2,
    collections::easycoll::{M1, M2},
    get,
    getopt,
    set,
    m1, m2,
};

////////////////////////////////////////////////////////////////////////////////
//// Structure

pub struct LCABL {
    data: LCABLD,
}

/// LCA Binaru Lifting Data
struct LCABLD {
    acs: M2<usize, usize, usize>,
    d: M1<usize, usize>,
}


////////////////////////////////////////////////////////////////////////////////
//// Implementation

impl LCABL {
    pub fn new(g: &Graph, start: usize) -> Self {
        let data = lca_bl_setup(g, start);

        println!("depth: {:?}", data.d);
        println!("acs: {:#?}", data.acs);

        Self { data }
    }


    pub fn query(&self, mut p: usize, mut q: usize) -> usize {
        let d = &self.data.d;
        let acs = &self.data.acs;

        /* swap p to the depthest node */
        if get!(d => q) > get!(d => p) {
            (p, q) = (q, p);
        }

        /* binary lift q to the same depth of p */
        let mut diff = get!(d => p) - get!(d => q);
        let mut j = 0;
        while diff > 0 {
            // move base if the latest number is 1
            // So we can arrive at specific dsiatnce by 2 power
            if diff & 1 > 0 {
                p = get!(acs =>p, j);
            }

            j += 1;
            diff >>= 1
        }

        debug_assert_eq!(get!(d => p), get!(d => q));

        /* same path vertex return to avoid eval too big CA later */
        if p == q {
            return p;
        }

        /* binary lift p and q from high to low */

        for j in (0..uintlog2(get!(d => p)) as usize).rev() {
            // depth overflow or too big
            if getopt!(acs => p,j) != getopt!(acs => q,j) {
                p = get!(acs => p,j);
                q = get!(acs => q,j);
            }
        }

        p = get!(acs => p,0);
        q = get!(acs => q,0);

        debug_assert_eq!(p, q);

        p
    }
}


////////////////////////////////////////////////////////////////////////////////
//// Function



// #[macro_use]
// mod generated {
//     crate::gen_coll_macro!(2, m1, M1, new, insert);
// }


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
pub fn distance(g: &Graph, u: usize) -> HashMap<usize, isize> {
    let mut d = hashmap! {};
    let mut q = vec![(u, 0)];
    let mut visited = hashset! {u};

    d.insert(u, 0);

    while let Some((u, tot)) = q.pop() {
        for v in get!(g.e => u) {
            if visited.contains(&v) {
                continue;
            }
            visited.insert(v);
            let newtot = tot + get!(g.w => u, v);
            d.insert(v, newtot);

            q.push((v, newtot));
        }
    }

    d
}


/// Two pass bfs (depth)
pub fn diameter_2bfs_no_w(g: &Graph) -> isize {
    /* select entry point */
    let v0 = *g.e.0.keys().next().unwrap();

    /* find the furthest vertex from v0 */
    let (_d, v1) = furthest_vertex_no_w(g, v0);
    let (d, _v2) = furthest_vertex_no_w(g, v1[0]);

    d
}

/// Save the extra distance array, can't handle negtive weight
pub fn diameter_2dfs(g: &Graph) -> isize {
    let v0 = g.e.0.keys().next().unwrap().clone();
    // println!("v0: {v0}");
    let d = distance(g, v0);
    let (v1, _d_v1) = d.into_iter().max_by_key(|x| x.1).unwrap();

    // println!("v1: {v1}, d: {_d_v1}");

    let d = distance(g, v1);
    let (_v2, d_v2) = d.into_iter().max_by_key(|x| x.1).unwrap();

    d_v2
}


/// Find left longest distance and the right longest distance
///
/// That's means the longest and the longest but one
///
/// This method can handle negative weight case.
pub fn diameter_dp(g: &Graph) -> isize {
    // zero path weight
    let zpw = 0;

    let mut d1: HashMap<usize, isize> = hashmap! {};
    let mut d2: HashMap<usize, isize> = hashmap! {};
    let mut res = zpw; //
    let v0 = g.e.0.keys().next().unwrap().clone();
    // println!("start: {v0}");

    let mut stack = vec![(v0, v0)]
        .into_iter()
        .chain(get!(g.e => v0).into_iter().map(|v| (v, v0)))
        .collect::<Vec<(usize, usize)>>();

    while let Some((u, p)) = stack.pop() {
        // 树形结构是连通图，保证了一定会有至少一条边
        let subs = get!(g.e => u)
            .into_iter()
            .filter(|&v| v != p)
            .collect::<Vec<usize>>();

        // is leaf node
        if subs.is_empty() {
            // println!("U {u} as leaf");
            d1.insert(u, zpw);
            d2.insert(u, zpw);
            continue;
        }

        // before update
        if d1.get(&subs[0]).is_none() {
            // println!("Enstack {u}({p})");
            // for v in subs.iter() {
            //     println!("Enstack* {v}({u})");
            // }

            stack.push((u, p));
            stack.extend(subs.into_iter().map(|v| (v, u)));
            continue;
        }

        // after update
        for v in subs.into_iter() {
            let d1_v = d1.get(&v).unwrap();

            let mut d1_u = *d1.get(&u).unwrap_or(&zpw);
            let mut d2_u = *d2.get(&u).unwrap_or(&zpw);
            let w_u_v = get!(g.w => u, v);

            if d1_u < d1_v + w_u_v {
                d2_u = d1_u;
                d1_u = d1_v + w_u_v;
            } else if d2_u < d1_v + w_u_v {
                d2_u = d1_v + w_u_v;
            }

            res = max(res, d1_u + d2_u);
            d1.insert(u, d1_u);
            d2.insert(u, d2_u);

            // println!("U {u}(p: {p}, sub: {v}) to d1: {d1_u}, d2: {d2_u}, res: {res}");
        }
    }

    res
}



/// -> (depth map, ancestors map)
fn lca_bl_setup(g: &Graph, start: usize) -> LCABLD {
    /// Recursive version of LCA setup data
    fn setup_dfs_r(g: &Graph, u: usize, p: usize, data: LCABLD) -> LCABLD {
        let LCABLD { mut acs, mut d } = data;

        // println!("-> {u}");

        for v in get!(g.e => u) {
            if v == p {
                // println!("skip-{v}, ");
                continue;
            }
            // println!("{v}, ");

            let curd = get!(d => u => 0) + 1;
            set!(d => v => curd);

            let mut i = curd >> 1;
            let mut j = 1;
            set!(acs => v, 0 => u);

            while i > 0 {
                // acs[i][j] = acs[acs[i][j-1]][j-1]
                let oldstate = get!(acs => get!(acs => v, j-1), j-1);
                set!(acs => v, j => oldstate);

                i >>= 1;
                j += 1;
            }

            let data = setup_dfs_r(g, v, u, LCABLD { acs, d });
            acs = data.acs;
            d = data.d;
        }

        LCABLD { acs, d }
    }

    setup_dfs_r(
        g,
        start,
        start,
        LCABLD {
            acs: m2![start => 0 => start,],
            d: m1![start => 0,],
        },
    )
}




#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::{diameter_2bfs_no_w, furthest_vertex_no_w, LCABL};
    use crate::collections::graph::{
        to_undirected_vec,
        tree::{diameter_2dfs, diameter_dp},
        Graph,
    };


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

        let data =
            vec![(4, 1, vec![(13, 12, 1), (12, 4, 4), (13, 11, 6), (9, 12, 4)])];

        for (gin, start, qd) in data {
            let g = &graphs[gin];

            let lcabl = LCABL::new(g, start);
            for (p, q, res) in qd {
                assert_eq!(lcabl.query(p, q), res);
            }
        }
    }
}
