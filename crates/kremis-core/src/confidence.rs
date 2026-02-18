//! # Confidence Module
//!
//! Confidence scoring for Core verification.
//!
//! - Score based on graph density supporting the claim
//! - More edges confirming fact = higher confidence
//! - Threshold for "verified" vs "speculative" output

use crate::Artifact;
use crate::graph::Graph;

/// Default threshold for considering a result "verified".
///
/// Results with confidence >= this threshold are considered verified.
/// Results below are considered speculative.
pub const VERIFIED_THRESHOLD: u8 = 70;

/// Confidence score for a grounded result.
///
/// Uses integer scoring (0-100) to maintain CORE determinism.
/// No floating-point arithmetic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ConfidenceScore {
    /// Score from 0 to 100.
    pub score: u8,
    /// Number of supporting edges in the artifact.
    pub evidence_count: usize,
    /// Number of nodes in the evidence path.
    pub path_length: usize,
}

impl ConfidenceScore {
    /// Create a new confidence score.
    #[must_use]
    pub fn new(score: u8, evidence_count: usize, path_length: usize) -> Self {
        Self {
            score: score.min(100),
            evidence_count,
            path_length,
        }
    }

    /// Zero confidence (no evidence).
    #[must_use]
    pub fn zero() -> Self {
        Self::default()
    }

    /// Maximum confidence (100%).
    #[must_use]
    pub fn max() -> Self {
        Self {
            score: 100,
            evidence_count: 0,
            path_length: 0,
        }
    }

    /// Check if the result is above the verified threshold.
    #[must_use]
    pub fn is_verified(&self) -> bool {
        self.score >= VERIFIED_THRESHOLD
    }

    /// Check if the result is speculative (below threshold).
    #[must_use]
    pub fn is_speculative(&self) -> bool {
        self.score < VERIFIED_THRESHOLD
    }
}

/// Compute confidence score for an artifact.
///
/// Scoring algorithm:
/// - Base score from path existence: 50 if path exists, 0 otherwise
/// - Bonus from edge count: +1 per edge, capped at +30
/// - Bonus from path length: +2 per node, capped at +20
///
/// All arithmetic uses saturating operations for determinism.
#[must_use]
pub fn compute_confidence(artifact: &Artifact, _graph: &Graph) -> ConfidenceScore {
    let path_length = artifact.path.len();
    let evidence_count = artifact.subgraph.as_ref().map_or(0, Vec::len);

    if path_length == 0 {
        return ConfidenceScore::zero();
    }

    // Base score for having a path
    let mut score: u8 = 50;

    // Bonus from edge count (max +30)
    let edge_bonus = (evidence_count.min(30)) as u8;
    score = score.saturating_add(edge_bonus);

    // Bonus from path length (max +20)
    let path_bonus = ((path_length.min(10)) as u8).saturating_mul(2).min(20);
    score = score.saturating_add(path_bonus);

    ConfidenceScore::new(score, evidence_count, path_length)
}

/// Compute confidence for a path between two nodes.
///
/// Higher weight paths = higher confidence.
#[must_use]
pub fn compute_path_confidence(path: &[crate::NodeId], graph: &Graph) -> ConfidenceScore {
    if path.len() < 2 {
        return if path.is_empty() {
            ConfidenceScore::zero()
        } else {
            ConfidenceScore::new(50, 0, 1)
        };
    }

    let mut total_weight: i64 = 0;
    let mut edge_count: usize = 0;

    for window in path.windows(2) {
        if let Some(weight) = graph.get_edge_internal(window[0], window[1]) {
            total_weight = total_weight.saturating_add(weight.value());
            edge_count = edge_count.saturating_add(1);
        }
    }

    // Score based on average weight
    let avg_weight = if edge_count > 0 {
        total_weight / (edge_count as i64)
    } else {
        0
    };

    // Map weight to score (higher weight = higher score)
    // Weight 1-10 maps to 50-100
    let weight_score = ((avg_weight.clamp(0, 10)) as u8)
        .saturating_mul(5)
        .saturating_add(50);

    ConfidenceScore::new(weight_score.min(100), edge_count, path.len())
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::NodeId;

    #[test]
    fn zero_confidence() {
        let score = ConfidenceScore::zero();
        assert_eq!(score.score, 0);
        assert!(!score.is_verified());
        assert!(score.is_speculative());
    }

    #[test]
    fn max_confidence() {
        let score = ConfidenceScore::max();
        assert_eq!(score.score, 100);
        assert!(score.is_verified());
    }

    #[test]
    fn threshold_boundary() {
        let below = ConfidenceScore::new(69, 0, 0);
        let at = ConfidenceScore::new(70, 0, 0);
        let above = ConfidenceScore::new(71, 0, 0);

        assert!(!below.is_verified());
        assert!(at.is_verified());
        assert!(above.is_verified());
    }

    #[test]
    fn compute_confidence_empty_artifact() {
        let graph = Graph::new();
        let artifact = Artifact::with_path(vec![]);

        let score = compute_confidence(&artifact, &graph);
        assert_eq!(score.score, 0);
    }

    #[test]
    fn compute_confidence_with_path() {
        let graph = Graph::new();
        let artifact = Artifact::with_path(vec![NodeId(1), NodeId(2), NodeId(3)]);

        let score = compute_confidence(&artifact, &graph);
        assert!(score.score >= 50); // At least base score
    }

    // =========================================================================
    // M4 - compute_path_confidence tests
    // =========================================================================

    #[test]
    fn path_confidence_empty_path() {
        let graph = Graph::new();
        let path: Vec<NodeId> = vec![];

        let score = compute_path_confidence(&path, &graph);
        assert_eq!(score.score, 0);
        assert_eq!(score.evidence_count, 0);
        assert_eq!(score.path_length, 0);
        assert!(score.is_speculative());
    }

    #[test]
    fn path_confidence_single_node() {
        let graph = Graph::new();
        let path = vec![NodeId(1)];

        let score = compute_path_confidence(&path, &graph);
        assert_eq!(score.score, 50); // Single node gets base score
        assert_eq!(score.evidence_count, 0);
        assert_eq!(score.path_length, 1);
    }

    #[test]
    fn path_confidence_two_nodes_no_edge() {
        use crate::graph::GraphStore;

        let mut graph = Graph::new();
        let n1 = graph.insert_node(crate::EntityId(1)).expect("insert");
        let n2 = graph.insert_node(crate::EntityId(2)).expect("insert");
        // No edge between n1 and n2

        let path = vec![n1, n2];
        let score = compute_path_confidence(&path, &graph);

        // No edge found, so edge_count is 0
        assert_eq!(score.evidence_count, 0);
        assert_eq!(score.path_length, 2);
    }

    #[test]
    fn path_confidence_two_nodes_with_edge() {
        use crate::graph::GraphStore;
        use crate::{EdgeWeight, EntityId};

        let mut graph = Graph::new();
        let n1 = graph.insert_node(EntityId(1)).expect("insert");
        let n2 = graph.insert_node(EntityId(2)).expect("insert");
        graph
            .insert_edge(n1, n2, EdgeWeight::new(5))
            .expect("insert");

        let path = vec![n1, n2];
        let score = compute_path_confidence(&path, &graph);

        assert_eq!(score.evidence_count, 1);
        assert_eq!(score.path_length, 2);
        // Weight 5 -> score should be 50 + 5*5 = 75
        assert!(score.score >= 50);
    }

    #[test]
    fn path_confidence_long_path() {
        use crate::graph::GraphStore;
        use crate::{EdgeWeight, EntityId};

        let mut graph = Graph::new();
        let nodes: Vec<NodeId> = (0..10)
            .map(|i| graph.insert_node(EntityId(i)).expect("insert"))
            .collect();

        // Create edges between consecutive nodes with weight 5
        for i in 0..9 {
            graph
                .insert_edge(nodes[i], nodes[i + 1], EdgeWeight::new(5))
                .expect("insert");
        }

        let score = compute_path_confidence(&nodes, &graph);

        assert_eq!(score.evidence_count, 9); // 9 edges
        assert_eq!(score.path_length, 10); // 10 nodes
        assert!(score.score >= 50); // At least base score
    }

    #[test]
    fn path_confidence_high_weight_edges() {
        use crate::graph::GraphStore;
        use crate::{EdgeWeight, EntityId};

        let mut graph = Graph::new();
        let n1 = graph.insert_node(EntityId(1)).expect("insert");
        let n2 = graph.insert_node(EntityId(2)).expect("insert");
        let n3 = graph.insert_node(EntityId(3)).expect("insert");

        // High weight edges (10 is max for scoring)
        graph
            .insert_edge(n1, n2, EdgeWeight::new(10))
            .expect("insert");
        graph
            .insert_edge(n2, n3, EdgeWeight::new(10))
            .expect("insert");

        let path = vec![n1, n2, n3];
        let score = compute_path_confidence(&path, &graph);

        // Average weight 10 -> score should be 50 + 10*5 = 100
        assert_eq!(score.score, 100);
        assert!(score.is_verified());
    }

    #[test]
    fn path_confidence_low_weight_edges() {
        use crate::graph::GraphStore;
        use crate::{EdgeWeight, EntityId};

        let mut graph = Graph::new();
        let n1 = graph.insert_node(EntityId(1)).expect("insert");
        let n2 = graph.insert_node(EntityId(2)).expect("insert");
        let n3 = graph.insert_node(EntityId(3)).expect("insert");

        // Low weight edges
        graph
            .insert_edge(n1, n2, EdgeWeight::new(1))
            .expect("insert");
        graph
            .insert_edge(n2, n3, EdgeWeight::new(1))
            .expect("insert");

        let path = vec![n1, n2, n3];
        let score = compute_path_confidence(&path, &graph);

        // Average weight 1 -> score should be 50 + 1*5 = 55
        assert_eq!(score.score, 55);
        assert!(!score.is_verified()); // Below 70 threshold
    }

    #[test]
    fn path_confidence_mixed_weights() {
        use crate::graph::GraphStore;
        use crate::{EdgeWeight, EntityId};

        let mut graph = Graph::new();
        let n1 = graph.insert_node(EntityId(1)).expect("insert");
        let n2 = graph.insert_node(EntityId(2)).expect("insert");
        let n3 = graph.insert_node(EntityId(3)).expect("insert");
        let n4 = graph.insert_node(EntityId(4)).expect("insert");

        // Mixed weights: 2, 8, 5 -> average = 5
        graph
            .insert_edge(n1, n2, EdgeWeight::new(2))
            .expect("insert");
        graph
            .insert_edge(n2, n3, EdgeWeight::new(8))
            .expect("insert");
        graph
            .insert_edge(n3, n4, EdgeWeight::new(5))
            .expect("insert");

        let path = vec![n1, n2, n3, n4];
        let score = compute_path_confidence(&path, &graph);

        assert_eq!(score.evidence_count, 3);
        assert_eq!(score.path_length, 4);
        // Average weight 5 -> score should be 50 + 5*5 = 75
        assert_eq!(score.score, 75);
        assert!(score.is_verified());
    }

    #[test]
    fn path_confidence_very_high_weight_capped() {
        use crate::graph::GraphStore;
        use crate::{EdgeWeight, EntityId};

        let mut graph = Graph::new();
        let n1 = graph.insert_node(EntityId(1)).expect("insert");
        let n2 = graph.insert_node(EntityId(2)).expect("insert");

        // Very high weight (above 10, should be capped)
        graph
            .insert_edge(n1, n2, EdgeWeight::new(1000))
            .expect("insert");

        let path = vec![n1, n2];
        let score = compute_path_confidence(&path, &graph);

        // Weight capped at 10 -> score should be 100 (50 + 10*5)
        assert_eq!(score.score, 100);
    }

    #[test]
    fn path_confidence_negative_weight() {
        use crate::graph::GraphStore;
        use crate::{EdgeWeight, EntityId};

        let mut graph = Graph::new();
        let n1 = graph.insert_node(EntityId(1)).expect("insert");
        let n2 = graph.insert_node(EntityId(2)).expect("insert");

        // Negative weight
        graph
            .insert_edge(n1, n2, EdgeWeight::new(-5))
            .expect("insert");

        let path = vec![n1, n2];
        let score = compute_path_confidence(&path, &graph);

        // Negative weight, min(x, 10) still works, then multiplied
        assert!(score.score >= 50); // At least base
    }

    #[test]
    fn path_confidence_zero_weight() {
        use crate::graph::GraphStore;
        use crate::{EdgeWeight, EntityId};

        let mut graph = Graph::new();
        let n1 = graph.insert_node(EntityId(1)).expect("insert");
        let n2 = graph.insert_node(EntityId(2)).expect("insert");

        // Zero weight
        graph
            .insert_edge(n1, n2, EdgeWeight::new(0))
            .expect("insert");

        let path = vec![n1, n2];
        let score = compute_path_confidence(&path, &graph);

        // Zero weight -> score should be 50 + 0*5 = 50
        assert_eq!(score.score, 50);
    }

    #[test]
    fn path_confidence_partial_path_edges() {
        use crate::graph::GraphStore;
        use crate::{EdgeWeight, EntityId};

        let mut graph = Graph::new();
        let n1 = graph.insert_node(EntityId(1)).expect("insert");
        let n2 = graph.insert_node(EntityId(2)).expect("insert");
        let n3 = graph.insert_node(EntityId(3)).expect("insert");
        let n4 = graph.insert_node(EntityId(4)).expect("insert");

        // Only some edges exist: n1->n2 and n3->n4, but NOT n2->n3
        graph
            .insert_edge(n1, n2, EdgeWeight::new(10))
            .expect("insert");
        graph
            .insert_edge(n3, n4, EdgeWeight::new(10))
            .expect("insert");

        let path = vec![n1, n2, n3, n4];
        let score = compute_path_confidence(&path, &graph);

        // Only 2 out of 3 edges exist
        assert_eq!(score.evidence_count, 2);
        assert_eq!(score.path_length, 4);
    }

    #[test]
    fn path_confidence_deterministic() {
        use crate::graph::GraphStore;
        use crate::{EdgeWeight, EntityId};

        let mut graph = Graph::new();
        let n1 = graph.insert_node(EntityId(1)).expect("insert");
        let n2 = graph.insert_node(EntityId(2)).expect("insert");
        let n3 = graph.insert_node(EntityId(3)).expect("insert");

        graph
            .insert_edge(n1, n2, EdgeWeight::new(7))
            .expect("insert");
        graph
            .insert_edge(n2, n3, EdgeWeight::new(3))
            .expect("insert");

        let path = vec![n1, n2, n3];

        // Multiple calls should return identical results
        let score1 = compute_path_confidence(&path, &graph);
        let score2 = compute_path_confidence(&path, &graph);
        let score3 = compute_path_confidence(&path, &graph);

        assert_eq!(score1, score2);
        assert_eq!(score2, score3);
    }
}
