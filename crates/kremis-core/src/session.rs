//! # Session Module
//!
//! Session management combining Graph and Buffer.
//!
//! Per ROADMAP.md Section 5.2.3:
//! - Buffer is volatile, session-local state
//! - Never serialized to disk
//! - Cleared on session reset
//! - Does not contribute to state persistence
//!
//! ## Storage Backends
//!
//! Session supports two storage backends:
//! - `InMemory`: Uses in-memory `Graph` (fast, volatile unless explicitly saved)
//! - `Persistent`: Uses `RedbGraph` for disk-backed ACID storage

use crate::graph::{Graph, GraphStore};
use crate::ingestor::Ingestor;
use crate::storage::RedbGraph;
use crate::{Artifact, Buffer, EdgeWeight, EntityId, KremisError, NodeId, Signal};
use std::path::Path;

// =============================================================================
// ERROR LOGGING HELPERS
// =============================================================================

/// Log an I/O error and convert Result to Option.
///
/// This helper ensures that storage errors are logged before being converted
/// to Option::None, preventing silent error swallowing.
///
/// # M2 Fix
///
/// Uses stderr logging for CORE (no external dependencies).
/// The app layer should configure proper tracing if needed.
#[inline]
fn log_and_convert<T>(result: Result<T, KremisError>, context: &str) -> Option<T> {
    match result {
        Ok(v) => Some(v),
        Err(e) => {
            // M2 FIX: Use structured logging format for easier parsing
            // Note: CORE avoids tracing dependency to stay minimal.
            // App layer should redirect stderr to tracing if needed.
            eprintln!(
                "{{\"level\":\"warn\",\"target\":\"kremis_core::session\",\"message\":\"I/O error in {}: {}\"}}",
                context, e
            );
            None
        }
    }
}

/// Log an I/O error and convert Result<T, E> to default value.
///
/// This helper ensures that storage errors are logged before being converted
/// to a default value, preventing silent error swallowing.
///
/// # M2 Fix
///
/// Uses stderr logging for CORE (no external dependencies).
/// The app layer should configure proper tracing if needed.
#[inline]
fn log_and_default<T: Default>(result: Result<T, KremisError>, context: &str) -> T {
    match result {
        Ok(v) => v,
        Err(e) => {
            // M2 FIX: Use structured logging format for easier parsing
            eprintln!(
                "{{\"level\":\"warn\",\"target\":\"kremis_core::session\",\"message\":\"I/O error in {}: {}\"}}",
                context, e
            );
            T::default()
        }
    }
}

/// Storage backend for a Session.
///
/// Per ROADMAP.md, supports both in-memory and persistent storage.
#[derive(Debug)]
pub enum StorageBackend {
    /// In-memory graph (fast, volatile).
    InMemory(Graph),
    /// Disk-backed graph using redb (ACID, persistent).
    Persistent(RedbGraph),
}

impl Default for StorageBackend {
    fn default() -> Self {
        Self::InMemory(Graph::new())
    }
}

// NOTE: StorageBackend does NOT implement Clone.
// RedbGraph (database handle) cannot be safely cloned.
// Use Session::try_clone() for explicit cloning with proper error handling.

/// A Session combines a Graph with a volatile Buffer.
///
/// The Session provides a high-level interface for:
/// - Ingesting signals
/// - Managing active context
/// - Composing output artifacts
///
/// Note: Session does NOT implement Clone directly.
/// Use `try_clone()` for explicit cloning with proper error handling.
#[derive(Debug, Default)]
pub struct Session {
    /// The storage backend (in-memory or persistent).
    backend: StorageBackend,
    /// The volatile session buffer (active context).
    buffer: Buffer,
}

impl Session {
    /// Create a new empty session with in-memory storage.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a session with an existing in-memory graph.
    #[must_use]
    pub fn with_graph(graph: Graph) -> Self {
        Self {
            backend: StorageBackend::InMemory(graph),
            buffer: Buffer::new(),
        }
    }

    /// Create a session with persistent redb storage.
    ///
    /// Opens or creates a redb database at the given path.
    /// All changes are automatically persisted to disk.
    pub fn with_redb(path: impl AsRef<Path>) -> Result<Self, KremisError> {
        let redb = RedbGraph::open(path)?;
        Ok(Self {
            backend: StorageBackend::Persistent(redb),
            buffer: Buffer::new(),
        })
    }

    /// Create a session with an existing RedbGraph.
    #[must_use]
    pub fn with_redb_graph(redb: RedbGraph) -> Self {
        Self {
            backend: StorageBackend::Persistent(redb),
            buffer: Buffer::new(),
        }
    }

    /// Check if using persistent storage.
    #[must_use]
    pub fn is_persistent(&self) -> bool {
        matches!(self.backend, StorageBackend::Persistent(_))
    }

    /// Get a reference to the in-memory graph (if using in-memory backend).
    ///
    /// **DEPRECATED**: Use `graph_opt()` which returns `Option<&Graph>` for explicit
    /// handling of persistent backends.
    ///
    /// # Warning
    /// For persistent backends, this returns a static empty graph, which is almost
    /// certainly not what you want. Use `graph_opt()` for new code.
    #[deprecated(
        since = "0.2.0",
        note = "Use graph_opt() for explicit None handling with persistent backends"
    )]
    #[must_use]
    pub fn graph(&self) -> &Graph {
        match &self.backend {
            StorageBackend::InMemory(g) => g,
            StorageBackend::Persistent(_) => {
                // Return a static empty graph for backward compatibility
                // New code should use graph_opt() instead
                static EMPTY: std::sync::OnceLock<Graph> = std::sync::OnceLock::new();
                EMPTY.get_or_init(Graph::new)
            }
        }
    }

    /// Get an optional reference to the in-memory graph.
    ///
    /// Returns `Some(&Graph)` for in-memory backends, `None` for persistent backends.
    /// This is the preferred method for accessing the graph directly.
    ///
    /// # Example
    /// ```
    /// use kremis_core::Session;
    ///
    /// let session = Session::new();
    /// assert!(session.graph_opt().is_some()); // In-memory session has graph
    /// ```
    #[must_use]
    pub fn graph_opt(&self) -> Option<&Graph> {
        match &self.backend {
            StorageBackend::InMemory(g) => Some(g),
            StorageBackend::Persistent(_) => None,
        }
    }

    /// Check if the session can provide direct graph access.
    ///
    /// Returns `true` for in-memory backends, `false` for persistent.
    /// Use this to check before calling `graph_opt()` or operations that
    /// require direct graph access.
    #[must_use]
    pub fn has_direct_graph_access(&self) -> bool {
        matches!(self.backend, StorageBackend::InMemory(_))
    }

    /// Try to clone the session.
    ///
    /// Returns `Some(Session)` for in-memory backends with cloned graph and buffer.
    /// Returns `None` for persistent backends (database handles cannot be safely cloned).
    ///
    /// # Example
    /// ```
    /// use kremis_core::Session;
    ///
    /// let session = Session::new();
    /// if let Some(cloned) = session.try_clone() {
    ///     // Work with the cloned session
    /// } else {
    ///     // Handle persistent backend case
    /// }
    /// ```
    #[must_use]
    pub fn try_clone(&self) -> Option<Self> {
        match &self.backend {
            StorageBackend::InMemory(g) => Some(Self {
                backend: StorageBackend::InMemory(g.clone()),
                buffer: self.buffer.clone(),
            }),
            StorageBackend::Persistent(_) => None,
        }
    }

    /// Get a mutable reference to the in-memory graph.
    ///
    /// Returns `None` if using persistent storage.
    /// Callers should use session methods directly for persistent backends.
    ///
    /// Per AGENTS.md Section 5.7: No unsafe blocks in Core.
    #[must_use]
    pub fn graph_mut(&mut self) -> Option<&mut Graph> {
        match &mut self.backend {
            StorageBackend::InMemory(g) => Some(g),
            StorageBackend::Persistent(_) => None,
        }
    }

    /// Get a reference to the buffer.
    #[must_use]
    pub fn buffer(&self) -> &Buffer {
        &self.buffer
    }

    /// Get a reference to the storage backend.
    #[must_use]
    pub fn backend(&self) -> &StorageBackend {
        &self.backend
    }

    // =========================================================================
    // INGESTION
    // =========================================================================

    /// Ingest a signal and add its node to the active context.
    pub fn ingest(&mut self, signal: &Signal) -> Result<NodeId, KremisError> {
        let node_id = match &mut self.backend {
            StorageBackend::InMemory(graph) => Ingestor::ingest_signal(graph, signal)?,
            StorageBackend::Persistent(redb) => Ingestor::ingest_signal(redb, signal)?,
        };
        self.buffer.activate(node_id);
        Ok(node_id)
    }

    /// Ingest a sequence of signals.
    ///
    /// Creates edges between adjacent signals per ASSOCIATION_WINDOW.
    /// All resulting nodes are added to active context.
    pub fn ingest_sequence(&mut self, signals: &[Signal]) -> Result<Vec<NodeId>, KremisError> {
        let nodes = match &mut self.backend {
            StorageBackend::InMemory(graph) => Ingestor::ingest_sequence(graph, signals)?,
            StorageBackend::Persistent(redb) => Ingestor::ingest_sequence(redb, signals)?,
        };
        for &node in &nodes {
            self.buffer.activate(node);
        }
        Ok(nodes)
    }

    // =========================================================================
    // COMPOSITION
    // =========================================================================

    /// Compose an artifact from a starting node.
    pub fn compose(&self, start: NodeId, depth: usize) -> Option<Artifact> {
        let result = match &self.backend {
            StorageBackend::InMemory(graph) => graph.traverse(start, depth),
            StorageBackend::Persistent(redb) => redb.traverse(start, depth),
        };
        log_and_convert(result, "compose").flatten()
    }

    /// Compose from an active context node.
    ///
    /// Uses the first active node if available.
    pub fn compose_from_active(&self, depth: usize) -> Option<Artifact> {
        let start = self.buffer.active_nodes.first()?;
        self.compose(*start, depth)
    }

    /// Extract path between two nodes.
    pub fn extract_path(&self, start: NodeId, end: NodeId) -> Option<Artifact> {
        let path_result = match &self.backend {
            StorageBackend::InMemory(graph) => graph.strongest_path(start, end),
            StorageBackend::Persistent(redb) => redb.strongest_path(start, end),
        };
        let path = log_and_convert(path_result, "extract_path").flatten()?;

        // Collect edges along the path for the artifact
        let mut subgraph = Vec::new();
        for window in path.windows(2) {
            let from = window[0];
            let to = window[1];
            if let Some(weight) = self.get_edge(from, to) {
                subgraph.push((from, to, weight));
            }
        }

        Some(Artifact::with_subgraph(path, subgraph))
    }

    /// Find intersection of active context nodes.
    pub fn intersect_active(&self) -> Artifact {
        let nodes: Vec<_> = self.buffer.active_nodes.iter().copied().collect();
        let result = match &self.backend {
            StorageBackend::InMemory(graph) => graph.intersect(&nodes),
            StorageBackend::Persistent(redb) => redb.intersect(&nodes),
        };
        Artifact::with_path(log_and_default(result, "intersect_active"))
    }

    // =========================================================================
    // CONTEXT MANAGEMENT
    // =========================================================================

    /// Activate a node in the current context.
    pub fn activate(&mut self, node: NodeId) {
        self.buffer.activate(node);
    }

    /// Deactivate a node from the current context.
    pub fn deactivate(&mut self, node: &NodeId) {
        self.buffer.deactivate(node);
    }

    /// Check if a node is active.
    #[must_use]
    pub fn is_active(&self, node: &NodeId) -> bool {
        self.buffer.is_active(node)
    }

    /// Clear the active context (session reset).
    ///
    /// Per ROADMAP.md:
    /// - Buffer is volatile
    /// - Cleared on reset
    /// - Graph persists
    pub fn clear_context(&mut self) {
        self.buffer.clear();
    }

    /// Get the number of active nodes.
    #[must_use]
    pub fn active_count(&self) -> usize {
        self.buffer.active_nodes.len()
    }

    // =========================================================================
    // LOOKUP
    // =========================================================================

    /// Lookup a node by entity ID.
    pub fn lookup_entity(&self, entity: EntityId) -> Option<NodeId> {
        match &self.backend {
            StorageBackend::InMemory(graph) => graph.get_node_by_entity(entity),
            StorageBackend::Persistent(redb) => redb.get_node_by_entity(entity),
        }
    }

    /// Get edge weight between two nodes.
    pub fn get_edge(&self, from: NodeId, to: NodeId) -> Option<EdgeWeight> {
        let result = match &self.backend {
            StorageBackend::InMemory(graph) => graph.get_edge(from, to),
            StorageBackend::Persistent(redb) => redb.get_edge(from, to),
        };
        log_and_convert(result, "get_edge").flatten()
    }

    // =========================================================================
    // METRICS (for stage assessment)
    // =========================================================================

    /// Get the node count.
    #[must_use]
    pub fn node_count(&self) -> usize {
        let result = match &self.backend {
            StorageBackend::InMemory(graph) => graph.node_count(),
            StorageBackend::Persistent(redb) => redb.node_count(),
        };
        log_and_default(result, "node_count")
    }

    /// Get the edge count.
    #[must_use]
    pub fn edge_count(&self) -> usize {
        let result = match &self.backend {
            StorageBackend::InMemory(graph) => graph.edge_count(),
            StorageBackend::Persistent(redb) => redb.edge_count(),
        };
        log_and_default(result, "edge_count")
    }

    /// Traverse from a starting node.
    pub fn traverse(&self, start: NodeId, depth: usize) -> Option<Artifact> {
        let result = match &self.backend {
            StorageBackend::InMemory(graph) => graph.traverse(start, depth),
            StorageBackend::Persistent(redb) => redb.traverse(start, depth),
        };
        log_and_convert(result, "traverse").flatten()
    }

    /// Traverse with minimum weight filter.
    pub fn traverse_filtered(
        &self,
        start: NodeId,
        depth: usize,
        min_weight: EdgeWeight,
    ) -> Option<Artifact> {
        let result = match &self.backend {
            StorageBackend::InMemory(graph) => graph.traverse_filtered(start, depth, min_weight),
            StorageBackend::Persistent(redb) => redb.traverse_filtered(start, depth, min_weight),
        };
        log_and_convert(result, "traverse_filtered").flatten()
    }

    /// Find strongest path between two nodes.
    pub fn strongest_path(&self, start: NodeId, end: NodeId) -> Option<Vec<NodeId>> {
        let result = match &self.backend {
            StorageBackend::InMemory(graph) => graph.strongest_path(start, end),
            StorageBackend::Persistent(redb) => redb.strongest_path(start, end),
        };
        log_and_convert(result, "strongest_path").flatten()
    }

    /// Find intersection of nodes.
    pub fn intersect(&self, nodes: &[NodeId]) -> Vec<NodeId> {
        let result = match &self.backend {
            StorageBackend::InMemory(graph) => graph.intersect(nodes),
            StorageBackend::Persistent(redb) => redb.intersect(nodes),
        };
        log_and_default(result, "intersect")
    }

    // =========================================================================
    // EXPORT SUPPORT (M3 FIX)
    // =========================================================================

    /// Build an in-memory Graph snapshot for export purposes.
    ///
    /// This method works with both in-memory and persistent backends:
    /// - For in-memory: clones the existing graph
    /// - For persistent: iterates all nodes/edges and builds a new Graph
    ///
    /// # M3 Fix
    ///
    /// This enables export functionality for persistent (redb) backends,
    /// which was previously unsupported via the HTTP API.
    ///
    /// # Errors
    ///
    /// Returns an error if the persistent backend fails to iterate nodes/edges.
    pub fn export_graph_snapshot(&self) -> Result<Graph, KremisError> {
        match &self.backend {
            StorageBackend::InMemory(graph) => Ok(graph.clone()),
            StorageBackend::Persistent(redb) => {
                use crate::graph::GraphStore;

                let mut graph = Graph::new();

                // Import all nodes
                for node in redb.nodes()? {
                    // Insert node preserving original NodeId
                    graph.import_node(node);
                }

                // Import all edges
                for (from, to, weight) in redb.edges()? {
                    let _ = graph.insert_edge(from, to, weight);
                }

                Ok(graph)
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
    use crate::{Attribute, Value};

    fn make_signal(entity_id: u64, attr: &str, val: &str) -> Signal {
        Signal::new(EntityId(entity_id), Attribute::new(attr), Value::new(val))
    }

    #[test]
    fn ingest_adds_to_active_context() {
        let mut session = Session::new();
        let signal = make_signal(1, "name", "Alice");

        let node = session.ingest(&signal).expect("ingest");

        assert!(session.is_active(&node));
        assert_eq!(session.active_count(), 1);
    }

    #[test]
    fn clear_context_removes_active_nodes() {
        let mut session = Session::new();
        let signal = make_signal(1, "name", "Alice");

        session.ingest(&signal).expect("ingest");
        assert_eq!(session.active_count(), 1);

        session.clear_context();
        assert_eq!(session.active_count(), 0);
    }

    #[test]
    fn graph_persists_after_context_clear() {
        let mut session = Session::new();
        let signal = make_signal(1, "name", "Alice");

        let node = session.ingest(&signal).expect("ingest");
        session.clear_context();

        // Graph still has the node
        let graph = session
            .graph_opt()
            .expect("in-memory session should have graph");
        assert!(graph.lookup(node).expect("lookup").is_some());
        // But it's not in active context
        assert!(!session.is_active(&node));
    }

    #[test]
    fn ingest_sequence_creates_edges() {
        let mut session = Session::new();
        let signals = vec![
            make_signal(1, "type", "word"),
            make_signal(2, "type", "word"),
        ];

        let nodes = session.ingest_sequence(&signals).expect("ingest");

        assert!(session.get_edge(nodes[0], nodes[1]).is_some());
    }
}
