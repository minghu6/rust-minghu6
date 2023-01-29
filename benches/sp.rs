#![feature(test)]

extern crate test;
use lazy_static::lazy_static;
use minghu6::{
    collections::graph::{
        sp::{sp_fa, sp_fa_early_termination, SPFA, SPDijkstra},
        Graph, batch_graph, GraphGenOptions
    },
};
use test::Bencher;


lazy_static! {
    static ref DIR_NEGATIVE_GRAPH: Vec<Graph> = prepare_data_detect_negative_cycle();
    static ref DIR_POSITIVE_GRAPH: Vec<Graph> = prepare_data_dir_positive();
}


fn prepare_data_detect_negative_cycle() -> Vec<Graph> {
    batch_graph(45, 35, -40..60, &GraphGenOptions::dir_conn())
}


fn prepare_data_dir_positive() -> Vec<Graph> {
    batch_graph(50, 50, 1..100, &GraphGenOptions::dir_conn())
}



#[bench]
fn bench_spfa_detect_nagtive_cycle_origin(b: &mut Bencher) {
    let gs = &*DIR_NEGATIVE_GRAPH;

    b.iter(|| {
        for g in gs.into_iter() {
            for v in g.vertexs() {
                if let Err(..) = sp_fa(g, v) {}
            }
        }
    });
}


#[bench]
fn bench_spfa_detect_nagtive_cycle_early_termination(b: &mut Bencher) {
    let gs = &*DIR_NEGATIVE_GRAPH;

    b.iter(|| {
        for g in gs.into_iter() {
            for v in g.vertexs() {
                if let Err(..) = sp_fa_early_termination(g, v) {}
            }
        }
    });
}


#[bench]
fn bench_sp_spfa(b: &mut Bencher) {
    let gs = &*DIR_POSITIVE_GRAPH;

    b.iter(|| {
        for g in gs.into_iter() {
            for u in g.vertexs() {
                if u % 3 != 0 { continue }

                let spfa = SPFA::new(g, u).unwrap();

                for v in g.vertexs() {
                    spfa.query(v);
                }
            }
        }
    });
}


#[bench]
fn bench_sp_dijkstra(b: &mut Bencher) {
    let gs = &*DIR_POSITIVE_GRAPH;

    b.iter(|| {
        for g in gs.into_iter() {
            for u in g.vertexs() {
                if u % 3 != 0 { continue }

                let spdijkstra = SPDijkstra::new(g, u);

                for v in g.vertexs() {
                    spdijkstra.query(v);
                }
            }
        }
    });
}
