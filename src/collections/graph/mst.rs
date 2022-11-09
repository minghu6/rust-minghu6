//! Mini Spanning Tree  (无向连通图包含所有点的是生成树，边权和最小的是最小生成树)
//!


////////////////////////////////////////////////////////////////////////////////
//// Function

use crate::collections::{union_find::{UnionFind, MergeBy}, heap::fib::FibHeap};

use super::Graph;

///
/// 逐边地贪心算法
///
/// 并查集 + 堆
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
