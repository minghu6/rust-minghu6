//! Biconnected Components (undirected graph)

use std::{cmp::min, collections::HashSet};

use coll::{
    aux::{VerifyError, VerifyResult},
    get, ordered_insert,
};
use common::Itertools;

use crate::Graph;


impl Graph {
    /// Verify bccs for undirected graph
    ///
    /// 1. 边集不交且为合集
    ///
    /// 2. 每个bcc内部没有交点(articulation point/cut point)
    ///
    /// 3. bcc之间连接的是交点
    ///
    pub fn verify_undir_bccs(
        &self,
        bccs: &[Vec<(usize, usize)>],
    ) -> VerifyResult {
        debug_assert!(!self.is_dir);

        /* 1. edges cover and disjoint */

        let all_edges = self
            .edges()
            .map(|(u, v, _)| if u < v { (u, v) } else { (v, u) })
            .collect::<HashSet<(usize, usize)>>();

        let mut acc = HashSet::new();

        for bcc in bccs {
            let bcc_set = HashSet::from_iter(bcc.clone());

            let intersection: Vec<(usize, usize)> =
                acc.intersection(&bcc_set).cloned().collect();

            if !intersection.is_empty() {
                return Err(VerifyError::Fail(format!(
                    "Overlapped bcc edges: {intersection:?}"
                )));
            }

            acc = acc.union(&bcc_set).cloned().collect();
        }

        if all_edges != acc {
            return Err(
                VerifyError::Fail(
                    format!(
                        "Unmatched complete edges:\n all_edges: {all_edges:?}\n bccs_edges: {acc:?}"
                    )
                )
            );
        }

        /* 2. check cut point in each bcc */

        let mut g_bccs = vec![];

        for bcc in bccs {
            let mut cuts = vec![];

            let g_bcc = Graph::from_undirected_iter(
                bcc.into_iter().cloned().map(|(u, v)| (u, v, 1)),
            );

            for v in g_bcc.vertexs() {
                if g_bcc.test_cut_vertex(v) {
                    cuts.push(v);
                }
            }

            if !cuts.is_empty() {
                return Err(VerifyError::Fail(format!(
                    "there are cut points {cuts:?}\n in {bcc:?}"
                )));
            }

            g_bccs.push(g_bcc);
        }

        /* 3. ensure intersection vertexs are articulation points */

        let raw_components = self.components().len();
        let mut excluded = vec![false; self.e.len()];

        for bcc in bccs {
            if bcc.len() == 1 {
                excluded[bcc[0].0] = true;
                excluded[bcc[0].1] = true;
            }
        }

        for (i, bcc) in bccs.iter().enumerate() {
            let bcc_edges_set: HashSet<(usize, usize)> =
                bcc.iter().cloned().collect();

            let rem_edges_set: HashSet<(usize, usize)> = self
                .edges()
                .map(|(u, v, _)| (u, v))
                .filter(|(u, v)| {
                    !bcc_edges_set.contains(&(*u, *v))
                        && !bcc_edges_set.contains(&(*v, *u))
                })
                .collect();

            let rem_vertexs_set: HashSet<usize> = rem_edges_set
                .iter()
                .cloned()
                .flat_map(|(u, v)| [u, v])
                .collect();

            let bcc_vertexs_set: HashSet<usize> =
                g_bccs[i].vertexs().collect();

            for v in bcc_vertexs_set.intersection(&rem_vertexs_set).cloned() {
                if !excluded[v] {
                    let g1 = Graph::from_undirected_iter(
                        self.edges()
                            .filter(|(u1, v1, _)| *u1 != v && *v1 != v),
                    );

                    if g1.components().len() == raw_components {
                        return Err(VerifyError::Fail(format!(
                            "Found none cut point {v:?}\n for {bcc:?}",
                        )));
                    }

                    excluded[v] = true;
                }
            }
        }

        Ok(())
    }

    pub fn test_cut_vertex(&self, v: usize) -> bool {
        let g1_edges = self
            .edges()
            .filter(|(u1, v1, _)| *u1 != v && *v1 != v);

        let g1 = if self.is_dir {
            Graph::from_directed_iter(g1_edges)
        } else {
            Graph::from_undirected_iter(g1_edges)
        };

        self.components().len() < g1.components().len()
    }
}


/// for undirected graph
pub fn normalize_undir_edges_comps(
    a: Vec<Vec<(usize, usize)>>,
) -> Vec<Vec<(usize, usize)>> {
    assert!(a.iter().all(|x| !x.is_empty()));

    a.into_iter()
        .map(|comps| {
            comps
                .into_iter()
                .map(|(u, v)| if u < v { (u, v) } else { (v, u) })
                .sorted_unstable()
                .dedup()
                .collect::<Vec<(usize, usize)>>()
        })
        .sorted_unstable_by_key(|comp| comp[0])
        .collect()
}


pub fn bcc_tarjan(g: &Graph) -> Vec<Vec<(usize, usize)>> {
    #[derive(Default, Clone, Copy)]
    struct DFSMeta {
        index: Option<usize>,
        lowpt: usize,
    }

    fn dfs_bcc(
        g: &Graph,
        u: usize,
        p: usize,
        mut bccs: &mut Vec<Vec<(usize, usize)>>,
        stack: &mut Vec<(usize, usize)>,
        index: &mut usize,
        vertexs: &mut Vec<DFSMeta>,
    ) {
        *index += 1;

        vertexs[u] = DFSMeta {
            index: Some(*index),
            lowpt: *index,
        };

        for v in get!(g.e => u => vec![]) {
            if vertexs[v].index.is_none() {
                stack.push((u, v));
                dfs_bcc(g, v, u, bccs, stack, index, vertexs);

                vertexs[u].lowpt =
                    min(vertexs[u].lowpt, vertexs[v].lowpt);

                if vertexs[u].index.unwrap() == vertexs[u].lowpt {
                    // u is cut point
                    // get biconnected component

                    let mut bcc = vec![];

                    loop {
                        let (v1, v2) = stack.pop().unwrap();
                        ordered_insert!(
                            &mut bcc,
                            if v1 < v2 { (v1, v2) } else { (v2, v1) }
                        );

                        if v1 == u {
                            break;
                        }
                    }

                    ordered_insert!(&mut bccs, bcc, |bcc: &Vec<(usize, usize)>| bcc[0]);
                }
            } else if vertexs[v].index < vertexs[u].index && v != p {
                stack.push((u, v));
                vertexs[u].lowpt =
                    min(vertexs[u].lowpt, vertexs[v].index.unwrap());
            }
        }
    }


    let mut bccs = Vec::new();

    if g.e.is_empty() {
        return bccs;
    }

    let mut stack = vec![];
    let mut index = 0;

    let max_v = g.vertexs().max().unwrap();

    let mut vertexs = vec![DFSMeta::default(); max_v + 1];

    for u in g.vertexs() {
        if vertexs[u].index.is_none() {
            dfs_bcc(
                g,
                u,
                max_v + 1,
                &mut bccs,
                &mut stack,
                &mut index,
                &mut vertexs,
            )
        }
    }

    bccs
}




#[cfg(test)]
mod tests {

    use coll::aux::VerifyError::*;
    use common::{random_range, Itertools};

    use super::normalize_undir_edges_comps;
    use crate::{bcc::bcc_tarjan, test::GraphGenOptions, Graph};


    fn setup_data() -> Vec<Graph> {
        let opt = GraphGenOptions {
            is_dir: false,
            allow_cycle: true,
            non_negative_cycle: false,
            weak_conn: true,
        };

        let mut gs = vec![];

        /* setup sample data */

        let g0 = vec![
            (1, 2, 1),
            (1, 6, 1),
            (1, 7, 1),
            (2, 3, 1),
            (2, 4, 1),
            (2, 5, 1),
            (3, 4, 1),
            (5, 6, 1),
            (7, 8, 1),
            (7, 9, 1),
            (8, 9, 1),
        ];
        gs.push(Graph::from_undirected_iter(g0));

        for _ in 0..50 {
            let sparsity = random_range!(1..3);
            let g = Graph::gen(&opt, 30, sparsity, 1..2);
            gs.push(g);
        }

        gs
    }


    #[test]
    fn test_find_bcc() {
        let mut samples = vec![];

        let g0 = vec![
            (1, 2, 1),
            (1, 6, 1),
            (1, 7, 1),
            (2, 3, 1),
            (2, 4, 1),
            (2, 5, 1),
            (3, 4, 1),
            (5, 6, 1),
            (7, 8, 1),
            (7, 9, 1),
            (8, 9, 1),
        ];
        let res0 = vec![
            vec![(1, 7)],
            vec![(1, 2), (2, 5), (5, 6), (6, 1)],
            vec![(2, 3), (2, 4), (3, 4)],
            vec![(7, 8), (7, 9), (8, 9)],
        ];

        samples.push((
            Graph::from_undirected_iter(g0),
            normalize_undir_edges_comps(res0),
        ));

        for (g, expect) in samples {
            let bccs = bcc_tarjan(&g);

            assert_eq!(bccs, expect);
            g.verify_undir_bccs(&bccs).unwrap();
        }

        for g in setup_data() {
            let bccs = bcc_tarjan(&g);

            // println!("bccs len: {}", bccs.len());

            if let Err(err) = g.verify_undir_bccs(&bccs) {
                eprintln!(
                    "Graph:    {:?}\n\nbccs:   {bccs:?}",
                    g
                    .edges()
                    .map(|(u, v, _)| if u < v { (u, v)} else { (v, u) })
                    .sorted()
                    .dedup()
                    .collect_vec()
                );

                match err {
                    Inv(inv) => panic!("Invalid {inv}"),
                    Fail(fail) => panic!("Fail {fail}"),
                }
            }
        }
    }
}
