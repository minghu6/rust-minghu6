//! Shortest paths problem




////////////////////////////////////////////////////////////////////////////////
//// Structure

use std::collections::HashSet;

use super::Graph;
use crate::{
    apush,
    collections::{easycoll::{M1, MV}, heap::dary::DaryHeap5},
    concat, get, getopt, set, stack,
};


/// Floyd algorithm (DP) 全源最短路径
#[allow(unused)]
pub struct SPFlod<'a> {
    g: &'a Graph,
    /// shortest path weight
    spw: M1<(usize, usize), isize>,
    //// shortest path paths
    spp: MV<(usize, usize), usize>,
}


#[allow(unused)]
pub struct SPBellmanFord<'a> {
    g: &'a Graph,
    /// shortest path weight
    src: usize,
    spw: M1<usize, isize>,
    //// shortest path paths
    pre: M1<usize, usize>,
}


#[allow(unused)]
pub struct SPFA<'a> {
    g: &'a Graph,
    /// shortest path weight
    src: usize,
    spw: M1<usize, isize>,
    //// shortest path paths
    pre: M1<usize, usize>,
}


/// /ˈdaɪkstrəz/
#[allow(unused)]
pub struct SPDijkstra<'a> {
    g: &'a Graph,
    /// shortest path weight
    src: usize,
    spw: M1<usize, isize>,
    //// shortest path paths
    pre: M1<usize, usize>,
}



////////////////////////////////////////////////////////////////////////////////
//// Implementation

impl<'a> SPFA<'a> {
    pub fn new(g: &'a Graph, src: usize) -> Result<Self, Vec<usize>>{
        let (spw, pre) = sp_fa(g, src)?;

        Ok(Self { g, src, spw, pre })
    }

    pub fn query(&self, dst: usize) -> (isize, Vec<usize>) {
        (get!(self.spw => dst), pre_to_path(dst, &self.pre))
    }
}

impl<'a> SPBellmanFord<'a> {
    pub fn new(g: &'a Graph, src: usize) -> Result<Self, Vec<usize>>{
        let (spw, pre) = sp_bellman_ford(g, src)?;

        Ok(Self { g, src, spw, pre })
    }

    pub fn query(&self, dst: usize) -> (isize, Vec<usize>) {
        (get!(self.spw => dst), pre_to_path(dst, &self.pre))
    }
}

impl<'a> SPDijkstra<'a> {
    pub fn new(g: &'a Graph, src: usize) -> Result<Self, Vec<usize>>{
        let (spw, pre) = sp_dijkstra(g, src);

        Ok(Self { g, src, spw, pre })
    }

    pub fn query(&self, dst: usize) -> (isize, Vec<usize>) {
        (get!(self.spw => dst), pre_to_path(dst, &self.pre))
    }
}

impl<'a> SPFlod<'a> {
    pub fn new(g: &'a Graph) -> Self {
        let (spw, spp) = sp_floyd(g);

        SPFlod { g, spw, spp }
    }

    /// (total_weight, (from), .. , k1, k2, .. to)
    pub fn query(&self, from: usize, to: usize) -> (isize, Vec<usize>) {
        (get!(self.spw => (from, to)), get!(self.spp => (from, to)))
    }
}



////////////////////////////////////////////////////////////////////////////////
//// Function

pub fn sp_floyd(
    g: &Graph,
) -> (M1<(usize, usize), isize>, MV<(usize, usize), usize>) {
    let mut vertexs: Vec<usize> = g.vertexs().collect();
    vertexs.sort_unstable();

    let n = vertexs.len();
    let mut spw = M1::<(usize, usize), isize>::new();
    let mut spp = MV::<(usize, usize), usize>::new();

    let infi = isize::MAX / 2;

    /* init sp */
    for x in 0..n {
        for y in 0..n {
            let x = vertexs[x];
            let y = vertexs[y];

            if let Some(w) = getopt!(g.w => (x, y)) {
                set!(spw => (x, y) => w);
                apush!(spp => (x, y) => y); // 便于算法实现
            } else if x == y {
                set!(spw => (x, y) => 0);
                set!(spp => (x, y) => vec![]);
            } else {
                set!(spw => (x, y) => infi);
            }
        }
    }

    // println!("INIT SPP: {:?}", spp);

    for k in 0..n {
        for x in 0..n {
            for y in 0..n {
                if k == x || k == y || x == y {
                    continue;
                }

                let x = vertexs[x];
                let y = vertexs[y];
                let k = vertexs[k];

                let w = get!(spw => (x, k)) + get!(spw => (k, y));
                if w < get!(spw => (x, y)) {
                    set!(spw => (x, y) => w);
                    // 确保都可达
                    let p_x_k = get!(spp => (x, k));
                    let p_k_y = get!(spp => (k, y));

                    set!(spp => (x, y) => concat!(p_x_k => p_k_y));
                }
            }
        }
    }

    (spw, spp)
}



pub fn sp_bellman_ford(
    g: &Graph,
    src: usize
) -> Result<(M1<usize, isize>, M1<usize, usize>), Vec<usize>> {
    let mut pre = M1::new();
    let mut dis = M1::new();
    let n = g.vertexs().count();
    let infi = isize::MAX / 2;

    set!(dis => src => 0);

    for _ in 1..=n-1 {
        for (u, v, w) in g.edges() {
            let dis_u = get!(dis => u => infi);
            let dis_v = get!(dis => v => infi);

            if dis_u + w < dis_v {
                set!(dis => v => dis_u + w);
                set!(pre => v => u);
            }
        }
    }

    for (u, v, w) in g.edges() {
        let dis_u = get!(dis => u => infi);
        let dis_v = get!(dis => v => infi);

        if dis_u + w < dis_v {
            // negative cycle found!
            let mut c = v;

            // 确保在环上
            for _ in 1..=n {
                c = get!(pre => c);
            }

            let mut cycle = Vec::new();
            let mut cur = c;
            while cycle.is_empty() || cur != c {
                cycle.push(cur);
                cur = get!(pre => cur);
            }

            return Err(cycle)
        }
    }

    Ok((dis, pre))
}



/// Shortest Path Faster Algorithm, imporoved Bellman-Ford algorithm
///
/// O(ev), single source,
pub fn sp_fa(g: &Graph, src: usize) -> Result<(M1<usize, isize>, M1<usize, usize>), Vec<usize>> {
    let mut pre = M1::new();
    let mut dis = M1::new();
    let mut cnt: M1<usize, usize> = M1::new();  // 穿过的边数
    let n = g.vertexs().count();
    let infi = isize::MAX / 2;

    set!(dis => src => 0);

    // use stack instead of queue to quick find negative circle
    let mut vis = HashSet::new();
    let mut stack = stack![src];

    let mut c = None;

    while let Some(u) = stack.pop() {
        vis.remove(&u);

        for v in get!(g.e => u) {
            let w = get!(dis => u => infi) + get!(g.w => (u, v));
            if w < get!(dis => v => infi) {
                set!(dis => v => w);
                set!(pre => v => u);

                if get!(cnt => v => 0) >= n {
                    c = Some(v);
                    break;
                }

                set!(cnt => v => get!(cnt => u => 0) + 1);

                if !vis.contains(&v) {
                    stack.push(v);
                    vis.insert(v);
                }
            }
        }
    }

    if let Some(mut c) = c {
        for _ in 1..=n {
            c = get!(pre => c);
        }

        let mut cycle = Vec::new();
        let mut cur = c;
        while cycle.is_empty() || cur != c {
            cycle.push(cur);
            cur = get!(pre => cur);
        }

        Err(cycle)
    }
    else {
        Ok((dis, pre))
    }

}


pub fn sp_dijkstra(g: &Graph, src: usize) -> (M1<usize, isize>, M1<usize, usize>) {
    // let mut pre = M1::new();
    let mut dis = DaryHeap5::new();
    let infi = isize::MAX / 2;

    for v in g.vertexs() {
        if v == src {
            dis.insert(v, 0);
        }
        else {
            dis.insert(v, infi);
        }
    }

    while let Some((u, w_u)) = dis.pop_item() {
        for v in get!(g.e => u) {

            // dis.decrease_key(v);
        }
    }

    todo!()
}


pub(crate) fn pre_to_path(dst: usize, pre: &M1<usize, usize>) -> Vec<usize> {
    let mut path = Vec::new();

    let mut cur = dst;

    while let Some(prev) = getopt!(pre => cur) {
        path.push(cur);
        cur = prev;
    }

    path.reverse();

    path
}



#[cfg(test)]
mod tests {
    use std::fs::File;

    use super::{SPFlod, SPBellmanFord};
    use crate::{collections::graph::{Graph, sp::SPFA}, test::graph::{
        batch_graph,
        ucgopt, ucg_nncycle_opt
    }};

    fn get_graph(p: &str) -> Graph {
        let mut file = File::open(p).unwrap();
        let g = Graph::read_from_csv(&mut file).unwrap();
        g
    }

    #[test]
    fn test_sp_fixeddata() {
        /* simple classic positive graph */
        // let g = get_graph("res/sp1.csv");
        // let sp = SPFlod::new(&g);

        /* negative circle graph */
        // let g = get_graph("res/sp2.csv");
        // let sp = SPFlod::new(&g);

        let g = get_graph("res/sp3.csv");
        let sp = SPFlod::new(&g);
        println!("spp: {:?}", sp.spp);
    }

    #[test]
    fn test_sp_randomdata_pw() {
        let mut i = 0;

        for g in batch_graph(15, 50, 1..100, &ucgopt()) {
            let sp_flod = SPFlod::new(&g);

            for src in g.vertexs() {
                let sp_bellmanford = SPBellmanFord::new(&g, src).unwrap();
                let sp_fa = SPFA::new(&g, src).unwrap();

                for dst in g.vertexs() {
                    let (w_bellmanford, p_bellmanford) = sp_bellmanford.query(dst);
                    let (w_spfa, p_spfa) = sp_fa.query(dst);
                    let (w_flod, p_flod) = sp_flod.query(src, dst);

                    assert_eq!(w_bellmanford, w_flod);
                    assert_eq!(w_bellmanford, w_spfa);

                    g.verify_path(src, dst, &p_bellmanford).unwrap();
                    g.verify_path(src, dst, &p_flod).unwrap();
                    g.verify_path(src, dst, &p_spfa).unwrap();

                }
                // println!("     {src:03} pass.");
            }

            println!("g {i:03} pass.");
            i += 1;
        }
    }

    #[test]
    fn test_sp_randomdata_nw() {
        let graphs = batch_graph(25, 40, -15..85, &ucg_nncycle_opt());
        println!("generate graphs done.");
        for (i, g) in graphs.iter().enumerate() {
            println!("g {i:03} sparisity {}", g.sparisity(false));
        }
        println!();

        for (i, g) in graphs.into_iter().enumerate() {
            print!("-> {i:03} ...");

            let sp_flod = SPFlod::new(&g);

            for src in g.vertexs() {
                let sp_bellmanford = SPBellmanFord::new(&g, src).unwrap();
                let sp_fa = SPFA::new(&g, src).unwrap();

                for dst in g.vertexs() {
                    let (w_bellmanford, p_bellmanford) = sp_bellmanford.query(dst);
                    let (w_spfa, p_spfa) = sp_fa.query(dst);

                    let (w_flod, p_flod) = sp_flod.query(src, dst);

                    assert_eq!(w_bellmanford, w_flod);
                    assert_eq!(w_bellmanford, w_spfa);

                    g.verify_path(src, dst, &p_bellmanford).unwrap();
                    g.verify_path(src, dst, &p_flod).unwrap();
                    g.verify_path(src, dst, &p_spfa).unwrap();
                }
                // println!("     {src:03} pass.");
            }

            println!("pass");
        }
    }

    #[test]
    fn test_sp_negative_cycle_detect() {
        let mut i = 0;

        for g in batch_graph(20, 40, -40..60, &ucgopt()) {
            for src in g.vertexs() {
                if let Err(ncycle) = SPBellmanFord::new(&g, src) {
                    g.verify_negative_cycle(false, &ncycle).unwrap();
                }
                if let Err(ncycle) = SPFA::new(&g, src) {
                    g.verify_negative_cycle(false, &ncycle).unwrap();
                }
            }
            println!("g {i:03} pass.");
            i += 1;
        }
    }

}
