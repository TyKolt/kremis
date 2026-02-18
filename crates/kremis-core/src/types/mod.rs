//! # Core Type Definitions
//!
//! MERGED FROM: kremis-types crate
//!
//! This module contains all core types for the Kremis deterministic graph substrate:
//! - Entity and graph identifiers (`EntityId`, `NodeId`, `EdgeWeight`)
//! - Signal representation (`Signal`, `Attribute`, `Value`)
//! - Output structures (`Artifact`, `Buffer`)
//! - Error types (`KremisError`)
//! - Facet trait
//!
//! ## Determinism Guarantees
//!
//! All types in this module:
//! - Use integer arithmetic only (no floating-point)
//! - Implement `Ord` for deterministic ordering in `BTreeMap`/`BTreeSet`
//! - Use saturating arithmetic for counters to prevent overflow

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use thiserror::Error;

// =============================================================================
// ENTITY & GRAPH IDENTIFIERS
// =============================================================================

/// Unique identifier for an entity in the external world.
/// Entities are the semantic units that signals refer to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct EntityId(pub u64);

/// Unique identifier for a node in the internal graph.
/// Nodes are the structural representation of entities within the CORE.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct NodeId(pub u64);

/// Weight of a directed edge in the graph.
/// Uses i64 with saturating arithmetic to prevent overflow.
/// Higher weight indicates stronger association (more co-occurrences).
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default,
)]
pub struct EdgeWeight(pub i64);

impl EdgeWeight {
    /// Create a new edge weight with the given value.
    #[must_use]
    pub const fn new(weight: i64) -> Self {
        Self(weight)
    }

    /// Increment the edge weight by 1 using saturating arithmetic.
    /// This is the ONLY allowed mutation for edge weights.
    #[must_use]
    pub const fn increment(self) -> Self {
        Self(self.0.saturating_add(1))
    }

    /// Get the raw weight value.
    #[must_use]
    pub const fn value(self) -> i64 {
        self.0
    }
}

// =============================================================================
// NODE
// =============================================================================

/// A Node in the graph, representing a structural entity.
///
/// A Node contains only an EntityId.
/// The NodeId is the internal identifier used for graph operations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Node {
    /// The internal node identifier.
    pub id: NodeId,
    /// The external entity this node represents.
    pub entity: EntityId,
}

impl Node {
    /// Create a new node.
    #[must_use]
    pub const fn new(id: NodeId, entity: EntityId) -> Self {
        Self { id, entity }
    }
}

// =============================================================================
// SIGNAL COMPONENTS
// =============================================================================

/// Attribute component of a signal.
/// Represents the relationship type between entity and value.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Attribute(pub String);

impl Attribute {
    /// Create a new attribute from a string.
    #[must_use]
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    /// Get the attribute as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Value component of a signal.
/// Represents the data associated with an entity-attribute pair.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Value(pub String);

impl Value {
    /// Create a new value from a string.
    #[must_use]
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    /// Get the value as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

// =============================================================================
// SIGNAL
// =============================================================================

/// A Signal is the fundamental unit of input to the CORE.
///
/// Signals are normalized representations of external events in the form:
/// `[Entity | Attribute | Value]`
///
/// If input cannot be represented in this form,
/// it must be discarded. No interpretation or semantic inference is allowed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Signal {
    /// The entity this signal refers to.
    pub entity: EntityId,
    /// The attribute (relationship type) of this signal.
    pub attribute: Attribute,
    /// The value associated with the entity-attribute pair.
    pub value: Value,
}

impl Signal {
    /// Create a new signal.
    #[must_use]
    pub fn new(entity: EntityId, attribute: Attribute, value: Value) -> Self {
        Self {
            entity,
            attribute,
            value,
        }
    }
}

// =============================================================================
// ARTIFACT
// =============================================================================

/// An Artifact is the output of a graph traversal operation.
///
/// The Compositor outputs raw symbolic structures only.
/// No language, text, or meaning generation is allowed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Artifact {
    /// The path of nodes traversed.
    pub path: Vec<NodeId>,
    /// Optional subgraph extracted (edges with weights).
    pub subgraph: Option<Vec<(NodeId, NodeId, EdgeWeight)>>,
}

impl Artifact {
    /// Create a new empty artifact.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create an artifact with just a path.
    #[must_use]
    pub fn with_path(path: Vec<NodeId>) -> Self {
        Self {
            path,
            subgraph: None,
        }
    }

    /// Create an artifact with both path and subgraph.
    #[must_use]
    pub fn with_subgraph(path: Vec<NodeId>, subgraph: Vec<(NodeId, NodeId, EdgeWeight)>) -> Self {
        Self {
            path,
            subgraph: Some(subgraph),
        }
    }

    /// Check if the artifact is empty (no path, no subgraph).
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.path.is_empty() && self.subgraph.as_ref().is_none_or(Vec::is_empty)
    }
}

// =============================================================================
// BUFFER (Active Context)
// =============================================================================

/// Buffer is the volatile, session-local working memory.
///
/// - Active Context is VOLATILE
/// - Never serialized to disk
/// - Cleared on `Buffer::clear()`
/// - Does not contribute to state persistence
/// - Cannot be used for "long-term memory"
#[derive(Debug, Clone, Default)]
pub struct Buffer {
    /// Currently activated nodes in this session.
    /// Uses BTreeSet for deterministic ordering.
    pub active_nodes: BTreeSet<NodeId>,
}

impl Buffer {
    /// Create a new empty buffer.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Clear all active nodes from the buffer.
    /// This resets the session-local state.
    pub fn clear(&mut self) {
        self.active_nodes.clear();
    }

    /// Add a node to the active context.
    pub fn activate(&mut self, node: NodeId) {
        self.active_nodes.insert(node);
    }

    /// Remove a node from the active context.
    pub fn deactivate(&mut self, node: &NodeId) {
        self.active_nodes.remove(node);
    }

    /// Check if a node is currently active.
    #[must_use]
    pub fn is_active(&self, node: &NodeId) -> bool {
        self.active_nodes.contains(node)
    }
}

// =============================================================================
// FACET TRAIT
// =============================================================================

/// The Facet trait defines the interface between external world and CORE.
///
/// - `ingest`: Transforms raw bytes into a Signal
/// - `emit`: Transforms an Artifact into raw bytes
///
/// Facets must be `Send + Sync` for thread safety.
///
/// # Extension Point
///
/// This trait is intentionally defined without in-crate implementations.
/// It serves as an extension point for external adapters (file parsers,
/// network protocols, database connectors) that need to interface with
/// the CORE. Implementors should be stateless and pure.
pub trait Facet: Send + Sync {
    /// Ingest raw input and produce a normalized Signal.
    ///
    /// This is a pure transformation with no side effects.
    /// Returns `KremisError::InvalidSignal` if input cannot be parsed.
    fn ingest(&self, raw: &[u8]) -> Result<Signal, KremisError>;

    /// Emit an Artifact as raw bytes for output.
    ///
    /// This is a pure transformation with no side effects.
    fn emit(&self, artifact: &Artifact) -> Result<Vec<u8>, KremisError>;
}

// =============================================================================
// ERROR TYPES
// =============================================================================

/// Errors that can occur in the Kremis system.
///
/// - No silent failures
/// - Use `Result<T, KremisError>` for fallible operations
/// - The CORE should never panic; all errors must be recoverable
#[derive(Debug, Error)]
pub enum KremisError {
    /// The input signal format is invalid and cannot be parsed.
    #[error("Invalid signal format")]
    InvalidSignal,

    /// The requested node was not found in the graph.
    #[error("Node not found: {0:?}")]
    NodeNotFound(NodeId),

    /// The requested edge was not found in the graph.
    #[error("Edge not found: {0:?} -> {1:?}")]
    EdgeNotFound(NodeId, NodeId),

    /// A traversal operation failed to complete.
    #[error("Traversal failed")]
    TraversalFailed,

    /// A serialization or deserialization error occurred.
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// A deserialization error occurred.
    #[error("Deserialization error: {0}")]
    DeserializationError(String),

    /// An I/O error occurred.
    #[error("I/O error: {0}")]
    IoError(String),
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn edge_weight_saturating_increment() {
        let weight = EdgeWeight::new(i64::MAX);
        let incremented = weight.increment();
        assert_eq!(incremented.value(), i64::MAX);
    }

    #[test]
    fn edge_weight_normal_increment() {
        let weight = EdgeWeight::new(0);
        let incremented = weight.increment();
        assert_eq!(incremented.value(), 1);
    }

    #[test]
    fn buffer_operations() {
        let mut buffer = Buffer::new();
        let node = NodeId(1);

        buffer.activate(node);
        assert!(buffer.is_active(&node));

        buffer.deactivate(&node);
        assert!(!buffer.is_active(&node));

        buffer.activate(node);
        buffer.clear();
        assert!(!buffer.is_active(&node));
    }

    #[test]
    fn buffer_deterministic_ordering() {
        let mut buffer = Buffer::new();
        buffer.activate(NodeId(3));
        buffer.activate(NodeId(1));
        buffer.activate(NodeId(2));

        let nodes: Vec<_> = buffer.active_nodes.iter().copied().collect();
        assert_eq!(nodes, vec![NodeId(1), NodeId(2), NodeId(3)]);
    }

    #[test]
    fn artifact_is_empty() {
        let empty = Artifact::new();
        assert!(empty.is_empty());

        let with_path = Artifact::with_path(vec![NodeId(1)]);
        assert!(!with_path.is_empty());
    }
}
