//! # Canonical Export Module
//!
//! > **The "Redb Compromise":**
//! > - Runtime: CORE uses `redb` for performance and ACID transactions.
//! > - Verification: `redb` files are NOT guaranteed bit-identical across runs.
//! > - Mandate: Implement `export_canonical()` that serializes to bit-exact `postcard` stream.
//! >   **This export is the Source of Truth for verification.**
//!
//! This module provides deterministic, bit-exact serialization for graph verification.

use crate::graph::{Graph, GraphStore};
use crate::{EdgeWeight, EntityId, KremisError, Node, NodeId};
use serde::{Deserialize, Serialize};

// =============================================================================
// CANONICAL FORMAT
// =============================================================================

/// Magic bytes for canonical export format.
pub const CANONICAL_MAGIC: [u8; 4] = *b"KREX"; // Kremis Export

/// Current canonical format version.
pub const CANONICAL_VERSION: u8 = 2;

/// Maximum allowed node count in canonical imports.
///
/// This prevents memory exhaustion from malicious or corrupted data.
/// 1 million nodes is a reasonable upper bound for most use cases.
pub const MAX_IMPORT_NODE_COUNT: u64 = 1_000_000;

/// Maximum allowed edge count in canonical imports.
///
/// This prevents memory exhaustion from malicious or corrupted data.
/// 10 million edges is a reasonable upper bound (10x node count).
pub const MAX_IMPORT_EDGE_COUNT: u64 = 10_000_000;

/// Header for canonical export files.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CanonicalHeader {
    /// Magic bytes to identify the format.
    pub magic: [u8; 4],

    /// Format version for compatibility.
    pub version: u8,

    /// Number of nodes in the export.
    pub node_count: u64,

    /// Number of edges in the export.
    pub edge_count: u64,

    /// Checksum of the data section (simple XOR-based for determinism).
    pub checksum: u64,
}

impl CanonicalHeader {
    /// Create a new header with the given counts.
    #[must_use]
    pub fn new(node_count: u64, edge_count: u64, checksum: u64) -> Self {
        Self {
            magic: CANONICAL_MAGIC,
            version: CANONICAL_VERSION,
            node_count,
            edge_count,
            checksum,
        }
    }

    /// Validate the header.
    ///
    /// # Security Note
    ///
    /// Error messages are intentionally generic to avoid leaking format details
    /// to potential attackers.
    pub fn validate(&self) -> Result<(), KremisError> {
        if self.magic != CANONICAL_MAGIC {
            return Err(KremisError::SerializationError(
                "Invalid file format".to_string(),
            ));
        }
        if self.version != 1 && self.version != CANONICAL_VERSION {
            return Err(KremisError::SerializationError(
                "Unsupported file version".to_string(),
            ));
        }
        Ok(())
    }
}

// =============================================================================
// CANONICAL NODE & EDGE (Sorted, Deterministic)
// =============================================================================

/// A node in canonical format.
///
/// Sorted by NodeId for deterministic ordering.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct CanonicalNode {
    /// The node ID (sort key).
    pub id: u64,

    /// The entity ID this node represents.
    pub entity: u64,
}

impl From<&Node> for CanonicalNode {
    fn from(node: &Node) -> Self {
        Self {
            id: node.id.0,
            entity: node.entity.0,
        }
    }
}

impl From<CanonicalNode> for Node {
    fn from(cn: CanonicalNode) -> Self {
        Node::new(NodeId(cn.id), EntityId(cn.entity))
    }
}

/// An edge in canonical format.
///
/// Sorted by (from, to) for deterministic ordering.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct CanonicalEdge {
    /// Source node ID.
    pub from: u64,

    /// Target node ID.
    pub to: u64,

    /// Edge weight.
    pub weight: i64,
}

impl CanonicalEdge {
    /// Create a new canonical edge.
    #[must_use]
    pub fn new(from: NodeId, to: NodeId, weight: EdgeWeight) -> Self {
        Self {
            from: from.0,
            to: to.0,
            weight: weight.value(),
        }
    }
}

/// A property in canonical format.
///
/// Sorted by (node_id, attribute, value) for deterministic ordering.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct CanonicalProperty {
    /// The node ID this property belongs to.
    pub node_id: u64,

    /// The attribute name.
    pub attribute: String,

    /// The value.
    pub value: String,
}

// =============================================================================
// CANONICAL GRAPH (Sorted, Deterministic)
// =============================================================================

/// V1 canonical graph format (without properties) for backward compatibility.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct CanonicalGraphV1 {
    nodes: Vec<CanonicalNode>,
    edges: Vec<CanonicalEdge>,
    next_node_id: u64,
}

/// A graph in canonical format for bit-exact serialization.
///
/// > "The System MUST implement a `export_canonical()` function that serializes
/// > the graph into a sorted, bit-exact `postcard` stream."
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CanonicalGraph {
    /// Nodes sorted by NodeId.
    pub nodes: Vec<CanonicalNode>,

    /// Edges sorted by (from, to).
    pub edges: Vec<CanonicalEdge>,

    /// Next node ID counter.
    pub next_node_id: u64,

    /// Properties sorted by (node_id, attribute, value).
    pub properties: Vec<CanonicalProperty>,
}

impl CanonicalGraph {
    /// Create a canonical graph from a regular graph.
    ///
    /// This ensures deterministic ordering by sorting all elements.
    #[must_use]
    pub fn from_graph(graph: &Graph) -> Self {
        // Collect and sort nodes
        let mut nodes: Vec<CanonicalNode> = graph.nodes().map(CanonicalNode::from).collect();
        nodes.sort();

        // Collect and sort edges
        let mut edges: Vec<CanonicalEdge> = graph
            .edges()
            .map(|(from, to, weight)| CanonicalEdge::new(from, to, weight))
            .collect();
        edges.sort();

        // Collect and sort properties
        let mut properties: Vec<CanonicalProperty> = Vec::new();
        for node in &nodes {
            if let Ok(props) = graph.get_properties(NodeId(node.id)) {
                for (attr, val) in props {
                    properties.push(CanonicalProperty {
                        node_id: node.id,
                        attribute: attr.as_str().to_string(),
                        value: val.as_str().to_string(),
                    });
                }
            }
        }
        properties.sort();

        Self {
            nodes,
            edges,
            next_node_id: graph.next_node_id(),
            properties,
        }
    }

    /// Convert back to a regular graph, preserving original NodeIds.
    #[must_use]
    pub fn to_graph(&self) -> Graph {
        Graph::from_canonical(self)
    }

    /// Compute a deterministic checksum of the data.
    ///
    /// Uses XOR-based hashing for simplicity and determinism.
    /// No floating point, no randomness.
    ///
    /// # Security Note
    ///
    /// This is **NOT** a cryptographic hash. It is designed for:
    /// - Detecting accidental data corruption
    /// - Verifying export/import integrity
    /// - Quick equality checks
    ///
    /// It is **NOT** designed for:
    /// - Detecting intentional tampering
    /// - Security-sensitive applications
    /// - Collision resistance
    ///
    /// For security-sensitive use cases, compute an additional cryptographic
    /// hash (e.g., SHA-256, BLAKE3) externally on the exported bytes.
    #[must_use]
    pub fn checksum(&self) -> u64 {
        let mut hash: u64 = 0;

        // Hash nodes
        for node in &self.nodes {
            hash ^= node.id.rotate_left(13);
            hash ^= node.entity.rotate_left(7);
        }

        // Hash edges
        for edge in &self.edges {
            hash ^= edge.from.rotate_left(17);
            hash ^= edge.to.rotate_left(11);
            hash ^= (edge.weight as u64).rotate_left(5);
        }

        // Hash properties
        for prop in &self.properties {
            hash ^= prop.node_id.rotate_left(19);
            for byte in prop.attribute.as_bytes() {
                hash ^= (*byte as u64).rotate_left(23);
            }
            for byte in prop.value.as_bytes() {
                hash ^= (*byte as u64).rotate_left(29);
            }
        }

        // Hash metadata
        hash ^= self.next_node_id.rotate_left(3);

        hash
    }
}

// =============================================================================
// EXPORT FUNCTIONS
// =============================================================================

/// Export a graph to canonical postcard format.
///
/// This is the primary export function.
///
/// Format:
/// ```text
/// [CanonicalHeader (postcard)] [CanonicalGraph (postcard)]
/// ```
///
/// # Errors
///
/// Returns `KremisError::SerializationError` if serialization fails.
pub fn export_canonical(graph: &Graph) -> Result<Vec<u8>, KremisError> {
    let canonical = CanonicalGraph::from_graph(graph);
    let checksum = canonical.checksum();

    let header = CanonicalHeader::new(
        canonical.nodes.len() as u64,
        canonical.edges.len() as u64,
        checksum,
    );

    // Serialize header
    let header_bytes = postcard::to_allocvec(&header)
        .map_err(|e| KremisError::SerializationError(format!("Header: {}", e)))?;

    // Serialize data
    let data_bytes = postcard::to_allocvec(&canonical)
        .map_err(|e| KremisError::SerializationError(format!("Data: {}", e)))?;

    // Combine: [header_len: u32] [header] [data]
    let mut result = Vec::with_capacity(4 + header_bytes.len() + data_bytes.len());
    result.extend_from_slice(&(header_bytes.len() as u32).to_le_bytes());
    result.extend_from_slice(&header_bytes);
    result.extend_from_slice(&data_bytes);

    Ok(result)
}

/// Import a graph from canonical postcard format.
///
/// # Errors
///
/// Returns `KremisError::SerializationError` if deserialization fails
/// or the data is corrupted.
pub fn import_canonical(data: &[u8]) -> Result<Graph, KremisError> {
    if data.len() < 4 {
        return Err(KremisError::SerializationError(
            "Data too short".to_string(),
        ));
    }

    // Read header length
    let header_len = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;

    if data.len() < 4 + header_len {
        return Err(KremisError::SerializationError(
            "Data too short for header".to_string(),
        ));
    }

    // Deserialize header
    let header: CanonicalHeader = postcard::from_bytes(&data[4..4 + header_len])
        .map_err(|e| KremisError::SerializationError(format!("Header: {}", e)))?;

    header.validate()?;

    // Validate size limits BEFORE deserializing the full graph to prevent DoS
    if header.node_count > MAX_IMPORT_NODE_COUNT {
        return Err(KremisError::SerializationError(format!(
            "Node count {} exceeds maximum allowed {}",
            header.node_count, MAX_IMPORT_NODE_COUNT
        )));
    }
    if header.edge_count > MAX_IMPORT_EDGE_COUNT {
        return Err(KremisError::SerializationError(format!(
            "Edge count {} exceeds maximum allowed {}",
            header.edge_count, MAX_IMPORT_EDGE_COUNT
        )));
    }

    // Deserialize data based on version
    let canonical: CanonicalGraph = if header.version == 1 {
        // V1 format: no properties field
        let v1: CanonicalGraphV1 = postcard::from_bytes(&data[4 + header_len..])
            .map_err(|e| KremisError::SerializationError(format!("Data: {}", e)))?;
        CanonicalGraph {
            nodes: v1.nodes,
            edges: v1.edges,
            next_node_id: v1.next_node_id,
            properties: Vec::new(),
        }
    } else {
        postcard::from_bytes(&data[4 + header_len..])
            .map_err(|e| KremisError::SerializationError(format!("Data: {}", e)))?
    };

    // Verify checksum: for v1 imports, recompute using v1's checksum logic (no properties)
    let computed_checksum = if header.version == 1 {
        // Recompute without properties (v1 checksum logic)
        let v1_canonical = CanonicalGraph {
            nodes: canonical.nodes.clone(),
            edges: canonical.edges.clone(),
            next_node_id: canonical.next_node_id,
            properties: Vec::new(),
        };
        v1_canonical.checksum()
    } else {
        canonical.checksum()
    };
    if computed_checksum != header.checksum {
        return Err(KremisError::SerializationError(format!(
            "Checksum mismatch: expected {}, got {}",
            header.checksum, computed_checksum
        )));
    }

    // Verify counts
    if canonical.nodes.len() as u64 != header.node_count {
        return Err(KremisError::SerializationError(
            "Node count mismatch".to_string(),
        ));
    }
    if canonical.edges.len() as u64 != header.edge_count {
        return Err(KremisError::SerializationError(
            "Edge count mismatch".to_string(),
        ));
    }

    Ok(canonical.to_graph())
}

/// Verify that a graph matches its canonical export.
///
/// This is used to verify `redb` data against the canonical format.
pub fn verify_canonical(graph: &Graph, canonical_data: &[u8]) -> Result<bool, KremisError> {
    let imported = import_canonical(canonical_data)?;

    // Compare node counts
    if graph.node_count()? != imported.node_count()? {
        return Ok(false);
    }

    // Compare edge counts
    if graph.edge_count()? != imported.edge_count()? {
        return Ok(false);
    }

    // Compare canonical representations
    let original_canonical = CanonicalGraph::from_graph(graph);
    let imported_canonical = CanonicalGraph::from_graph(&imported);

    Ok(original_canonical == imported_canonical)
}

/// Compute the canonical checksum of a graph.
///
/// This can be used to quickly compare two graphs for equality.
#[must_use]
pub fn canonical_checksum(graph: &Graph) -> u64 {
    CanonicalGraph::from_graph(graph).checksum()
}

// =============================================================================
// M1 FIX: CRYPTOGRAPHIC HASH SUPPORT
// =============================================================================

/// Compute a BLAKE3 cryptographic hash of the canonical export.
///
/// # M1 Fix
///
/// This provides a collision-resistant hash for security-sensitive use cases,
/// complementing the faster XOR-based checksum for integrity checking.
///
/// Returns the hash as a hex string (64 characters).
///
/// # Requires
///
/// This function is only available with the `crypto-hash` feature enabled.
/// Add `kremis-core = { version = "...", features = ["crypto-hash"] }` to enable.
#[cfg(feature = "crypto-hash")]
#[must_use]
pub fn canonical_crypto_hash(graph: &Graph) -> String {
    let data = export_canonical(graph).unwrap_or_default();
    let hash = blake3::hash(&data);
    hash.to_hex().to_string()
}

/// Verify a graph against a BLAKE3 hash.
///
/// # M1 Fix
///
/// Returns `true` if the graph's canonical export matches the provided hash.
///
/// # Requires
///
/// This function is only available with the `crypto-hash` feature enabled.
#[cfg(feature = "crypto-hash")]
pub fn verify_crypto_hash(graph: &Graph, expected_hash: &str) -> bool {
    let actual_hash = canonical_crypto_hash(graph);
    // Constant-time comparison would be ideal here for security,
    // but for integrity verification (not authentication), timing attacks
    // are not a concern.
    actual_hash == expected_hash
}

/// Compute a BLAKE3 hash of raw bytes.
///
/// # M1 Fix
///
/// Utility function for hashing arbitrary data.
///
/// # Requires
///
/// This function is only available with the `crypto-hash` feature enabled.
#[cfg(feature = "crypto-hash")]
#[must_use]
pub fn compute_blake3_hash(data: &[u8]) -> String {
    let hash = blake3::hash(data);
    hash.to_hex().to_string()
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::panic)]
mod tests {
    use super::*;
    use crate::graph::GraphStore;

    fn create_test_graph() -> Graph {
        let mut graph = Graph::new();
        let a = graph.insert_node(EntityId(1)).expect("insert");
        let b = graph.insert_node(EntityId(2)).expect("insert");
        let c = graph.insert_node(EntityId(3)).expect("insert");

        graph
            .insert_edge(a, b, EdgeWeight::new(10))
            .expect("insert");
        graph
            .insert_edge(b, c, EdgeWeight::new(20))
            .expect("insert");
        graph.insert_edge(a, c, EdgeWeight::new(5)).expect("insert");

        graph
    }

    #[test]
    fn canonical_roundtrip() {
        let graph = create_test_graph();

        let exported = export_canonical(&graph).expect("export should succeed");
        let imported = import_canonical(&exported).expect("import should succeed");

        assert_eq!(
            graph.node_count().expect("count"),
            imported.node_count().expect("count")
        );
        assert_eq!(
            graph.edge_count().expect("count"),
            imported.edge_count().expect("count")
        );
    }

    #[test]
    fn canonical_checksum_deterministic() {
        let graph = create_test_graph();

        let checksum1 = canonical_checksum(&graph);
        let checksum2 = canonical_checksum(&graph);

        assert_eq!(checksum1, checksum2);
    }

    #[test]
    fn canonical_export_deterministic() {
        let graph = create_test_graph();

        let export1 = export_canonical(&graph).expect("export 1");
        let export2 = export_canonical(&graph).expect("export 2");

        assert_eq!(export1, export2, "Exports must be bit-identical");
    }

    #[test]
    fn verify_canonical_success() {
        let graph = create_test_graph();
        let exported = export_canonical(&graph).expect("export");

        let result = verify_canonical(&graph, &exported).expect("verify");
        assert!(result);
    }

    #[test]
    fn verify_canonical_detects_corruption() {
        let graph = create_test_graph();
        let mut exported = export_canonical(&graph).expect("export");

        // Corrupt the data
        if let Some(last) = exported.last_mut() {
            *last ^= 0xFF;
        }

        // Should fail verification
        let result = import_canonical(&exported);
        assert!(result.is_err());
    }

    #[test]
    fn canonical_nodes_sorted() {
        let mut graph = Graph::new();
        // Insert in non-sorted order
        graph.insert_node(EntityId(100)).expect("insert");
        graph.insert_node(EntityId(1)).expect("insert");
        graph.insert_node(EntityId(50)).expect("insert");

        let canonical = CanonicalGraph::from_graph(&graph);

        // Nodes should be sorted by NodeId
        for i in 1..canonical.nodes.len() {
            assert!(canonical.nodes[i - 1].id <= canonical.nodes[i].id);
        }
    }

    #[test]
    fn canonical_edges_sorted() {
        let mut graph = Graph::new();
        let a = graph.insert_node(EntityId(1)).expect("insert");
        let b = graph.insert_node(EntityId(2)).expect("insert");
        let c = graph.insert_node(EntityId(3)).expect("insert");

        // Insert edges in non-sorted order
        graph.insert_edge(c, a, EdgeWeight::new(1)).expect("insert");
        graph.insert_edge(a, b, EdgeWeight::new(2)).expect("insert");
        graph.insert_edge(b, c, EdgeWeight::new(3)).expect("insert");

        let canonical = CanonicalGraph::from_graph(&graph);

        // Edges should be sorted by (from, to)
        for i in 1..canonical.edges.len() {
            let prev = &canonical.edges[i - 1];
            let curr = &canonical.edges[i];
            assert!(
                (prev.from, prev.to) <= (curr.from, curr.to),
                "Edges should be sorted"
            );
        }
    }

    #[test]
    fn header_validation() {
        let header = CanonicalHeader::new(10, 5, 12345);
        assert!(header.validate().is_ok());

        let bad_magic = CanonicalHeader {
            magic: *b"XXXX",
            version: CANONICAL_VERSION,
            node_count: 0,
            edge_count: 0,
            checksum: 0,
        };
        assert!(bad_magic.validate().is_err());

        let bad_version = CanonicalHeader {
            magic: CANONICAL_MAGIC,
            version: 99,
            node_count: 0,
            edge_count: 0,
            checksum: 0,
        };
        assert!(bad_version.validate().is_err());
    }

    #[test]
    fn empty_graph_export() {
        let graph = Graph::new();

        let exported = export_canonical(&graph).expect("export empty");
        let imported = import_canonical(&exported).expect("import empty");

        assert_eq!(imported.node_count().expect("count"), 0);
        assert_eq!(imported.edge_count().expect("count"), 0);
    }

    // =========================================================================
    // M5 - Corrupted imports tests
    // =========================================================================

    #[test]
    fn corrupted_import_empty_data() {
        let result = import_canonical(&[]);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, KremisError::SerializationError(_)));
    }

    #[test]
    fn corrupted_import_too_short_for_header_len() {
        // Only 3 bytes, need at least 4 for header length
        let result = import_canonical(&[0x01, 0x02, 0x03]);
        assert!(result.is_err());
    }

    #[test]
    fn corrupted_import_header_length_exceeds_data() {
        // Header length says 1000 bytes, but we only have a few
        let mut data = vec![0xe8, 0x03, 0x00, 0x00]; // 1000 in little-endian u32
        data.extend_from_slice(&[0x00, 0x00, 0x00]); // Only 3 bytes of "header"

        let result = import_canonical(&data);
        assert!(result.is_err());
    }

    #[test]
    fn corrupted_import_invalid_magic_bytes() {
        let graph = create_test_graph();
        let mut exported = export_canonical(&graph).expect("export");

        // Corrupt magic bytes (bytes 4-7 after header length)
        if exported.len() > 7 {
            exported[4] = 0xFF;
            exported[5] = 0xFF;
            exported[6] = 0xFF;
            exported[7] = 0xFF;
        }

        let result = import_canonical(&exported);
        assert!(result.is_err());
        let err_msg = format!("{:?}", result.unwrap_err());
        assert!(
            err_msg.contains("Invalid file format") || err_msg.contains("Header"),
            "Expected format error, got: {}",
            err_msg
        );
    }

    #[test]
    fn corrupted_import_invalid_version() {
        let graph = create_test_graph();
        let exported = export_canonical(&graph).expect("export");

        // Create a modified export with wrong version
        let mut modified = exported.clone();
        // Version is after magic bytes (4 bytes), so at offset 4 + 4 = 8 in header
        if modified.len() > 4 + 8 {
            modified[4 + 4] = 99; // Set version to 99 (invalid)
        }

        let result = import_canonical(&modified);
        assert!(result.is_err());
    }

    #[test]
    fn corrupted_import_checksum_mismatch() {
        let graph = create_test_graph();
        let mut exported = export_canonical(&graph).expect("export");

        // Corrupt the last byte of data section (will cause checksum mismatch)
        if let Some(last) = exported.last_mut() {
            *last ^= 0xFF;
        }

        let result = import_canonical(&exported);
        assert!(result.is_err());
        let err_msg = format!("{:?}", result.unwrap_err());
        // Could be checksum mismatch or deserialization error
        assert!(
            err_msg.contains("Checksum") || err_msg.contains("Data"),
            "Expected checksum or data error, got: {}",
            err_msg
        );
    }

    #[test]
    fn corrupted_import_truncated_data_section() {
        let graph = create_test_graph();
        let exported = export_canonical(&graph).expect("export");

        // Read header length
        let header_len =
            u32::from_le_bytes([exported[0], exported[1], exported[2], exported[3]]) as usize;

        // Truncate data section (keep header but remove most of data)
        let truncated = exported[..4 + header_len + 1].to_vec();

        let result = import_canonical(&truncated);
        assert!(result.is_err());
    }

    #[test]
    fn corrupted_import_garbage_data_section() {
        let graph = create_test_graph();
        let exported = export_canonical(&graph).expect("export");

        // Read header length
        let header_len =
            u32::from_le_bytes([exported[0], exported[1], exported[2], exported[3]]) as usize;

        // Keep header but replace data with garbage
        let mut corrupted = exported[..4 + header_len].to_vec();
        corrupted.extend_from_slice(&[0xDE, 0xAD, 0xBE, 0xEF, 0x00, 0x00, 0x00, 0x00]);

        let result = import_canonical(&corrupted);
        assert!(result.is_err());
    }

    #[test]
    fn corrupted_import_node_count_mismatch() {
        // Create a valid export, then manually create one with wrong node count in header
        let graph = create_test_graph();
        let exported = export_canonical(&graph).expect("export");

        // Read header length
        let header_len =
            u32::from_le_bytes([exported[0], exported[1], exported[2], exported[3]]) as usize;

        // Deserialize header, modify node_count, reserialize
        let mut header: CanonicalHeader =
            postcard::from_bytes(&exported[4..4 + header_len]).expect("parse header");

        // Corrupt node count to be wrong
        header.node_count = header.node_count.saturating_add(100);

        let new_header_bytes = postcard::to_allocvec(&header).expect("serialize header");

        // Rebuild export with corrupted header
        let mut corrupted = Vec::new();
        corrupted.extend_from_slice(&(new_header_bytes.len() as u32).to_le_bytes());
        corrupted.extend_from_slice(&new_header_bytes);
        corrupted.extend_from_slice(&exported[4 + header_len..]);

        let result = import_canonical(&corrupted);
        assert!(result.is_err());
        let err_msg = format!("{:?}", result.unwrap_err());
        assert!(
            err_msg.contains("Node count")
                || err_msg.contains("mismatch")
                || err_msg.contains("Checksum"),
            "Expected count mismatch error, got: {}",
            err_msg
        );
    }

    #[test]
    fn corrupted_import_edge_count_mismatch() {
        let graph = create_test_graph();
        let exported = export_canonical(&graph).expect("export");

        let header_len =
            u32::from_le_bytes([exported[0], exported[1], exported[2], exported[3]]) as usize;

        let mut header: CanonicalHeader =
            postcard::from_bytes(&exported[4..4 + header_len]).expect("parse header");

        // Corrupt edge count
        header.edge_count = header.edge_count.saturating_add(50);

        let new_header_bytes = postcard::to_allocvec(&header).expect("serialize header");

        let mut corrupted = Vec::new();
        corrupted.extend_from_slice(&(new_header_bytes.len() as u32).to_le_bytes());
        corrupted.extend_from_slice(&new_header_bytes);
        corrupted.extend_from_slice(&exported[4 + header_len..]);

        let result = import_canonical(&corrupted);
        assert!(result.is_err());
    }

    #[test]
    fn corrupted_import_excessive_node_count() {
        // Create a header claiming more nodes than allowed
        let header = CanonicalHeader {
            magic: CANONICAL_MAGIC,
            version: CANONICAL_VERSION,
            node_count: MAX_IMPORT_NODE_COUNT + 1, // Exceeds limit
            edge_count: 0,
            checksum: 0,
        };

        let header_bytes = postcard::to_allocvec(&header).expect("serialize");
        let mut data = Vec::new();
        data.extend_from_slice(&(header_bytes.len() as u32).to_le_bytes());
        data.extend_from_slice(&header_bytes);
        // Empty data section
        data.extend_from_slice(&[0u8; 10]);

        let result = import_canonical(&data);
        assert!(result.is_err());
        let err_msg = format!("{:?}", result.unwrap_err());
        assert!(
            err_msg.contains("exceeds maximum") || err_msg.contains("Node count"),
            "Expected size limit error, got: {}",
            err_msg
        );
    }

    #[test]
    fn corrupted_import_excessive_edge_count() {
        let header = CanonicalHeader {
            magic: CANONICAL_MAGIC,
            version: CANONICAL_VERSION,
            node_count: 10,
            edge_count: MAX_IMPORT_EDGE_COUNT + 1, // Exceeds limit
            checksum: 0,
        };

        let header_bytes = postcard::to_allocvec(&header).expect("serialize");
        let mut data = Vec::new();
        data.extend_from_slice(&(header_bytes.len() as u32).to_le_bytes());
        data.extend_from_slice(&header_bytes);
        data.extend_from_slice(&[0u8; 10]);

        let result = import_canonical(&data);
        assert!(result.is_err());
        let err_msg = format!("{:?}", result.unwrap_err());
        assert!(
            err_msg.contains("exceeds maximum") || err_msg.contains("Edge count"),
            "Expected size limit error, got: {}",
            err_msg
        );
    }

    #[test]
    fn corrupted_import_both_counts_at_limit() {
        // Counts exactly at limit should pass validation (if data is valid)
        let header = CanonicalHeader {
            magic: CANONICAL_MAGIC,
            version: CANONICAL_VERSION,
            node_count: MAX_IMPORT_NODE_COUNT, // Exactly at limit
            edge_count: MAX_IMPORT_EDGE_COUNT, // Exactly at limit
            checksum: 0,
        };

        let header_bytes = postcard::to_allocvec(&header).expect("serialize");
        let mut data = Vec::new();
        data.extend_from_slice(&(header_bytes.len() as u32).to_le_bytes());
        data.extend_from_slice(&header_bytes);
        data.extend_from_slice(&[0u8; 10]);

        let result = import_canonical(&data);
        // Should fail on data deserialization, not on size validation
        assert!(result.is_err());
        let err_msg = format!("{:?}", result.unwrap_err());
        // Should NOT be about exceeding limits
        assert!(
            !err_msg.contains("exceeds maximum"),
            "Should not fail on size validation: {}",
            err_msg
        );
    }

    #[test]
    fn corrupted_import_random_bytes() {
        // Completely random data should fail gracefully
        let random_data: Vec<u8> = (0..100).map(|i| (i * 17 + 31) as u8).collect();

        let result = import_canonical(&random_data);
        assert!(result.is_err());
    }

    #[test]
    fn corrupted_import_valid_header_invalid_postcard_data() {
        // Valid header but invalid postcard-encoded graph data
        let header = CanonicalHeader {
            magic: CANONICAL_MAGIC,
            version: CANONICAL_VERSION,
            node_count: 3,
            edge_count: 2,
            checksum: 12345,
        };

        let header_bytes = postcard::to_allocvec(&header).expect("serialize");
        let mut data = Vec::new();
        data.extend_from_slice(&(header_bytes.len() as u32).to_le_bytes());
        data.extend_from_slice(&header_bytes);
        // Invalid postcard data (valid varint prefix but wrong structure)
        data.extend_from_slice(&[0x03, 0x00, 0x01, 0x02, 0x03, 0xFF, 0xFF]);

        let result = import_canonical(&data);
        assert!(result.is_err());
    }

    #[test]
    fn corrupted_import_partial_header() {
        // Header claims to be larger than the actual data
        let mut data = Vec::new();
        data.extend_from_slice(&100u32.to_le_bytes()); // Header length = 100
        data.extend_from_slice(&[0x4B, 0x52, 0x45, 0x58]); // "KREX" magic
        // Only 4 more bytes, but header_len says 100

        let result = import_canonical(&data);
        assert!(result.is_err());
    }

    #[test]
    fn verify_canonical_returns_false_for_different_graphs() {
        let graph1 = create_test_graph();

        let mut graph2 = Graph::new();
        let a = graph2.insert_node(EntityId(100)).expect("insert");
        let b = graph2.insert_node(EntityId(200)).expect("insert");
        graph2
            .insert_edge(a, b, EdgeWeight::new(999))
            .expect("insert");

        let exported1 = export_canonical(&graph1).expect("export");

        // Verifying graph2 against graph1's export should return false
        let result = verify_canonical(&graph2, &exported1).expect("verify");
        assert!(!result);
    }

    #[test]
    fn canonical_graph_checksum_changes_with_data() {
        let mut graph1 = Graph::new();
        let a = graph1.insert_node(EntityId(1)).expect("insert");
        let b = graph1.insert_node(EntityId(2)).expect("insert");
        graph1
            .insert_edge(a, b, EdgeWeight::new(10))
            .expect("insert");

        let mut graph2 = Graph::new();
        let c = graph2.insert_node(EntityId(1)).expect("insert");
        let d = graph2.insert_node(EntityId(2)).expect("insert");
        graph2
            .insert_edge(c, d, EdgeWeight::new(20))
            .expect("insert"); // Different weight

        let checksum1 = canonical_checksum(&graph1);
        let checksum2 = canonical_checksum(&graph2);

        assert_ne!(
            checksum1, checksum2,
            "Different data should produce different checksums"
        );
    }

    #[test]
    fn canonical_node_from_conversion() {
        use crate::Node;

        let node = Node::new(NodeId(42), EntityId(100));
        let canonical: CanonicalNode = (&node).into();

        assert_eq!(canonical.id, 42);
        assert_eq!(canonical.entity, 100);

        let back: Node = canonical.into();
        assert_eq!(back.id, NodeId(42));
        assert_eq!(back.entity, EntityId(100));
    }

    #[test]
    fn canonical_edge_new() {
        let edge = CanonicalEdge::new(NodeId(1), NodeId(2), EdgeWeight::new(50));

        assert_eq!(edge.from, 1);
        assert_eq!(edge.to, 2);
        assert_eq!(edge.weight, 50);
    }

    // =========================================================================
    // Properties export/import tests (Bug 10)
    // =========================================================================

    #[test]
    fn canonical_roundtrip_with_properties() {
        use crate::{Attribute, Value};

        let mut graph = create_test_graph();
        let node_a = NodeId(0);

        // Store properties via the GraphStore trait
        graph
            .store_property(node_a, Attribute::new("name"), Value::new("Alice"))
            .expect("store property");
        graph
            .store_property(node_a, Attribute::new("role"), Value::new("admin"))
            .expect("store property");

        let exported = export_canonical(&graph).expect("export should succeed");
        let imported = import_canonical(&exported).expect("import should succeed");

        // Verify properties survived the roundtrip
        let props = imported.get_properties(node_a).expect("get properties");
        assert_eq!(props.len(), 2);
        assert!(props.contains(&(Attribute::new("name"), Value::new("Alice"))));
        assert!(props.contains(&(Attribute::new("role"), Value::new("admin"))));
    }

    #[test]
    fn canonical_import_v1_backward_compat() {
        // Create a v1-format export (no properties)
        let graph = create_test_graph();
        let v1 = CanonicalGraphV1 {
            nodes: {
                let mut nodes: Vec<CanonicalNode> =
                    graph.nodes().map(CanonicalNode::from).collect();
                nodes.sort();
                nodes
            },
            edges: {
                let mut edges: Vec<CanonicalEdge> = graph
                    .edges()
                    .map(|(from, to, weight)| CanonicalEdge::new(from, to, weight))
                    .collect();
                edges.sort();
                edges
            },
            next_node_id: graph.next_node_id(),
        };

        // Compute v1 checksum (same algorithm, no properties)
        let v1_as_canonical = CanonicalGraph {
            nodes: v1.nodes.clone(),
            edges: v1.edges.clone(),
            next_node_id: v1.next_node_id,
            properties: Vec::new(),
        };
        let checksum = v1_as_canonical.checksum();

        let header = CanonicalHeader {
            magic: CANONICAL_MAGIC,
            version: 1,
            node_count: v1.nodes.len() as u64,
            edge_count: v1.edges.len() as u64,
            checksum,
        };

        // Serialize as v1
        let header_bytes = postcard::to_allocvec(&header).expect("header");
        let data_bytes = postcard::to_allocvec(&v1).expect("data");

        let mut data = Vec::new();
        data.extend_from_slice(&(header_bytes.len() as u32).to_le_bytes());
        data.extend_from_slice(&header_bytes);
        data.extend_from_slice(&data_bytes);

        // Import should succeed with empty properties
        let imported = import_canonical(&data).expect("import v1 should succeed");
        assert_eq!(imported.node_count().expect("count"), 3);
        assert_eq!(imported.edge_count().expect("count"), 3);

        // Properties should be empty
        let props = imported.get_properties(NodeId(0)).expect("get props");
        assert!(props.is_empty());
    }

    #[test]
    fn canonical_properties_included_in_checksum() {
        use crate::{Attribute, Value};

        let mut graph1 = create_test_graph();
        let graph2 = create_test_graph();

        // Add a property only to graph1
        graph1
            .store_property(NodeId(0), Attribute::new("name"), Value::new("Alice"))
            .expect("store");

        let checksum1 = canonical_checksum(&graph1);
        let checksum2 = canonical_checksum(&graph2);

        assert_ne!(
            checksum1, checksum2,
            "Properties should affect the checksum"
        );
    }
}
