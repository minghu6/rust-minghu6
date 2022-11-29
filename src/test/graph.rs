use std::{collections::HashSet, ops::Range};

use crate::{
    algs::{random, random_range},
    collections::{
        graph::{to_undirected_vec, Graph, sp::SPBellmanFord, Path},
        union_find::{MergeBy, UnionFind}, easycoll::{MS},
    }, apush, del, set,
};

use rand::prelude::*;


////////////////////////////////////////////////////////////////////////////////
//// Structure

pub struct GraphGenOptions {
    pub dir: bool,
    /// if it's a tree
    pub cyclic: bool,
    pub nonnegative_cycle: bool
}



////////////////////////////////////////////////////////////////////////////////
//// Implementation

impl Graph {
    /// Generate simple connected graph
    ///
    /// vertex in 1..=vrange
    ///
    /// weight in wrange
    ///
    /// sparsity: 1-10,
    ///
    ///
    /// it ranges exponentially
    pub fn gen(
        opt: &GraphGenOptions,
        vrange: usize,
        sparsity: usize,
        wrange: Range<isize>,
    ) -> Graph {
        debug_assert!(1 <= sparsity && sparsity <= 10);
        let mut g;

        if sparsity == 10 {
            assert!(opt.cyclic);
            g = gen_scc_graph(vrange, wrange.clone());
        }
        else {
            let mut max = vrange * (vrange - 1);
            let min = vrange - 1;
            if !opt.dir {
                max /= 2;
            }

            let peace = (max - min) / 10;

            let edge_limit = min + sparsity * peace;

            g = Graph::new();
            let mut dsu = UnionFind::new(Some(MergeBy::SZ));

            for v in 1..=vrange {
                dsu.insert(v);
            }

            let mut rem_edges = MS::new();
            for u in 1..=vrange {
                for v in 1..=vrange {
                    if u != v {
                        apush!(rem_edges => u => v);
                    }
                }
            }

            for _ in 0..edge_limit {
                let u = *rem_edges
                    .0
                    .keys()
                    .choose(&mut rand::thread_rng())
                    .unwrap();

                let mut rem_vertexs = del!(rem_edges => u);

                let v = *rem_vertexs
                    .iter()
                    .choose(&mut rand::thread_rng())
                    .unwrap();

                rem_vertexs.remove(&v);
                if !rem_vertexs.is_empty() {
                    set!(rem_edges => u => rem_vertexs);
                }

                let mut othset = del!(rem_edges => v);
                othset.remove(&u);
                if !othset.is_empty() {
                    set!(rem_edges => v => othset);
                }

                dsu.cunion(u, v);

                let w = random_range(wrange.clone());
                g.insert_edge((u, v), w);

                if !opt.dir {
                    g.insert_edge((v, u), w);
                }
            }

            /* ensure connected */

            let mut comps = HashSet::new();

            for v in 1..=vrange {
                comps.insert(dsu.cfind(v));
            }

            if comps.len() > 1 {
                let mut comps_iter = comps.into_iter();

                let u = comps_iter.next().unwrap();

                for v in comps_iter {
                    let w = random_range(wrange.clone());

                    g.insert_edge((u, v), w);
                    dsu.cunion(u, v);

                    if !opt.dir {
                        g.insert_edge((v, u), w);
                    }
                }
            }
        }

        /* gen spanning tree */

        if !opt.cyclic {
            let mut visited = HashSet::new();
            let mut edges = vec![];

            for (u, v, w) in g.into_iter() {
                if !visited.contains(&u) || !visited.contains(&v) {
                    visited.insert(u);
                    visited.insert(v);

                    edges.push((u, v, w));
                }
            }

            if !opt.dir {
                edges = to_undirected_vec(edges);
            }

            g = Graph::from_iter(edges);
        }
        else {
            debug_assert!(g.is_connected());

            if wrange.start < 0 && opt.nonnegative_cycle {
                g.fix_negative_cycle(opt.dir);
            }
        }

        g
    }


    pub(crate) fn fix_negative_cycle(&mut self, dir: bool) {
        loop {
            let src = self.anypoint();

            if let Err(ncycle) = SPBellmanFord::new(&self, src) {
                assert!(ncycle.len() > 1);

                let p = Path::from_cycle(self, &ncycle).freeze();
                let mut totw = p.weight();
                assert!(totw < 0);

                for (u ,v, w) in p.iter() {
                    if w < 0 {  // reverse weight
                        totw += 2*(-w);
                        set!(self.w => (u, v) => -w);

                        if !dir {
                            set!(self.w => (v, u) => -w);
                        }
                    }
                }
            }
            else {
                break;
            }
        }
    }

}



////////////////////////////////////////////////////////////////////////////////
//// Function

/// Generate (Undirected) Simple Completed Connected Graph
fn gen_scc_graph(
    vrange: usize,
    wrange: Range<isize>,
) -> Graph
{
    let mut edges = vec![];

    for u in 1..vrange {
        for v in u + 1..=vrange {
            let w = random_range(wrange.clone());
            edges.push((u, v, w));
            edges.push((v, u, w));
        }
    }

    edges.into_iter().collect()
}


/// Undirected Connected Simple Graph Options
pub fn ucgopt() -> GraphGenOptions {
    GraphGenOptions {
        dir: false,
        cyclic: true,
        nonnegative_cycle: false
    }
}

pub fn ucg_nncycle_opt() -> GraphGenOptions {
    GraphGenOptions {
        dir: false,
        cyclic: true,
        nonnegative_cycle: true
    }
}

pub fn batch_graph(
    n: usize,
    vrange: usize,
    wrange: Range<isize>,
    opt: &GraphGenOptions,
) -> Vec<Graph> {
    let mut res = vec![];

    for _ in 0..n {
        let sparsity = random::<usize>() % 10 + 1;

        let g = Graph::gen(opt, vrange, sparsity, wrange.clone());
        res.push(g);
    }

    res
}



#[cfg(test)]
mod tests {
    use std::fs::File;

    use super::*;
    use crate::{collections::graph::Graph, debug::graphviz::RenderOptions};


    #[test]
    fn test_gen_ud_graph() {

        let g = Graph::gen(&ucgopt(), 30, 4, 1..15);

        assert!(g.is_connected());

        let mut dotf = File::create("out.dot").unwrap();
        let render_opt = RenderOptions {
            dir: false,
            weight_edge: true,
        };
        let mut csvf = File::create("out.csv").unwrap();
        g.write_to_csv(&mut csvf).unwrap();

        let mut csvf = File::open("out.csv").unwrap();
        let g = Graph::read_from_csv(&mut csvf).unwrap();

        g.render(&render_opt, &mut dotf).unwrap();
    }

    #[test]
    fn test_batch_graph() {
        for _g in batch_graph(100, 100, 1..50, &ucgopt()) {

        }
        // for _ in 0..100 {
        //     // let sparsity = random() % 10 + 1;

        //     // println!("sparsity: {sparsity}");

        //     let g = Graph::gen(&ucgopt(), 100, 10, -10..20);
        // }

        // let g = Graph::gen(&ucgopt(), 10, 10, -10..20);

    }

}
