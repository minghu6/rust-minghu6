use std::{collections::HashMap, iter::once};

use coll::{aux::{VerifyResult, VerifyError}, get};

use crate::Graph;



impl Graph {
    ////////////////////////////////////////////////////////////////////////////
    /// Verify

    /// verify spanning tree
    pub fn verify_st(&self, st: &[(usize, usize)]) -> VerifyResult {
        let mut vertx = self
            .vertexs()
            .map(|v| (v, ()))
            .collect::<HashMap<usize, ()>>();

        for (u, v) in st {
            if !self.contains_edge((*u, *v)) {
                return Err(VerifyError::Inv(format!("No edge {u}->{v}")));
            }

            vertx.remove(u);
            vertx.remove(v);
        }

        if vertx.is_empty() {
            Ok(())
        } else {
            Err(VerifyError::Fail(format!("remains vertex: {vertx:?}",)))
        }
    }


    /// verify minimal spanning tree
    pub fn verify_mst(
        &self,
        min: isize,
        st: &[(usize, usize)],
    ) -> VerifyResult {
        self.verify_st(st)?;

        let tot: isize = st.into_iter().map(|x| get!(self.w => x)).sum();

        if tot == min {
            Ok(())
        } else if tot < min {
            Err(VerifyError::Inv(format!("cur_tot: {tot} < min: {min}")))
        } else {
            Err(VerifyError::Fail(format!("too big")))
        }
    }

    pub fn verify_path(
        &self,
        src: usize,
        dst: usize,
        path: &[usize],
    ) -> VerifyResult {
        let mut u = src;

        for v in path {
            if self.contains_edge((u, *v)) {
                u = *v;
            } else {
                return Err(VerifyError::Inv(format!(
                    "No edge {u}-{v} for {path:?} src: {src} dst: {dst}"
                )));
            }
        }

        if u == dst {
            Ok(())
        } else {
            Err(VerifyError::Inv(format!("Not end in {dst}, found {u}")))
        }
    }

    pub fn verify_cycle(&self, cycle: &[usize]) -> VerifyResult {
        if cycle.len() == 0 {
            return Err(VerifyError::Inv(format!("Empty negative cycle")));
        }
        if cycle.len() == 1 {
            return Err(VerifyError::Inv(format!("Self loop")));
        }
        if cycle.len() == 2 && !self.is_dir {
            return Err(VerifyError::Fail(format!("Single edge cycle for undirected graph")));
        }

        let src = *cycle.first().unwrap();
        let dst = *cycle.last().unwrap();

        self.verify_path(src, dst, &cycle[1..])?;

        if self.contains_edge((dst, src)) {
            Ok(())
        } else {
            Err(VerifyError::Fail(format!("{dst} can't goback to {src}")))
        }
    }

    pub fn verify_negative_cycle(
        &self,
        cycle: &[usize],
    ) -> VerifyResult {
        self.verify_cycle(cycle)?;

        /* verify negative weights sumeration */

        let sumw: isize = cycle
            .iter()
            .cloned()
            .zip(cycle[1..].iter().cloned().chain(once(cycle[0])))
            .map(|(u, v)| get!(self.w => (u, v)))
            .sum();

        if sumw < 0 {
            Ok(())
        } else {
            Err(VerifyError::Fail(format!("weight sum: {sumw} >= 0, for cycle {cycle:?}")))
        }
    }
}
