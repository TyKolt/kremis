//! # Validation Tier Tests (T0-T3)
//!
//! If ANY tier fails, the system is INVALID.
//!
//! ## Tiers
//! - T0: Signal Integrity
//! - T1: Deterministic Edge Creation
//! - T2: Single-Hop Traversal
//! - T3: Multi-Hop Traversal

use kremis_core::{
    Attribute, EdgeWeight, EntityId, Graph, GraphStore, KremisError, NodeId, Signal, Value,
};

// =============================================================================
// TIER T0: SIGNAL INTEGRITY
// =============================================================================

mod t0_signal_integrity {
    use super::*;
    use kremis_core::Ingestor;

    /// T0.1: Signal validation accepts valid signals.
    #[test]
    fn valid_signal_accepted() {
        let signal = Signal::new(EntityId(1), Attribute::new("name"), Value::new("Alice"));

        assert!(Ingestor::validate(&signal).is_ok());
    }

    /// T0.2: Signal validation rejects empty attribute.
    #[test]
    fn empty_attribute_rejected() {
        let signal = Signal::new(EntityId(1), Attribute::new(""), Value::new("value"));

        let result = Ingestor::validate(&signal);
        assert!(matches!(result, Err(KremisError::InvalidSignal)));
    }

    /// T0.3: Signal validation rejects empty value.
    #[test]
    fn empty_value_rejected() {
        let signal = Signal::new(EntityId(1), Attribute::new("attr"), Value::new(""));

        let result = Ingestor::validate(&signal);
        assert!(matches!(result, Err(KremisError::InvalidSignal)));
    }

    /// T0.4: Signal ingestion is idempotent.
    #[test]
    fn signal_ingestion_idempotent() {
        let mut graph = Graph::new();
        let signal = Signal::new(EntityId(42), Attribute::new("test"), Value::new("value"));

        let node1 = Ingestor::ingest_signal(&mut graph, &signal).expect("first");
        let node2 = Ingestor::ingest_signal(&mut graph, &signal).expect("second");

        // Same signal produces same node
        assert_eq!(node1, node2);
        assert_eq!(graph.node_count().expect("count"), 1);
    }

    /// T0.5: Duplicate detection works.
    #[test]
    fn duplicate_detection() {
        let mut graph = Graph::new();
        let signal = Signal::new(EntityId(1), Attribute::new("a"), Value::new("b"));

        assert!(!Ingestor::is_duplicate(&graph, &signal));
        let _ = Ingestor::ingest_signal(&mut graph, &signal);
        assert!(Ingestor::is_duplicate(&graph, &signal));
    }
}

// =============================================================================
// TIER T1: DETERMINISTIC EDGE CREATION
// =============================================================================

mod t1_deterministic_edges {
    use super::*;

    /// T1.1: Same signals produce same edges.
    #[test]
    fn same_signals_same_edges() {
        let make_graph = || {
            let mut graph = Graph::new();
            let a = graph.insert_node(EntityId(1)).expect("insert");
            let b = graph.insert_node(EntityId(2)).expect("insert");
            let c = graph.insert_node(EntityId(3)).expect("insert");
            graph.insert_edge(a, b, EdgeWeight::new(5)).expect("edge");
            graph.insert_edge(b, c, EdgeWeight::new(3)).expect("edge");
            graph
        };

        let graph1 = make_graph();
        let graph2 = make_graph();

        // Same structure
        assert_eq!(
            graph1.node_count().expect("count"),
            graph2.node_count().expect("count")
        );
        assert_eq!(
            graph1.edge_count().expect("count"),
            graph2.edge_count().expect("count")
        );

        // Same edges
        let a1 = graph1.get_node_by_entity(EntityId(1)).expect("a1");
        let b1 = graph1.get_node_by_entity(EntityId(2)).expect("b1");
        let a2 = graph2.get_node_by_entity(EntityId(1)).expect("a2");
        let b2 = graph2.get_node_by_entity(EntityId(2)).expect("b2");

        assert_eq!(
            graph1.get_edge(a1, b1).expect("edge"),
            graph2.get_edge(a2, b2).expect("edge")
        );
    }

    /// T1.2: Weight increments are exact integers.
    #[test]
    fn weight_increments_exact() {
        let mut graph = Graph::new();
        let a = graph.insert_node(EntityId(1)).expect("insert");
        let b = graph.insert_node(EntityId(2)).expect("insert");

        // Initial weight
        graph.insert_edge(a, b, EdgeWeight::new(1)).expect("edge");
        assert_eq!(graph.get_edge(a, b).expect("get"), Some(EdgeWeight::new(1)));

        // Increment
        graph.increment_edge(a, b).expect("inc");
        assert_eq!(graph.get_edge(a, b).expect("get"), Some(EdgeWeight::new(2)));

        // More increments
        for _ in 0..8 {
            graph.increment_edge(a, b).expect("inc");
        }
        assert_eq!(
            graph.get_edge(a, b).expect("get"),
            Some(EdgeWeight::new(10))
        );
    }

    /// T1.3: Edge creation is reproducible across runs.
    #[test]
    fn edge_creation_reproducible() {
        // Run twice, compare results
        for _ in 0..2 {
            let mut graph = Graph::new();
            let nodes: Vec<NodeId> = (0..10)
                .map(|i| graph.insert_node(EntityId(i as u64)).expect("insert"))
                .collect();

            for i in 0..9 {
                graph.increment_edge(nodes[i], nodes[i + 1]).expect("inc");
            }

            assert_eq!(graph.edge_count().expect("count"), 9);
        }
    }

    /// T1.4: Saturating arithmetic prevents overflow.
    #[test]
    fn saturating_arithmetic() {
        let max_weight = EdgeWeight::new(i64::MAX);
        let incremented = max_weight.increment();

        // Should saturate, not overflow
        assert_eq!(incremented.value(), i64::MAX);
    }
}

// =============================================================================
// TIER T2: SINGLE-HOP TRAVERSAL
// =============================================================================

mod t2_single_hop {
    use super::*;

    /// T2.1: neighbors() returns correct adjacent nodes.
    #[test]
    fn neighbors_returns_correct() {
        let mut graph = Graph::new();
        let a = graph.insert_node(EntityId(1)).expect("insert");
        let b = graph.insert_node(EntityId(2)).expect("insert");
        let c = graph.insert_node(EntityId(3)).expect("insert");

        graph.insert_edge(a, b, EdgeWeight::new(1)).expect("edge");
        graph.insert_edge(a, c, EdgeWeight::new(1)).expect("edge");

        let neighbors = graph.neighbors(a).expect("neighbors");
        assert_eq!(neighbors.len(), 2);
        assert!(neighbors.iter().any(|(n, _)| *n == b));
        assert!(neighbors.iter().any(|(n, _)| *n == c));
    }

    /// T2.2: Missing node returns empty vec.
    #[test]
    fn missing_node_empty() {
        let graph = Graph::new();
        let fake_node = NodeId(999);

        let neighbors = graph.neighbors(fake_node).expect("neighbors");
        assert!(neighbors.is_empty());
    }

    /// T2.3: Neighbor order is deterministic (sorted by NodeId).
    #[test]
    fn neighbor_order_deterministic() {
        let mut graph = Graph::new();
        let a = graph.insert_node(EntityId(1)).expect("insert");
        let b = graph.insert_node(EntityId(2)).expect("insert");
        let c = graph.insert_node(EntityId(3)).expect("insert");
        let d = graph.insert_node(EntityId(4)).expect("insert");

        // Insert in random order
        graph.insert_edge(a, d, EdgeWeight::new(1)).expect("edge");
        graph.insert_edge(a, b, EdgeWeight::new(1)).expect("edge");
        graph.insert_edge(a, c, EdgeWeight::new(1)).expect("edge");

        let neighbors: Vec<_> = graph
            .neighbors(a)
            .expect("neighbors")
            .into_iter()
            .map(|(n, _)| n)
            .collect();

        // Should be sorted by NodeId
        assert_eq!(neighbors, vec![b, c, d]);
    }

    /// T2.4: lookup() returns correct node.
    #[test]
    fn lookup_returns_correct() {
        let mut graph = Graph::new();
        let node = graph.insert_node(EntityId(42)).expect("insert");

        let result = graph.lookup(node).expect("lookup");
        assert!(result.is_some());
        assert_eq!(result.map(|n| n.entity), Some(EntityId(42)));
    }

    /// T2.5: lookup() returns None for missing node.
    #[test]
    fn lookup_missing_returns_none() {
        let graph = Graph::new();
        let result = graph.lookup(NodeId(999)).expect("lookup");
        assert!(result.is_none());
    }
}

// =============================================================================
// TIER T3: MULTI-HOP TRAVERSAL
// =============================================================================

mod t3_multi_hop {
    use super::*;

    /// T3.1: BFS produces correct paths.
    #[test]
    fn bfs_correct_paths() {
        let mut graph = Graph::new();
        let a = graph.insert_node(EntityId(1)).expect("insert");
        let b = graph.insert_node(EntityId(2)).expect("insert");
        let c = graph.insert_node(EntityId(3)).expect("insert");

        graph.insert_edge(a, b, EdgeWeight::new(1)).expect("edge");
        graph.insert_edge(b, c, EdgeWeight::new(1)).expect("edge");

        let artifact = graph
            .traverse(a, 3)
            .expect("traverse")
            .expect("should traverse");

        // Path should contain all reachable nodes
        assert!(artifact.path.contains(&a));
        assert!(artifact.path.contains(&b));
        assert!(artifact.path.contains(&c));
    }

    /// T3.2: DFS produces correct paths.
    #[test]
    fn dfs_correct_paths() {
        let mut graph = Graph::new();
        let a = graph.insert_node(EntityId(1)).expect("insert");
        let b = graph.insert_node(EntityId(2)).expect("insert");
        let c = graph.insert_node(EntityId(3)).expect("insert");

        graph.insert_edge(a, b, EdgeWeight::new(1)).expect("edge");
        graph.insert_edge(b, c, EdgeWeight::new(1)).expect("edge");

        let artifact = graph.traverse_dfs(a, 3).expect("should traverse");

        assert!(artifact.path.contains(&a));
        assert!(artifact.path.contains(&b));
        assert!(artifact.path.contains(&c));
    }

    /// T3.3: Traversal depth is respected.
    #[test]
    fn traversal_depth_respected() {
        let mut graph = Graph::new();
        let nodes: Vec<_> = (0..10)
            .map(|i| graph.insert_node(EntityId(i as u64)).expect("insert"))
            .collect();

        for i in 0..9 {
            graph
                .insert_edge(nodes[i], nodes[i + 1], EdgeWeight::new(1))
                .expect("edge");
        }

        // Depth 2 should only reach first 3 nodes
        let artifact = graph
            .traverse(nodes[0], 2)
            .expect("traverse")
            .expect("artifact");
        assert!(artifact.path.len() <= 3);
    }

    /// T3.4: Artifact contains complete path trace.
    #[test]
    fn artifact_complete_trace() {
        let mut graph = Graph::new();
        let a = graph.insert_node(EntityId(1)).expect("insert");
        let b = graph.insert_node(EntityId(2)).expect("insert");

        graph.insert_edge(a, b, EdgeWeight::new(5)).expect("edge");

        let artifact = graph.traverse(a, 2).expect("traverse").expect("artifact");

        assert!(!artifact.path.is_empty());
        assert!(artifact.subgraph.is_some());

        let subgraph = artifact.subgraph.as_ref().expect("subgraph");
        assert!(!subgraph.is_empty());
    }

    /// T3.5: Cyclic graphs do not cause infinite loops.
    #[test]
    fn cyclic_graphs_handled() {
        let mut graph = Graph::new();
        let a = graph.insert_node(EntityId(1)).expect("insert");
        let b = graph.insert_node(EntityId(2)).expect("insert");
        let c = graph.insert_node(EntityId(3)).expect("insert");

        // Create cycle: a -> b -> c -> a
        graph.insert_edge(a, b, EdgeWeight::new(1)).expect("edge");
        graph.insert_edge(b, c, EdgeWeight::new(1)).expect("edge");
        graph.insert_edge(c, a, EdgeWeight::new(1)).expect("edge");

        // This should complete without hanging
        let artifact = graph.traverse(a, 10).expect("traverse");
        assert!(artifact.is_some());

        // DFS should also handle cycles
        let artifact_dfs = graph.traverse_dfs(a, 10);
        assert!(artifact_dfs.is_some());
    }

    /// T3.6: strongest_path works correctly.
    #[test]
    fn strongest_path_works() {
        let mut graph = Graph::new();
        let a = graph.insert_node(EntityId(1)).expect("insert");
        let b = graph.insert_node(EntityId(2)).expect("insert");
        let c = graph.insert_node(EntityId(3)).expect("insert");

        graph.insert_edge(a, b, EdgeWeight::new(10)).expect("edge");
        graph.insert_edge(b, c, EdgeWeight::new(10)).expect("edge");

        let path = graph.strongest_path(a, c).expect("path");
        assert!(path.is_some());
        let path = path.expect("path");
        assert_eq!(path.first(), Some(&a));
        assert_eq!(path.last(), Some(&c));
    }

    /// T3.7: Missing path returns None.
    #[test]
    fn missing_path_returns_none() {
        let mut graph = Graph::new();
        let a = graph.insert_node(EntityId(1)).expect("insert");
        let b = graph.insert_node(EntityId(2)).expect("insert");
        // No edge between a and b

        let path = graph.strongest_path(a, b).expect("path");
        assert!(path.is_none());
    }
}
