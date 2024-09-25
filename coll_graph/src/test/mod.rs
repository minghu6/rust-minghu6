use std::{collections::HashSet, ops::Range};

use coll::union_find::{MergeBy, UnionFind};
use common::*;

use crate::{
    to_undirected_vec, Graph,
};


pub mod path;
mod sp;
mod verify;

////////////////////////////////////////////////////////////////////////////////
//// Structures

#[derive(Debug)]
pub struct GraphGenOptions {
    pub is_dir: bool,
    pub allow_cycle: bool,
    pub non_negative_cycle: bool,
    pub weak_conn: bool,
}



////////////////////////////////////////////////////////////////////////////////
//// Implementations

impl GraphGenOptions {
    pub fn non_negative_cycle(mut self) -> Self {
        self.non_negative_cycle = true;
        self
    }

    pub fn weak_conn(mut self) -> Self {
        self.weak_conn = true;
        self
    }
}


impl GraphGenOptions {
    /// 无向连通图
    pub fn undir_conn() -> Self {
        Self {
            is_dir: false,
            allow_cycle: true,
            non_negative_cycle: false,
            weak_conn: true,
        }
    }

    /// 有向强连通图
    pub fn dir_conn() -> Self {
        Self {
            is_dir: true,
            allow_cycle: true,
            non_negative_cycle: false,
            weak_conn: true,
        }
    }
}



impl Graph {
    /// Generate simple graph
    ///
    /// vertex in 1..=vrange
    ///
    /// weight in wrange
    ///
    /// sparsity: 0-10,
    ///
    ///
    /// it ranges exponentially
    pub fn gen(
        opt: &GraphGenOptions,
        vrange: usize,
        sparsity: usize,
        wrange: Range<isize>,
    ) -> Graph {
        debug_assert!(sparsity <= 10);
        let mut g;

        if sparsity == 10 {
            assert!(opt.allow_cycle);
            g = gen_scc_graph(vrange, wrange.clone());
            g.is_dir = opt.is_dir;
        } else {
            let mut max = vrange * (vrange - 1);
            let min = vrange - 1;
            // 无向图 or 强连通有向图
            max /= 2;

            let piece = (max - min) / 10;

            let edge_limit = min + sparsity * piece;

            g = Graph::new();
            g.is_dir = opt.is_dir;

            let mut dsu = UnionFind::new(Some(MergeBy::SZ));

            for i in 1..=vrange {
                dsu.insert(i);
            }

            let mut rem_edges = HashSet::new();

            for u in 1..=vrange-1 {
                for v in u+1..=vrange {
                    rem_edges.insert((u, v));
                    rem_edges.insert((v, u));
                }
            }

            for _ in 0..edge_limit {
                let (u, v) = rem_edges
                    .iter()
                    .choose(&mut common::thread_rng())
                    .cloned()
                    .unwrap();
                let w = random_range!(wrange.clone());

                g.insert((u, v), w);
                rem_edges.remove(&(u, v));
                dsu.cunion(u, v);

                if !opt.is_dir {
                    g.insert((v, u), w);
                    rem_edges.remove(&(v, u));
                }
            }

            if opt.weak_conn {
                /* ensure weak connected */

                let mut comps = HashSet::new();

                for v in 1..=vrange {
                    comps.insert(dsu.cfind(v));
                }

                if comps.len() > 1 {
                    let mut comps_iter = comps.into_iter();

                    let u = comps_iter.next().unwrap();

                    for v in comps_iter {
                        let w = random_range!(wrange.clone());

                        g.insert((u, v), w);
                        dsu.cunion(u, v);

                        if !opt.is_dir {
                            g.insert((v, u), w);
                        } else {
                            let w2;
                            if opt.non_negative_cycle && w < 0 {
                                w2 = w.abs() + random_range!(0..wrange.end);
                            } else {
                                w2 = random_range!(wrange.clone());
                            }

                            g.insert((v, u), w2);
                        }
                    }
                }
            }
        }

        /* gen spanning tree */

        if !opt.allow_cycle {
            let mut visited = HashSet::new();
            let mut edges = vec![];

            for (u, v, w) in g.edges() {
                if !visited.contains(&u) || !visited.contains(&v) {
                    visited.insert(u);
                    visited.insert(v);

                    edges.push((u, v, w));
                }
            }

            if !opt.is_dir {
                edges = to_undirected_vec(edges);
            }

            if opt.is_dir {
                g = Graph::from_directed_iter(edges);
            }
            else {
                g = Graph::from_directed_iter(edges);
            }
        } else {
            if wrange.start < 0 && opt.non_negative_cycle {
                g.fix_negative_cycle_spfa(false);
            }
        }

        g
    }

}



////////////////////////////////////////////////////////////////////////////////
//// Functions

/// Generate (Undirected) Simple Completed Connected Graph
fn gen_scc_graph(vrange: usize, wrange: Range<isize>) -> Graph {
    let mut edges = vec![];

    for u in 1..vrange {
        for v in u + 1..=vrange {
            let w = random_range!(wrange.clone());
            edges.push((u, v, w));
            edges.push((v, u, w));
        }
    }

    Graph::from_undirected_iter(edges)
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
    use crate::{debug::graphviz::RenderOptions, Graph};


    #[test]
    fn test_gen_ud_graph() {
        let g = Graph::gen(&GraphGenOptions::undir_conn(), 30, 4, 1..15);

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
        for _g in batch_graph(100, 100, 1..50, &GraphGenOptions::undir_conn())
        {
        }
        // for _ in 0..100 {
        //     // let sparsity = random() % 10 + 1;

        //     // println!("sparsity: {sparsity}");

        //     let g = Graph::gen(&ucgopt(), 100, 10, -10..20);
        // }

        // let g = Graph::gen(&ucgopt(), 10, 10, -10..20);
    }
}
