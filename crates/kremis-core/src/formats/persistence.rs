//! # Persistence Format
//!
//! Binary serialization for Kremis graphs.
//!
//! MIGRATED FROM: kremis-facet-std/src/persistence.rs
//!
//! Per ROADMAP.md Section 4.1.3, persistence format is defined here.
//! File I/O operations are in the app layer.
//!
//! Format: Header (5 bytes) + postcard-serialized graph data.
//! - 4 bytes: Magic ("KREM")
//! - 1 byte: Version
//!
//! ## Security (H7 Fix)
//!
//! This module implements pre-deserialization validation to prevent DoS attacks:
//! - Maximum payload size limit (`MAX_PERSISTENCE_PAYLOAD_SIZE`)
//! - Header validation before payload parsing
//! - Graceful error handling for corrupted data

use crate::{primitives, Graph, KremisError, SerializableGraph};

// =============================================================================
// SECURITY LIMITS (H7 Fix)
// =============================================================================

/// Maximum allowed payload size for persistence format.
///
/// This prevents memory exhaustion from malicious or corrupted data.
/// 500 MB is a reasonable upper bound for graph data.
///
/// **Security Note**: This limit is validated BEFORE attempting deserialization
/// to prevent allocation-based DoS attacks.
pub const MAX_PERSISTENCE_PAYLOAD_SIZE: usize = 500 * 1024 * 1024; // 500 MB

/// Minimum valid file size (header only).
const MIN_FILE_SIZE: usize = 5;

// =============================================================================
// FILE HEADER
// =============================================================================

/// The persistence header precedes all graph data.
#[derive(Debug, Clone, Copy)]
pub struct PersistenceHeader {
    pub magic: [u8; 4],
    pub version: u8,
}

impl PersistenceHeader {
    /// Create a new header with current format version.
    #[must_use]
    pub fn new() -> Self {
        Self {
            magic: *primitives::MAGIC_BYTES,
            version: primitives::FORMAT_VERSION,
        }
    }

    /// Validate the header.
    pub fn validate(&self) -> Result<(), KremisError> {
        if &self.magic != primitives::MAGIC_BYTES {
            return Err(KremisError::SerializationError(
                "Invalid magic bytes".to_string(),
            ));
        }
        if self.version != primitives::FORMAT_VERSION {
            return Err(KremisError::SerializationError(format!(
                "Unsupported version: {} (expected {})",
                self.version,
                primitives::FORMAT_VERSION
            )));
        }
        Ok(())
    }

    /// Write header to bytes.
    pub fn to_bytes(&self) -> [u8; 5] {
        let mut bytes = [0u8; 5];
        bytes[0..4].copy_from_slice(&self.magic);
        bytes[4] = self.version;
        bytes
    }

    /// Read header from bytes.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, KremisError> {
        if bytes.len() < 5 {
            return Err(KremisError::SerializationError(
                "Header too short".to_string(),
            ));
        }
        let mut magic = [0u8; 4];
        magic.copy_from_slice(&bytes[0..4]);
        Ok(Self {
            magic,
            version: bytes[4],
        })
    }
}

impl Default for PersistenceHeader {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// SERIALIZATION FUNCTIONS
// =============================================================================

/// Serialize a graph to bytes (header + payload).
///
/// This is a pure transformation - no file I/O.
pub fn graph_to_bytes(graph: &Graph) -> Result<Vec<u8>, KremisError> {
    let header = PersistenceHeader::new();
    let serializable = SerializableGraph::from(graph);

    let payload = postcard::to_stdvec(&serializable)
        .map_err(|e| KremisError::SerializationError(e.to_string()))?;

    let mut result = Vec::with_capacity(5 + payload.len());
    result.extend_from_slice(&header.to_bytes());
    result.extend_from_slice(&payload);

    Ok(result)
}

/// Deserialize a graph from bytes.
///
/// This is a pure transformation - no file I/O.
///
/// # Security (H7 Fix)
///
/// This function validates:
/// 1. Minimum data size (header must be present)
/// 2. Maximum payload size (prevents memory exhaustion DoS)
/// 3. Header magic bytes and version
///
/// All validation occurs BEFORE attempting payload deserialization.
pub fn graph_from_bytes(bytes: &[u8]) -> Result<Graph, KremisError> {
    // H7 FIX: Validate minimum size
    if bytes.len() < MIN_FILE_SIZE {
        return Err(KremisError::SerializationError(
            "Data too short: minimum 5 bytes required".to_string(),
        ));
    }

    // H7 FIX: Validate maximum size BEFORE any processing
    if bytes.len() > MAX_PERSISTENCE_PAYLOAD_SIZE {
        return Err(KremisError::SerializationError(format!(
            "Data size {} bytes exceeds maximum allowed {} bytes",
            bytes.len(),
            MAX_PERSISTENCE_PAYLOAD_SIZE
        )));
    }

    // Validate header BEFORE processing payload
    let header = PersistenceHeader::from_bytes(bytes)?;
    header.validate()?;

    // Now safe to deserialize (size has been validated)
    let payload = &bytes[5..];
    let serializable: SerializableGraph = postcard::from_bytes(payload).map_err(|e| {
        KremisError::SerializationError(format!("Failed to deserialize graph data: {}", e))
    })?;

    Ok(Graph::from(serializable))
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{EdgeWeight, EntityId, GraphStore};

    #[test]
    fn header_roundtrip() {
        let header = PersistenceHeader::new();
        let bytes = header.to_bytes();
        let restored = PersistenceHeader::from_bytes(&bytes).expect("parse header");

        assert_eq!(restored.magic, *primitives::MAGIC_BYTES);
        assert_eq!(restored.version, primitives::FORMAT_VERSION);
    }

    #[test]
    fn bytes_roundtrip_bit_exact() {
        let mut graph = Graph::new();
        let a = graph.insert_node(EntityId(1)).expect("insert");
        let b = graph.insert_node(EntityId(2)).expect("insert");
        graph
            .insert_edge(a, b, EdgeWeight::new(10))
            .expect("insert");

        // First serialization
        let bytes1 = graph_to_bytes(&graph).expect("first serialize");

        // Deserialize and reserialize
        let restored = graph_from_bytes(&bytes1).expect("deserialize");
        let bytes2 = graph_to_bytes(&restored).expect("second serialize");

        // Must be bit-exact
        assert_eq!(
            bytes1, bytes2,
            "save -> load -> save must produce identical bytes"
        );
    }

    #[test]
    fn invalid_magic_rejected() {
        let mut bytes = vec![0u8; 10];
        bytes[0..4].copy_from_slice(b"XXXX"); // Wrong magic

        let result = graph_from_bytes(&bytes);
        assert!(result.is_err());
    }
}
