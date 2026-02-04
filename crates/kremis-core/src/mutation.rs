//! # Mutation Engine
//!
//! Consolidates graph mutation operations for the Kremis CORE.
//!
//! Per ROADMAP.md Section 6:
//! > This is NOT learning.
//! > This is deterministic recording of signal occurrence.
//!
//! All mutations are:
//! - Deterministic
//! - Based on hardcoded rules
//! - Applied uniformly to all nodes and edges

use crate::graph::{Graph, GraphStore};
use crate::ingestor::Ingestor;
use crate::primitives::{ASSOCIATION_WINDOW, PROMOTION_THRESHOLD};
use crate::{EdgeWeight, KremisError, NodeId, Signal};

/// The MutationEngine consolidates all graph mutation operations.
///
/// Per KREMIS.md, the CORE is a closed system. All mutations follow
/// deterministic, hardcoded rules.
pub struct MutationEngine;

impl MutationEngine {
    /// Process a single signal and apply mutations to the graph.
    ///
    /// This is a convenience wrapper around `Ingestor::ingest_signal`.
    pub fn process_signal(graph: &mut Graph, signal: &Signal) -> Result<NodeId, KremisError> {
        Ingestor::ingest_signal(graph, signal)
    }

    /// Process a sequence of signals with automatic edge creation.
    ///
    /// Per ROADMAP.md Section 6.3.2:
    /// - Edges are created on co-occurrence
    /// - Weight increments on repetition using `saturating_add(1)`
    /// - Links formed only between adjacent signals (ASSOCIATION_WINDOW = 1)
    pub fn process_sequence(
        graph: &mut Graph,
        signals: &[Signal],
    ) -> Result<Vec<NodeId>, KremisError> {
        Ingestor::ingest_sequence(graph, signals)
    }

    /// Check if an edge weight is above the promotion threshold.
    ///
    /// Per ROADMAP.md Section 6.3.4:
    /// - Edges with `weight >= PROMOTION_THRESHOLD` are considered "Stable"
    /// - This is used by FACETS to determine edge significance
    /// - The CORE does not make decisions based on this; it only exposes the check
    #[must_use]
    pub fn is_stable_edge(weight: EdgeWeight) -> bool {
        weight.value() >= PROMOTION_THRESHOLD
    }

    /// Get the association window size.
    #[must_use]
    pub const fn association_window() -> usize {
        ASSOCIATION_WINDOW
    }

    /// Get the promotion threshold.
    #[must_use]
    pub const fn promotion_threshold() -> i64 {
        PROMOTION_THRESHOLD
    }

    /// Apply co-occurrence linking between two signals.
    ///
    /// This increments the edge weight between the two nodes by 1.
    pub fn link_signals(graph: &mut Graph, from: NodeId, to: NodeId) -> Result<(), KremisError> {
        graph.increment_edge(from, to)
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Attribute, EntityId, Value};

    fn make_signal(entity_id: u64, attr: &str, val: &str) -> Signal {
        Signal::new(EntityId(entity_id), Attribute::new(attr), Value::new(val))
    }

    #[test]
    fn process_signal_creates_node() {
        let mut graph = Graph::new();
        let signal = make_signal(1, "name", "Alice");

        let node = MutationEngine::process_signal(&mut graph, &signal).expect("process");

        assert!(graph.lookup(node).expect("lookup").is_some());
    }

    #[test]
    fn process_sequence_creates_edges() {
        let mut graph = Graph::new();
        let signals = vec![
            make_signal(1, "type", "word"),
            make_signal(2, "type", "word"),
        ];

        let nodes = MutationEngine::process_sequence(&mut graph, &signals).expect("process");

        assert_eq!(nodes.len(), 2);
        assert!(graph.get_edge(nodes[0], nodes[1]).expect("get").is_some());
    }

    #[test]
    fn is_stable_edge_checks_threshold() {
        assert!(!MutationEngine::is_stable_edge(EdgeWeight::new(5)));
        assert!(MutationEngine::is_stable_edge(EdgeWeight::new(10)));
        assert!(MutationEngine::is_stable_edge(EdgeWeight::new(100)));
    }

    #[test]
    fn link_signals_increments_weight() {
        let mut graph = Graph::new();
        let a = graph.insert_node(EntityId(1)).expect("insert");
        let b = graph.insert_node(EntityId(2)).expect("insert");

        MutationEngine::link_signals(&mut graph, a, b).expect("link");
        assert_eq!(graph.get_edge(a, b).expect("get"), Some(EdgeWeight::new(1)));

        MutationEngine::link_signals(&mut graph, a, b).expect("link");
        assert_eq!(graph.get_edge(a, b).expect("get"), Some(EdgeWeight::new(2)));
    }

    #[test]
    fn constants_are_correct() {
        assert_eq!(MutationEngine::association_window(), 1);
        assert_eq!(MutationEngine::promotion_threshold(), 10);
    }
}
