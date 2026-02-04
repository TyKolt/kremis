//! # Property-Based Tests
//!
//! Verification tests using proptest for Phase 6.
//!
//! These tests ensure determinism and correctness invariants.

use kremis_core::{EdgeWeight, EntityId, Graph, GraphStore};
use proptest::collection::vec;
use proptest::prelude::*;
use std::collections::BTreeSet;

// =============================================================================
// PROPERTY TESTS
// =============================================================================

proptest! {
    /// Same sequence of insertions produces identical graph structure.
    #[test]
    fn determinism_identical_input_produces_identical_output(
        entity_ids in vec(0u64..10000, 1..50)
    ) {
        let entities: Vec<EntityId> = entity_ids.iter().map(|&id| EntityId(id)).collect();

        let mut graph1 = Graph::new();
        let mut graph2 = Graph::new();

        for entity in &entities {
            graph1.insert_node(*entity).expect("insert");
            graph2.insert_node(*entity).expect("insert");
        }

        prop_assert_eq!(graph1.node_count().expect("count"), graph2.node_count().expect("count"));

        // Same entities should produce same NodeIds
        for entity in &entities {
            let node1 = graph1.get_node_by_entity(*entity);
            let node2 = graph2.get_node_by_entity(*entity);
            prop_assert_eq!(node1, node2);
        }
    }

    /// Inserting same entity twice returns the same NodeId.
    #[test]
    fn duplicate_entity_returns_same_node(id in 0u64..10000) {
        let entity = EntityId(id);
        let mut graph = Graph::new();

        let node1 = graph.insert_node(entity).expect("insert");
        let node2 = graph.insert_node(entity).expect("insert");

        prop_assert_eq!(node1, node2);
        prop_assert_eq!(graph.node_count().expect("count"), 1);
    }

    /// Edge weights increment correctly with saturating arithmetic.
    #[test]
    fn edge_weight_saturating_increment(weight_val in 1i64..1000) {
        let weight = EdgeWeight::new(weight_val);
        let incremented = weight.increment();

        // Should be at most 1 more (saturating)
        prop_assert!(incremented.value() >= weight.value());
        prop_assert!(incremented.value() <= weight.value().saturating_add(1));
    }

    /// Graph traversal is deterministic - same start produces same result.
    #[test]
    fn traversal_deterministic(
        entity_ids in vec(0u64..10000, 2..20),
        depth in 1usize..10
    ) {
        let entities: Vec<EntityId> = entity_ids.iter().map(|&id| EntityId(id)).collect();
        let mut graph = Graph::new();
        let mut previous_node = None;

        for entity in &entities {
            let node = graph.insert_node(*entity).expect("insert");
            if let Some(prev) = previous_node {
                graph.increment_edge(prev, node).expect("inc");
            }
            previous_node = Some(node);
        }

        let start = graph.insert_node(entities[0]).expect("insert");
        let result1 = graph.traverse(start, depth).expect("traverse");
        let result2 = graph.traverse(start, depth).expect("traverse");

        // Same traversal twice should produce identical results
        prop_assert_eq!(result1.is_some(), result2.is_some());
        if let (Some(a1), Some(a2)) = (result1, result2) {
            prop_assert_eq!(a1.path, a2.path);
        }
    }

    /// Node count equals unique entities inserted.
    #[test]
    fn node_count_reflects_unique_entities(entity_ids in vec(0u64..10000, 0..100)) {
        let mut graph = Graph::new();

        for id in &entity_ids {
            graph.insert_node(EntityId(*id)).expect("insert");
        }

        // Unique entities
        let unique_count = entity_ids.iter().collect::<BTreeSet<_>>().len();
        prop_assert_eq!(graph.node_count().expect("count"), unique_count);
    }

    /// Edge insertion creates valid edges.
    #[test]
    fn edge_insertion_creates_valid_edges(
        e1_id in 0u64..10000,
        e2_id in 0u64..10000,
        weight_val in 1i64..1000
    ) {
        let e1 = EntityId(e1_id);
        let e2 = EntityId(e2_id);
        let weight = EdgeWeight::new(weight_val);

        let mut graph = Graph::new();
        let n1 = graph.insert_node(e1).expect("insert");
        let n2 = graph.insert_node(e2).expect("insert");

        graph.insert_edge(n1, n2, weight).expect("edge");

        prop_assert!(graph.contains_edge(n1, n2));
        prop_assert_eq!(graph.get_edge(n1, n2).expect("get"), Some(weight));
    }
}
