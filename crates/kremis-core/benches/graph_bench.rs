//! # Graph Benchmarks
//!
//! Performance benchmarks for kremis-core graph operations.
//!
//! Run with: `cargo bench -p kremis-core`

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use kremis_core::graph::{Graph, GraphStore};
use kremis_core::{
    Attribute, EdgeWeight, EntityId, Ingestor, RedbGraph, Session, Signal, Value,
    canonical_checksum, export_canonical, import_canonical,
};
use std::hint::black_box;

// =============================================================================
// HELPERS
// =============================================================================

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

/// Create a dense graph where each node connects to the next 5 nodes.
fn create_dense_graph(size: usize) -> Graph {
    let mut graph = Graph::new();
    let mut nodes = Vec::with_capacity(size);

    for i in 0..size {
        let node = graph.insert_node(EntityId(i as u64)).expect("insert");
        nodes.push(node);
    }

    for i in 0..size {
        for j in 1..=5 {
            if i + j < size {
                graph
                    .insert_edge(nodes[i], nodes[i + j], EdgeWeight::new(10))
                    .expect("edge");
            }
        }
    }

    graph
}

/// Generate a vector of unique signals.
fn generate_signals(count: usize) -> Vec<Signal> {
    (0..count)
        .map(|i| {
            Signal::new(
                EntityId(i as u64),
                Attribute::new(format!("attr_{}", i % 50)),
                Value::new(format!("val_{i}")),
            )
        })
        .collect()
}

// =============================================================================
// EXISTING BENCHMARKS
// =============================================================================

fn bench_node_insertion(c: &mut Criterion) {
    let mut group = c.benchmark_group("node_insertion");

    for size in [100, 1_000, 10_000, 50_000, 100_000].iter() {
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

    for size in [100, 1_000, 10_000, 50_000, 100_000].iter() {
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

    for size in [100, 1_000, 10_000].iter() {
        let graph = create_linear_graph(*size);

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| {
                let entity = EntityId((size / 2) as u64);
                black_box(graph.get_node_by_entity(entity))
            });
        });
    }

    group.finish();
}

fn bench_traverse(c: &mut Criterion) {
    let mut group = c.benchmark_group("traverse");

    for size in [100, 500, 1_000].iter() {
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

    for size in [100, 500, 1_000].iter() {
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

    for size in [100, 500, 1_000].iter() {
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
    let mut group = c.benchmark_group("export_canonical");

    for size in [100, 500, 1_000].iter() {
        let graph = create_linear_graph(*size);

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| black_box(export_canonical(&graph)));
        });
    }

    group.finish();
}

// =============================================================================
// NEW BENCHMARKS
// =============================================================================

fn bench_signal_ingestion(c: &mut Criterion) {
    let mut group = c.benchmark_group("signal_ingestion");

    // Single-signal ingestion via Ingestor (the real user path)
    for size in [1_000, 10_000, 50_000].iter() {
        let signals = generate_signals(*size);

        group.bench_with_input(BenchmarkId::new("single", size), &signals, |b, signals| {
            b.iter(|| {
                let mut graph = Graph::new();
                for signal in signals {
                    let _ = Ingestor::ingest_signal(&mut graph, signal);
                }
                black_box(graph)
            });
        });
    }

    // Batch ingestion via ingest_sequence (capped at MAX_SEQUENCE_LENGTH = 10_000)
    for size in [1_000, 5_000, 10_000].iter() {
        let signals = generate_signals(*size);

        group.bench_with_input(
            BenchmarkId::new("sequence", size),
            &signals,
            |b, signals| {
                b.iter(|| {
                    let mut graph = Graph::new();
                    let _ = Ingestor::ingest_sequence(&mut graph, signals);
                    black_box(graph)
                });
            },
        );
    }

    // Session-level ingestion (includes context tracking overhead)
    for size in [1_000, 10_000].iter() {
        let signals = generate_signals(*size);

        group.bench_with_input(BenchmarkId::new("session", size), &signals, |b, signals| {
            b.iter(|| {
                let mut session = Session::new();
                for signal in signals {
                    let _ = session.ingest(signal);
                }
                black_box(session)
            });
        });
    }

    group.finish();
}

fn bench_import_canonical(c: &mut Criterion) {
    let mut group = c.benchmark_group("import_canonical");

    for size in [100, 1_000, 10_000].iter() {
        let graph = create_linear_graph(*size);
        let data = export_canonical(&graph).expect("export");

        group.bench_with_input(BenchmarkId::from_parameter(size), &data, |b, data| {
            b.iter(|| black_box(import_canonical(data)));
        });
    }

    group.finish();
}

fn bench_properties(c: &mut Criterion) {
    let mut group = c.benchmark_group("properties");

    // Store properties
    for size in [100, 1_000, 10_000].iter() {
        group.bench_with_input(BenchmarkId::new("store", size), size, |b, &size| {
            b.iter(|| {
                let mut graph = Graph::new();
                let mut nodes = Vec::with_capacity(size);
                for i in 0..size {
                    let node = graph.insert_node(EntityId(i as u64)).expect("insert");
                    nodes.push(node);
                }
                for (i, &node) in nodes.iter().enumerate() {
                    let _ = graph.store_property(
                        node,
                        Attribute::new(format!("key_{}", i % 10)),
                        Value::new(format!("val_{i}")),
                    );
                }
                black_box(graph)
            });
        });
    }

    // Retrieve properties
    for size in [100, 1_000, 10_000].iter() {
        let mut graph = Graph::new();
        let mut nodes = Vec::with_capacity(*size);
        for i in 0..*size {
            let node = graph.insert_node(EntityId(i as u64)).expect("insert");
            graph
                .store_property(
                    node,
                    Attribute::new(format!("key_{}", i % 10)),
                    Value::new(format!("val_{i}")),
                )
                .expect("store");
            nodes.push(node);
        }

        group.bench_with_input(
            BenchmarkId::new("get", size),
            &(graph, nodes),
            |b, (graph, nodes)| {
                b.iter(|| {
                    for &node in nodes {
                        black_box(graph.get_properties(node).expect("get"));
                    }
                });
            },
        );
    }

    group.finish();
}

fn bench_increment_edge(c: &mut Criterion) {
    let mut group = c.benchmark_group("increment_edge");

    for size in [100, 1_000, 10_000].iter() {
        // Pre-build a linear graph, then bench incrementing all its edges
        let base = create_linear_graph(*size);
        let edge_pairs: Vec<_> = (0..*size - 1)
            .map(|i| {
                let from = base.get_node_by_entity(EntityId(i as u64)).expect("from node");
                let to = base
                    .get_node_by_entity(EntityId((i + 1) as u64))
                    .expect("to node");
                (from, to)
            })
            .collect();

        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            &edge_pairs,
            |b, pairs| {
                b.iter(|| {
                    let mut graph = create_linear_graph(*size);
                    for &(from, to) in pairs {
                        let _ = graph.increment_edge(from, to);
                    }
                    black_box(graph)
                });
            },
        );
    }

    group.finish();
}

fn bench_checksum(c: &mut Criterion) {
    let mut group = c.benchmark_group("checksum");

    for size in [100, 1_000, 10_000].iter() {
        let graph = create_linear_graph(*size);

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| black_box(canonical_checksum(&graph)));
        });
    }

    group.finish();
}

fn bench_redb_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("redb_operations");
    // Disk is slower — use reduced sizes
    let sizes = [100, 1_000];

    // Insert nodes
    for size in sizes.iter() {
        group.bench_with_input(BenchmarkId::new("insert_nodes", size), size, |b, &size| {
            b.iter(|| {
                let dir = tempfile::tempdir().expect("tmpdir");
                let mut redb = RedbGraph::open(dir.path().join("bench.redb")).expect("open");
                for i in 0..size {
                    let _ = redb.insert_node(EntityId(i as u64));
                }
                black_box(&redb);
            });
        });
    }

    // Insert edges (linear graph pattern)
    for size in sizes.iter() {
        group.bench_with_input(BenchmarkId::new("insert_edges", size), size, |b, &size| {
            b.iter(|| {
                let dir = tempfile::tempdir().expect("tmpdir");
                let mut redb = RedbGraph::open(dir.path().join("bench.redb")).expect("open");
                let mut prev = None;
                for i in 0..size {
                    let node = redb.insert_node(EntityId(i as u64)).expect("insert");
                    if let Some(p) = prev {
                        let _ = redb.insert_edge(p, node, EdgeWeight::new(10));
                    }
                    prev = Some(node);
                }
                black_box(&redb);
            });
        });
    }

    // Lookup
    for size in sizes.iter() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let mut redb = RedbGraph::open(dir.path().join("bench.redb")).expect("open");
        for i in 0..*size {
            redb.insert_node(EntityId(i as u64)).expect("insert");
        }

        group.bench_with_input(BenchmarkId::new("lookup", size), size, |b, &size| {
            b.iter(|| {
                let entity = EntityId((size / 2) as u64);
                black_box(redb.get_node_by_entity(entity))
            });
        });
    }

    // Traverse
    for size in sizes.iter() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let mut redb = RedbGraph::open(dir.path().join("bench.redb")).expect("open");
        let mut prev = None;
        for i in 0..*size {
            let node = redb.insert_node(EntityId(i as u64)).expect("insert");
            if let Some(p) = prev {
                redb.insert_edge(p, node, EdgeWeight::new(10))
                    .expect("edge");
            }
            prev = Some(node);
        }
        let start = redb
            .get_node_by_entity(EntityId(0))
            .expect("node 0 should exist");

        group.bench_with_input(
            BenchmarkId::new("traverse_depth10", size),
            &start,
            |b, &start| {
                b.iter(|| black_box(redb.traverse(start, 10)));
            },
        );
    }

    group.finish();
}

fn bench_dense_graph_traverse(c: &mut Criterion) {
    let mut group = c.benchmark_group("dense_graph_traverse");

    for size in [100, 500, 1_000].iter() {
        let graph = create_dense_graph(*size);
        let start = graph
            .get_node_by_entity(EntityId(0))
            .expect("node 0 should exist");

        group.bench_with_input(
            BenchmarkId::new("depth_5", size),
            &(start, 5),
            |b, &(start, depth)| {
                b.iter(|| black_box(graph.traverse(start, depth)));
            },
        );

        group.bench_with_input(
            BenchmarkId::new("depth_10", size),
            &(start, 10),
            |b, &(start, depth)| {
                b.iter(|| black_box(graph.traverse(start, depth)));
            },
        );
    }

    group.finish();
}

// =============================================================================
// CRITERION GROUPS
// =============================================================================

criterion_group!(
    benches,
    bench_node_insertion,
    bench_edge_insertion,
    bench_node_lookup,
    bench_traverse,
    bench_strongest_path,
    bench_intersect,
    bench_export_canonical,
    bench_signal_ingestion,
    bench_import_canonical,
    bench_properties,
    bench_increment_edge,
    bench_checksum,
    bench_redb_operations,
    bench_dense_graph_traverse,
);

criterion_main!(benches);
