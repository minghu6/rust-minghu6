use std::{collections::HashSet, ops::Range};

use crate::{
    algs::{random, random_range},
    collections::{
        graph::{to_undirected_vec, Graph},
        union_find::{MergeBy, UnionFind},
    },
};



////////////////////////////////////////////////////////////////////////////////
//// Structure

pub struct GraphGenOptions {
    pub dir: bool,
    pub cyclic: bool,
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
    /// 1: n-1 edge
    /// 10: (n-1)(n-1) edge
    ///
    /// it ranges exponentially
    pub fn gen(
        opt: &GraphGenOptions,
        vrange: usize,
        sparsity: usize,
        wrange: Range<isize>,
    ) -> Graph {
        debug_assert!(1 <= sparsity && sparsity <= 10);

        if sparsity == 10 {
            if !opt.dir && opt.cyclic {
                return gen_scc_graph(vrange, wrange);
            }
            else {
                unreachable!()
            }
        }

        fn edges_limit(vrange: usize, sparsity: usize) -> usize {
            if vrange == 1 {
                return 0;
            }

            let sv_edge = vrange - 1;

            if sparsity == 1 {
                return sv_edge;
            }
            debug_assert!(sparsity != 10, "shouldn't generate complete connected graph");
            // if sparsity == 10 {
            //     return sv_edge * sv_edge;
            // }

            if sv_edge < 10 {
                let slice = ((sv_edge - 1) as f64) / 10.0;

                1 + (slice * sparsity as f64) as usize
            } else {
                let base = (sv_edge as f64).powf(0.1);

                sv_edge * base.powf(0.1 * sparsity as f64) as usize
            }
        }

        let edge_limit = edges_limit(vrange, sparsity);
        let sv_edge = vrange - 1;

        debug_assert!(
            sv_edge <= edge_limit && vrange <= sv_edge * sv_edge,
            "sv_edge: {sv_edge}, edge_limit: {edge_limit}"
        );

        let mut g = Graph::new();
        let mut dsu = UnionFind::new(Some(MergeBy::SZ));

        for v in 1..=vrange {
            dsu.insert(v);
        }

        for _ in 0..edge_limit {
            loop {
                let u = random::<usize>() % vrange + 1;
                let v = random::<usize>() % vrange + 1;

                if u == v {
                    continue;
                }

                if g.contains_edge((u, v)) {
                    continue;
                }

                dsu.cunion(u, v);

                let w = random_range(wrange.clone());
                g.insert_edge((u, v), w);

                if !opt.dir {
                    g.insert_edge((v, u), w);
                }

                break;
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
            debug_assert!(g.is_connected(), "Faield with ");
        }

        g
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

    for u in 1..=vrange {
        for v in 1..=vrange {
            if u == v { continue }

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
