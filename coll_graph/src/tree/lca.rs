use std::collections::HashSet;

use coll::{
    apush, get, getopt, m1, mv, set,
    {
        easycoll::{M1, MV},
        union_find::UnionFind,
    },
};

use super::{hpd::HPD, Graph};

////////////////////////////////////////////////////////////////////////////////
//// Structure

/// Binary lifting, (improved version of native) 对数(2)式跳转
///
/// Obviously, it's online algorithm.
pub struct LCADP {
    data: LCADPD,
}

/// LCA Binary Lifting Data
struct LCADPD {
    /// u, 2^(idx) th ancestor, ancestor id
    acs: MV<usize, usize>,
    /// depth
    depth: M1<usize, usize>,
}


/// LCA Tarjan using
pub struct LCATarjan<'a> {
    g: &'a Graph,
    root: usize,
    dsu: UnionFind<usize>,
}


////////////////////////////////////////////////////////////////////////////////
//// Implementation

impl LCADP {
    /// O(n*log(n))
    pub fn new(g: &Graph, root: usize) -> Self {
        /// Recursive version of LCA setup data
        fn setup_dfs_r(g: &Graph, u: usize, p: usize, data: LCADPD) -> LCADPD {
            let LCADPD { mut acs, mut depth } = data;

            for v in get!(g.e => u) {
                if v == p {
                    continue;
                }

                let v_depth = get!(depth => u => 0) + 1;

                set!(depth => v => v_depth);
                apush!(acs => v => u);

                for j in 1..=v_depth.ilog2() as usize {
                    // acs[i][j] = acs[acs[i][j-1]][j-1]
                    let oldstate
                    = get!(acs =>
                        get!(acs =>
                            v,
                            j-1
                        ),
                        j-1
                    );
                    apush!(acs => v => oldstate);
                }

                let data = setup_dfs_r(g, v, u, LCADPD { acs, depth });

                acs = data.acs;
                depth = data.depth;
            }

            LCADPD { acs, depth }
        }

        let data = setup_dfs_r(
            g,
            root,
            root,
            LCADPD {
                acs: mv![root => 0 => root],
                depth: m1![root => 0],
            },
        );

        Self { data }
    }

    /// O(logn) + online
    pub fn query(&self, mut p: usize, mut q: usize) -> usize {
        let depth = &self.data.depth;
        let acs = &self.data.acs;

        /* swap p to the depthest node */

        if get!(depth => q) > get!(depth => p) {
            (p, q) = (q, p);
        }

        /* binary lift q to the same depth of p */

        let mut diff = get!(depth => p) - get!(depth => q);
        let mut j = 0;

        /* binary decomposition of `diff`
            accumulate from low bit to high bit
        */

        while diff > 0 {

            if diff & 1 > 0 {
                p = get!(acs => p, j);
            }

            j += 1;
            diff >>= 1
        }

        debug_assert_eq!(get!(depth => p), get!(depth => q));

        if p == q {
            return p;
        }

        /* binary decomposition of unknown x depth before lca */

        for j in (1..=get!(depth => p).ilog2() as usize).rev() {
            if get!(acs => p,j) != get!(acs => q,j) {
                p = get!(acs => p,j);
                q = get!(acs => q,j);
            }
        }

        if p != q {
            p = get!(acs => p, 0);
            q = get!(acs => q, 0);
        }

        debug_assert_eq!(p, q);

        p
    }
}



impl<'a> LCATarjan<'a> {
    pub fn new(g: &'a Graph, root: usize) -> Self {
        let mut dsu = UnionFind::new(None);

        for n in g.vertexs() {
            dsu.insert(n);
        }

        Self { dsu, g, root: root }
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
        ) {
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

        tarjan(self.g, self.root, &mut visited, &mut dsu, &q, &mut res);

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
