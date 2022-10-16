////////////////////////////////////////////////////////////////////////////////
//// Function

use std::{cmp::max, collections::HashMap};

use super::{distance, furthest_vertex_no_w};
use crate::{collections::graph::Graph, get, hashmap};


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
    let d = distance(g, v0);
    let (v1, _d_v1) = d.0.into_iter().max_by_key(|x| x.1).unwrap();

    let d = distance(g, v1);
    let (_v2, d_v2) = d.0.into_iter().max_by_key(|x| x.1).unwrap();

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
        }
    }

    res
}
