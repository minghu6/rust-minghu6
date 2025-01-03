//! Mini Spanning Tree  (无向连通图包含所有点的是生成树，边权和最小的是最小生成树)
//!


////////////////////////////////////////////////////////////////////////////////
//// Functions

use std::collections::{HashMap, HashSet};

use coll::{
    get,
    union_find::{MergeBy, UnionFind},
};
use coll_heap::dary::DaryHeap;

use super::Graph;


///
/// 逐边地贪心算法
///
/// 边排序 + 并查集（两点是否相连）
///
/// O(eloge) + O(elogv) = O(eloge)
///
pub fn mst_kruskal(g: &Graph) -> Vec<(usize, usize)> {
    /* init sorted edge set */
    // let mut sorted_edges = FibHeap::new();
    let mut sorted_edges = vec![];

    for (u, v, w) in g.edges() {
        sorted_edges.push((u, v, w));
    }

    sorted_edges.sort_unstable_by_key(|x| x.2);

    /* init disjoint set */
    let mut ds = UnionFind::new(Some(MergeBy::SZ));

    for v in g.vertexs() {
        ds.insert(v);
    }

    let mut res = vec![];

    for (u, v, _w) in sorted_edges {
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
/// | Fib Heap | Binary Heap | Dary2^5 Heap |
/// | --- | --- | --- |
/// | O(vlogv + e) | O(vlogv + elogv) | O(v + e) |
///
///
pub fn mst_prim(g: &Graph) -> Vec<(usize, usize)> {
    debug_assert!(g.is_connected());

    let mut res = vec![];

    /* choose an arbitray node as root */

    let mut viter = g.vertexs();
    let root;

    if let Some(_root) = viter.next() {
        root = _root;
    } else {
        return res;
    }

    /* setup rest collection */

    let mut rest: HashSet<usize> = HashSet::new();

    /* init dis heap && dis edge map */

    // let mut dis = FibHeap::new();
    let mut dis = DaryHeap::<3, usize, isize>::with_capacity(rest.len() + 1);

    let mut dis_edge = HashMap::new();

    dis.insert(root, 0);
    dis_edge.insert(root, Some(root));

    for v in viter {
        rest.insert(v);
        dis.insert(v, isize::MAX);
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

            if w_uv < get!(dis => v) {
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

    for v in g.vertexs() {
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

            let pu_min_edge = comp_min_edges.get(&pu).cloned().unwrap_or(None);

            if pu_min_edge.is_none() || Some((w, u, v)) < pu_min_edge {
                comp_min_edges.insert(pu, Some((w, u, v)));
            }

            let pv_min_edge = comp_min_edges.get(&pv).cloned().unwrap_or(None);

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
    use super::{super::test::*, *};

    pub(crate) fn setup_ud_g_data() -> Vec<Graph> {
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
            .map(|x| Graph::from_undirected_iter(x))
            .collect::<Vec<Graph>>()
    }


    #[test]
    fn verify_option_partial_ord() {
        assert!(Some(0) < Some(1));
        assert!(Some(0) > None);
        assert!(None::<usize> == None);
    }

    #[test]
    fn test_mst_fixed() {
        let g = setup_ud_g_data();

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

    #[test]
    fn test_mst_random() {
        for g in batch_graph(50, 100, -10..20, &GraphGenOptions::undir_conn())
        {
            /* verify krusal (edge) algorithm */
            let st = mst_kruskal(&g);
            // use krusal as standard mst algorithm
            let min =
                st.iter().cloned().map(|e| get!(g.w => e)).sum::<isize>();

            assert_eq!(g.verify_mst(min, &st), Ok(()));

            /* verify prim (vertex) algorithm */
            let st = mst_prim(&g);
            assert_eq!(g.verify_mst(min, &st), Ok(()));

            /* verify boruvka algorithm */
            let st = mst_boruvka(&g);
            assert_eq!(g.verify_mst(min, &st), Ok(()));
        }
    }
}
