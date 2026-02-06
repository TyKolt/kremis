//! # Compositor Module
//!
//! Output assembly protocol for Kremis CORE.
//!
//! Per ROADMAP.md Section 5.2.2:
//! - Output raw structure only
//! - No natural language generation
//! - No formatting logic in the Core
//! - Return `Result<Option<Artifact>, KremisError>` for traversal results

use crate::graph::GraphStore;
use crate::{Artifact, EdgeWeight, KremisError, NodeId};

/// The Compositor handles output assembly from the graph.
///
/// Per AGENTS.md Section 3.3, the Compositor:
/// - Traverses the graph from active nodes
/// - Extracts paths or subgraphs
/// - Assembles Graph Artifacts
/// - Does NOT generate language, text, or meaning
pub struct Compositor;

impl Compositor {
    /// Compose an artifact by traversing from a starting node.
    ///
    /// Returns `Ok(None)` if the node doesn't exist.
    pub fn compose<G: GraphStore>(
        graph: &G,
        start: NodeId,
        depth: usize,
    ) -> Result<Option<Artifact>, KremisError> {
        graph.traverse(start, depth)
    }

    /// Compose an artifact with weight filtering.
    ///
    /// Only includes edges with weight >= min_weight.
    pub fn compose_filtered<G: GraphStore>(
        graph: &G,
        start: NodeId,
        depth: usize,
        min_weight: EdgeWeight,
    ) -> Result<Option<Artifact>, KremisError> {
        graph.traverse_filtered(start, depth, min_weight)
    }

    /// Extract a path between two nodes.
    ///
    /// Uses strongest_path algorithm (maximizes edge weights).
    pub fn extract_path<G: GraphStore>(
        graph: &G,
        start: NodeId,
        end: NodeId,
    ) -> Result<Option<Artifact>, KremisError> {
        let path = match graph.strongest_path(start, end)? {
            Some(p) => p,
            None => return Ok(None),
        };

        // Collect edges along the path
        let mut subgraph = Vec::new();
        for window in path.windows(2) {
            let from = window[0];
            let to = window[1];
            if let Some(weight) = graph.get_edge(from, to)? {
                subgraph.push((from, to, weight));
            }
        }

        Ok(Some(Artifact::with_subgraph(path, subgraph)))
    }

    /// Find common connections between multiple nodes.
    ///
    /// Returns an artifact containing the intersection nodes.
    pub fn find_intersection<G: GraphStore>(
        graph: &G,
        nodes: &[NodeId],
    ) -> Result<Artifact, KremisError> {
        let common = graph.intersect(nodes)?;
        Ok(Artifact::with_path(common))
    }

    /// Extract a related subgraph from a starting point.
    pub fn related_context<G: GraphStore>(
        graph: &G,
        start: NodeId,
        depth: usize,
    ) -> Result<Option<Artifact>, KremisError> {
        graph.traverse(start, depth)
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::EntityId;
    use crate::graph::{Graph, GraphStore};

    #[test]
    fn compose_returns_none_for_missing_node() {
        let graph = Graph::new();
        let result = Compositor::compose(&graph, NodeId(999), 5).expect("compose");
        assert!(result.is_none());
    }

    #[test]
    fn compose_returns_artifact_for_existing_node() {
        let mut graph = Graph::new();
        let node = graph.insert_node(EntityId(1)).expect("insert");

        let result = Compositor::compose(&graph, node, 1).expect("compose");
        assert!(result.is_some());
        assert!(!result.as_ref().map(|a| a.path.is_empty()).unwrap_or(true));
    }

    #[test]
    fn extract_path_finds_route() {
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

        let artifact = Compositor::extract_path(&graph, a, c).expect("extract");
        assert!(artifact.is_some());

        let path = artifact.as_ref().map(|a| &a.path);
        assert_eq!(path, Some(&vec![a, b, c]));
    }

    #[test]
    fn find_intersection_returns_common_neighbors() {
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

        let artifact = Compositor::find_intersection(&graph, &[a, b]).expect("intersect");
        assert_eq!(artifact.path, vec![common]);
    }
}
