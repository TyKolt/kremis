//! # API Endpoint Handlers
//!
//! This module implements the actual HTTP endpoint handlers.

use super::{
    AppState,
    types::{
        ExportResponse, HealthResponse, IngestRequest, IngestResponse, PropertyJson, QueryRequest,
        QueryResponse, RetractRequest, RetractResponse, StageResponse, StatusResponse,
    },
};
use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use kremis_core::{
    Artifact, EdgeWeight, EntityId, KremisError, NodeId, Session,
    export::{canonical_checksum, canonical_crypto_hash, export_canonical},
    primitives::{MAX_INTERSECT_NODES, MAX_TRAVERSAL_DEPTH},
    system::{GraphMetrics, Stage, StageAssessor},
};
use std::collections::BTreeSet;

// =============================================================================
// HEALTH HANDLER
// =============================================================================

/// Health check endpoint.
pub async fn health_handler() -> impl IntoResponse {
    Json(HealthResponse::default())
}

// =============================================================================
// STATUS HANDLER
// =============================================================================

/// Get graph status.
pub async fn status_handler(State(state): State<AppState>) -> impl IntoResponse {
    let session = state.session.read().await;
    let metrics = GraphMetrics::from_session(&session);

    let response = StatusResponse {
        node_count: metrics.node_count,
        edge_count: metrics.edge_count,
        stable_edges: metrics.stable_edge_count,
        density_millionths: metrics.density_millionths,
    };

    (StatusCode::OK, Json(response))
}

// =============================================================================
// STAGE HANDLER
// =============================================================================

/// Get developmental stage.
pub async fn stage_handler(State(state): State<AppState>) -> impl IntoResponse {
    let session = state.session.read().await;
    let assessor = StageAssessor::new();
    let progress = assessor.progress_to_next_session(&session);

    let response = StageResponse {
        stage: format!("{:?}", progress.current),
        name: progress.current.name().to_string(),
        progress_percent: progress.percent,
        stable_edges_needed: progress.stable_edges_needed,
        stable_edges_current: progress.stable_edges_current,
    };

    (StatusCode::OK, Json(response))
}

// =============================================================================
// INGEST HANDLER
// =============================================================================

/// Ingest a signal.
pub async fn ingest_handler(
    State(state): State<AppState>,
    Json(request): Json<IngestRequest>,
) -> impl IntoResponse {
    // Validate and convert request to signal
    let signal = match request.to_signal() {
        Ok(s) => s,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(IngestResponse::error(format!("Invalid signal: {}", e))),
            );
        }
    };

    // Get write lock and ingest
    let mut session = state.session.write().await;
    match session.ingest(&signal) {
        Ok(node_id) => (StatusCode::OK, Json(IngestResponse::success(node_id))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(IngestResponse::error(format!("Ingest failed: {}", e))),
        ),
    }
}

// =============================================================================
// RETRACT HANDLER
// =============================================================================

/// Retract a signal — decrement the weight of an edge between two entities.
///
/// Returns 404 if either entity or the edge does not exist.
pub async fn retract_handler(
    State(state): State<AppState>,
    Json(request): Json<RetractRequest>,
) -> impl IntoResponse {
    let mut session = state.session.write().await;

    let from_node = match session.lookup_entity(EntityId(request.from_entity)) {
        Some(n) => n,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(RetractResponse::error("from_entity not found")),
            );
        }
    };
    let to_node = match session.lookup_entity(EntityId(request.to_entity)) {
        Some(n) => n,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(RetractResponse::error("to_entity not found")),
            );
        }
    };

    match session.decrement_edge(from_node, to_node) {
        Ok(()) => {
            let new_weight = session
                .get_edge(from_node, to_node)
                .map(|w| w.value())
                .unwrap_or(0);
            (StatusCode::OK, Json(RetractResponse::success(new_weight)))
        }
        Err(KremisError::EdgeNotFound(_, _)) => (
            StatusCode::NOT_FOUND,
            Json(RetractResponse::error("edge not found")),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(RetractResponse::error(format!("retract failed: {}", e))),
        ),
    }
}

// =============================================================================
// QUERY HANDLER
// =============================================================================

/// Execute a query.
pub async fn query_handler(
    State(state): State<AppState>,
    Json(request): Json<QueryRequest>,
) -> impl IntoResponse {
    let session = state.session.read().await;
    match execute_query_session(&session, &request) {
        Ok(response) => (StatusCode::OK, Json(response)),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(QueryResponse::error(format!("Query failed: {}", e))),
        ),
    }
}

/// Validate that depth is within bounds to prevent DoS.
fn validate_depth(depth: usize) -> Result<(), KremisError> {
    if depth > MAX_TRAVERSAL_DEPTH {
        return Err(KremisError::InvalidSignal);
    }
    Ok(())
}

/// Apply top-K filtering to an artifact: keep only the K highest-weight edges.
///
/// Ordering is deterministic: weight descending, then `from` ascending, then `to` ascending.
/// The path is rebuilt to include only nodes that appear in the top-K edges, plus the
/// original start node (first element of the original path) if it was present.
fn apply_top_k(mut artifact: Artifact, top_k: Option<usize>) -> Artifact {
    let k = match top_k {
        None | Some(0) => return artifact,
        Some(k) => k,
    };
    let mut edges = match artifact.subgraph.take() {
        None => return artifact,
        Some(e) => e,
    };
    edges.sort_by(|a, b| {
        b.2.value()
            .cmp(&a.2.value())
            .then_with(|| a.0.cmp(&b.0))
            .then_with(|| a.1.cmp(&b.1))
    });
    edges.truncate(k);
    let in_edges: BTreeSet<NodeId> = edges.iter().flat_map(|(f, t, _)| [*f, *t]).collect();
    let start = artifact.path.first().copied();
    let path: Vec<NodeId> = artifact
        .path
        .into_iter()
        .filter(|n| in_edges.contains(n) || Some(*n) == start)
        .collect();
    Artifact {
        path,
        subgraph: Some(edges),
    }
}

/// Classify grounding based on query type and whether data was found.
fn classify_grounding(request: &QueryRequest, found: bool) -> &'static str {
    if !found {
        return "unknown";
    }
    match request {
        QueryRequest::Lookup { .. } | QueryRequest::Properties { .. } => "fact",
        _ => "inference",
    }
}

/// Execute a query using Session methods (works with both InMemory and Persistent backends).
fn execute_query_session(
    session: &Session,
    request: &QueryRequest,
) -> Result<QueryResponse, KremisError> {
    let mut response = execute_query_inner(session, request)?;
    response.grounding = classify_grounding(request, response.found).to_string();
    Ok(response)
}

fn execute_query_inner(
    session: &Session,
    request: &QueryRequest,
) -> Result<QueryResponse, KremisError> {
    match request {
        QueryRequest::Lookup { entity_id } => match session.lookup_entity(EntityId(*entity_id)) {
            Some(node_id) => Ok(QueryResponse::with_path(vec![node_id])),
            None => Ok(QueryResponse::not_found().with_diagnostic("entity_not_found")),
        },

        QueryRequest::Traverse { node_id, depth } => {
            // Validate depth to prevent DoS
            validate_depth(*depth)?;
            match session.traverse(NodeId(*node_id), *depth) {
                Some(artifact) => Ok(QueryResponse::with_artifact(&artifact)),
                None => Ok(QueryResponse::not_found().with_diagnostic("node_not_found")),
            }
        }

        QueryRequest::TraverseFiltered {
            node_id,
            depth,
            min_weight,
            top_k,
        } => {
            // Validate depth to prevent DoS
            validate_depth(*depth)?;
            match session.traverse_filtered(NodeId(*node_id), *depth, EdgeWeight::new(*min_weight))
            {
                Some(artifact) => {
                    let artifact = apply_top_k(artifact, *top_k);
                    Ok(QueryResponse::with_artifact(&artifact))
                }
                None => Ok(QueryResponse::not_found().with_diagnostic("node_not_found")),
            }
        }

        QueryRequest::StrongestPath { start, end } => {
            match session.strongest_path(NodeId(*start), NodeId(*end)) {
                Some(path) => Ok(QueryResponse::with_path(path)),
                None => {
                    let reason = if session.traverse(NodeId(*start), 0).is_none() {
                        "start_not_found"
                    } else if session.traverse(NodeId(*end), 0).is_none() {
                        "end_not_found"
                    } else {
                        "no_path"
                    };
                    Ok(QueryResponse::not_found().with_diagnostic(reason))
                }
            }
        }

        QueryRequest::Intersect { nodes } => {
            // Validate node count limit
            if nodes.len() > MAX_INTERSECT_NODES {
                return Err(KremisError::InvalidSignal);
            }
            let node_ids: Vec<NodeId> = nodes.iter().map(|n| NodeId(*n)).collect();
            let result = session.intersect(&node_ids);
            let is_empty = result.is_empty();
            let mut response = QueryResponse::with_path(result);
            if is_empty {
                response = response.with_diagnostic("no_common_neighbors");
            }
            Ok(response)
        }

        QueryRequest::Related { node_id, depth } => {
            // Validate depth to prevent DoS
            validate_depth(*depth)?;
            // For Related queries, use compose which handles both backends
            match session.compose(NodeId(*node_id), *depth) {
                Some(artifact) => Ok(QueryResponse::with_artifact(&artifact)),
                None => Ok(QueryResponse::not_found().with_diagnostic("node_not_found")),
            }
        }

        QueryRequest::Properties { node_id } => match session.get_properties(NodeId(*node_id)) {
            Ok(props) => {
                let properties: Vec<PropertyJson> = props
                    .into_iter()
                    .map(|(attr, val)| PropertyJson {
                        attribute: attr.as_str().to_string(),
                        value: val.as_str().to_string(),
                    })
                    .collect();
                Ok(QueryResponse::with_properties(properties))
            }
            Err(KremisError::NodeNotFound(_)) => {
                Ok(QueryResponse::not_found().with_diagnostic("node_not_found"))
            }
            Err(e) => Err(e),
        },
    }
}

// =============================================================================
// HASH HANDLER
// =============================================================================

/// Compute BLAKE3 cryptographic hash of graph canonical export.
pub async fn hash_handler(State(state): State<AppState>) -> impl IntoResponse {
    let session = state.session.read().await;
    let graph = match session.export_graph_snapshot() {
        Ok(g) => g,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(
                    serde_json::json!({"success": false, "error": format!("Snapshot failed: {}", e)}),
                ),
            );
        }
    };
    let hash = canonical_crypto_hash(&graph);
    let checksum = canonical_checksum(&graph);
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "success": true,
            "hash": hash,
            "algorithm": "blake3",
            "checksum": checksum
        })),
    )
}

// =============================================================================
// METRICS HANDLER
// =============================================================================

/// Prometheus-compatible metrics endpoint.
pub async fn metrics_handler(State(state): State<AppState>) -> impl IntoResponse {
    let session = state.session.read().await;
    let metrics = GraphMetrics::from_session(&session);
    let assessor = StageAssessor::new();
    let progress = assessor.progress_to_next_session(&session);
    let stage_num = match progress.current {
        Stage::S0 => 0u8,
        Stage::S1 => 1u8,
        Stage::S2 => 2u8,
        Stage::S3 => 3u8,
    };
    let body = format!(
        "# HELP kremis_node_count Total number of nodes in the graph\n\
         # TYPE kremis_node_count gauge\n\
         kremis_node_count {}\n\
         # HELP kremis_edge_count Total number of edges in the graph\n\
         # TYPE kremis_edge_count gauge\n\
         kremis_edge_count {}\n\
         # HELP kremis_stable_edges Edges with weight >= stable threshold\n\
         # TYPE kremis_stable_edges gauge\n\
         kremis_stable_edges {}\n\
         # HELP kremis_density_millionths Graph density (edges*1M/nodes)\n\
         # TYPE kremis_density_millionths gauge\n\
         kremis_density_millionths {}\n\
         # HELP kremis_stage Current developmental stage (0=S0 1=S1 2=S2 3=S3)\n\
         # TYPE kremis_stage gauge\n\
         kremis_stage {}\n\
         # HELP kremis_stage_progress_percent Progress toward next stage\n\
         # TYPE kremis_stage_progress_percent gauge\n\
         kremis_stage_progress_percent {}\n",
        metrics.node_count,
        metrics.edge_count,
        metrics.stable_edge_count,
        metrics.density_millionths,
        stage_num,
        progress.percent,
    );
    (
        StatusCode::OK,
        [(
            axum::http::header::CONTENT_TYPE,
            "text/plain; version=0.0.4",
        )],
        body,
    )
}

// =============================================================================
// EXPORT HANDLER
// =============================================================================

/// Export graph in canonical format.
///
/// # M3 Fix
///
/// This handler now supports both in-memory and persistent backends
/// by using `export_graph_snapshot()` which builds a graph snapshot
/// from any backend type.
pub async fn export_handler(State(state): State<AppState>) -> impl IntoResponse {
    let session = state.session.read().await;

    // M3 FIX: Use export_graph_snapshot() which works with both backends
    let graph = match session.export_graph_snapshot() {
        Ok(g) => g,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ExportResponse::error(format!(
                    "Failed to build graph snapshot: {}",
                    e
                ))),
            );
        }
    };

    match export_canonical(&graph) {
        Ok(data) => {
            let checksum = canonical_checksum(&graph);
            (
                StatusCode::OK,
                Json(ExportResponse::success(data, checksum)),
            )
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ExportResponse::error(format!("Export failed: {}", e))),
        ),
    }
}
