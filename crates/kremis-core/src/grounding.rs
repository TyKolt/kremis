//! # Grounding Module
//!
//! Hypothesis verification for CORTEX-CORE interaction.
//!
//! Per ROADMAP.md Section 10.7.3:
//! - CORTEX generates hypothesis
//! - CORE validates via graph traversal
//! - Result annotated with evidence path
//! - Unverifiable claims explicitly marked

use crate::confidence::{ConfidenceScore, compute_confidence, compute_path_confidence};
use crate::graph::{Graph, GraphStore};
use crate::query::{Query, QueryType};
use crate::{Artifact, NodeId};

/// Result of hypothesis verification.
///
/// Contains the artifact (if found), confidence score,
/// and verification status.
#[derive(Debug, Clone)]
pub struct GroundedResult {
    /// The artifact from the query, if any.
    pub artifact: Option<Artifact>,
    /// Confidence score for the result.
    pub confidence: ConfidenceScore,
    /// Whether the result is verified (above threshold).
    pub verified: bool,
    /// The evidence path supporting the result.
    pub evidence_path: Vec<NodeId>,
}

impl GroundedResult {
    /// Create an unverified (no evidence) result.
    #[must_use]
    pub fn unverified() -> Self {
        Self {
            artifact: None,
            confidence: ConfidenceScore::zero(),
            verified: false,
            evidence_path: Vec::new(),
        }
    }

    /// Create a verified result with artifact.
    #[must_use]
    pub fn with_artifact(artifact: Artifact, confidence: ConfidenceScore) -> Self {
        let verified = confidence.is_verified();
        let evidence_path = artifact.path.clone();
        Self {
            artifact: Some(artifact),
            confidence,
            verified,
            evidence_path,
        }
    }
}

/// Execute a query and return a grounded result.
///
/// This is the main entry point for CORTEX-CORE interaction.
/// It executes the query, computes confidence, and annotates
/// the result with verification status.
#[must_use]
pub fn verify_hypothesis(graph: &Graph, query: Query) -> GroundedResult {
    match query.query_type {
        QueryType::Lookup(entity) => {
            if let Some(node_id) = graph.get_node_by_entity(entity) {
                let artifact = Artifact::with_path(vec![node_id]);
                let confidence = ConfidenceScore::new(100, 0, 1);
                GroundedResult::with_artifact(artifact, confidence)
            } else {
                GroundedResult::unverified()
            }
        }

        QueryType::Traverse { start, depth } => match graph.traverse(start, depth) {
            Ok(Some(artifact)) => {
                let confidence = compute_confidence(&artifact, graph);
                GroundedResult::with_artifact(artifact, confidence)
            }
            _ => GroundedResult::unverified(),
        },

        QueryType::TraverseFiltered {
            start,
            depth,
            min_weight,
        } => match graph.traverse_filtered(start, depth, min_weight) {
            Ok(Some(artifact)) => {
                let confidence = compute_confidence(&artifact, graph);
                GroundedResult::with_artifact(artifact, confidence)
            }
            _ => GroundedResult::unverified(),
        },

        QueryType::StrongestPath { start, end } => match graph.strongest_path(start, end) {
            Ok(Some(path)) => {
                let confidence = compute_path_confidence(&path, graph);
                let artifact = Artifact::with_path(path);
                GroundedResult::with_artifact(artifact, confidence)
            }
            _ => GroundedResult::unverified(),
        },

        QueryType::Intersect(ref nodes) => match graph.intersect(nodes) {
            Ok(common) if !common.is_empty() => {
                let artifact = Artifact::with_path(common);
                let confidence = compute_confidence(&artifact, graph);
                GroundedResult::with_artifact(artifact, confidence)
            }
            _ => GroundedResult::unverified(),
        },

        QueryType::TraverseDfs { start, depth } => {
            if let Some(artifact) = graph.traverse_dfs(start, depth) {
                let confidence = compute_confidence(&artifact, graph);
                GroundedResult::with_artifact(artifact, confidence)
            } else {
                GroundedResult::unverified()
            }
        }
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{EdgeWeight, EntityId};

    #[test]
    fn verify_lookup_existing() {
        let mut graph = Graph::new();
        let entity = EntityId(42);
        graph.insert_node(entity).expect("insert");

        let query = Query::lookup(entity);
        let result = verify_hypothesis(&graph, query);

        assert!(result.verified);
        assert!(result.artifact.is_some());
        assert_eq!(result.confidence.score, 100);
    }

    #[test]
    fn verify_lookup_missing() {
        let graph = Graph::new();
        let query = Query::lookup(EntityId(999));
        let result = verify_hypothesis(&graph, query);

        assert!(!result.verified);
        assert!(result.artifact.is_none());
        assert_eq!(result.confidence.score, 0);
    }

    #[test]
    fn verify_traverse() {
        let mut graph = Graph::new();
        let a = graph.insert_node(EntityId(1)).expect("insert");
        let b = graph.insert_node(EntityId(2)).expect("insert");
        graph.insert_edge(a, b, EdgeWeight::new(5)).expect("insert");

        let query = Query::traverse(a, 2);
        let result = verify_hypothesis(&graph, query);

        assert!(result.artifact.is_some());
        assert!(result.confidence.score >= 50);
    }

    #[test]
    fn verify_strongest_path() {
        let mut graph = Graph::new();
        let a = graph.insert_node(EntityId(1)).expect("insert");
        let b = graph.insert_node(EntityId(2)).expect("insert");
        let c = graph.insert_node(EntityId(3)).expect("insert");
        graph
            .insert_edge(a, b, EdgeWeight::new(10))
            .expect("insert");
        graph
            .insert_edge(b, c, EdgeWeight::new(10))
            .expect("insert");

        let query = Query::strongest_path(a, c);
        let result = verify_hypothesis(&graph, query);

        assert!(result.artifact.is_some());
        assert!(result.verified);
        assert!(!result.evidence_path.is_empty());
    }

    #[test]
    fn verify_intersect() {
        let mut graph = Graph::new();
        let a = graph.insert_node(EntityId(1)).expect("insert");
        let b = graph.insert_node(EntityId(2)).expect("insert");
        let common = graph.insert_node(EntityId(100)).expect("insert");
        graph
            .insert_edge(a, common, EdgeWeight::new(1))
            .expect("insert");
        graph
            .insert_edge(b, common, EdgeWeight::new(1))
            .expect("insert");

        let query = Query::intersect(vec![a, b]);
        let result = verify_hypothesis(&graph, query);

        assert!(result.artifact.is_some());
        let path = result.artifact.as_ref().map(|a| &a.path);
        assert_eq!(path, Some(&vec![common]));
    }
}
