//! Heavy path decomposition (heavy-light decomposition)
//!

use coll::{
    easycoll::M1,
    m1, set, get, contains, getopt,
};

use crate::Graph;


////////////////////////////////////////////////////////////////////////////////
//// Structure (Enumration)

/// Heavy (node size, vs Long, depth) path decomposition
///
///
///
pub struct HPD {
    /// depth
    pub d: M1<usize, usize>,

    /// parent
    pub p: M1<usize, usize>,

    /// tree node size
    pub sz: M1<usize, usize>,

    /// heavy son
    pub hson: M1<usize, usize>,

    /// heavy link root
    pub top: M1<usize, usize>,

    /// id of heavy link
    pub id: M1<usize, usize>,

    /// rank (rev of id of heavy link)
    pub rk: Vec<usize>,
}



////////////////////////////////////////////////////////////////////////////////
//// Implementation

impl HPD {
    /// two pass dfs init
    pub fn new(g: &Graph, start: Option<usize>) -> Self {
        let start = start.unwrap_or(g.anypoint());

        let mut it = Self {
            d: m1!(),
            p: m1!(),
            sz: m1!(),
            hson: m1!(),
            top: m1!(),
            id: m1!(),
            rk: vec![0; g.vertexs().count()],
        };

        /// d, p, sz, hson,
        fn dfs1(g: &Graph, u: usize, p: usize, d: usize,  it: &mut HPD) {
            set!(it.d => u => d);
            set!(it.p => u => p);
            set!(it.sz => u => 1);

            for v in get!(g.e => u) {
                if v != p {
                    dfs1(g, v, u, d + 1, it);
                    set!(it.sz => u => get!(it.sz => u) + get!(it.sz => v));

                    if !contains!(it.hson => u) {
                        set!(it.hson => u => v);
                    }
                    else if get!(it.sz => v) > get!(it.sz => get!(it.hson => u)) {
                        set!(it.hson => u => v);
                    }
                }
            }

        }

        /// top, id, rk,
        ///
        /// Heavy link first search
        fn dfs2(g: &Graph, u: usize, top: usize, id: &mut usize, it: &mut HPD) {
            set!(it.top => u => top);
            set!(it.id => u => *id);
            it.rk[*id] = u;
            *id += 1;

            if let Some(hson) = getopt!(it.hson => u) {
                // heavy link id shoulbe continous in dfs
                dfs2(g, hson, top, id, it);

                for v in get!(g.e => u) {
                    if v != get!(it.p => u) && v != hson {
                        dfs2(g, v, v, id, it)
                    }

                }
            }

        }

        dfs1(g, start, start, 1, &mut it);
        dfs2(g, start, start, &mut 0, &mut it);

        it
    }
}
