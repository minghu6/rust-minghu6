//! Shortest paths problem

////////////////////////////////////////////////////////////////////////////////
//// Structure

use std::collections::HashSet;

use coll::{
    apush,
    easycoll::{M1, M2},
    get, getopt, set, stack,
};
use coll_heap::dary::DaryHeap5;

use super::Graph;


macro_rules! pre_to_path {
    ($dst:expr, $pre:expr) => {
        {
            let pre = $pre;
            let dst = $dst;

            let mut path = indexmap::IndexSet::new();

            let mut cur = dst;

            while let Some(prev) = coll::rgetopt!(pre => cur) && !path.contains(&cur) {
                path.insert(cur);
                cur = prev;
            }

            path.into_iter().rev().collect::<Vec<usize>>()
        }

    };
}
pub(crate) use pre_to_path;


/// Floyd algorithm (DP) 全源最短路径
#[allow(unused)]
pub struct SPFloyd<'a> {
    g: &'a Graph,
    /// shortest path weight
    spw: M1<(usize, usize), isize>,
    //// shortest path paths
    next: M2<usize, usize, usize>,
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


#[allow(unused)]
pub struct SPJohnson<'a> {
    g: &'a Graph,
    h: M1<usize, isize>,
    spw: M2<usize, usize, isize>,
    //// shortest path paths
    sppre: M2<usize, usize, usize>,
}



////////////////////////////////////////////////////////////////////////////////
//// Implementation

impl<'a> SPFA<'a> {
    pub fn new(g: &'a Graph, src: usize) -> Result<Self, Vec<usize>> {
        let (spw, pre) = sp_fa_early_termination(g, src)?;

        Ok(Self { g, src, spw, pre })
    }

    pub fn query(&self, dst: usize) -> (isize, Vec<usize>) {
        (get!(self.spw => dst), pre_to_path!(dst, &self.pre))
    }
}


impl<'a> SPBellmanFord<'a> {
    pub fn new(g: &'a Graph, src: usize) -> Result<Self, Vec<usize>> {
        let (spw, pre) = sp_bellman_ford(g, src)?;

        Ok(Self { g, src, spw, pre })
    }

    pub fn query(&self, dst: usize) -> (isize, Vec<usize>) {
        (get!(self.spw => dst), pre_to_path!(dst, &self.pre))
    }
}


impl<'a> SPDijkstra<'a> {
    pub fn new(g: &'a Graph, src: usize) -> Self {
        let (spw, pre) = sp_dijkstra(g, src);

        Self { g, src, spw, pre }
    }

    pub fn query(&self, dst: usize) -> (isize, Vec<usize>) {
        (get!(self.spw => dst), pre_to_path!(dst, &self.pre))
    }
}


/// 对于无向图，探不到负环，或者说每条边都是负环
impl<'a> SPFloyd<'a> {
    pub fn new(g: &'a Graph) -> Result<Self, Vec<usize>> {
        let (spw, next) = sp_floyd(g)?;

        Ok(Self { g, spw, next })
    }

    /// (total_weight, (src), .. , k1, k2, .. dst)
    pub fn query(&self, src: usize, dst: usize) -> (isize, Vec<usize>) {
        (
            get!(self.spw => (src, dst)),
            next_to_path((src, dst), &self.next),
        )
    }
}


impl<'a> SPJohnson<'a> {
    pub fn new(g: &'a Graph) -> Result<Self, (Graph, Vec<usize>)> {
        let (h, spw, sppre) = sp_johnson(g)?;

        Ok(Self { g, h, spw, sppre })
    }

    /// (total_weight, (src), .. , k1, k2, .. dst)
    pub fn query(&self, src: usize, dst: usize) -> (isize, Vec<usize>) {
        (
            get!(self.spw => (src, dst)) - get!(self.h => src)
                + get!(self.h => dst),
            pre_to_path!(dst, self.sppre.0.get(&src).unwrap()),
        )
    }
}


////////////////////////////////////////////////////////////////////////////////
//// Function

fn sp_floyd(
    g: &Graph,
) -> Result<(M1<(usize, usize), isize>, M2<usize, usize, usize>), Vec<usize>> {
    let mut vertexs: Vec<usize> = g.vertexs().collect();
    vertexs.sort_unstable();

    let n = vertexs.len();
    let mut spw = M1::<(usize, usize), isize>::new();
    let mut next = M2::<usize, usize, usize>::new();

    /* init sp */
    for x in 0..n {
        for y in 0..n {
            let x = vertexs[x];
            let y = vertexs[y];

            if let Some(w) = getopt!(g.w => (x, y)) {
                set!(spw => (x, y) => w);
                set!(next => (x, y) => y); // 便于算法实现
            } else if x == y {
                set!(spw => (x, y) => 0);
            }
        }
    }

    for k in 0..n {
        for x in 0..n {
            for y in 0..n {
                let x = vertexs[x];
                let y = vertexs[y];
                let k = vertexs[k];

                if let Some(w_xk) = getopt!(spw => (x, k)) &&
                   let Some(w_ky) = getopt!(spw => (k, y))
                {
                    if getopt!(spw => (x, y)).is_none() || w_xk + w_ky < get!(spw => (x, y)) {
                        set!(spw => (x, y) => w_xk + w_ky);
                        set!(next => (x, y) => get!(next => (x, k)));

                        if x == y && w_xk + w_ky < 0 {
                            let mut cycle = vec![x];
                            let mut c = get!(next => (x, y));

                            while c != y {
                                cycle.push(c);
                                c = get!(next => (c, y));
                            }

                            return Err(cycle);
                        }
                    }
                }
            }
        }
    }

    Ok((spw, next))
}


fn sp_bellman_ford(
    g: &Graph,
    src: usize,
) -> Result<(M1<usize, isize>, M1<usize, usize>), Vec<usize>> {
    let mut pre = M1::new();
    let mut dis = M1::new();
    let n = g.vertexs().count();

    set!(dis => src => 0);

    for _ in 1..=n - 1 {
        for (u, v, w) in g.edges() {
            if let Some(dis_u) = getopt!(dis => u) {
                if getopt!(dis => v).is_none() || dis_u + w < get!(dis => v) {
                    set!(dis => v => dis_u + w);
                    set!(pre => v => u);
                }
            }
        }
    }

    for (u, v, w) in g.edges() {
        if let Some(dis_u) = getopt!(dis => u) {
            if getopt!(dis => v).is_none() || dis_u + w < get!(dis => v) {
                // negative cycle found!
                let mut c = v;

                // 确保在环上
                for _ in 1..=n {
                    c = get!(pre => c);
                }

                return Err(install_cycle(c, &pre));
            }
        }
    }

    Ok((dis, pre))
}



/// Shortest Path Faster Algorithm, imporoved Bellman-Ford algorithm
///
/// O(ev), single source,
pub fn sp_fa(
    g: &Graph,
    src: usize,
) -> Result<(M1<usize, isize>, M1<usize, usize>), Vec<usize>> {
    let mut pre = M1::new();
    let mut dis = M1::new();
    let mut cnt: M1<usize, usize> = M1::new(); // 穿过的边数
    let n = g.vertexs().count();

    set!(dis => src => 0);

    // use stack instead of queue dst quick find negative circle
    let mut vis = HashSet::new();
    let mut stack = stack![src];

    let mut c = None;

    'outer: while let Some(u) = stack.pop() {
        vis.remove(&u);
        // println!("test through {u}");

        for v in get!(g.e => u => vec![]) {
            // 无向图不存在这条路径
            if !g.is_dir && getopt!(pre => u) == Some(v) {
                continue;
            }

            if let Some(dis_u) = getopt!(dis => u) {
                if getopt!(dis => v).is_none()
                    || dis_u + get!(g.w => (u, v)) < get!(dis => v)
                {
                    set!(dis => v => dis_u + get!(g.w => (u, v)));
                    set!(pre => v => u);
                    set!(cnt => v => get!(cnt => u => 0) + 1);

                    if get!(cnt => v => 0) >= n {
                        c = Some(v);
                        break 'outer;
                    }

                    if !vis.contains(&v) {
                        stack.push(v);
                        vis.insert(v);
                    }
                }
            }
        }

        // /* print pre */
        // for v in g.vertexs() {
        //     println!("{src}->{v}: {:?}", pre_to_path!(v, &pre));
        // }
        // println!("=============== round {r:02} end  ==============\n");
        // r += 1;
    }

    if let Some(mut c) = c {
        for _ in 1..=n {
            c = get!(pre => c);
        }

        Err(install_cycle(c, &pre))
    } else {
        Ok((dis, pre))
    }
}


/// Shortest Path Faster Algorithm, imporoved Bellman-Ford algorithm
///
/// O(ev), single source,
pub fn sp_fa_early_termination(
    g: &Graph,
    src: usize,
) -> Result<(M1<usize, isize>, M1<usize, usize>), Vec<usize>> {
    let mut pre = M1::new();
    let mut dis = M1::new();
    let n = g.vertexs().count();

    set!(dis => src => 0);

    // use stack instead of queue dst quick find negative circle
    let mut vis = HashSet::new();
    let mut stack = vec![src];

    let mut i = 0;

    while let Some(u) = stack.pop() {
        vis.remove(&u);

        for v in get!(g.e => u => vec![]) {
            // 无向图不存在这条路径
            if !g.is_dir && getopt!(pre => u) == Some(v) {
                continue;
            }

            if let Some(dis_u) = getopt!(dis => u) {
                if getopt!(dis => v).is_none()
                    || dis_u + get!(g.w => (u, v)) < get!(dis => v)
                {
                    set!(dis => v => dis_u + get!(g.w => (u, v)));
                    set!(pre => v => u);

                    i += 1;
                    if i == n {
                        if let Some(cycle) = detect_negative_cycle(n, v, &pre)
                        {
                            return Err(cycle);
                        }

                        i = 0;
                    }

                    if !vis.contains(&v) {
                        stack.push(v);
                        vis.insert(v);
                    }
                }
            }
        }
    }

    Ok((dis, pre))
}


fn sp_dijkstra(g: &Graph, src: usize) -> (M1<usize, isize>, M1<usize, usize>) {
    let mut pre = M1::new();
    let mut dis_m1 = M1::new();

    let mut dis = DaryHeap5::new();
    let mut rest = HashSet::new();

    let infi = isize::MAX / 2;

    for v in g.vertexs() {
        rest.insert(v);
        if v == src {
            dis.insert(v, 0);
        } else {
            dis.insert(v, infi);
        }
    }

    while let Some((u, dis_u)) = dis.pop_item() {
        rest.remove(&u);
        set!(dis_m1 => u => dis_u);

        for v in get!(g.e => u) {
            if rest.contains(&v) {
                let w = dis_u + get!(g.w => (u, v));
                if w < *get!(dis => v) {
                    dis.decrease_key(v, w);
                    set!(pre => v => u);
                }
            }
        }
    }

    (dis_m1, pre)
}


fn sp_johnson(
    g: &Graph,
) -> Result<
    (
        M1<usize, isize>,
        M2<usize, usize, isize>,
        M2<usize, usize, usize>,
    ),
    (Graph, Vec<usize>),
> {
    /* Create a new map with special node q*/
    let mut g2 = g.clone();

    let mut vertexs: Vec<usize> = g2.vertexs().collect();
    vertexs.sort_unstable();

    let q = *vertexs.last().unwrap() + 1;
    for v in vertexs.iter().cloned() {
        set!(g2.w => (q, v) => 0);
        apush!(g2.e => q => v);
    }

    /* Using SPFA dst calc h((q, v)) */
    let h = match sp_fa(&g2, q) {
        Ok((h, _)) => h,
        Err(cycle) => return Err((g2, cycle)),
    };

    /* Reweight */
    for (u, v, w) in g.edges() {
        set!(g2.w => (u, v) => w + get!(h => u) - get!(h => v));
    }

    /* Calc spw and spp usign Dijkstra */

    let mut spw = M2::<usize, usize, isize>::new();
    let mut sppre = M2::<usize, usize, usize>::new();

    for v in vertexs {
        let (sspw, sspp) = sp_dijkstra(&g2, v);

        set!(spw => v => sspw);
        set!(sppre => v => sspp);
    }

    Ok((h, spw, sppre))
}


fn next_to_path(
    edge: (usize, usize),
    next: &M2<usize, usize, usize>,
) -> Vec<usize> {
    let (src, dst) = edge;

    let mut path = vec![];
    let mut cur = src;

    while cur != dst {
        cur = get!(next => (cur, dst));
        path.push(cur);
    }

    path
}


fn install_cycle(c: usize, pre: &M1<usize, usize>) -> Vec<usize> {
    let mut cycle = vec![c];
    let mut cur = get!(pre => c);

    while cur != c {
        cycle.push(cur);
        cur = get!(pre => cur);
    }

    cycle.reverse();

    cycle
}

/// for pre array
fn detect_negative_cycle(
    n: usize,
    mut cur: usize,
    pre: &M1<usize, usize>,
) -> Option<Vec<usize>> {
    let mut i = 0;

    let c = loop {
        if let Some(next) = getopt!(pre => cur) {
            cur = next;
        } else {
            break None;
        }

        i += 1;

        if i >= n {
            break Some(cur);
        }
    };

    c.and_then(|c| Some(install_cycle(c, pre)))
}



#[cfg(test)]
mod tests {
    use coll::{ min, same };

    use super::{SPBellmanFord, SPFloyd};
    use crate::{
        test:: { batch_graph, GraphGenOptions },
        sp::{SPDijkstra, SPJohnson, SPFA},
        Graph,
    };


    #[test]
    fn test_sp_fixeddata() {
        let mut g = Graph::read_from_csv_file("../res/sp5.csv").unwrap();
        g.is_dir = false;
        assert!(g.is_connected());

        if let Err(cycle) = SPFA::new(&g, 8) {
            if let Err(_err) = g.verify_negative_cycle(&cycle) {
                println!("detect negative cycle failed {:?}", cycle);
                assert!(false);
            } else {
                println!("detect negative cycle {:?}", cycle);
                // flag = true;
            }
        }

        if let Err((g2, cycle)) = SPJohnson::new(&g) {
            if let Err(_err) = g2.verify_negative_cycle(&cycle) {
                println!("detect negative cycle failed {:?} for G2", cycle);
                assert!(false);
            } else {
                println!("detect negative cycle {:?}", cycle);
                // flag = true;
            }
        }
    }


    #[test]
    fn test_sp_randomdata_pw() {
        let mut i = 0;

        for g in batch_graph(45, 30, 1..100, &GraphGenOptions::undir_conn()) {
            let sp_flod = SPFloyd::new(&g).unwrap();
            let sp_johnson = SPJohnson::new(&g).unwrap();

            for src in g.vertexs() {
                let sp_bellmanford = SPBellmanFord::new(&g, src).unwrap();
                let sp_fa = SPFA::new(&g, src).unwrap();
                let sp_dijkstra = SPDijkstra::new(&g, src);

                for dst in g.vertexs() {
                    let (w_bellmanford, p_bellmanford) =
                        sp_bellmanford.query(dst);
                    let (w_spfa, p_spfa) = sp_fa.query(dst);
                    let (w_flod, p_flod) = sp_flod.query(src, dst);
                    let (w_johnson, p_johnson) = sp_johnson.query(src, dst);
                    let (w_spdijkstra, p_spdijkstra) = sp_dijkstra.query(dst);

                    g.verify_path(src, dst, &p_bellmanford).unwrap();
                    g.verify_path(src, dst, &p_flod).unwrap();
                    g.verify_path(src, dst, &p_spfa).unwrap();
                    g.verify_path(src, dst, &p_spdijkstra).unwrap();
                    g.verify_path(src, dst, &p_johnson).unwrap();

                    assert_eq!(w_bellmanford, w_flod);
                    assert_eq!(w_bellmanford, w_spfa);
                    assert_eq!(w_bellmanford, w_spdijkstra);
                    assert_eq!(w_bellmanford, w_johnson);
                }
            }

            println!("g {i:03} pass");
            i += 1;
        }
    }


    #[test]
    fn test_sp_randomdata_nw() {
        let graphs = batch_graph(
            25,
            35,
            -35..65,
            &GraphGenOptions::undir_conn().non_negative_cycle(),
        );
        println!("Generate Graphs Done:");

        let graphs: Vec<Graph> = graphs
            .into_iter()
            .filter(|g| {
                let negative_edges =
                    g.w.0.values().filter(|&&w| w < 0).count();
                negative_edges > 0
            })
            .collect();

        for (i, g) in graphs.iter().enumerate() {
            // assert!(g.is_connected());
            let negative_edges = g.w.0.values().filter(|&&w| w < 0).count();

            println!("g {i:03} sparisity {:02}, negative edges: {negative_edges:02}",
                g.sparisity()
            );
        }
        println!();

        for (i, g) in graphs.into_iter().enumerate() {
            print!("-> {i:03} ...");

            let sp_johnson = match SPJohnson::new(&g) {
                Ok(it) => it,
                Err(_) => {
                    println!(
                        "Found negative cycle for spjohnson... skip {i:03}"
                    );
                    continue;
                }
            };
            let sp_floyd = match SPFloyd::new(&g) {
                Ok(it) => it,
                Err(_) => {
                    println!(
                        "Found negative cycle for spfloyd... skip {i:03}"
                    );
                    continue;
                }
            };

            for src in g.vertexs() {
                let sp_bellmanford = SPBellmanFord::new(&g, src).unwrap();
                let sp_fa = SPFA::new(&g, src).unwrap();

                for dst in g.vertexs() {
                    let (w_bellmanford, p_bellmanford) =
                        sp_bellmanford.query(dst);
                    let (w_spfa, p_spfa) = sp_fa.query(dst);
                    let (w_johnson, p_johnson) = sp_johnson.query(src, dst);
                    let (w_floyd, p_floyd) = sp_floyd.query(src, dst);

                    g.verify_path(src, dst, &p_bellmanford).unwrap();
                    g.verify_path(src, dst, &p_spfa).unwrap();
                    g.verify_path(src, dst, &p_johnson).unwrap();
                    g.verify_path(src, dst, &p_floyd).unwrap();

                    let w_min = min!(w_floyd, w_johnson, w_spfa);

                    if !same!(w_floyd, w_spfa, w_johnson, w_bellmanford) {
                        println!("w_min: {w_min}");
                        println!("w_floyd: {w_floyd}");
                        println!("w_johnson: {w_johnson}");
                        println!("w_spfa: {w_spfa}");
                        println!("w_bellmanford: {w_bellmanford}");

                        assert!(false);
                    }
                }
                // println!("pass {src}")
            }

            println!("pass");
        }
    }


    #[test]
    fn test_sp_negative_cycle_detect() {
        let mut i = 0;

        for g in batch_graph(45, 35, -40..60, &GraphGenOptions::dir_conn()) {
            for src in g.vertexs() {
                if let Err(cycle) = SPBellmanFord::new(&g, src) {
                    g.verify_negative_cycle(&cycle).unwrap();
                }
                if let Err(cycle) = SPFA::new(&g, src) {
                    g.verify_negative_cycle(&cycle).unwrap();
                }
            }
            if let Err((_g2, cycle)) = SPJohnson::new(&g) {
                g.verify_negative_cycle(&cycle).unwrap();
            }
            if let Err(cycle) = SPFloyd::new(&g) {
                g.verify_negative_cycle(&cycle).unwrap();
            }
            println!("g {i:03} pass");
            i += 1;
        }
    }

    fn detect_negative_cycle(g: &Graph) {
        let mut flags = false;
        for src in g.vertexs() {
            if let Err(cycle) = SPBellmanFord::new(&g, src) {
                eprintln!("SPBellmanFord detect it!");
                g.verify_negative_cycle(&cycle).unwrap();
                flags = true;
            }
            if let Err(cycle) = SPFA::new(&g, src) {
                eprintln!("SPFA detect it!");
                g.verify_negative_cycle(&cycle).unwrap();
                flags = true;
            }
        }
        if let Err((g2, cycle)) = SPJohnson::new(&g) {
            eprintln!("SPJohnson detect it!");
            g2.verify_negative_cycle(&cycle).unwrap();
            flags = true;
        }
        if let Err(cycle) = SPFloyd::new(&g) {
            eprintln!("SPFloyd detect it!");
            g.verify_negative_cycle(&cycle).unwrap();
            flags = true;
        }

        assert!(!flags);
    }

    #[test]
    fn test_sp_negative_cycle_fix_floyd() {
        let mut i = 0;

        for mut g in batch_graph(45, 35, -40..60, &GraphGenOptions::dir_conn())
        {
            g.fix_negative_cycle_floyd(true);

            let negative_edges = g.edges().filter(|&x| x.2 < 0).count();

            detect_negative_cycle(&g);

            println!("g {i:03} pass negative_edges: {negative_edges}");
            i += 1;
        }
    }

    #[test]
    fn test_sp_negative_cycle_fix_bellmanford() {
        let mut i = 0;

        for mut g in batch_graph(45, 35, -40..60, &GraphGenOptions::dir_conn())
        {
            g.fix_negative_cycle_bellmanford(true);

            let negative_edges = g.edges().filter(|&x| x.2 < 0).count();

            detect_negative_cycle(&g);

            println!("g {i:03} pass negative_edges: {negative_edges}");
            i += 1;
        }
    }

    #[test]
    fn test_sp_negative_cycle_fix_spfa() {
        let mut i = 0;

        for mut g in batch_graph(45, 35, -40..60, &GraphGenOptions::dir_conn())
        {
            g.fix_negative_cycle_spfa(true);

            let negative_edges = g.edges().filter(|&x| x.2 < 0).count();

            detect_negative_cycle(&g);

            println!("g {i:03} pass negative_edges: {negative_edges}");
            i += 1;
        }
    }

    #[test]
    fn test_sp_negative_cycle_fix_johnson() {
        let mut i = 0;

        for mut g in batch_graph(45, 35, -40..60, &GraphGenOptions::dir_conn())
        {
            g.fix_negative_cycle_johnson(true);

            let negative_edges = g.edges().filter(|&x| x.2 < 0).count();

            detect_negative_cycle(&g);

            println!("g {i:03} pass negative_edges: {negative_edges}");
            i += 1;
        }
    }
}
