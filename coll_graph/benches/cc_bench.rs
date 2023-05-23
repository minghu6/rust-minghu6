#![feature(test)]

extern crate test;

use common::random;
use m6_coll_graph::{ test::GraphGenOptions, Graph, scc::scc_tarjan};
use test::{Bencher, black_box};

const BATCH_NUM: usize = 0_100;
const V_RANGE: usize = 10;

lazy_static::lazy_static! {
    pub static ref GRAPH_GEN_OPT: GraphGenOptions = GraphGenOptions {
        is_dir: false,
        allow_cycle: true,
        non_negative_cycle: false,
        weak_conn: false,
    };

    pub static ref LOW_DENSITY_GRAPHS: Vec<Graph> = {
        prepare_low_density_graphs()
    };

    pub static ref HIGH_DENSITY_GRAPHS: Vec<Graph> = {
        prepare_high_density_graphs()
    };
}


fn prepare_low_density_graphs() -> Vec<Graph> {
    let mut res = vec![];

    for _ in 0..BATCH_NUM {
        let sparsity = random::<usize>() % 3;

        let g = Graph::gen(&GRAPH_GEN_OPT, V_RANGE, sparsity, 1..2);
        res.push(g);
    }

    res
}


fn prepare_high_density_graphs() -> Vec<Graph> {
    let mut res = vec![];

    for _ in 0..BATCH_NUM {
        let sparsity = random::<usize>() % 3 + 6;

        let g = Graph::gen(&GRAPH_GEN_OPT, V_RANGE, sparsity, 1..2);
        res.push(g);
    }

    res
}


#[bench]
fn bench_undircc_lowdensity_msu(b: &mut Bencher) {
    let gs = &*LOW_DENSITY_GRAPHS;

    b.iter(|| {
        for g in gs.into_iter() {
            black_box(g.components());
        }
    })
}

#[bench]
fn bench_undircc_lowdensity_tarjan(b: &mut Bencher) {
    let gs = &*LOW_DENSITY_GRAPHS;

    b.iter(|| {
        for g in gs.into_iter() {
            black_box(scc_tarjan(g));
        }
    })
}

#[bench]
fn bench_undircc_highdensity_msu(b: &mut Bencher) {
    let gs = &*HIGH_DENSITY_GRAPHS;

    b.iter(|| {
        for g in gs.into_iter() {
            black_box(g.components());
        }
    })
}

#[bench]
fn bench_undircc_highdensity_tarjan(b: &mut Bencher) {
    let gs = &*HIGH_DENSITY_GRAPHS;

    b.iter(|| {
        for g in gs.into_iter() {
            black_box(scc_tarjan(g));
        }
    })
}
