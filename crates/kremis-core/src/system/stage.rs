//! # Stage Assessment
//!
//! Per KREMIS.md Section 9: Developmental Stages (FACET - Capability Maturation)
//!
//! ## Current Implementation Status
//!
//! **Stages are informational metrics only.** They indicate graph "maturity" based on
//! stable edge counts, but do not gate or restrict any functionality. All graph
//! operations are available regardless of the current stage.
//!
//! The [`StageCapability`] enum documents which capabilities *conceptually* belong
//! to each stage, but no runtime checks are performed. This is intentional for v0.2.0.
//!
//! ## Stage Definitions
//!
//! | Stage | Name | Graph Threshold | Behavior |
//! |-------|------|-----------------|----------|
//! | S0 | Signal Segmentation | 0 nodes | Informational |
//! | S1 | Pattern Crystallization | ~100 stable edges | Informational |
//! | S2 | Causal Chaining | ~1000 stable edges | Informational |
//! | S3 | Recursive Optimization | ~5000 stable edges | Informational |
//!
//! ## Important Note
//!
//! Per KREMIS.md: The edge counts (100, 1000, 5000) are illustrative placeholders.
//! Real-world thresholds may be orders of magnitude higher.

use crate::{Graph, GraphStore, Session, StorageBackend};
use serde::{Deserialize, Serialize};

// =============================================================================
// STAGE THRESHOLDS (Configurable Reference Values)
// =============================================================================

/// Threshold for S1: Pattern Crystallization
pub const S1_THRESHOLD: usize = 100;

/// Threshold for S2: Causal Chaining
pub const S2_THRESHOLD: usize = 1000;

/// Threshold for S3: Recursive Optimization
pub const S3_THRESHOLD: usize = 5000;

/// Weight threshold for "stable" edges
/// Per KREMIS.md Section 6: Stable Layer = weight >= STABLE_THRESHOLD
pub const STABLE_THRESHOLD: i64 = 10;

// =============================================================================
// STAGE ENUM
// =============================================================================

/// Developmental stages per KREMIS.md Section 9.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Stage {
    /// S0: Signal Segmentation
    S0,
    /// S1: Pattern Crystallization
    S1,
    /// S2: Causal Chaining
    S2,
    /// S3: Recursive Optimization
    S3,
}

impl Stage {
    /// Get the stage name.
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Stage::S0 => "Signal Segmentation",
            Stage::S1 => "Pattern Crystallization",
            Stage::S2 => "Causal Chaining",
            Stage::S3 => "Recursive Optimization",
        }
    }

    /// Get the minimum stable edge threshold for this stage.
    #[must_use]
    pub fn threshold(&self) -> usize {
        match self {
            Stage::S0 => 0,
            Stage::S1 => S1_THRESHOLD,
            Stage::S2 => S2_THRESHOLD,
            Stage::S3 => S3_THRESHOLD,
        }
    }

    /// Get the next stage, if any.
    #[must_use]
    pub fn next(&self) -> Option<Stage> {
        match self {
            Stage::S0 => Some(Stage::S1),
            Stage::S1 => Some(Stage::S2),
            Stage::S2 => Some(Stage::S3),
            Stage::S3 => None,
        }
    }

    /// Get the previous stage, if any.
    #[must_use]
    pub fn previous(&self) -> Option<Stage> {
        match self {
            Stage::S0 => None,
            Stage::S1 => Some(Stage::S0),
            Stage::S2 => Some(Stage::S1),
            Stage::S3 => Some(Stage::S2),
        }
    }

    /// Check if this stage is terminal (S3).
    #[must_use]
    pub fn is_terminal(&self) -> bool {
        matches!(self, Stage::S3)
    }
}

impl std::fmt::Display for Stage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}: {}", self, self.name())
    }
}

// =============================================================================
// GRAPH METRICS
// =============================================================================

/// Metrics extracted from a graph for stage assessment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphMetrics {
    /// Total number of nodes in the graph.
    pub node_count: usize,
    /// Total number of edges in the graph.
    pub edge_count: usize,
    /// Number of edges with weight >= STABLE_THRESHOLD.
    pub stable_edge_count: usize,
    /// Graph density: edge_count / node_count (0 if no nodes).
    /// Stored as fixed-point: density * 1_000_000 (integer only per AGENTS.md).
    pub density_millionths: u64,
    /// Maximum traversal depth achievable from any node.
    pub max_depth: usize,
}

impl GraphMetrics {
    /// Create new metrics with all zeros.
    #[must_use]
    pub fn empty() -> Self {
        Self {
            node_count: 0,
            edge_count: 0,
            stable_edge_count: 0,
            density_millionths: 0,
            max_depth: 0,
        }
    }

    /// Compute metrics from a graph.
    #[must_use]
    pub fn from_graph(graph: &Graph) -> Self {
        let node_count = graph.node_count().unwrap_or(0);
        let edge_count = graph.edge_count().unwrap_or(0);

        // Count stable edges (weight >= STABLE_THRESHOLD)
        let stable_edge_count = graph
            .edges()
            .filter(|(_, _, w)| w.value() >= STABLE_THRESHOLD)
            .count();

        // Density as millionths (integer math only)
        let density_millionths = if node_count > 0 {
            ((edge_count as u64).saturating_mul(1_000_000)) / (node_count as u64)
        } else {
            0
        };

        // Compute max depth via sampling (bounded computation)
        let max_depth = compute_max_depth(graph);

        Self {
            node_count,
            edge_count,
            stable_edge_count,
            density_millionths,
            max_depth,
        }
    }

    /// Get density as parts per thousand (integer only, no floats).
    #[must_use]
    pub fn density_per_thousand(&self) -> u64 {
        self.density_millionths / 1000
    }

    /// Compute metrics from a Session.
    #[must_use]
    pub fn from_session(session: &Session) -> Self {
        match session.backend() {
            StorageBackend::InMemory(graph) => Self::from_graph(graph),
            StorageBackend::Persistent(redb) => {
                let node_count = redb.node_count().unwrap_or(0);
                let edge_count = redb.edge_count().unwrap_or(0);
                let stable_edge_count = redb.stable_edge_count(STABLE_THRESHOLD).unwrap_or(0);

                let density_millionths = if node_count > 0 {
                    ((edge_count as u64).saturating_mul(1_000_000)) / (node_count as u64)
                } else {
                    0
                };

                let max_depth = 0; // Skip for redb (performance)

                Self {
                    node_count,
                    edge_count,
                    stable_edge_count,
                    density_millionths,
                    max_depth,
                }
            }
        }
    }
}

/// Compute maximum depth by sampling nodes (bounded computation).
fn compute_max_depth(graph: &Graph) -> usize {
    use std::collections::{BTreeSet, VecDeque};

    let mut max_depth = 0;
    let sample_size = 10.min(graph.node_count().unwrap_or(0));

    for (i, node) in graph.nodes().enumerate() {
        if i >= sample_size {
            break;
        }

        let mut visited = BTreeSet::new();
        let mut queue = VecDeque::new();
        let mut local_max = 0;

        queue.push_back((node.id, 0usize));
        visited.insert(node.id);

        while let Some((current, depth)) = queue.pop_front() {
            local_max = local_max.max(depth);

            if depth >= 100 {
                continue;
            }

            for (neighbor, _) in graph.neighbors_internal(current) {
                if !visited.contains(&neighbor) {
                    visited.insert(neighbor);
                    queue.push_back((neighbor, depth.saturating_add(1)));
                }
            }
        }

        max_depth = max_depth.max(local_max);
    }

    max_depth
}

// =============================================================================
// STAGE ASSESSOR
// =============================================================================

/// Stage Assessor - Pure function to determine current stage.
pub struct StageAssessor {
    s1_threshold: usize,
    s2_threshold: usize,
    s3_threshold: usize,
}

impl Default for StageAssessor {
    fn default() -> Self {
        Self::new()
    }
}

impl StageAssessor {
    /// Create a new assessor with default thresholds.
    #[must_use]
    pub fn new() -> Self {
        Self {
            s1_threshold: S1_THRESHOLD,
            s2_threshold: S2_THRESHOLD,
            s3_threshold: S3_THRESHOLD,
        }
    }

    /// Create an assessor with custom thresholds.
    #[must_use]
    pub fn with_thresholds(s1: usize, s2: usize, s3: usize) -> Self {
        Self {
            s1_threshold: s1,
            s2_threshold: s2,
            s3_threshold: s3,
        }
    }

    /// Assess the current stage based on graph metrics.
    #[must_use]
    pub fn assess(&self, graph: &Graph) -> Stage {
        let metrics = GraphMetrics::from_graph(graph);
        self.assess_from_metrics(&metrics)
    }

    /// Assess stage from pre-computed metrics.
    #[must_use]
    pub fn assess_from_metrics(&self, metrics: &GraphMetrics) -> Stage {
        if metrics.stable_edge_count >= self.s3_threshold {
            Stage::S3
        } else if metrics.stable_edge_count >= self.s2_threshold {
            Stage::S2
        } else if metrics.stable_edge_count >= self.s1_threshold {
            Stage::S1
        } else {
            Stage::S0
        }
    }

    /// Check if a specific stage is reached.
    #[must_use]
    pub fn has_reached(&self, graph: &Graph, target: Stage) -> bool {
        self.assess(graph) >= target
    }

    /// Get progress toward next stage.
    #[must_use]
    pub fn progress_to_next(&self, graph: &Graph) -> StageProgress {
        let metrics = GraphMetrics::from_graph(graph);
        self.progress_from_metrics(metrics)
    }

    /// Get progress toward next stage from a Session.
    #[must_use]
    pub fn progress_to_next_session(&self, session: &Session) -> StageProgress {
        let metrics = GraphMetrics::from_session(session);
        self.progress_from_metrics(metrics)
    }

    fn progress_from_metrics(&self, metrics: GraphMetrics) -> StageProgress {
        let current = self.assess_from_metrics(&metrics);

        let (next, current_threshold, next_threshold) = match current {
            Stage::S0 => (Stage::S1, 0, self.s1_threshold),
            Stage::S1 => (Stage::S2, self.s1_threshold, self.s2_threshold),
            Stage::S2 => (Stage::S3, self.s2_threshold, self.s3_threshold),
            Stage::S3 => return StageProgress::terminal(current, metrics),
        };

        let range = next_threshold.saturating_sub(current_threshold);
        let progress_in_range = metrics.stable_edge_count.saturating_sub(current_threshold);

        let percent = if range > 0 {
            ((progress_in_range as u64).saturating_mul(100) / (range as u64)) as u8
        } else {
            100
        };

        StageProgress {
            current,
            next: Some(next),
            percent: percent.min(100),
            stable_edges_current: metrics.stable_edge_count,
            stable_edges_needed: next_threshold,
            metrics,
        }
    }
}

/// Progress information toward the next stage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageProgress {
    pub current: Stage,
    pub next: Option<Stage>,
    pub percent: u8,
    pub stable_edges_current: usize,
    pub stable_edges_needed: usize,
    pub metrics: GraphMetrics,
}

impl StageProgress {
    fn terminal(current: Stage, metrics: GraphMetrics) -> Self {
        Self {
            current,
            next: None,
            percent: 100,
            stable_edges_current: metrics.stable_edge_count,
            stable_edges_needed: metrics.stable_edge_count,
            metrics,
        }
    }
}

// =============================================================================
// STAGE CAPABILITY (Reference Pattern)
// =============================================================================

/// Capabilities that conceptually belong to each developmental stage.
///
/// **Note:** This enum is a reference pattern documenting the intended capability
/// progression. No runtime enforcement is performed in v0.2.0 — all operations
/// are available regardless of the current stage. Use [`StageCapability::required_stage`]
/// to query the conceptual stage requirement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StageCapability {
    SignalSegmentation,
    PrimitiveLinking,
    GrammarInduction,
    PatternGeneration,
    CausalityDetection,
    TemporalMemory,
    CausalChainExtraction,
    GoalPlanning,
    FacetTriggers,
    WorldModification,
}

impl StageCapability {
    /// Get the required stage for this capability.
    #[must_use]
    pub fn required_stage(&self) -> Stage {
        match self {
            StageCapability::SignalSegmentation | StageCapability::PrimitiveLinking => Stage::S0,
            StageCapability::GrammarInduction | StageCapability::PatternGeneration => Stage::S1,
            StageCapability::CausalityDetection
            | StageCapability::TemporalMemory
            | StageCapability::CausalChainExtraction => Stage::S2,
            StageCapability::GoalPlanning
            | StageCapability::FacetTriggers
            | StageCapability::WorldModification => Stage::S3,
        }
    }

    /// Get a description of this capability.
    #[must_use]
    pub fn description(&self) -> &'static str {
        match self {
            StageCapability::SignalSegmentation => "Basic signal segmentation into discrete units",
            StageCapability::PrimitiveLinking => "Creating directed edges between sequential units",
            StageCapability::GrammarInduction => "Inducing grammar from patterns",
            StageCapability::PatternGeneration => "Generating simple patterns from structure",
            StageCapability::CausalityDetection => "Detecting causal relationships",
            StageCapability::TemporalMemory => "Accessing temporal memory patterns",
            StageCapability::CausalChainExtraction => "Extracting causal chains from graph",
            StageCapability::GoalPlanning => "Planning goals via external systems",
            StageCapability::FacetTriggers => "Triggering external facet operations",
            StageCapability::WorldModification => "Modifying external world state",
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

    fn create_graph_with_stable_edges(count: usize) -> Graph {
        let mut graph = Graph::new();

        for i in 0..count {
            let from = graph.insert_node(EntityId(i as u64 * 2)).expect("insert");
            let to = graph
                .insert_node(EntityId(i as u64 * 2 + 1))
                .expect("insert");
            graph
                .insert_edge(from, to, EdgeWeight::new(STABLE_THRESHOLD))
                .expect("insert");
        }

        graph
    }

    #[test]
    fn stage_ordering() {
        assert!(Stage::S0 < Stage::S1);
        assert!(Stage::S1 < Stage::S2);
        assert!(Stage::S2 < Stage::S3);
    }

    #[test]
    fn assess_empty_graph_is_s0() {
        let graph = Graph::new();
        let assessor = StageAssessor::new();
        assert_eq!(assessor.assess(&graph), Stage::S0);
    }

    #[test]
    fn assess_s1_threshold() {
        let graph = create_graph_with_stable_edges(S1_THRESHOLD);
        let assessor = StageAssessor::new();
        assert_eq!(assessor.assess(&graph), Stage::S1);
    }

    #[test]
    fn stage_display() {
        assert_eq!(format!("{}", Stage::S0), "S0: Signal Segmentation");
        assert_eq!(format!("{}", Stage::S3), "S3: Recursive Optimization");
    }
}
