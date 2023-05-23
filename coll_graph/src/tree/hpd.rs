//! Heavy path decomposition (heavy-light decomposition)
//!

use std::collections::HashMap;

use coll::{
    set, get, contains, getopt, hashmap,
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
    pub d: HashMap<usize, usize>,

    /// parent
    pub p: HashMap<usize, usize>,

    /// tree node size
    pub sz: HashMap<usize, usize>,

    /// heavy son
    pub hson: HashMap<usize, usize>,

    /// heavy link root
    pub top: HashMap<usize, usize>,

    /// id of heavy link
    pub id: HashMap<usize, usize>,

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
            d: hashmap! {},
            p: hashmap! {},
            sz: hashmap! {},
            hson: hashmap! {},
            top: hashmap! {},
            id: hashmap! {},
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
