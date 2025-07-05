#![feature(test)]
#![allow(dead_code)]

extern crate test;

use common::random;

use m6_coll_graph::{mst::*, Graph, test::GraphGenOptions};

use test::Bencher;

const BATCH_NUM: usize = 0_100;


lazy_static::lazy_static! {
    pub static ref LOW_DENSITY_GRAPHS: Vec<Graph> = {
        prepare_low_density_graphs()
    };

    pub static ref HIGH_DENSITY_GRAPHS: Vec<Graph> = {
        prepare_high_density_graphs()
    };
}



fn prepare_low_density_graphs() -> Vec<Graph> {
    let opt = GraphGenOptions::undir_conn();
    let wrange = 1..1000;
    let vrange = 100;
    let mut res = vec![];

    for _ in 0..BATCH_NUM {
        let sparsity = random::<usize>() % 3 + 1;

        let g = Graph::generate(&opt, vrange, sparsity, wrange.clone());
        res.push(g);
    }

    res
}


fn prepare_high_density_graphs() -> Vec<Graph> {
    let opt = GraphGenOptions::undir_conn();
    let wrange = 1..1000;
    let vrange = 100;
    let mut res = vec![];

    for _ in 0..BATCH_NUM {
        let sparsity = random::<usize>() % 3 + 8;

        let g = Graph::generate(&opt, vrange, sparsity, wrange.clone());
        res.push(g);
    }

    res
}



#[bench]
fn bench_mst_lowdensity_krusal(b: &mut Bencher) {
    let gs = &*LOW_DENSITY_GRAPHS;

    b.iter(|| {
        for g in gs.into_iter() {
            mst_kruskal(g);
        }
    })
}


#[bench]
fn bench_mst_lowdensity_prim(b: &mut Bencher) {
    let gs = &*LOW_DENSITY_GRAPHS;

    b.iter(|| {
        for g in gs.into_iter() {
            mst_prim(g);
        }
    })
}



#[bench]
fn bench_mst_lowdensity_boruvka(b: &mut Bencher) {
    let gs = &*LOW_DENSITY_GRAPHS;

    b.iter(|| {
        for g in gs.into_iter() {
            mst_boruvka(g);
        }
    })
}



#[bench]
fn bench_mst_highdensity_krusal(b: &mut Bencher) {
    let gs = &*HIGH_DENSITY_GRAPHS;

    b.iter(|| {
        for g in gs.into_iter() {
            mst_kruskal(g);
        }
    })
}


#[bench]
fn bench_mst_highdensity_prim(b: &mut Bencher) {
    let gs = &*HIGH_DENSITY_GRAPHS;

    b.iter(|| {
        for g in gs.into_iter() {
            mst_prim(g);
        }
    })
}



#[bench]
fn bench_mst_highdensity_boruvka(b: &mut Bencher) {
    let gs = &*HIGH_DENSITY_GRAPHS;

    b.iter(|| {
        for g in gs.into_iter() {
            mst_boruvka(g);
        }
    })
}

