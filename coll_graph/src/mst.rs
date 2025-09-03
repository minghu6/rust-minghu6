//! Mini Spanning Tree  (无向连通图包含所有点的是生成树，边权和最小的是最小生成树)


////////////////////////////////////////////////////////////////////////////////
//// Functions

use std::collections::{HashMap, HashSet};

use coll::{
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
pub fn mst_kruskal(g: &Graph) -> Vec<(usize, usize)> {
    /* init sorted edge set */
    // let mut sorted_edges = FibHeap::new();
    let mut sorted_edges = vec![];

    for (u, v, w) in g.undir_edges() {
        sorted_edges.push((u, v, w));
    }

    sorted_edges.sort_unstable_by_key(|x| x.2);

    /* init disjoint set */
    let mut ds = UnionFind::new(Some(MergeBy::SZ));

    for v in 1..g.e.len() {
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
pub fn mst_prim(g: &Graph) -> Vec<(usize, usize)> {
    // g is connected means that g is not empty.
    debug_assert!(g.is_connected());
    debug_assert!(g.e[0].is_empty());

    let mut res = vec![];

    /* setup rest collection */

    let mut rest = vec![true; g.e.len()];
    // vertex start from 1
    rest[1] = false;

    /* init dis heap && dis edge map */

    // let mut dis = FibHeap::new();
    let mut dis = DaryHeap::<3, usize, isize>::with_capacity(rest.len());
    let mut dis_edge = vec![0; g.e.len()];

    let mut u = 1;

    loop {
        for v in g.e[u].iter().filter(|&&v| rest[v]).cloned() {
            let maybe_dis_v = dis.get(&v).cloned();

            if maybe_dis_v.is_none() {
                dis.insert(v, g.w[&(u, v)]);
                dis_edge[v] = u;
            } else if let Some(dis_v) = maybe_dis_v
                && g.w[&(u, v)] < dis_v
            {
                dis.decrease_key(&v, g.w[&(u, v)]);
                dis_edge[v] = u;
            }
        }

        if let Some((u_, _)) = dis.pop_item() {
            u = u_
        } else {
            break;
        };

        rest[u] = false;

        res.push((u, dis_edge[u]));
    }

    res
}

/// Boruvka: bo ru s ga (for unique edge weight)
///
/// 最小生成森林（非连通图）/ 最小生成树（连通图）
pub fn mst_boruvka(g: &Graph) -> Vec<(usize, usize)> {
    // let mut res = Vec::new();
    let mut res = HashSet::new();

    // using lexicograph order
    let mut dsu = UnionFind::new(Some(MergeBy::SZ));

    for v in 1..g.e.len() {
        dsu.insert(v);
    }

    // components cheapest edges: (weight, usize)
    let mut cand_edges: HashSet<(usize, usize, isize)> =
        g.undir_edges().collect();

    loop {
        let mut comp_min_edges: HashMap<usize, (isize, usize, usize)> =
            HashMap::new();

        for (u, v, w) in cand_edges.iter().cloned() {
            let pu = dsu.cfind(u);
            let pv = dsu.cfind(v);

            if pu == pv {
                continue;
            }

            let pu_min_edge = comp_min_edges.get(&pu).cloned();

            if pu_min_edge.is_none() || Some((w, u, v)) < pu_min_edge {
                comp_min_edges.insert(pu,(w, u, v));
            }

            let pv_min_edge = comp_min_edges.get(&pv).cloned();

            if pv_min_edge.is_none() || Some((w, u, v)) < pv_min_edge {
                comp_min_edges.insert(pv, (w, u, v));
            }
        }

        let mut continue_flag = false;

        for (_, (w, u, v)) in comp_min_edges.into_iter() {
            res.insert((u, v));
            dsu.cunion(u, v);
            cand_edges.remove(&(u, v, w));

            continue_flag = true;
        }

        if !continue_flag {
            break;
        }
    }

    // res
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
            let min = edges.into_iter().map(|x| g[gi].w[&x]).sum();

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
        for g in batch_graph(50, 100, -10..20, &GraphGenOptions::undir_conn()) {
            /* verify krusal (edge) algorithm */
            // let st = mst_prim(&g);
            let st = mst_kruskal(&g);
            // use krusal as standard mst algorithm
            let min = st.iter().cloned().map(|e| g.w[&e]).sum::<isize>();

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
