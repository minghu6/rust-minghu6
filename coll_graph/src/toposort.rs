//! Can used for detect ring
//!

use coll::get;

use crate::Graph;


impl Graph {
    #[cfg(test)]
    fn verify_toposort(&self, res: Option<Vec<usize>>) {
        debug_assert!(self.is_dir);

        if self.components().into_iter().any(|comp| comp.len() > 1) {
            assert!(res.is_none());
            return;
        }

        let mut cond = std::collections::HashSet::new();

        for u in res.unwrap() {
            for p in get!(self.rev => u => vec![]) {
                if !cond.contains(&p) {
                    panic!("Lacking cond {p} for {u}");
                }
            }

            cond.insert(u);
        }
    }
}



pub fn toposort_tarjan(g: &Graph) -> Result<Vec<usize>, (usize, usize)> {
    #[repr(u8)]
    #[derive(Default, Clone, Copy, PartialEq, Eq)]
    enum DFSMeta {
        #[default]
        Green,
        Yellow,
        Red,
    }
    use DFSMeta::*;

    fn dfs(
        g: &Graph,
        u: usize,
        vertexs: &mut Vec<DFSMeta>,
        ans: &mut Vec<usize>,
    ) -> Result<(), (usize, usize)> {
        vertexs[u] = Yellow;

        for v in get!(g.e => u => vec![]) {
            match vertexs[v] {
                Green => {
                    dfs(g, v, vertexs, ans)?;
                }
                Yellow => {
                    return Err((u, v)); // found edge on cycle
                }
                Red => (), // just skip
            }
        }

        vertexs[u] = Red;
        ans.push(u);

        Ok(())
    }

    let mut ans = Vec::new();

    let max_v = g.vertexs().max().unwrap();

    let mut vertexs = vec![DFSMeta::default(); max_v + 1];

    for u in g.vertexs() {
        if vertexs[u] == Green {
            dfs(g, u, &mut vertexs, &mut ans)?;
        }
    }

    ans.reverse();

    Ok(ans)
}



#[cfg(test)]
mod tests {
    use crate::{test::GraphGenOptions, toposort::toposort_tarjan, Graph};

    fn setup_dir_data() -> Vec<Graph> {
        let opt = GraphGenOptions {
            is_dir: true,
            allow_cycle: true,
            non_negative_cycle: false,
            weak_conn: false,
        };

        let mut gs = vec![];

        for _ in 0..50 {
            let g = Graph::gen(&opt, 30, 0, 1..2);
            gs.push(g);
        }

        gs
    }


    #[test]
    fn test_toposort() {
        for (i, g) in setup_dir_data().into_iter().enumerate() {
            // let topo_kahn = toposort_kahn(&g);
            let topo_dfs = toposort_tarjan(&g).ok();

            if topo_dfs.is_none() {
                println!("... g-{i:02}");
            } else {
                println!(">>> g-{i:02}");
            }

            // g.verify_toposort(topo_kahn);
            g.verify_toposort(topo_dfs);
        }
    }
}
