//! Can used for detect ring
//!

use super::Graph;
use crate::{
    collections::easycoll::M1,
    get, m1, queue, set,
};


/// Kahn (pronounce can) algorithm (assume a DIRECTED graph) O(E + V)
pub fn toposort_kahn(g: &Graph) -> Option<Vec<usize>> {
    /* build ein map */
    let mut ein = m1!();

    for u in g.bfs(None) {
        for v in get!(g.e => u) {
            set!(ein => v => get!(ein => v => 0) + 1);
        }
    }

    // // remove extra field from
    // for (_, x) in ein.0.iter_mut() {
    //     *x -= 1;
    // }

    /* init S ({v| ein(v) = 0 }) */
    #[allow(non_snake_case)]
    let mut S = queue!();

    for u in ein.0.keys().cloned() {
        if get!(ein => u) == 0 {
            S.enq(u);
        }
    }

    #[allow(non_snake_case)]
    let mut L = vec![];

    while let Some(u) = S.deq() {
        L.push(u);

        for v in get!(g.e => u) {
            set!(ein => v => get!(ein => v) - 1);

            if get!(ein => v) == 0 {
                S.enq(v);
            }
        }
    }

    if L.len() == g.vertexs().count() {
        Some(L)
    } else {
        None
    }
}


/// O(V + E)
#[allow(non_snake_case)]
pub fn toposort_dfs(g: &Graph) -> Option<Vec<usize>> {
    let mut L = vec![];
    let mut marks = m1!();

    #[derive(PartialEq, Clone, Copy)]
    enum Mark {
        UMark,
        Tmark,
        PMark,
    }

    fn dfs(
        g: &Graph,
        u: usize,
        marks: &mut M1<usize, Mark>,
        L: &mut Vec<usize>,
    ) -> Result<(), ()> {
        match get!(marks => u) {
            Mark::PMark => return Ok(()),
            Mark::Tmark => return Err(()),
            _ => (),
        }

        set!(marks => u => Mark::Tmark);

        for v in get!(g.e => u) {
            dfs(g, v, marks, L)?;
        }

        set!(marks => u => Mark::PMark);
        L.push(u);

        Ok(())
    }

    for u in g.bfs(None) {
        if get!(marks => u) == Mark::UMark {
            if let Err(()) = dfs(g, u, &mut marks, &mut L) {
                return None;
            }
        }
    }

    L.reverse();

    Some(L)
}
