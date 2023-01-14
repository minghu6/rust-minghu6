//! Strong Connected Components
//!
//!

use std::{
    cmp::min,
    collections::{HashMap, HashSet},
};

use super::Graph;
use crate::{
    apush,
    collections::{aux::VerifyResult, easycoll::MS},
    get, set,
};


impl Graph {
    pub fn verify_scc(&self, _scc: &[usize]) -> VerifyResult {
        debug_assert!(self.dir);

        Ok(())
    }
}


pub fn scc_tarjan(g: &Graph) -> Vec<HashSet<usize>> {
    let mut stack = vec![];
    let mut index = 0;

    #[derive(Default)]
    struct Meta {
        index: Option<usize>,
        lowlink: Option<usize>,
        on_stack: bool,
    }

    let mut vertexs = g
        .vertexs()
        .map(|u| (u, Meta::default()))
        .collect::<HashMap<usize, Meta>>();

    let mut comps: Vec<HashSet<usize>> = Vec::new();

    for u in g.vertexs() {
        if get!(vertexs => u).index.is_none() {
            find_comp(g, u, &mut stack, &mut index, &mut vertexs, &mut comps)
        }
    }

    fn find_comp(
        g: &Graph,
        u: usize,
        stack: &mut Vec<usize>,
        index: &mut usize,
        vertexs: &mut HashMap<usize, Meta>,
        comps: &mut Vec<HashSet<usize>>,
    ) {
        set!(vertexs => u => Meta {
            index: Some(*index),
            lowlink: Some(*index),
            on_stack: true
        });

        *index += 1;
        stack.push(u);


        for v in get!(g.e => u => vec![]) {
            if get!(vertexs => v).index.is_none() {
                find_comp(g, v, stack, index, vertexs, comps);

                vertexs.get_mut(&u).unwrap().lowlink = Some(min(
                    get!(vertexs => u).lowlink.unwrap(),
                    get!(vertexs => v).lowlink.unwrap(),
                ));
            } else if get!(vertexs => v).on_stack {
                vertexs.get_mut(&u).unwrap().lowlink = Some(min(
                    get!(vertexs => u).lowlink.unwrap(),
                    // It says v.index not v.lowlink;
                    // that is deliberate and from the original paper
                    get!(vertexs => v).index.unwrap(),
                ));
            }
        }

        if get!(vertexs => u).lowlink.unwrap()
            == get!(vertexs => u).index.unwrap()
        {
            /* start a new scc */
            let mut set = HashSet::new();

            while let Some(s) = stack.pop() {
                set.insert(s);
                vertexs.get_mut(&s).unwrap().on_stack = false;

                if s == u {
                    break;
                }
            }

            comps.push(set);
        }
    }


    comps
}


/// or Gabow
pub fn scc_path_based(g: &Graph) -> Vec<HashSet<usize>> {
    let mut stack = vec![];
    let mut path = vec![];
    let mut index = 0;
    let mut vertexs = g
        .vertexs()
        .map(|u| (u, None))
        .collect::<HashMap<usize, Option<usize>>>();

    let mut assigned = HashSet::new();
    let mut comps: Vec<HashSet<usize>> = Vec::new();

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


    fn find_comp(
        g: &Graph,
        u: usize,
        stack: &mut Vec<usize>,
        path: &mut Vec<usize>,
        index: &mut usize,
        vertexs: &mut HashMap<usize, Option<usize>>,
        assigned: &mut HashSet<usize>,
        comps: &mut Vec<HashSet<usize>>,
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
            let mut new_comp = HashSet::new();

            while let Some(s) = stack.pop() {
                new_comp.insert(s);
                assigned.insert(s);

                if s == u { break }
            }

            comps.push(new_comp);
            path.pop();
        }
    }

    comps
}


/// 大概读作 'kao ser ra zhu'
pub fn scc_kosaraju(g: &Graph) -> Vec<HashSet<usize>> {
    /* DFS 1 */

    let mut vis = HashSet::new();
    let mut l: Vec<usize> = vec![];
    let mut comps: MS<usize, usize> = MS::new();

    for u in g.vertexs() {
        visit(g, u, &mut vis, &mut l);
    }

    let gt = g.t();
    vis.clear();

    while let Some(u) = l.pop() {
        assign(&gt, u, u, &mut comps, &mut vis);
    }

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
        comps: &mut MS<usize, usize>,
        vis: &mut HashSet<usize>,
    ) {
        if !vis.contains(&u) {
            vis.insert(u);
            apush!(comps => root => u);

            for v in get!(g.e => u => vec![]) {
                assign(g, v, root, comps, vis);
            }
        }
    }

    comps.0.into_iter().map(|x| x.1).collect()
}


pub fn normalize_comps(mut a: Vec<HashSet<usize>>) -> Vec<HashSet<usize>> {
    a.sort_unstable_by_key(|set| *set.iter().min().unwrap());
    a
}


// pub fn comps_eq(a: Vec<HashSet<usize>>, oth: Vec<HashSet<usize>>) -> bool {
//     normalize_comps(a) == normalize_comps(oth)
// }



#[cfg(test)]
mod tests {
    use super::{scc_kosaraju, scc_tarjan};
    use crate::{
        algs::random_range,
        collections::graph::{
            scc::{normalize_comps, scc_path_based},
            Graph,
        },
        test::graph::GraphGenOptions,
    };


    fn setup_data() -> Vec<Graph> {
        let opt = GraphGenOptions {
            dir: true,
            cyclic: true,
            non_negative_cycle: false,
            weak: true,
        };

        let mut gs = vec![];

        for _ in 0..50 {
            let sparsity = random_range(1..3);
            let g = Graph::gen(&opt, 30, sparsity, 1..2);
            gs.push(g);
        }

        gs
    }


    #[test]
    fn test_find_scc() {
        for g in setup_data() {
            let comps_kosaraju = scc_kosaraju(&g);
            let comps_tarjan = scc_tarjan(&g);
            let comps_path_based = scc_path_based(&g);

            let comps_kosaraju = normalize_comps(comps_kosaraju);
            let comps_tarjan = normalize_comps(comps_tarjan);
            let comps_path_based = normalize_comps(comps_path_based);

            assert_eq!(comps_kosaraju, comps_tarjan);
            assert_eq!(comps_kosaraju, comps_path_based);
        }
    }
}

