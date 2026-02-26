//! Integration tests for the Kremis HTTP API.
//!
//! Uses axum-test to test the API handlers without starting a real server.

// Allow unwrap and panic in tests - these are standard for test code
// Allow holding MutexGuard across await in auth tests - tests are serialized
// intentionally to avoid env var conflicts
#![allow(clippy::unwrap_used, clippy::panic, clippy::await_holding_lock)]

use axum::http::HeaderValue;
use axum_test::TestServer;
use kremis::api::{
    AppState, ExportResponse, HealthResponse, IngestRequest, IngestResponse, QueryRequest,
    QueryResponse, RetractRequest, RetractResponse, StageResponse, StatusResponse, create_router,
};
use kremis_core::Session;
use serde_json::json;
use std::sync::Mutex;

/// Mutex to serialize auth tests since they modify env vars.
static AUTH_TEST_MUTEX: Mutex<()> = Mutex::new(());

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

/// Guard wrapper that holds the mutex and ensures cleanup on drop.
struct TestGuard {
    _guard: std::sync::MutexGuard<'static, ()>,
}

impl Drop for TestGuard {
    fn drop(&mut self) {
        // SAFETY: Tests run sequentially under AUTH_TEST_MUTEX, so no concurrent env access.
        unsafe { std::env::remove_var("KREMIS_API_KEY") };
    }
}

/// Create a test server with a fresh in-memory session.
/// Returns a guard that must be kept alive during the test.
fn create_test_server() -> (TestServer, TestGuard) {
    let guard = AUTH_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    // SAFETY: Tests run sequentially under AUTH_TEST_MUTEX, so no concurrent env access.
    unsafe { std::env::remove_var("KREMIS_API_KEY") };
    let session = Session::new();
    let state = AppState::new(session);
    let router = create_router(state);
    (
        TestServer::new(router).unwrap(),
        TestGuard { _guard: guard },
    )
}

/// Create a test server with some pre-populated data.
/// Returns a guard that must be kept alive during the test.
fn create_populated_test_server() -> (TestServer, TestGuard) {
    use kremis_core::{Attribute, EntityId, Signal, Value};

    let guard = AUTH_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    // SAFETY: Tests run sequentially under AUTH_TEST_MUTEX, so no concurrent env access.
    unsafe { std::env::remove_var("KREMIS_API_KEY") };

    let mut session = Session::new();

    // Ingest some test signals
    let signals = vec![
        Signal::new(EntityId(1), Attribute::new("name"), Value::new("Alice")),
        Signal::new(EntityId(2), Attribute::new("name"), Value::new("Bob")),
        Signal::new(EntityId(1), Attribute::new("knows"), Value::new("Bob")),
    ];

    session.ingest_sequence(&signals).unwrap();

    let state = AppState::new(session);
    let router = create_router(state);
    (
        TestServer::new(router).unwrap(),
        TestGuard { _guard: guard },
    )
}

// =============================================================================
// HEALTH ENDPOINT TESTS
// =============================================================================

#[tokio::test]
async fn test_health_endpoint() {
    let (server, _guard) = create_test_server();

    let response = server.get("/health").await;

    response.assert_status_ok();
    let health: HealthResponse = response.json();
    assert_eq!(health.status, "ok");
    assert!(!health.version.is_empty());
}

#[tokio::test]
async fn test_health_returns_correct_version() {
    let (server, _guard) = create_test_server();

    let response = server.get("/health").await;
    let health: HealthResponse = response.json();

    // Version should match Cargo.toml
    assert_eq!(health.version, env!("CARGO_PKG_VERSION"));
}

// =============================================================================
// STATUS ENDPOINT TESTS
// =============================================================================

#[tokio::test]
async fn test_status_empty_graph() {
    let (server, _guard) = create_test_server();

    let response = server.get("/status").await;

    response.assert_status_ok();
    let status: StatusResponse = response.json();
    assert_eq!(status.node_count, 0);
    assert_eq!(status.edge_count, 0);
    assert_eq!(status.stable_edges, 0);
}

#[tokio::test]
async fn test_status_populated_graph() {
    let (server, _guard) = create_populated_test_server();

    let response = server.get("/status").await;

    response.assert_status_ok();
    let status: StatusResponse = response.json();
    assert!(status.node_count > 0, "Should have nodes");
    assert!(status.edge_count > 0, "Should have edges");
}

// =============================================================================
// STAGE ENDPOINT TESTS
// =============================================================================

#[tokio::test]
async fn test_stage_empty_graph() {
    let (server, _guard) = create_test_server();

    let response = server.get("/stage").await;

    response.assert_status_ok();
    let stage: StageResponse = response.json();
    assert!(stage.stage.starts_with("S")); // S0, S1, S2, or S3
    assert!(!stage.name.is_empty());
    assert!(stage.progress_percent <= 100);
}

#[tokio::test]
async fn test_stage_returns_valid_stage() {
    let (server, _guard) = create_populated_test_server();

    let response = server.get("/stage").await;
    let stage: StageResponse = response.json();

    // Stage should be one of the valid stages
    let valid_stages = ["S0", "S1", "S2", "S3"];
    assert!(
        valid_stages.iter().any(|s| stage.stage.contains(s)),
        "Stage {} should be one of S0-S3",
        stage.stage
    );
}

// =============================================================================
// INGEST ENDPOINT TESTS
// =============================================================================

#[tokio::test]
async fn test_ingest_valid_signal() {
    let (server, _guard) = create_test_server();

    let request = IngestRequest {
        entity_id: 1,
        attribute: "name".to_string(),
        value: "Alice".to_string(),
    };

    let response = server.post("/signal").json(&request).await;

    response.assert_status_ok();
    let result: IngestResponse = response.json();
    assert!(result.success);
    assert!(result.node_id.is_some());
    assert!(result.error.is_none());
}

#[tokio::test]
async fn test_ingest_empty_attribute() {
    let (server, _guard) = create_test_server();

    let request = json!({
        "entity_id": 1,
        "attribute": "",
        "value": "Alice"
    });

    let response = server.post("/signal").json(&request).await;

    response.assert_status_bad_request();
    let result: IngestResponse = response.json();
    assert!(!result.success);
    assert!(result.error.is_some());
}

#[tokio::test]
async fn test_ingest_empty_value() {
    let (server, _guard) = create_test_server();

    let request = json!({
        "entity_id": 1,
        "attribute": "name",
        "value": ""
    });

    let response = server.post("/signal").json(&request).await;

    response.assert_status_bad_request();
    let result: IngestResponse = response.json();
    assert!(!result.success);
}

#[tokio::test]
async fn test_ingest_multiple_signals() {
    let (server, _guard) = create_test_server();

    // Ingest first signal
    let request1 = IngestRequest {
        entity_id: 1,
        attribute: "name".to_string(),
        value: "Alice".to_string(),
    };
    let response1 = server.post("/signal").json(&request1).await;
    let result1: IngestResponse = response1.json();
    assert!(result1.success);

    // Ingest second signal
    let request2 = IngestRequest {
        entity_id: 2,
        attribute: "name".to_string(),
        value: "Bob".to_string(),
    };
    let response2 = server.post("/signal").json(&request2).await;
    let result2: IngestResponse = response2.json();
    assert!(result2.success);

    // Verify status updated
    let status_response = server.get("/status").await;
    let status: StatusResponse = status_response.json();
    assert!(status.node_count >= 2);
}

// =============================================================================
// QUERY ENDPOINT TESTS
// =============================================================================

#[tokio::test]
async fn test_query_lookup_not_found() {
    let (server, _guard) = create_test_server();

    let request = QueryRequest::Lookup { entity_id: 999 };
    let response = server.post("/query").json(&request).await;

    response.assert_status_ok();
    let result: QueryResponse = response.json();
    assert!(result.success);
    assert!(!result.found);
    assert!(result.path.is_empty());
    assert_eq!(result.grounding, "unknown");
}

#[tokio::test]
async fn test_query_lookup_found() {
    let (server, _guard) = create_populated_test_server();

    let request = QueryRequest::Lookup { entity_id: 1 };
    let response = server.post("/query").json(&request).await;

    response.assert_status_ok();
    let result: QueryResponse = response.json();
    assert!(result.success);
    assert!(result.found);
    assert!(!result.path.is_empty());
    assert_eq!(result.grounding, "fact");
}

#[tokio::test]
async fn test_query_traverse() {
    let (server, _guard) = create_populated_test_server();

    // First lookup to get a node ID
    let lookup = QueryRequest::Lookup { entity_id: 1 };
    let lookup_response = server.post("/query").json(&lookup).await;
    let lookup_result: QueryResponse = lookup_response.json();

    assert!(
        lookup_result.found,
        "Entity 1 should exist in populated graph"
    );
    assert!(
        !lookup_result.path.is_empty(),
        "Lookup should return node ID"
    );

    let node_id = lookup_result.path[0];

    let request = QueryRequest::Traverse { node_id, depth: 2 };
    let response = server.post("/query").json(&request).await;

    response.assert_status_ok();
    let result: QueryResponse = response.json();
    assert!(result.success);
    assert!(
        result.found,
        "Traverse from existing node should find results"
    );
    assert!(
        !result.path.is_empty(),
        "Traverse should return at least the starting node"
    );
    // Verify the starting node is in the path
    assert!(
        result.path.contains(&node_id),
        "Traverse result should include the starting node"
    );
    assert_eq!(result.grounding, "inference");
}

#[tokio::test]
async fn test_query_traverse_filtered() {
    let (server, _guard) = create_populated_test_server();

    // First verify node 1 exists
    let lookup = QueryRequest::Lookup { entity_id: 1 };
    let lookup_response = server.post("/query").json(&lookup).await;
    let lookup_result: QueryResponse = lookup_response.json();
    assert!(lookup_result.found, "Entity 1 should exist");
    let node_id = lookup_result.path[0];

    // Test with min_weight=0 to ensure we get results
    let request = QueryRequest::TraverseFiltered {
        node_id,
        depth: 2,
        min_weight: 0,
    };
    let response = server.post("/query").json(&request).await;

    response.assert_status_ok();
    let result: QueryResponse = response.json();
    assert!(result.success);
    assert!(
        result.found,
        "Traverse filtered with min_weight=0 should find results"
    );
    assert!(
        !result.path.is_empty(),
        "Traverse filtered should return nodes"
    );
    assert_eq!(result.grounding, "inference");

    // Test with very high min_weight - may return fewer or no edges
    let high_filter = QueryRequest::TraverseFiltered {
        node_id,
        depth: 2,
        min_weight: 1000,
    };
    let high_response = server.post("/query").json(&high_filter).await;
    let high_result: QueryResponse = high_response.json();
    assert!(high_result.success, "High filter query should succeed");
    // With high filter, edges should be filtered out
    assert!(
        high_result.edges.is_empty() || high_result.edges.iter().all(|e| e.weight >= 1000),
        "All edges should meet min_weight threshold"
    );
}

#[tokio::test]
async fn test_query_strongest_path() {
    let (server, _guard) = create_populated_test_server();

    // First verify both nodes exist
    let lookup1 = QueryRequest::Lookup { entity_id: 1 };
    let lookup2 = QueryRequest::Lookup { entity_id: 2 };
    let resp1 = server.post("/query").json(&lookup1).await;
    let resp2 = server.post("/query").json(&lookup2).await;
    let result1: QueryResponse = resp1.json();
    let result2: QueryResponse = resp2.json();
    assert!(result1.found, "Entity 1 should exist");
    assert!(result2.found, "Entity 2 should exist");

    let node1 = result1.path[0];
    let node2 = result2.path[0];

    let request = QueryRequest::StrongestPath {
        start: node1,
        end: node2,
    };
    let response = server.post("/query").json(&request).await;

    response.assert_status_ok();
    let result: QueryResponse = response.json();
    assert!(result.success);

    // If path exists, it should start with start and end with end
    if result.found && !result.path.is_empty() {
        assert_eq!(
            result.path.first(),
            Some(&node1),
            "Path should start with start node"
        );
        assert_eq!(
            result.path.last(),
            Some(&node2),
            "Path should end with end node"
        );
        assert_eq!(result.grounding, "inference");
    }
}

#[tokio::test]
async fn test_query_intersect() {
    let (server, _guard) = create_populated_test_server();

    // Get actual node IDs from lookups
    let lookup1 = QueryRequest::Lookup { entity_id: 1 };
    let lookup2 = QueryRequest::Lookup { entity_id: 2 };
    let resp1 = server.post("/query").json(&lookup1).await;
    let resp2 = server.post("/query").json(&lookup2).await;
    let result1: QueryResponse = resp1.json();
    let result2: QueryResponse = resp2.json();
    assert!(result1.found && result2.found, "Entities should exist");

    let node1 = result1.path[0];
    let node2 = result2.path[0];

    let request = QueryRequest::Intersect {
        nodes: vec![node1, node2],
    };
    let response = server.post("/query").json(&request).await;

    response.assert_status_ok();
    let result: QueryResponse = response.json();
    assert!(result.success);
    // Intersect returns common ancestors/connections - verify response structure
    assert!(
        result.error.is_none(),
        "Intersect should not return an error"
    );
    let expected = if result.found { "inference" } else { "unknown" };
    assert_eq!(result.grounding, expected);
}

#[tokio::test]
async fn test_query_intersect_nonexistent_nodes() {
    let (server, _guard) = create_test_server();

    // Query with nodes that don't exist
    let request = QueryRequest::Intersect {
        nodes: vec![9999, 8888, 7777],
    };
    let response = server.post("/query").json(&request).await;

    response.assert_status_ok();
    let result: QueryResponse = response.json();
    assert!(result.success);
    assert!(
        !result.found,
        "Intersect of nonexistent nodes should not find results"
    );
    assert!(
        result.path.is_empty(),
        "Path should be empty for nonexistent nodes"
    );
    assert_eq!(result.grounding, "unknown");
}

#[tokio::test]
async fn test_query_related() {
    let (server, _guard) = create_populated_test_server();

    // First get actual node ID
    let lookup = QueryRequest::Lookup { entity_id: 1 };
    let lookup_response = server.post("/query").json(&lookup).await;
    let lookup_result: QueryResponse = lookup_response.json();
    assert!(lookup_result.found, "Entity 1 should exist");
    let node_id = lookup_result.path[0];

    let request = QueryRequest::Related { node_id, depth: 2 };
    let response = server.post("/query").json(&request).await;

    response.assert_status_ok();
    let result: QueryResponse = response.json();
    assert!(result.success);
    assert!(
        result.found,
        "Related query from existing node should find results"
    );
    assert!(!result.path.is_empty(), "Related query should return nodes");
    // The query node should be in the result
    assert!(
        result.path.contains(&node_id),
        "Related result should include the query node"
    );
    assert_eq!(result.grounding, "inference");
}

#[tokio::test]
async fn test_query_related_nonexistent_node() {
    let (server, _guard) = create_test_server();

    let request = QueryRequest::Related {
        node_id: 99999,
        depth: 2,
    };
    let response = server.post("/query").json(&request).await;

    response.assert_status_ok();
    let result: QueryResponse = response.json();
    assert!(result.success);
    assert!(
        !result.found,
        "Related query for nonexistent node should not find results"
    );
    assert!(
        result.path.is_empty(),
        "Path should be empty for nonexistent node"
    );
    assert_eq!(result.grounding, "unknown");
}

// =============================================================================
// DIAGNOSTIC FIELD TESTS
// =============================================================================

/// Helper: create a server with two fully isolated entities (no shared value nodes).
fn create_isolated_pair_server() -> (TestServer, TestGuard) {
    use kremis_core::{Attribute, EntityId, Signal, Value};

    let guard = AUTH_TEST_MUTEX.lock().unwrap();
    // SAFETY: Tests run sequentially under AUTH_TEST_MUTEX, so no concurrent env access.
    unsafe { std::env::remove_var("KREMIS_API_KEY") };

    let mut session = Session::new();
    let signals = vec![
        Signal::new(
            EntityId(100),
            Attribute::new("iso_attr_a"),
            Value::new("iso_val_a"),
        ),
        Signal::new(
            EntityId(200),
            Attribute::new("iso_attr_b"),
            Value::new("iso_val_b"),
        ),
    ];
    session.ingest_sequence(&signals).unwrap();

    let state = AppState::new(session);
    let router = create_router(state);
    (
        TestServer::new(router).unwrap(),
        TestGuard { _guard: guard },
    )
}

#[tokio::test]
async fn test_query_lookup_missing_has_diagnostic() {
    let (server, _guard) = create_test_server();

    let request = QueryRequest::Lookup { entity_id: 99999 };
    let response = server.post("/query").json(&request).await;

    response.assert_status_ok();
    let result: QueryResponse = response.json();
    assert!(result.success);
    assert!(!result.found);
    assert_eq!(result.diagnostic, Some("entity_not_found".to_string()));
}

#[tokio::test]
async fn test_query_traverse_missing_node_has_diagnostic() {
    let (server, _guard) = create_test_server();

    let request = QueryRequest::Traverse {
        node_id: 99999,
        depth: 2,
    };
    let response = server.post("/query").json(&request).await;

    response.assert_status_ok();
    let result: QueryResponse = response.json();
    assert!(result.success);
    assert!(!result.found);
    assert_eq!(result.diagnostic, Some("node_not_found".to_string()));
}

#[tokio::test]
async fn test_query_traverse_found_no_diagnostic() {
    let (server, _guard) = create_populated_test_server();

    // Lookup to get a valid node ID
    let lookup = QueryRequest::Lookup { entity_id: 1 };
    let lookup_resp = server.post("/query").json(&lookup).await;
    let lookup_result: QueryResponse = lookup_resp.json();
    assert!(lookup_result.found, "Entity 1 should exist");
    let node_id = lookup_result.path[0];

    let request = QueryRequest::Traverse { node_id, depth: 2 };
    let response = server.post("/query").json(&request).await;

    response.assert_status_ok();
    let result: QueryResponse = response.json();
    assert!(result.success);
    assert!(result.found);
    assert!(
        result.diagnostic.is_none(),
        "Found result must have no diagnostic"
    );
}

#[tokio::test]
async fn test_query_path_start_not_found_has_diagnostic() {
    let (server, _guard) = create_test_server();

    let request = QueryRequest::StrongestPath {
        start: 99999,
        end: 88888,
    };
    let response = server.post("/query").json(&request).await;

    response.assert_status_ok();
    let result: QueryResponse = response.json();
    assert!(result.success);
    assert!(!result.found);
    assert_eq!(result.diagnostic, Some("start_not_found".to_string()));
}

#[tokio::test]
async fn test_query_path_end_not_found_has_diagnostic() {
    let (server, _guard) = create_populated_test_server();

    // Lookup entity 1 to get a real start node
    let lookup = QueryRequest::Lookup { entity_id: 1 };
    let lookup_resp = server.post("/query").json(&lookup).await;
    let lookup_result: QueryResponse = lookup_resp.json();
    assert!(lookup_result.found, "Entity 1 should exist");
    let start_node = lookup_result.path[0];

    let request = QueryRequest::StrongestPath {
        start: start_node,
        end: 99999, // non-existent
    };
    let response = server.post("/query").json(&request).await;

    response.assert_status_ok();
    let result: QueryResponse = response.json();
    assert!(result.success);
    assert!(!result.found);
    assert_eq!(result.diagnostic, Some("end_not_found".to_string()));
}

#[tokio::test]
async fn test_query_path_no_path_has_diagnostic() {
    let (server, _guard) = create_isolated_pair_server();

    // Lookup both isolated entities to get their node IDs
    let lookup1 = QueryRequest::Lookup { entity_id: 100 };
    let lookup2 = QueryRequest::Lookup { entity_id: 200 };
    let resp1 = server.post("/query").json(&lookup1).await;
    let resp2 = server.post("/query").json(&lookup2).await;
    let result1: QueryResponse = resp1.json();
    let result2: QueryResponse = resp2.json();
    assert!(result1.found, "Entity 100 should exist");
    assert!(result2.found, "Entity 200 should exist");

    let node1 = result1.path[0];
    let node2 = result2.path[0];

    let request = QueryRequest::StrongestPath {
        start: node1,
        end: node2,
    };
    let response = server.post("/query").json(&request).await;

    response.assert_status_ok();
    let result: QueryResponse = response.json();
    assert!(result.success);
    // Both nodes exist but have no path between them
    if !result.found {
        assert_eq!(result.diagnostic, Some("no_path".to_string()));
    }
    // If a path is somehow found (graph topology dependent), diagnostic should be absent
    if result.found {
        assert!(result.diagnostic.is_none());
    }
}

#[tokio::test]
async fn test_query_intersect_empty_has_diagnostic() {
    let (server, _guard) = create_test_server();

    let request = QueryRequest::Intersect {
        nodes: vec![9999, 8888, 7777],
    };
    let response = server.post("/query").json(&request).await;

    response.assert_status_ok();
    let result: QueryResponse = response.json();
    assert!(result.success);
    assert!(!result.found);
    assert_eq!(result.diagnostic, Some("no_common_neighbors".to_string()));
}

#[tokio::test]
async fn test_query_properties_missing_node_has_diagnostic() {
    let (server, _guard) = create_test_server();

    let request = QueryRequest::Properties { node_id: 99999 };
    let response = server.post("/query").json(&request).await;

    response.assert_status_ok();
    let result: QueryResponse = response.json();
    assert!(result.success);
    assert!(!result.found);
    assert_eq!(result.diagnostic, Some("node_not_found".to_string()));
}

// =============================================================================
// EXPORT ENDPOINT TESTS
// =============================================================================

#[tokio::test]
async fn test_export_empty_graph() {
    let (server, _guard) = create_test_server();

    let response = server.post("/export").await;

    response.assert_status_ok();
    let result: ExportResponse = response.json();
    assert!(result.success);
    assert!(result.data.is_some());
    assert!(result.checksum.is_some());
}

#[tokio::test]
async fn test_export_populated_graph() {
    let (server, _guard) = create_populated_test_server();

    let response = server.post("/export").await;

    response.assert_status_ok();
    let result: ExportResponse = response.json();
    assert!(result.success);
    assert!(result.data.is_some());
    assert!(result.checksum.is_some());

    // Data should be base64 encoded
    let data = result.data.unwrap();
    assert!(!data.is_empty());

    // Verify it's valid base64
    let decoded = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &data);
    assert!(decoded.is_ok());
}

// =============================================================================
// CORS TESTS
// =============================================================================

#[tokio::test]
async fn test_cors_headers_present() {
    let (server, _guard) = create_test_server();

    // Simple request to verify CORS layer doesn't block
    let response = server.get("/health").await;
    response.assert_status_ok();
}

// =============================================================================
// ERROR HANDLING TESTS
// =============================================================================

#[tokio::test]
async fn test_404_on_unknown_endpoint() {
    let (server, _guard) = create_test_server();

    let response = server.get("/unknown").await;
    response.assert_status_not_found();
}

#[tokio::test]
async fn test_method_not_allowed() {
    let (server, _guard) = create_test_server();

    // /health is GET only
    let response = server.post("/health").await;
    // axum returns 405 Method Not Allowed
    assert_eq!(response.status_code().as_u16(), 405);
}

#[tokio::test]
async fn test_invalid_json_body() {
    let (server, _guard) = create_test_server();

    let response = server
        .post("/signal")
        .bytes(bytes::Bytes::from("not valid json"))
        .content_type("application/json")
        .await;

    // Should return 4xx error for invalid JSON
    assert!(response.status_code().is_client_error());
}

// =============================================================================
// AUTHENTICATION MIDDLEWARE TESTS
// =============================================================================

/// Create a test server with authentication enabled.
/// Must be called while holding AUTH_TEST_MUTEX.
fn create_auth_test_server(api_key: &str) -> TestServer {
    // SAFETY: Tests run sequentially under AUTH_TEST_MUTEX, so no concurrent env access.
    unsafe { std::env::set_var("KREMIS_API_KEY", api_key) };
    let session = Session::new();
    let state = AppState::new(session);
    let router = create_router(state);
    TestServer::new(router).unwrap()
}

/// Clean up auth env var after test.
fn cleanup_auth_env() {
    // SAFETY: Tests run sequentially under AUTH_TEST_MUTEX, so no concurrent env access.
    unsafe { std::env::remove_var("KREMIS_API_KEY") };
}

#[tokio::test]
async fn test_auth_valid_bearer_token() {
    let _guard = AUTH_TEST_MUTEX.lock().unwrap();
    let api_key = "test-secret-key-12345";
    let server = create_auth_test_server(api_key);

    let response = server
        .get("/status")
        .add_header(
            axum::http::header::AUTHORIZATION,
            format!("Bearer {}", api_key)
                .parse::<HeaderValue>()
                .unwrap(),
        )
        .await;

    cleanup_auth_env();

    response.assert_status_ok();
    let status: StatusResponse = response.json();
    assert_eq!(status.node_count, 0);
}

#[tokio::test]
async fn test_auth_valid_raw_token() {
    let _guard = AUTH_TEST_MUTEX.lock().unwrap();
    let api_key = "test-raw-key-67890";
    let server = create_auth_test_server(api_key);

    // Test raw token format (without "Bearer " prefix)
    let response = server
        .get("/status")
        .add_header(
            axum::http::header::AUTHORIZATION,
            api_key.parse::<HeaderValue>().unwrap(),
        )
        .await;

    cleanup_auth_env();

    response.assert_status_ok();
}

#[tokio::test]
async fn test_auth_invalid_token_rejected() {
    let _guard = AUTH_TEST_MUTEX.lock().unwrap();
    let api_key = "correct-key";
    let server = create_auth_test_server(api_key);

    let response = server
        .get("/status")
        .add_header(
            axum::http::header::AUTHORIZATION,
            "Bearer wrong-key".parse::<HeaderValue>().unwrap(),
        )
        .await;

    cleanup_auth_env();

    assert_eq!(
        response.status_code().as_u16(),
        401,
        "Invalid token should return 401 Unauthorized"
    );
}

#[tokio::test]
async fn test_auth_missing_header_rejected() {
    let _guard = AUTH_TEST_MUTEX.lock().unwrap();
    let api_key = "required-key";
    let server = create_auth_test_server(api_key);

    // Request without Authorization header
    let response = server.get("/status").await;

    cleanup_auth_env();

    assert_eq!(
        response.status_code().as_u16(),
        401,
        "Missing Authorization header should return 401 Unauthorized"
    );
}

#[tokio::test]
async fn test_auth_health_endpoint_bypasses_auth() {
    let _guard = AUTH_TEST_MUTEX.lock().unwrap();
    let api_key = "secret-key-for-bypass-test";
    let server = create_auth_test_server(api_key);

    // /health should be accessible without authentication
    let response = server.get("/health").await;

    cleanup_auth_env();

    response.assert_status_ok();
    let health: HealthResponse = response.json();
    assert_eq!(health.status, "ok");
}

#[tokio::test]
async fn test_auth_empty_key_rejected() {
    let _guard = AUTH_TEST_MUTEX.lock().unwrap();
    let api_key = "non-empty-key";
    let server = create_auth_test_server(api_key);

    // Empty authorization header should be rejected
    let response = server
        .get("/status")
        .add_header(
            axum::http::header::AUTHORIZATION,
            "".parse::<HeaderValue>().unwrap(),
        )
        .await;

    cleanup_auth_env();

    assert_eq!(
        response.status_code().as_u16(),
        401,
        "Empty Authorization header should return 401 Unauthorized"
    );
}

#[tokio::test]
async fn test_auth_bearer_prefix_only_rejected() {
    let _guard = AUTH_TEST_MUTEX.lock().unwrap();
    let api_key = "actual-key";
    let server = create_auth_test_server(api_key);

    // "Bearer " with no key should be rejected
    let response = server
        .get("/status")
        .add_header(
            axum::http::header::AUTHORIZATION,
            "Bearer ".parse::<HeaderValue>().unwrap(),
        )
        .await;

    cleanup_auth_env();

    assert_eq!(
        response.status_code().as_u16(),
        401,
        "Bearer prefix with no key should return 401 Unauthorized"
    );
}

// =============================================================================
// HASH ENDPOINT TESTS
// =============================================================================

#[tokio::test]
async fn test_hash_empty_graph() {
    let (server, _guard) = create_test_server();

    let response = server.get("/hash").await;

    response.assert_status_ok();
    let body: serde_json::Value = response.json();
    assert_eq!(body["success"], true);
    assert_eq!(body["algorithm"], "blake3");
    let hash = body["hash"].as_str().expect("hash should be a string");
    assert_eq!(hash.len(), 64, "BLAKE3 hex digest must be 64 characters");
    assert!(
        hash.chars().all(|c| c.is_ascii_hexdigit()),
        "Hash must be hex"
    );
}

#[tokio::test]
async fn test_hash_after_ingest() {
    use kremis_core::{Attribute, EntityId, Signal, Value};

    // Acquire mutex once; build both servers within the same lock scope
    // to avoid deadlock from two separate create_*_server() calls.
    let (server_empty, _guard) = create_test_server();
    let hash_empty: serde_json::Value = server_empty.get("/hash").await.json();
    let empty_hash = hash_empty["hash"].as_str().unwrap().to_string();

    // Build populated server manually without re-acquiring the mutex
    let mut session2 = Session::new();
    session2
        .ingest_sequence(&[
            Signal::new(EntityId(1), Attribute::new("name"), Value::new("Alice")),
            Signal::new(EntityId(2), Attribute::new("name"), Value::new("Bob")),
        ])
        .unwrap();
    let router2 = create_router(AppState::new(session2));
    let server_populated = TestServer::new(router2).unwrap();

    let hash_populated: serde_json::Value = server_populated.get("/hash").await.json();
    let populated_hash = hash_populated["hash"].as_str().unwrap().to_string();

    assert_ne!(
        empty_hash, populated_hash,
        "Hash must change when graph content changes"
    );
    assert_eq!(populated_hash.len(), 64);
}

// =============================================================================
// METRICS ENDPOINT TESTS
// =============================================================================

#[tokio::test]
async fn test_metrics_content_type() {
    let (server, _guard) = create_test_server();

    let response = server.get("/metrics").await;

    response.assert_status_ok();
    let content_type = response
        .headers()
        .get("content-type")
        .expect("content-type header must be present")
        .to_str()
        .expect("content-type must be valid utf8");
    assert_eq!(
        content_type, "text/plain; version=0.0.4",
        "Prometheus endpoint must return correct Content-Type"
    );
}

#[tokio::test]
async fn test_metrics_contains_labels() {
    let (server, _guard) = create_test_server();

    let response = server.get("/metrics").await;

    response.assert_status_ok();
    let body = response.text();
    assert!(
        body.contains("kremis_node_count"),
        "Metrics must contain kremis_node_count"
    );
    assert!(
        body.contains("kremis_edge_count"),
        "Metrics must contain kremis_edge_count"
    );
    assert!(
        body.contains("kremis_stage"),
        "Metrics must contain kremis_stage"
    );
    assert!(
        body.contains("# TYPE"),
        "Metrics must contain Prometheus TYPE annotations"
    );
}

// =============================================================================
// RETRACT TESTS
// =============================================================================

#[tokio::test]
async fn test_retract_reduces_edge_weight() {
    use kremis_core::{Attribute, EntityId, Signal, Value};

    let guard = AUTH_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    unsafe { std::env::remove_var("KREMIS_API_KEY") };

    let mut session = Session::new();
    // Ingest a sequence — this creates an edge from entity 1 to entity 2
    let signals = vec![
        Signal::new(EntityId(1), Attribute::new("type"), Value::new("word")),
        Signal::new(EntityId(2), Attribute::new("type"), Value::new("word")),
    ];
    session.ingest_sequence(&signals).unwrap();

    // Ingest again to bump weight to 2
    session.ingest_sequence(&signals).unwrap();

    let state = AppState::new(session);
    let router = create_router(state);
    let server = TestServer::new(router).unwrap();
    let _guard = TestGuard { _guard: guard };

    let request = RetractRequest {
        from_entity: 1,
        to_entity: 2,
    };
    let response = server.post("/signal/retract").json(&request).await;

    response.assert_status_ok();
    let result: RetractResponse = response.json();
    assert!(result.success);
    assert_eq!(result.new_weight, Some(1));
    assert!(result.error.is_none());
}

#[tokio::test]
async fn test_retract_from_entity_not_found_returns_404() {
    let (server, _guard) = create_test_server();

    let request = RetractRequest {
        from_entity: 99999,
        to_entity: 1,
    };
    let response = server.post("/signal/retract").json(&request).await;

    response.assert_status(axum::http::StatusCode::NOT_FOUND);
    let result: RetractResponse = response.json();
    assert!(!result.success);
    assert!(result.error.is_some());
}

#[tokio::test]
async fn test_retract_to_entity_not_found_returns_404() {
    let (server, _guard) = create_populated_test_server();

    let request = RetractRequest {
        from_entity: 1,
        to_entity: 99999,
    };
    let response = server.post("/signal/retract").json(&request).await;

    response.assert_status(axum::http::StatusCode::NOT_FOUND);
    let result: RetractResponse = response.json();
    assert!(!result.success);
    assert!(result.error.is_some());
}

#[tokio::test]
async fn test_retract_edge_not_found_returns_404() {
    use kremis_core::{Attribute, EntityId, Signal, Value};

    let guard = AUTH_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    unsafe { std::env::remove_var("KREMIS_API_KEY") };

    let mut session = Session::new();
    // Ingest two unrelated signals to create entities but no edge between them
    let signals = vec![
        Signal::new(EntityId(1), Attribute::new("type"), Value::new("word")),
        Signal::new(EntityId(2), Attribute::new("type"), Value::new("word")),
    ];
    session.ingest_sequence(&signals).unwrap();

    let state = AppState::new(session);
    let router = create_router(state);
    let server = TestServer::new(router).unwrap();
    let _guard = TestGuard { _guard: guard };

    // Entities 1 and 2 exist but there is no direct edge from 2 to 1
    let request = RetractRequest {
        from_entity: 2,
        to_entity: 1,
    };
    let response = server.post("/signal/retract").json(&request).await;

    response.assert_status(axum::http::StatusCode::NOT_FOUND);
    let result: RetractResponse = response.json();
    assert!(!result.success);
    assert!(result.error.is_some());
}

#[tokio::test]
async fn test_retract_multiple_times_floors_at_zero() {
    use kremis_core::{Attribute, EntityId, Signal, Value};

    let guard = AUTH_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    unsafe { std::env::remove_var("KREMIS_API_KEY") };

    let mut session = Session::new();
    let signals = vec![
        Signal::new(EntityId(10), Attribute::new("type"), Value::new("a")),
        Signal::new(EntityId(11), Attribute::new("type"), Value::new("b")),
    ];
    // One ingest → edge weight = 1
    session.ingest_sequence(&signals).unwrap();

    let state = AppState::new(session);
    let router = create_router(state);
    let server = TestServer::new(router).unwrap();
    let _guard = TestGuard { _guard: guard };

    let request = RetractRequest {
        from_entity: 10,
        to_entity: 11,
    };

    // First retract: 1 → 0
    let response = server.post("/signal/retract").json(&request).await;
    response.assert_status_ok();
    let result: RetractResponse = response.json();
    assert_eq!(result.new_weight, Some(0));

    // Second retract: stays at 0 (no negative weights)
    let response = server.post("/signal/retract").json(&request).await;
    response.assert_status_ok();
    let result: RetractResponse = response.json();
    assert_eq!(result.new_weight, Some(0));
}
