#![feature(test)]

extern crate test;

use test::Bencher;

use m6_coll_graph::{
    test::{ batch_graph, GraphGenOptions },
    sp::{sp_fa, sp_fa_early_termination, SPDijkstra, SPFA},
    Graph,
};


#[cfg(test)]
lazy_static::lazy_static! {
    static ref DIR_NEGATIVE_GRAPH: Vec<Graph> = prepare_data_detect_negative_cycle();
    static ref DIR_POSITIVE_GRAPH: Vec<Graph> = prepare_data_dir_positive();
}


#[cfg(test)]
fn prepare_data_detect_negative_cycle() -> Vec<Graph> {
    batch_graph(45, 35, -40..60, &GraphGenOptions::dir_conn())
}

#[cfg(test)]
fn prepare_data_dir_positive() -> Vec<Graph> {
    batch_graph(50, 50, 1..100, &GraphGenOptions::dir_conn())
}


#[bench]
#[cfg(test)]
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


#[cfg(test)]
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

#[cfg(test)]
#[bench]
fn bench_sp_spfa(b: &mut Bencher) {
    let gs = &*DIR_POSITIVE_GRAPH;

    b.iter(|| {
        for g in gs.into_iter() {
            for u in g.vertexs() {
                if u % 3 != 0 {
                    continue;
                }

                let spfa = SPFA::new(g, u).unwrap();

                for v in g.vertexs() {
                    spfa.query(v);
                }
            }
        }
    });
}


#[cfg(test)]
#[bench]
fn bench_sp_dijkstra(b: &mut Bencher) {
    let gs = &*DIR_POSITIVE_GRAPH;

    b.iter(|| {
        for g in gs.into_iter() {
            for u in g.vertexs() {
                if u % 3 != 0 {
                    continue;
                }

                let spdijkstra = SPDijkstra::new(g, u);

                for v in g.vertexs() {
                    spdijkstra.query(v);
                }
            }
        }
    });
}
