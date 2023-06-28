//! Strong Connected Components
//!
//!

use std::{
    cmp::min,
    collections::{BTreeMap, HashMap, HashSet},
};

use coll::{
    aux::{VerifyError, VerifyResult},
    get, set,
};
use common::{hashset, Itertools, ordered_insert};

use super::Graph;


impl Graph {
    pub fn verify_sccs(&self, sccs: &[Vec<usize>]) -> VerifyResult {
        debug_assert!(self.is_dir);

        /* vertexs are disjoint and complete */

        let vertexs_origin = self.vertexs().sorted().collect::<Vec<usize>>();
        let mut vertexs_scc = vec![];

        for scc in sccs {
            vertexs_scc.extend(scc.iter().cloned());
        }

        vertexs_scc.sort();

        if vertexs_scc != vertexs_origin {
            return Err(VerifyError::Fail(
                "Unmatched scc vertexs set".to_owned(),
            ));
        }

        /* each scc */

        for scc in sccs {
            self.verify_scc(scc)?;
        }

        Ok(())
    }

    fn verify_scc(&self, scc: &[usize]) -> VerifyResult {
        if scc.is_empty() {
            return Err(VerifyError::Inv("Invalid empty scc".to_owned()));
        }

        let scc_vertexs = scc.iter().cloned().sorted().collect::<Vec<usize>>();

        let mut acc: HashSet<usize> = scc.into_iter().cloned().collect();

        for u in scc.iter().cloned() {
            let mut visited = hashset![u];
            let mut stack = vec![u];

            while let Some(u) = stack.pop() {
                for v in get!(self.e => u => vec![]) {
                    if !visited.contains(&v) {
                        visited.insert(v);
                        stack.push(v);
                    }
                }
            }

            acc = acc.intersection(&visited).cloned().collect();
        }

        let visited_vertexs = acc.into_iter().sorted().collect::<Vec<usize>>();

        if visited_vertexs != scc_vertexs {
            return Err(
                VerifyError::Fail(
                    format!("Wrong scc: \nscc    : {scc_vertexs:?} \nvisited: {visited_vertexs:?}\n")
                )
            );
        }

        Ok(())
    }
}


pub fn scc_tarjan(g: &Graph) -> Vec<Vec<usize>> {
    #[derive(Default, Clone, Copy)]
    struct DFSMeta {
        index: Option<usize>,
        lowlink: usize,
        on_stack: bool,
    }

    fn dfs_scc(
        g: &Graph,
        u: usize,
        comps: &mut Vec<Vec<usize>>,
        stack: &mut Vec<usize>,
        index: &mut usize,
        vertexs: &mut Vec<DFSMeta>,
    ) {
        vertexs[u] = DFSMeta {
            index: Some(*index),
            lowlink: *index,
            on_stack: true,
        };

        *index += 1;
        stack.push(u);

        for v in get!(g.e => u => vec![]) {
            if vertexs[v].index.is_none() {
                dfs_scc(g, v, comps, stack, index, vertexs);

                vertexs[u].lowlink =
                    min(vertexs[u].lowlink, vertexs[v].lowlink);
            } else if vertexs[v].index < vertexs[u].index
                && vertexs[v].on_stack
            {
                vertexs[u].lowlink =
                    min(vertexs[u].lowlink, vertexs[v].index.unwrap());
            }
        }

        /* start a new scc */

        if vertexs[u].lowlink == vertexs[u].index.unwrap() {
            let mut new_comp = Vec::new();

            while let Some(s) = stack.pop() {
                ordered_insert!(&mut new_comp, s);
                vertexs[s].on_stack = false;

                if s == u {
                    break;
                }
            }

            ordered_insert!(comps, new_comp, |x: &Vec<usize>| x[0]);
        }
    }


    let mut comps = Vec::new();

    if g.e.is_empty() {
        return comps;
    }

    let mut stack = vec![];
    let mut index = 0;

    let max_v = g.vertexs().max().unwrap();

    let mut vertexs = vec![DFSMeta::default(); max_v + 1];

    for u in g.vertexs() {
        if vertexs[u].index.is_none() {
            dfs_scc(g, u, &mut comps, &mut stack, &mut index, &mut vertexs)
        }
    }

    comps
}



/// or Gabow
pub fn scc_path_based(g: &Graph) -> Vec<Vec<usize>> {
    fn find_comp(
        g: &Graph,
        u: usize,
        stack: &mut Vec<usize>,
        path: &mut Vec<usize>,
        index: &mut usize,
        vertexs: &mut HashMap<usize, Option<usize>>,
        assigned: &mut HashSet<usize>,
        comps: &mut Vec<Vec<usize>>,
    ) {
        set!(vertexs => u => Some(*index));
        *index += 1;

        stack.push(u);
        path.push(u);

        for v in get!(g.e => u => vec![]) {
            if get!(vertexs => v).is_none() {
                find_comp(g, v, stack, path, index, vertexs, assigned, comps);
            } else if !assigned.contains(&v) {
                while let Some(p) = path.last()
                && get!(vertexs => *p).unwrap() > get!(vertexs => v).unwrap() {
                    path.pop();
                }
            }
        }

        if let Some(p) = path.last() && *p == u {
            let mut new_comp = Vec::new();

            while let Some(s) = stack.pop() {
                ordered_insert!(&mut new_comp, s);
                assigned.insert(s);

                if s == u { break }
            }

            ordered_insert!(comps, new_comp, |x: &Vec<usize>| x[0]);
            path.pop();
        }
    }


    let mut stack = vec![];
    let mut path = vec![];
    let mut index = 0;
    let mut vertexs = g
        .vertexs()
        .map(|u| (u, None))
        .collect::<HashMap<usize, Option<usize>>>();

    let mut assigned = HashSet::new();
    let mut comps: Vec<Vec<usize>> = Vec::new();

    for u in g.vertexs() {
        if get!(vertexs => u).is_none() {
            find_comp(
                g,
                u,
                &mut stack,
                &mut path,
                &mut index,
                &mut vertexs,
                &mut assigned,
                &mut comps,
            );
        }
    }

    comps
}


/// 大概读作 'kao ser ra zhu'
pub fn scc_kosaraju(g: &Graph) -> Vec<Vec<usize>> {
    /// Visit each Vertexs in Postfix order
    fn visit(
        g: &Graph,
        u: usize,
        vis: &mut HashSet<usize>,
        l: &mut Vec<usize>,
    ) {
        if !vis.contains(&u) {
            vis.insert(u);

            for v in get!(g.e => u => vec![]) {
                visit(g, v, vis, l);
            }

            l.push(u);
        }
    }

    /// Assign Components
    fn assign(
        g: &Graph,
        u: usize,
        root: usize,
        comps: &mut BTreeMap<usize, Vec<usize>>,
        vis: &mut HashSet<usize>,
    ) {
        if !vis.contains(&u) {
            vis.insert(u);

            let new_comp = comps.entry(root).or_insert(Vec::new());

            ordered_insert!(new_comp, u);

            for v in get!(g.e => u => vec![]) {
                assign(g, v, root, comps, vis);
            }
        }
    }

    /* DFS 1 */

    let mut vis = HashSet::new();
    let mut l: Vec<usize> = vec![];
    let mut comps: BTreeMap<usize, Vec<usize>> = BTreeMap::new();

    for u in g.vertexs() {
        visit(g, u, &mut vis, &mut l);
    }

    let gt = g.t();
    vis.clear();

    while let Some(u) = l.pop() {
        assign(&gt, u, u, &mut comps, &mut vis);
    }

    comps
        .into_iter()
        .map(|x| x.1)
        .sorted_unstable_by_key(|x| x[0])
        .collect()
}



#[cfg(test)]
mod tests {

    use coll::aux::VerifyError::*;
    use common::random_range;

    use super::*;
    use crate::{test::GraphGenOptions, Graph};

    fn setup_dir_data() -> Vec<Graph> {
        let opt = GraphGenOptions {
            is_dir: true,
            allow_cycle: true,
            non_negative_cycle: false,
            weak_conn: false,
        };

        let mut gs = vec![];

        for _ in 0..50 {
            let sparsity = random_range!(1..3);
            let g = Graph::gen(&opt, 50, sparsity, 1..2);
            gs.push(g);
        }

        gs
    }

    fn setup_undir_data() -> Vec<Graph> {
        let opt = GraphGenOptions {
            is_dir: false,
            allow_cycle: true,
            non_negative_cycle: false,
            weak_conn: false,
        };

        let mut gs = vec![];

        for _ in 0..50 {
            let g = Graph::gen(&opt, 50, 0, 1..2);
            gs.push(g);
        }

        gs
    }


    #[test]
    fn test_find_scc() {
        for g in setup_dir_data() {
            let comps_kosaraju = scc_kosaraju(&g);
            let comps_tarjan = scc_tarjan(&g);
            // let comps_tarjan2 = scc_tarjan2(&g);
            let comps_path_based = scc_path_based(&g);

            // println!("{}", comps_kosaraju.len());

            if let Err(err) = g.verify_sccs(&comps_tarjan) {
                match err {
                    Inv(inv) => panic!("Invalid {inv}"),
                    Fail(fail) => panic!("Fail {fail}"),
                }
            }

            // if let Err(err) = g.verify_sccs(&comps_tarjan2) {
            //     match err {
            //         Inv(inv) => panic!("Invalid {inv}"),
            //         Fail(fail) => panic!("Fail {fail}"),
            //     }
            // }

            assert_eq!(comps_kosaraju, comps_tarjan);
            // assert_eq!(comps_kosaraju, comps_tarjan2, "\ng:{g:?}");
            assert_eq!(comps_kosaraju, comps_path_based);
        }
    }

    #[test]
    fn test_find_undir_cc() {
        for g in setup_undir_data() {
            let comps_msu = g.components();
            let comps_tarjan = scc_tarjan(&g);

            // println!("{}", comps_msu.len());
            assert_eq!(comps_msu, comps_tarjan);
        }
    }
}
