//! # kremis-core
//!
//! The deterministic Graph Engine for Kremis - THE LOGIC.
//!
//! This crate implements the CORE substrate - a minimal, deterministic,
//! graph-based cognitive substrate that functions solely as a mechanism
//! to record, associate, and retrieve structural relationships.
//!
//! ## Sidecar Architecture
//!
//! This crate merges the following from the old micro-crates architecture:
//! - `kremis-types` → `types` module
//! - `kremis-facet-std/persistence` → `formats` module
//! - `kremis-facet-system/stage` → `system` module
//!
//! ## Architectural Constraints
//!
//! Per AGENTS.md, the CORE:
//! - Is the ONLY place where memory exists (stateful)
//! - Is closed: no external logic may be injected
//! - Is minimal: if a feature is not essential to signal processing, it is removed
//! - Never initiates interaction; only reacts to explicit signals or ticks
//! - Has NO async, NO network dependencies (pure Rust)

// =============================================================================
// MODULES
// =============================================================================

pub mod compositor;
pub mod confidence;
pub mod export;
pub mod formats;
pub mod graph;
pub mod grounding;
pub mod ingestor;
pub mod mutation;
pub mod primitives;
pub mod query;
pub mod session;
pub mod storage;
pub mod system;
pub mod types;

// =============================================================================
// RE-EXPORTS: Core Types (from types module)
// =============================================================================

pub use types::{
    Artifact, Attribute, Buffer, EdgeWeight, EntityId, Facet, KremisError, Node, NodeId, Signal,
    Value,
};

// =============================================================================
// RE-EXPORTS: Graph Engine
// =============================================================================

pub use compositor::Compositor;
pub use confidence::ConfidenceScore;
pub use export::{
    CanonicalGraph, CanonicalHeader, canonical_checksum, export_canonical, import_canonical,
    verify_canonical,
};
pub use graph::{Graph, GraphStore, SerializableGraph};
pub use grounding::{GroundedResult, verify_hypothesis};
pub use ingestor::Ingestor;
pub use mutation::MutationEngine;
pub use query::{Query, QueryType};
pub use session::{Session, StorageBackend};
pub use storage::RedbGraph;

// =============================================================================
// RE-EXPORTS: Formats (from formats module)
// =============================================================================

pub use formats::{PersistenceHeader, graph_from_bytes, graph_to_bytes};

// =============================================================================
// RE-EXPORTS: System (from system module)
// =============================================================================

pub use system::{
    GraphMetrics, S1_THRESHOLD, S2_THRESHOLD, S3_THRESHOLD, STABLE_THRESHOLD, Stage, StageAssessor,
    StageCapability, StageProgress,
};
