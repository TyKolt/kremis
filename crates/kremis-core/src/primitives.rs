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

/// Maximum path length for pathfinding algorithms.
///
/// Limits the number of nodes in a single path to prevent
/// unbounded computation in strongest_path and similar queries.
pub const MAX_PATH_LENGTH: usize = 1000;

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
