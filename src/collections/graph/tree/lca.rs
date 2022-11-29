use std::collections::HashSet;

use super::{Graph, hpd::HPD};
use crate::{
    algs::math::uintlog2,
    collections::{
        easycoll::{M1, M2, MV},
        union_find::UnionFind,
    },
    get, getopt, m1, m2, set, mv, apush,
};

////////////////////////////////////////////////////////////////////////////////
//// Structure

/// Binary lifting, (improved version of native) 对数(2)式跳转
///
/// Obviously, it's online algorithm.
pub struct LCABL {
    data: LCABLD,
}

/// LCA Binaru Lifting Data
struct LCABLD {
    acs: M2<usize, usize, usize>,
    d: M1<usize, usize>,
}


/// LCA Tarjan [tray jn] using
pub struct LCATarjan<'a> {
    g: &'a Graph,
    start: usize,
    dsu: UnionFind<usize>,
}


////////////////////////////////////////////////////////////////////////////////
//// Implementation

impl LCABL {
    /// O(n*log(n))
    pub fn new(g: &Graph, start: usize) -> Self {
        let data = lca_bl_setup(g, start);

        Self { data }
    }

    /// O(logn) + online
    pub fn query(&self, mut p: usize, mut q: usize) -> usize {
        let d = &self.data.d;
        let acs = &self.data.acs;

        /* swap p to the depthest node */
        if get!(d => q) > get!(d => p) {
            (p, q) = (q, p);
        }

        /* binary lift q to the same depth of p */
        let mut diff = get!(d => p) - get!(d => q);
        let mut j = 0;
        while diff > 0 {
            // move base if the latest number is 1
            // So we can arrive at specific dsiatnce by 2 power
            if diff & 1 > 0 {
                p = get!(acs =>p, j);
            }

            j += 1;
            diff >>= 1
        }

        debug_assert_eq!(get!(d => p), get!(d => q));

        /* same path vertex return to avoid eval too big CA later */
        if p == q {
            return p;
        }

        /* binary lift p and q from high to low */

        for j in (0..uintlog2(get!(d => p)) as usize).rev() {
            // depth overflow or too big
            if getopt!(acs => p,j) != getopt!(acs => q,j) {
                p = get!(acs => p,j);
                q = get!(acs => q,j);
            }
        }

        p = get!(acs => p,0);
        q = get!(acs => q,0);

        debug_assert_eq!(p, q);

        p
    }
}



impl<'a> LCATarjan<'a> {
    pub fn new(g: &'a Graph, start: usize) -> Self {
        let mut dsu = UnionFind::new(None);

        for n in g.vertexs() {
            dsu.insert(n);
        }

        Self { dsu, g, start }
    }

    /// O((n+m) / m) + offline, 处理大量查询时有优势
    ///
    /// one pass query
    pub fn queries(&self, queries: &[(usize, usize)]) -> Vec<usize> {
        // get an independent version of usu for in-place find
        let mut dsu = self.dsu.clone();
        let mut res = vec![0; queries.len()];
        let mut visited = HashSet::new();

        let mut q = mv![];

        /* init q */
        for (i, (u, v)) in queries.into_iter().cloned().enumerate() {
            apush!(q => u => (i, v));
            apush!(q => v => (i, u));
        }

        fn tarjan(
            g: &Graph,
            u: usize,
            visited: &mut HashSet<usize>,
            dsu: &mut UnionFind<usize>,
            q: &MV<usize, (usize, usize)>,
            res: &mut Vec<usize>,
        )
        {
            visited.insert(u);

            /* dfs */
            for v in get!(g.e => u) {
                if !visited.contains(&v) {
                    tarjan(g, v, visited, dsu, q, res);
                    // println!("union {v} -> {u}");
                    // link v to u
                    dsu.cunion(u, v);
                }
            }

            /* query u-> ? */
            if let Some(tox) = getopt!(q => u) {
                for (qi, x) in tox {
                    if visited.contains(&x) {
                        // println!("dsu: {dsu:?}");
                        res[qi] = dsu.cfind(x);
                    }
                }
            }
        }

        tarjan(self.g, self.start, &mut visited, &mut dsu, &q, &mut res);

        res
    }
}


impl HPD {

    /// O(log(n))
    ///
    /// 同重链， O(1); 不同重链，O(log2(n))
    pub fn lca(&self, mut x: usize, mut y: usize) -> usize {
        /* x, topx is always deptheset */

        if get!(self.d => x) < get!(self.d => y) {
            (x, y) = (y, x)
        }
        let mut topx = get!(self.top => x);
        let mut topy = get!(self.top => y);

        while topx != topy {
            if get!(self.d => topx) < get!(self.d => topy) {
                (topx, topy) = (topy, topx)
            }

            x = get!(self.p => topx);
            topx = get!(self.top => x);
        }

        if get!(self.d => x) < get!(self.d => y) {
            (_, y) = (y, x)
        }

        y
    }

}


////////////////////////////////////////////////////////////////////////////////
//// Function



/// -> (depth map, ancestors map)
fn lca_bl_setup(g: &Graph, start: usize) -> LCABLD {
    /// Recursive version of LCA setup data
    fn setup_dfs_r(g: &Graph, u: usize, p: usize, data: LCABLD) -> LCABLD {
        let LCABLD { mut acs, mut d } = data;

        // println!("-> {u}");

        for v in get!(g.e => u) {
            if v == p {
                // println!("skip-{v}, ");
                continue;
            }
            // println!("{v}, ");

            let curd = get!(d => u => 0) + 1;
            set!(d => v => curd);

            let mut i = curd >> 1;
            let mut j = 1;
            set!(acs => v, 0 => u);

            while i > 0 {
                // acs[i][j] = acs[acs[i][j-1]][j-1]
                let oldstate = get!(acs => get!(acs => v, j-1), j-1);
                set!(acs => v, j => oldstate);

                i >>= 1;
                j += 1;
            }

            let data = setup_dfs_r(g, v, u, LCABLD { acs, d });
            acs = data.acs;
            d = data.d;
        }

        LCABLD { acs, d }
    }

    setup_dfs_r(
        g,
        start,
        start,
        LCABLD {
            acs: m2![start => 0 => start,],
            d: m1![start => 0,],
        },
    )
}

