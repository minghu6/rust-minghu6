
use std::{collections::HashSet, ops::Range};

use common::*;

use crate::sp::SPBellmanFord;
use crate::{sp::{SPFA, SPJohnson, SPFloyd}, to_undirected_vec, Graph, Path};

use coll::{
    apush,
    {
        easycoll::MS,
        union_find::{MergeBy, UnionFind},
    },
    del, set,
};


////////////////////////////////////////////////////////////////////////////////
//// Structure

#[derive(Debug)]
pub struct GraphGenOptions {
    pub dir: bool,
    /// if it's a tree
    pub cyclic: bool,
    pub non_negative_cycle: bool,
    pub weak: bool,
}



////////////////////////////////////////////////////////////////////////////////
//// Implementation

impl GraphGenOptions {
    pub fn non_negative_cycle(mut self) -> Self {
        self.non_negative_cycle = true;
        self
    }

    pub fn weak(mut self) -> Self {
        self.weak = true;
        self
    }
}


impl GraphGenOptions {
    /// 无向连通图
    pub fn undir_conn() -> Self {
        Self {
            dir: false,
            cyclic: true,
            non_negative_cycle: false,
            weak: false
        }
    }

    /// 有向强连通图
    pub fn dir_conn() -> Self {
        Self {
            dir: true,
            cyclic: true,
            non_negative_cycle: false,
            weak: false
        }
    }
}



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
            g.dir = opt.dir;
        } else {
            let mut max = vrange * (vrange - 1);
            let min = vrange - 1;
            // 无向图 or 强连通有向图
            max /= 2;

            let peace = (max - min) / 10;

            let edge_limit = min + sparsity * peace;

            g = Graph::new();
            g.dir = opt.dir;
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
                    .choose(&mut common::thread_rng())
                    .unwrap();

                let mut rem_vertexs = del!(rem_edges => u);

                let v = *rem_vertexs
                    .iter()
                    .choose(&mut common::thread_rng())
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

                let w = random_range!(wrange.clone());
                g.insert_edge((u, v), w);

                if !opt.dir {
                    g.insert_edge((v, u), w);
                }
                else if !opt.weak {
                    let w2;
                    if opt.non_negative_cycle && w < 0 {
                        w2 = w.abs() + random_range!(0..wrange.end);
                    }
                    else {
                        w2 = random_range!(wrange.clone());
                    }

                    g.insert_edge((v, u), w2);
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
                    let w = random_range!(wrange.clone());

                    g.insert_edge((u, v), w);
                    dsu.cunion(u, v);

                    if !opt.dir {
                        g.insert_edge((v, u), w);
                    }
                    else if !opt.weak {
                        let w2;
                        if opt.non_negative_cycle && w < 0 {
                            w2 = w.abs() + random_range!(0..wrange.end);
                        }
                        else {
                            w2 = random_range!(wrange.clone());
                        }

                        g.insert_edge((v, u), w2);
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
            g.dir = opt.dir;

        } else {
            if wrange.start < 0 && opt.non_negative_cycle {
                g.fix_negative_cycle_spfa(false);
            }
        }

        g
    }

    pub(crate) fn fix_one_negative_cycle_johnson(&mut self, g2: &mut Graph, cycle: &[usize], positive: bool) {
        g2.verify_negative_cycle(cycle).unwrap();

        let p = Path::from_cycle(&g2, cycle).freeze();
        let mut totw = p.weight();
        assert!(totw < 0);

        for (u, v, w) in p.iter() {
            if self.contains_edge((u, v)) {
                if w < 0 {
                    // reverse weight
                    totw += 2 * (-w);
                    set!(self.w => (u, v) => -w);

                    if !self.dir {
                        set!(self.w => (v, u) => -w);
                    }

                    if !positive {
                        if totw >= 0 {
                            break;
                        }
                    }
                }
            }
        }
    }

    pub(crate) fn fix_one_negative_cycle_normal(&mut self, cycle: &[usize], positive: bool) {
        self.verify_negative_cycle(cycle).unwrap();

        let p = Path::from_cycle(self, cycle).freeze();
        let mut totw = p.weight();
        assert!(totw < 0);

        for (u, v, w) in p.iter() {
            if w < 0 {
                // reverse weight
                totw += 2 * (-w);
                set!(self.w => (u, v) => -w);

                if !self.dir {
                    set!(self.w => (v, u) => -w);
                }

                if !positive {
                    if totw >= 0 {
                        break;
                    }
                }
            }
        }
    }

    pub fn fix_negative_cycle_floyd(&mut self, positive: bool) {
        loop {
            if let Err(cycle) = SPFloyd::new(&self) {
                self.fix_one_negative_cycle_normal(&cycle, positive);
            }
            else {
                break;
            }
        }
    }

    pub fn fix_negative_cycle_johnson(&mut self, positive: bool) {
        loop {
            if let Err((mut g2, cycle)) = SPJohnson::new(&self) {
                self.fix_one_negative_cycle_johnson(&mut g2, &cycle, positive);
            }
            else {
                break;
            }
        }
    }

    pub fn fix_negative_cycle_bellmanford(&mut self, positive: bool) {
        let vs: Vec<usize> = self.vertexs().collect();
        let n = vs.len();
        let mut i = 0;
        loop {
            if i >= n {
                break;
            }

            if let Err(cycle) = SPBellmanFord::new(&self, vs[i]) {
                self.fix_one_negative_cycle_normal(&cycle, positive);

                continue;
            } else {
                i += 1;
            }
        }
    }

    pub fn fix_negative_cycle_spfa(&mut self, positive: bool) {
        let vs: Vec<usize> = self.vertexs().collect();
        let n = vs.len();
        let mut i = 0;
        loop {
            if i >= n {
                break;
            }

            if let Err(cycle) = SPFA::new(&self, vs[i]) {
                self.fix_one_negative_cycle_normal(&cycle, positive);

                continue;
            } else {
                i += 1;
            }
        }
    }

}



////////////////////////////////////////////////////////////////////////////////
//// Function

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

    edges.into_iter().collect()
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
    use crate::{Graph, debug::graphviz::RenderOptions};


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
        for _g in batch_graph(
            100,
            100,
            1..50,
            &GraphGenOptions::undir_conn()
        ) {}
        // for _ in 0..100 {
        //     // let sparsity = random() % 10 + 1;

        //     // println!("sparsity: {sparsity}");

        //     let g = Graph::gen(&ucgopt(), 100, 10, -10..20);
        // }

        // let g = Graph::gen(&ucgopt(), 10, 10, -10..20);
    }
}
