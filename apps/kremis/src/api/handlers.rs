//! # API Endpoint Handlers
//!
//! This module implements the actual HTTP endpoint handlers.

use super::{
    AppState,
    types::{
        ExportResponse, HealthResponse, IngestRequest, IngestResponse, QueryRequest, QueryResponse,
        StageResponse, StatusResponse,
    },
};
use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use kremis_core::{
    EdgeWeight, EntityId, KremisError, NodeId, Session,
    export::{canonical_checksum, export_canonical},
    primitives::{MAX_INTERSECT_NODES, MAX_TRAVERSAL_DEPTH},
    system::{GraphMetrics, StageAssessor},
};

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

/// Execute a query using Session methods (works with both InMemory and Persistent backends).
fn execute_query_session(
    session: &Session,
    request: &QueryRequest,
) -> Result<QueryResponse, KremisError> {
    match request {
        QueryRequest::Lookup { entity_id } => match session.lookup_entity(EntityId(*entity_id)) {
            Some(node_id) => Ok(QueryResponse::with_path(vec![node_id])),
            None => Ok(QueryResponse::not_found()),
        },

        QueryRequest::Traverse { node_id, depth } => {
            // Validate depth to prevent DoS
            validate_depth(*depth)?;
            match session.traverse(NodeId(*node_id), *depth) {
                Some(artifact) => Ok(QueryResponse::with_artifact(&artifact)),
                None => Ok(QueryResponse::not_found()),
            }
        }

        QueryRequest::TraverseFiltered {
            node_id,
            depth,
            min_weight,
        } => {
            // Validate depth to prevent DoS
            validate_depth(*depth)?;
            match session.traverse_filtered(NodeId(*node_id), *depth, EdgeWeight::new(*min_weight))
            {
                Some(artifact) => Ok(QueryResponse::with_artifact(&artifact)),
                None => Ok(QueryResponse::not_found()),
            }
        }

        QueryRequest::StrongestPath { start, end } => {
            match session.strongest_path(NodeId(*start), NodeId(*end)) {
                Some(path) => Ok(QueryResponse::with_path(path)),
                None => Ok(QueryResponse::not_found()),
            }
        }

        QueryRequest::Intersect { nodes } => {
            // Validate node count limit
            if nodes.len() > MAX_INTERSECT_NODES {
                return Err(KremisError::InvalidSignal);
            }
            let node_ids: Vec<NodeId> = nodes.iter().map(|n| NodeId(*n)).collect();
            let result = session.intersect(&node_ids);
            Ok(QueryResponse::with_path(result))
        }

        QueryRequest::Related { node_id, depth } => {
            // Validate depth to prevent DoS
            validate_depth(*depth)?;
            // For Related queries, use compose which handles both backends
            match session.compose(NodeId(*node_id), *depth) {
                Some(artifact) => Ok(QueryResponse::with_artifact(&artifact)),
                None => Ok(QueryResponse::not_found()),
            }
        }
    }
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
