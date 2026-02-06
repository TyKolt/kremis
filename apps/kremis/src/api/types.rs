//! # API Request/Response Types
//!
//! This module defines the JSON structures for the HTTP API.

use kremis_core::{
    Artifact, Attribute, EntityId, KremisError, NodeId, Signal, Value,
    primitives::{MAX_ATTRIBUTE_LENGTH, MAX_VALUE_LENGTH},
};
use serde::{Deserialize, Serialize};

// =============================================================================
// HEALTH RESPONSE
// =============================================================================

/// Health check response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
}

impl Default for HealthResponse {
    fn default() -> Self {
        Self {
            status: "ok".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}

// =============================================================================
// STATUS RESPONSE
// =============================================================================

/// Graph status response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusResponse {
    pub node_count: usize,
    pub edge_count: usize,
    pub stable_edges: usize,
    pub density_millionths: u64,
}

// =============================================================================
// STAGE RESPONSE
// =============================================================================

/// Developmental stage response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageResponse {
    pub stage: String,
    pub name: String,
    pub progress_percent: u8,
    pub stable_edges_needed: usize,
    pub stable_edges_current: usize,
}

// =============================================================================
// INGEST REQUEST/RESPONSE
// =============================================================================

/// Signal ingest request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestRequest {
    pub entity_id: u64,
    pub attribute: String,
    pub value: String,
}

impl IngestRequest {
    /// Convert to a Signal, validating fields.
    ///
    /// # Validation (H2/H3 fix)
    ///
    /// This method validates:
    /// - `attribute` is non-empty and within `MAX_ATTRIBUTE_LENGTH` (256 bytes)
    /// - `value` is non-empty and within `MAX_VALUE_LENGTH` (65536 bytes)
    ///
    /// This prevents DoS attacks via oversized payloads at the API boundary,
    /// before data reaches the Core ingestor.
    pub fn to_signal(&self) -> Result<Signal, KremisError> {
        // H2 FIX: Validate attribute length
        if self.attribute.is_empty() {
            return Err(KremisError::InvalidSignal);
        }
        if self.attribute.len() > MAX_ATTRIBUTE_LENGTH {
            return Err(KremisError::SerializationError(format!(
                "Attribute length {} exceeds maximum {} bytes",
                self.attribute.len(),
                MAX_ATTRIBUTE_LENGTH
            )));
        }

        // H3 FIX: Validate value length
        if self.value.is_empty() {
            return Err(KremisError::InvalidSignal);
        }
        if self.value.len() > MAX_VALUE_LENGTH {
            return Err(KremisError::SerializationError(format!(
                "Value length {} exceeds maximum {} bytes",
                self.value.len(),
                MAX_VALUE_LENGTH
            )));
        }

        Ok(Signal::new(
            EntityId(self.entity_id),
            Attribute::new(&self.attribute),
            Value::new(&self.value),
        ))
    }
}

/// Signal ingest response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestResponse {
    pub success: bool,
    pub node_id: Option<u64>,
    pub error: Option<String>,
}

impl IngestResponse {
    pub fn success(node_id: NodeId) -> Self {
        Self {
            success: true,
            node_id: Some(node_id.0),
            error: None,
        }
    }

    pub fn error(msg: impl Into<String>) -> Self {
        Self {
            success: false,
            node_id: None,
            error: Some(msg.into()),
        }
    }
}

// =============================================================================
// QUERY REQUEST/RESPONSE
// =============================================================================

/// Query request (tagged union).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum QueryRequest {
    Lookup {
        entity_id: u64,
    },
    Traverse {
        node_id: u64,
        depth: usize,
    },
    TraverseFiltered {
        node_id: u64,
        depth: usize,
        min_weight: i64,
    },
    StrongestPath {
        start: u64,
        end: u64,
    },
    Intersect {
        nodes: Vec<u64>,
    },
    Related {
        node_id: u64,
        depth: usize,
    },
    Properties {
        node_id: u64,
    },
}

/// Property JSON representation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertyJson {
    pub attribute: String,
    pub value: String,
}

/// Query response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResponse {
    pub success: bool,
    pub found: bool,
    pub path: Vec<u64>,
    pub edges: Vec<EdgeJson>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub properties: Vec<PropertyJson>,
    pub error: Option<String>,
}

impl QueryResponse {
    pub fn not_found() -> Self {
        Self {
            success: true,
            found: false,
            path: vec![],
            edges: vec![],
            properties: vec![],
            error: None,
        }
    }

    pub fn with_path(path: Vec<NodeId>) -> Self {
        Self {
            success: true,
            found: !path.is_empty(),
            path: path.iter().map(|n| n.0).collect(),
            edges: vec![],
            properties: vec![],
            error: None,
        }
    }

    pub fn with_artifact(artifact: &Artifact) -> Self {
        let edges = artifact
            .subgraph
            .as_ref()
            .map(|sg| {
                sg.iter()
                    .map(|(from, to, weight)| EdgeJson {
                        from: from.0,
                        to: to.0,
                        weight: weight.value(),
                    })
                    .collect()
            })
            .unwrap_or_default();

        Self {
            success: true,
            found: !artifact.path.is_empty(),
            path: artifact.path.iter().map(|n| n.0).collect(),
            edges,
            properties: vec![],
            error: None,
        }
    }

    pub fn with_properties(properties: Vec<PropertyJson>) -> Self {
        Self {
            success: true,
            found: !properties.is_empty(),
            path: vec![],
            edges: vec![],
            properties,
            error: None,
        }
    }

    pub fn error(msg: impl Into<String>) -> Self {
        Self {
            success: false,
            found: false,
            path: vec![],
            edges: vec![],
            properties: vec![],
            error: Some(msg.into()),
        }
    }
}

/// Edge JSON representation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeJson {
    pub from: u64,
    pub to: u64,
    pub weight: i64,
}

// =============================================================================
// EXPORT RESPONSE
// =============================================================================

/// Export response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportResponse {
    pub success: bool,
    pub data: Option<String>, // Base64 encoded
    pub checksum: Option<u64>,
    pub error: Option<String>,
}

impl ExportResponse {
    pub fn success(data: Vec<u8>, checksum: u64) -> Self {
        Self {
            success: true,
            data: Some(base64::Engine::encode(
                &base64::engine::general_purpose::STANDARD,
                &data,
            )),
            checksum: Some(checksum),
            error: None,
        }
    }

    pub fn error(msg: impl Into<String>) -> Self {
        Self {
            success: false,
            data: None,
            checksum: None,
            error: Some(msg.into()),
        }
    }
}
