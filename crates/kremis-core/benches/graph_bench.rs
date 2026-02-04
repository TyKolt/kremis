//! # Graph Benchmarks
//!
//! Performance benchmarks for kremis-core graph operations.
//!
//! Run with: `cargo bench -p kremis-core`

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use kremis_core::graph::{Graph, GraphStore};
use kremis_core::{EdgeWeight, EntityId};
use std::hint::black_box;

/// Create a graph with N nodes and edges between consecutive nodes.
fn create_linear_graph(size: usize) -> Graph {
    let mut graph = Graph::new();
    let mut prev_node = None;

    for i in 0..size {
        let node = graph.insert_node(EntityId(i as u64)).expect("insert");
        if let Some(prev) = prev_node {
            graph
                .insert_edge(prev, node, EdgeWeight::new(10))
                .expect("edge");
        }
        prev_node = Some(node);
    }

    graph
}

/// Create a graph with N nodes and edges in a star pattern (hub-and-spoke).
fn create_star_graph(size: usize) -> Graph {
    let mut graph = Graph::new();
    let hub = graph.insert_node(EntityId(0)).expect("insert");

    for i in 1..size {
        let spoke = graph.insert_node(EntityId(i as u64)).expect("insert");
        graph
            .insert_edge(hub, spoke, EdgeWeight::new(10))
            .expect("edge");
    }

    graph
}

// =============================================================================
// BENCHMARKS
// =============================================================================

fn bench_node_insertion(c: &mut Criterion) {
    let mut group = c.benchmark_group("node_insertion");

    for size in [100, 1000, 10000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| {
                let mut graph = Graph::new();
                for i in 0..size {
                    let _ = graph.insert_node(EntityId(i as u64));
                }
                black_box(graph)
            });
        });
    }

    group.finish();
}

fn bench_edge_insertion(c: &mut Criterion) {
    let mut group = c.benchmark_group("edge_insertion");

    for size in [100, 1000, 10000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| {
                let graph = create_linear_graph(size);
                black_box(graph)
            });
        });
    }

    group.finish();
}

fn bench_node_lookup(c: &mut Criterion) {
    let mut group = c.benchmark_group("node_lookup");

    for size in [100, 1000, 10000].iter() {
        let graph = create_linear_graph(*size);

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| {
                // Lookup a node in the middle
                let entity = EntityId((size / 2) as u64);
                black_box(graph.get_node_by_entity(entity))
            });
        });
    }

    group.finish();
}

fn bench_traverse(c: &mut Criterion) {
    let mut group = c.benchmark_group("traverse");

    for size in [100, 500, 1000].iter() {
        let graph = create_linear_graph(*size);
        let start = graph
            .get_node_by_entity(EntityId(0))
            .expect("node 0 should exist");

        group.bench_with_input(
            BenchmarkId::new("depth_10", size),
            &(start, 10),
            |b, &(start, depth)| {
                b.iter(|| black_box(graph.traverse(start, depth)));
            },
        );

        group.bench_with_input(
            BenchmarkId::new("depth_50", size),
            &(start, 50),
            |b, &(start, depth)| {
                b.iter(|| black_box(graph.traverse(start, depth)));
            },
        );
    }

    group.finish();
}

fn bench_strongest_path(c: &mut Criterion) {
    let mut group = c.benchmark_group("strongest_path");

    for size in [100, 500, 1000].iter() {
        let graph = create_linear_graph(*size);
        let start = graph
            .get_node_by_entity(EntityId(0))
            .expect("start node should exist");
        let end = graph
            .get_node_by_entity(EntityId((*size - 1) as u64))
            .expect("end node should exist");

        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            &(start, end),
            |b, &(start, end)| {
                b.iter(|| black_box(graph.strongest_path(start, end)));
            },
        );
    }

    group.finish();
}

fn bench_intersect(c: &mut Criterion) {
    let mut group = c.benchmark_group("intersect");

    for size in [100, 500, 1000].iter() {
        let graph = create_star_graph(*size);
        let _hub = graph
            .get_node_by_entity(EntityId(0))
            .expect("hub should exist");
        let spoke1 = graph
            .get_node_by_entity(EntityId(1))
            .expect("spoke1 should exist");
        let spoke2 = graph
            .get_node_by_entity(EntityId(2))
            .expect("spoke2 should exist");

        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            &vec![spoke1, spoke2],
            |b, nodes| {
                b.iter(|| black_box(graph.intersect(nodes)));
            },
        );
    }

    group.finish();
}

fn bench_export_canonical(c: &mut Criterion) {
    use kremis_core::export::export_canonical;

    let mut group = c.benchmark_group("export_canonical");

    for size in [100, 500, 1000].iter() {
        let graph = create_linear_graph(*size);

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| black_box(export_canonical(&graph)));
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_node_insertion,
    bench_edge_insertion,
    bench_node_lookup,
    bench_traverse,
    bench_strongest_path,
    bench_intersect,
    bench_export_canonical,
);

criterion_main!(benches);
