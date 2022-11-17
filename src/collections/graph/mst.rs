//! Mini Spanning Tree  (无向连通图包含所有点的是生成树，边权和最小的是最小生成树)
//!


////////////////////////////////////////////////////////////////////////////////
//// Function

use std::collections::{HashMap, HashSet};

use super::Graph;
use crate::{
    collections::{
        heap::fib::FibHeap,
        union_find::{MergeBy, UnionFind},
    },
    get,
};

///
/// 逐边地贪心算法
///
/// 边排序 + 并查集（两点是否相连）
///
/// O(eloge) + O(elogv) = O(eloge)
///
pub fn mst_kruskal(g: &Graph) -> Vec<(usize, usize)> {
    /* init sorted edge set */
    let mut sorted_edges = FibHeap::new();

    for (u, v, w) in g.edges() {
        sorted_edges.push((u, v), w);
    }

    /* init disjoint set */
    let mut ds = UnionFind::new(Some(MergeBy::SZ));

    for v in g.vertexs() {
        ds.insert(*v);
    }

    let mut res = vec![];

    while let Some(((u, v), _w)) = sorted_edges.pop_item() {
        if ds.cfind(u) != ds.cfind(v) {
            ds.cunion(u, v);
            res.push((u, v));
        }
    }

    res
}


/// 逐点的贪心算法
///
/// hashset(剩余点集) + 小顶堆（最好支持 decrease-key，存贮剩余集合的每个点到已有生成树的距离）
///
/// v: |V|, e: |E|
///
/// 1. 稀疏图 e = v
///
/// 1. 稠密图 e = v^2
///
/// | Fib Heap | Binary Heap |
/// | --- | --- |
/// | O(vlogv + e) | O(vlogv + elogv) |
///
///
pub fn mst_prim(g: &Graph) -> Vec<(usize, usize)> {
    let mut res = vec![];

    /* choose an arbitray node as root */

    let mut viter = g.vertexs().cloned();
    let root;

    if let Some(_root) = viter.next() {
        root = _root;
    } else {
        return res;
    }

    /* setup rest collection */

    let mut rest: HashSet<usize> = HashSet::new();

    /* init dis heap && dis edge map */

    let mut dis = FibHeap::new();

    let mut dis_edge = HashMap::new();

    dis.push(root, 0);
    dis_edge.insert(root, Some(root));

    for v in viter {
        rest.insert(v);
        dis.push(v, isize::MAX);
        dis_edge.insert(v, None);
    }

    while !rest.is_empty() {
        // u is current vertex
        let (u, _uw) = dis.pop_item().unwrap().clone();

        // "decrease-key" (It's increase-key actually for min-heap)
        // dis.update(u, isize::MAX);
        rest.remove(&u);

        let u_pair = get!(dis_edge => u).unwrap();

        if u_pair != u {
            res.push((u, u_pair));
        }

        // calc adj

        let adjs = get!(g.e => u);

        /* update dis heap */
        for v in adjs.into_iter().filter(|v| rest.contains(v)) {
            let w_uv: isize = get!(g.w => (u, v));

            if w_uv < *get!(dis => v) {
                dis.decrease_key(v, w_uv);
                dis_edge.insert(v, Some(u));
            }
        }
    }

    res
}


/// Boruvka: bo ru s ga (for unique edge weight)
///
/// 最小生成森林（非连通图）/ 最小生成树（连通图）
///
pub fn mst_boruvka(g: &Graph) -> Vec<(usize, usize)> {
    let mut res = HashSet::new();

    // using lexicograph order
    let mut dsu = UnionFind::new(Some(MergeBy::SZ));

    for v in g.vertexs().cloned() {
        dsu.insert(v);
    }

    // components cheapest edges: (weight, usize)
    let mut cand_edges: HashSet<(usize, usize, isize)> = g.edges().collect();

    loop {
        let mut comp_min_edges: HashMap<usize, Option<(isize, usize, usize)>> =
            HashMap::new();

        for (u, v, w) in cand_edges.iter().cloned() {
            let pu = dsu.cfind(u);
            let pv = dsu.cfind(v);

            if pu == pv {
                continue;
            }

            let pu_min_edge = comp_min_edges
                .get(&pu)
                .cloned()
                .unwrap_or(None);

            if pu_min_edge.is_none() || Some((w, u, v)) < pu_min_edge {
                comp_min_edges.insert(pu, Some((w, u, v)));
            }

            let pv_min_edge = comp_min_edges
                .get(&pv)
                .cloned()
                .unwrap_or(None);

            if pv_min_edge.is_none() || Some((w, u, v)) < pv_min_edge {
                comp_min_edges.insert(pv, Some((w, u, v)));
            }
        }

        let mut continue_flag = false;

        for (_, opt) in comp_min_edges.into_iter() {
            if let Some((w, u, v)) = opt {
                res.insert((u, v));
                dsu.cunion(u, v);
                cand_edges.remove(&(u, v, w));

                continue_flag = true;
            }
        }

        if !continue_flag {
            break;
        }
    }

    res.into_iter().collect()
}



#[cfg(test)]
mod tests {

    #[test]
    fn verify_option_partial_ord() {
        assert!(Some(0) < Some(1));
        assert!(Some(0) > None);
        assert!(None::<usize> == None);
    }
}
