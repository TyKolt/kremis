//! # Innate Primitives
//!
//! Hardcoded runtime constants and primitives for the Kremis CORE.
//!
//! Kremis starts with zero data but fixed logic.
//! These primitives are compiled into the binary and are immutable at runtime.
//!
//! ## Primitives
//!
//! 1. **Segmentation Primitive**: Splits input streams into discrete units.
//! 2. **Linking Primitive**: Creates directed edges between sequential units.
//! 3. **Validation Primitive**: Checks graph continuity (Null Protocol).

/// The association window defines how many adjacent signals can form links.
///
/// - `ASSOCIATION_WINDOW = 1`: Links are formed only between strictly adjacent signals.
/// - Signal A connects to Signal B only if B immediately follows A.
///
/// This is the LINKING_PRIMITIVE constant.
pub const ASSOCIATION_WINDOW: usize = 1;

/// Magic bytes for the Kremis binary format header.
///
/// - File Header = Magic Bytes ("KREM") + Version (u8) before payload.
pub const MAGIC_BYTES: &[u8; 4] = b"KREM";

/// Current serialization format version.
///
/// Increment this when making breaking changes to the serialization format.
pub const FORMAT_VERSION: u8 = 1;

/// Default threshold for considering an edge "stable".
///
/// - Edges with `weight >= PROMOTION_THRESHOLD` are treated as "Stable" by FACETS.
/// - The CORE exposes weights but does not decide what constitutes a "Stable" edge.
/// - Promotion logic is implemented in FACETS.
///
/// This value is a reasonable default; FACETS may use custom thresholds.
pub const PROMOTION_THRESHOLD: i64 = 10;

/// Maximum traversal depth for graph queries.
///
/// - All queries must be computationally bounded.
/// - This prevents runaway traversals in large graphs.
pub const MAX_TRAVERSAL_DEPTH: usize = 100;

/// Maximum total node visits during strongest-path DFS.
///
/// Bounds the total work done by the DFS, preventing exponential blowup
/// on dense graphs where depth alone is insufficient.
/// When exhausted, the algorithm returns the best path found so far.
pub const MAX_VISIT_COUNT: usize = 50_000;

// =============================================================================
// INPUT VALIDATION LIMITS
// =============================================================================

/// Maximum length for attribute strings.
///
/// Attributes longer than this will be rejected by the Ingestor.
/// This prevents memory exhaustion from malicious or malformed input.
pub const MAX_ATTRIBUTE_LENGTH: usize = 256;

/// Maximum length for value strings.
///
/// Values longer than this (64KB) will be rejected by the Ingestor.
/// This prevents memory exhaustion from malicious or malformed input.
pub const MAX_VALUE_LENGTH: usize = 65536;

/// Maximum number of signals in a single ingestion sequence.
///
/// Sequences longer than this will be rejected to prevent DoS.
pub const MAX_SEQUENCE_LENGTH: usize = 10000;

/// Maximum number of distinct `(attribute, value)` properties per node.
///
/// Each node accumulates properties across successive ingestions. Length and
/// batch-size limits cap individual inputs, but without a per-node ceiling a
/// single targeted node could grow without bound, bloating the B-tree / entity
/// cache and exhausting disk/memory (slow DoS).
///
/// Adding a *new* property beyond this limit is rejected; idempotent re-inserts
/// of an already-stored pair remain allowed (they do not grow the node).
pub const MAX_PROPERTIES_PER_NODE: usize = 4096;

/// Minimum number of nodes in an Intersect query.
///
/// Intersection requires at least two sets to be meaningful.
pub const MIN_INTERSECT_NODES: usize = 2;

/// Maximum number of nodes in an Intersect query.
///
/// Limits the computational cost of intersection queries.
pub const MAX_INTERSECT_NODES: usize = 100;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn association_window_is_one() {
        // ASSOCIATION_WINDOW must be exactly 1
        assert_eq!(ASSOCIATION_WINDOW, 1);
    }

    #[test]
    fn magic_bytes_correct() {
        assert_eq!(MAGIC_BYTES, b"KREM");
    }
}
