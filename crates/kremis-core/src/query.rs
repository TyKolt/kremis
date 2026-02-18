//! # Query Module
//!
//! Structured query types for Core interaction.
//!
//! - Map user questions to structured traversal operations
//! - Deterministic query parsing (no semantic guessing)
//! - Support for complex queries

use crate::{EdgeWeight, EntityId, NodeId};

/// Query operation types supported by the CORE.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QueryType {
    /// Lookup a node by entity ID.
    Lookup(EntityId),

    /// Traverse from a starting node up to a depth limit.
    Traverse { start: NodeId, depth: usize },

    /// Traverse with minimum weight filter.
    TraverseFiltered {
        start: NodeId,
        depth: usize,
        min_weight: EdgeWeight,
    },

    /// Find the strongest path between two nodes.
    StrongestPath { start: NodeId, end: NodeId },

    /// Find nodes connected to ALL input nodes.
    Intersect(Vec<NodeId>),

    /// Depth-first traversal.
    TraverseDfs { start: NodeId, depth: usize },
}

/// A structured query with optional timeout.
#[derive(Debug, Clone)]
pub struct Query {
    /// The type of query operation.
    pub query_type: QueryType,
    /// Optional timeout in milliseconds.
    pub timeout_ms: Option<u64>,
}

impl Query {
    /// Create a new query with no timeout.
    #[must_use]
    pub fn new(query_type: QueryType) -> Self {
        Self {
            query_type,
            timeout_ms: None,
        }
    }

    /// Create a new query with a timeout.
    #[must_use]
    pub fn with_timeout(query_type: QueryType, timeout_ms: u64) -> Self {
        Self {
            query_type,
            timeout_ms: Some(timeout_ms),
        }
    }

    /// Lookup helper.
    #[must_use]
    pub fn lookup(entity: EntityId) -> Self {
        Self::new(QueryType::Lookup(entity))
    }

    /// Traverse helper.
    #[must_use]
    pub fn traverse(start: NodeId, depth: usize) -> Self {
        Self::new(QueryType::Traverse { start, depth })
    }

    /// Strongest path helper.
    #[must_use]
    pub fn strongest_path(start: NodeId, end: NodeId) -> Self {
        Self::new(QueryType::StrongestPath { start, end })
    }

    /// Intersect helper.
    #[must_use]
    pub fn intersect(nodes: Vec<NodeId>) -> Self {
        Self::new(QueryType::Intersect(nodes))
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn query_creation() {
        let q = Query::lookup(EntityId(42));
        assert_eq!(q.query_type, QueryType::Lookup(EntityId(42)));
        assert_eq!(q.timeout_ms, None);
    }

    #[test]
    fn query_with_timeout() {
        let q = Query::with_timeout(
            QueryType::Traverse {
                start: NodeId(1),
                depth: 5,
            },
            1000,
        );
        assert_eq!(q.timeout_ms, Some(1000));
    }

    #[test]
    fn query_helpers() {
        let _ = Query::traverse(NodeId(1), 10);
        let _ = Query::strongest_path(NodeId(1), NodeId(2));
        let _ = Query::intersect(vec![NodeId(1), NodeId(2)]);
    }
}
