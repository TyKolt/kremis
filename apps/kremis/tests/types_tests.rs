//! Unit tests for API types serialization/deserialization.

// Allow unwrap and panic in tests - these are standard for test code
#![allow(clippy::unwrap_used, clippy::panic)]

use kremis::api::{
    EdgeJson, ExportResponse, HealthResponse, IngestRequest, IngestResponse, QueryRequest,
    QueryResponse, StageResponse, StatusResponse,
};

// =============================================================================
// HEALTH RESPONSE TESTS
// =============================================================================

#[test]
fn test_health_response_default() {
    let health = HealthResponse::default();
    assert_eq!(health.status, "ok");
    assert!(!health.version.is_empty());
}

#[test]
fn test_health_response_serialization() {
    let health = HealthResponse {
        status: "ok".to_string(),
        version: "0.10.0".to_string(),
    };

    let json = serde_json::to_string(&health).unwrap();
    assert!(json.contains("\"status\":\"ok\""));
    assert!(json.contains("\"version\":\"0.10.0\""));
}

#[test]
fn test_health_response_deserialization() {
    let json = r#"{"status":"healthy","version":"1.0.0"}"#;
    let health: HealthResponse = serde_json::from_str(json).unwrap();

    assert_eq!(health.status, "healthy");
    assert_eq!(health.version, "1.0.0");
}

// =============================================================================
// STATUS RESPONSE TESTS
// =============================================================================

#[test]
fn test_status_response_serialization() {
    let status = StatusResponse {
        node_count: 100,
        edge_count: 250,
        stable_edges: 50,
        density_millionths: 250000,
    };

    let json = serde_json::to_string(&status).unwrap();
    assert!(json.contains("\"node_count\":100"));
    assert!(json.contains("\"edge_count\":250"));
    assert!(json.contains("\"stable_edges\":50"));
    assert!(json.contains("\"density_millionths\":250000"));
}

#[test]
fn test_status_response_deserialization() {
    let json = r#"{"node_count":10,"edge_count":15,"stable_edges":5,"density_millionths":150000}"#;
    let status: StatusResponse = serde_json::from_str(json).unwrap();

    assert_eq!(status.node_count, 10);
    assert_eq!(status.edge_count, 15);
    assert_eq!(status.stable_edges, 5);
    assert_eq!(status.density_millionths, 150000);
}

// =============================================================================
// STAGE RESPONSE TESTS
// =============================================================================

#[test]
fn test_stage_response_serialization() {
    let stage = StageResponse {
        stage: "S1".to_string(),
        name: "Pattern Crystallization".to_string(),
        progress_percent: 45,
        stable_edges_needed: 100,
        stable_edges_current: 45,
    };

    let json = serde_json::to_string(&stage).unwrap();
    assert!(json.contains("\"stage\":\"S1\""));
    assert!(json.contains("\"name\":\"Pattern Crystallization\""));
    assert!(json.contains("\"progress_percent\":45"));
}

#[test]
fn test_stage_response_deserialization() {
    let json = r#"{"stage":"S2","name":"Test Stage","progress_percent":75,"stable_edges_needed":200,"stable_edges_current":150}"#;
    let stage: StageResponse = serde_json::from_str(json).unwrap();

    assert_eq!(stage.stage, "S2");
    assert_eq!(stage.name, "Test Stage");
    assert_eq!(stage.progress_percent, 75);
    assert_eq!(stage.stable_edges_needed, 200);
    assert_eq!(stage.stable_edges_current, 150);
}

// =============================================================================
// INGEST REQUEST TESTS
// =============================================================================

#[test]
fn test_ingest_request_deserialization() {
    let json = r#"{"entity_id":1,"attribute":"name","value":"Alice"}"#;
    let request: IngestRequest = serde_json::from_str(json).unwrap();

    assert_eq!(request.entity_id, 1);
    assert_eq!(request.attribute, "name");
    assert_eq!(request.value, "Alice");
}

#[test]
fn test_ingest_request_to_signal_valid() {
    let request = IngestRequest {
        entity_id: 1,
        attribute: "name".to_string(),
        value: "Alice".to_string(),
    };

    let result = request.to_signal();
    assert!(result.is_ok());
}

#[test]
fn test_ingest_request_to_signal_empty_attribute() {
    let request = IngestRequest {
        entity_id: 1,
        attribute: "".to_string(),
        value: "Alice".to_string(),
    };

    let result = request.to_signal();
    assert!(result.is_err());
}

#[test]
fn test_ingest_request_to_signal_empty_value() {
    let request = IngestRequest {
        entity_id: 1,
        attribute: "name".to_string(),
        value: "".to_string(),
    };

    let result = request.to_signal();
    assert!(result.is_err());
}

// =============================================================================
// INGEST RESPONSE TESTS
// =============================================================================

#[test]
fn test_ingest_response_success() {
    use kremis_core::NodeId;

    let response = IngestResponse::success(NodeId(42));

    assert!(response.success);
    assert_eq!(response.node_id, Some(42));
    assert!(response.error.is_none());
}

#[test]
fn test_ingest_response_error() {
    let response = IngestResponse::error("Test error");

    assert!(!response.success);
    assert!(response.node_id.is_none());
    assert_eq!(response.error, Some("Test error".to_string()));
}

#[test]
fn test_ingest_response_serialization() {
    use kremis_core::NodeId;

    let response = IngestResponse::success(NodeId(42));
    let json = serde_json::to_string(&response).unwrap();

    assert!(json.contains("\"success\":true"));
    assert!(json.contains("\"node_id\":42"));
}

// =============================================================================
// QUERY REQUEST TESTS
// =============================================================================

#[test]
fn test_query_request_lookup_serialization() {
    let request = QueryRequest::Lookup { entity_id: 42 };
    let json = serde_json::to_string(&request).unwrap();

    assert!(json.contains("\"type\":\"lookup\""));
    assert!(json.contains("\"entity_id\":42"));
}

#[test]
fn test_query_request_traverse_serialization() {
    let request = QueryRequest::Traverse {
        node_id: 1,
        depth: 3,
    };
    let json = serde_json::to_string(&request).unwrap();

    assert!(json.contains("\"type\":\"traverse\""));
    assert!(json.contains("\"node_id\":1"));
    assert!(json.contains("\"depth\":3"));
}

#[test]
fn test_query_request_traverse_filtered_serialization() {
    let request = QueryRequest::TraverseFiltered {
        node_id: 1,
        depth: 2,
        min_weight: 50,
    };
    let json = serde_json::to_string(&request).unwrap();

    assert!(json.contains("\"type\":\"traverse_filtered\""));
    assert!(json.contains("\"min_weight\":50"));
}

#[test]
fn test_query_request_strongest_path_serialization() {
    let request = QueryRequest::StrongestPath { start: 1, end: 10 };
    let json = serde_json::to_string(&request).unwrap();

    assert!(json.contains("\"type\":\"strongest_path\""));
    assert!(json.contains("\"start\":1"));
    assert!(json.contains("\"end\":10"));
}

#[test]
fn test_query_request_intersect_serialization() {
    let request = QueryRequest::Intersect {
        nodes: vec![1, 2, 3],
    };
    let json = serde_json::to_string(&request).unwrap();

    assert!(json.contains("\"type\":\"intersect\""));
    assert!(json.contains("[1,2,3]"));
}

#[test]
fn test_query_request_related_serialization() {
    let request = QueryRequest::Related {
        node_id: 5,
        depth: 2,
    };
    let json = serde_json::to_string(&request).unwrap();

    assert!(json.contains("\"type\":\"related\""));
}

#[test]
fn test_query_request_deserialization() {
    let json = r#"{"type":"lookup","entity_id":42}"#;
    let request: QueryRequest = serde_json::from_str(json).unwrap();

    match request {
        QueryRequest::Lookup { entity_id } => assert_eq!(entity_id, 42),
        _ => panic!("Expected Lookup variant"),
    }
}

// =============================================================================
// QUERY RESPONSE TESTS
// =============================================================================

#[test]
fn test_query_response_not_found() {
    let response = QueryResponse::not_found();

    assert!(response.success);
    assert!(!response.found);
    assert!(response.path.is_empty());
    assert!(response.edges.is_empty());
    assert!(response.error.is_none());
}

#[test]
fn test_query_response_with_path() {
    use kremis_core::NodeId;

    let path = vec![NodeId(1), NodeId(2), NodeId(3)];
    let response = QueryResponse::with_path(path);

    assert!(response.success);
    assert!(response.found);
    assert_eq!(response.path, vec![1, 2, 3]);
}

#[test]
fn test_query_response_with_empty_path() {
    let response = QueryResponse::with_path(vec![]);

    assert!(response.success);
    assert!(!response.found);
    assert!(response.path.is_empty());
}

#[test]
fn test_query_response_error() {
    let response = QueryResponse::error("Test error");

    assert!(!response.success);
    assert!(!response.found);
    assert_eq!(response.error, Some("Test error".to_string()));
}

#[test]
fn test_query_response_serialization() {
    use kremis_core::NodeId;

    let path = vec![NodeId(1), NodeId(2)];
    let response = QueryResponse::with_path(path);
    let json = serde_json::to_string(&response).unwrap();

    assert!(json.contains("\"success\":true"));
    assert!(json.contains("\"found\":true"));
    assert!(json.contains("[1,2]"));
}

// =============================================================================
// GROUNDING FIELD TESTS
// =============================================================================

#[test]
fn test_query_response_not_found_grounding_is_unknown() {
    let response = QueryResponse::not_found();
    assert_eq!(response.grounding, "unknown");
}

#[test]
fn test_query_response_error_grounding_is_unknown() {
    let response = QueryResponse::error("something went wrong");
    assert_eq!(response.grounding, "unknown");
}

#[test]
fn test_query_response_with_path_grounding_default_is_unknown() {
    use kremis_core::NodeId;
    let response = QueryResponse::with_path(vec![NodeId(1), NodeId(2)]);
    // Default before handler override
    assert_eq!(response.grounding, "unknown");
}

#[test]
fn test_query_response_grounding_serialized() {
    let mut response = QueryResponse::not_found();
    response.grounding = "fact".to_string();
    let json = serde_json::to_string(&response).unwrap();
    assert!(json.contains("\"grounding\":\"fact\""));
}

#[test]
fn test_query_response_grounding_default_on_deserialize() {
    // Old response without grounding field should deserialize with "unknown"
    let json = r#"{"success":true,"found":false,"path":[],"edges":[],"error":null}"#;
    let response: QueryResponse = serde_json::from_str(json).unwrap();
    assert_eq!(response.grounding, "unknown");
}

// =============================================================================
// EDGE JSON TESTS
// =============================================================================

#[test]
fn test_edge_json_serialization() {
    let edge = EdgeJson {
        from: 1,
        to: 2,
        weight: 100,
    };

    let json = serde_json::to_string(&edge).unwrap();
    assert!(json.contains("\"from\":1"));
    assert!(json.contains("\"to\":2"));
    assert!(json.contains("\"weight\":100"));
}

#[test]
fn test_edge_json_deserialization() {
    let json = r#"{"from":5,"to":10,"weight":50}"#;
    let edge: EdgeJson = serde_json::from_str(json).unwrap();

    assert_eq!(edge.from, 5);
    assert_eq!(edge.to, 10);
    assert_eq!(edge.weight, 50);
}

// =============================================================================
// EXPORT RESPONSE TESTS
// =============================================================================

#[test]
fn test_export_response_success() {
    let data = vec![1, 2, 3, 4, 5];
    let response = ExportResponse::success(data, 12345);

    assert!(response.success);
    assert!(response.data.is_some());
    assert_eq!(response.checksum, Some(12345));
    assert!(response.error.is_none());
}

#[test]
fn test_export_response_error() {
    let response = ExportResponse::error("Export failed");

    assert!(!response.success);
    assert!(response.data.is_none());
    assert!(response.checksum.is_none());
    assert_eq!(response.error, Some("Export failed".to_string()));
}

#[test]
fn test_export_response_data_is_base64() {
    let data = vec![0, 1, 2, 255, 254, 253];
    let response = ExportResponse::success(data.clone(), 0);

    let base64_data = response.data.unwrap();

    // Decode and verify
    let decoded = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &base64_data)
        .expect("Should be valid base64");
    assert_eq!(decoded, data);
}

// =============================================================================
// ROUNDTRIP TESTS
// =============================================================================

#[test]
fn test_ingest_request_roundtrip() {
    let original = IngestRequest {
        entity_id: 42,
        attribute: "test_attr".to_string(),
        value: "test_value".to_string(),
    };

    let json = serde_json::to_string(&original).unwrap();
    let parsed: IngestRequest = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed.entity_id, original.entity_id);
    assert_eq!(parsed.attribute, original.attribute);
    assert_eq!(parsed.value, original.value);
}

#[test]
fn test_query_request_all_variants_roundtrip() {
    let variants = vec![
        QueryRequest::Lookup { entity_id: 1 },
        QueryRequest::Traverse {
            node_id: 2,
            depth: 3,
        },
        QueryRequest::TraverseFiltered {
            node_id: 4,
            depth: 5,
            min_weight: 10,
        },
        QueryRequest::StrongestPath { start: 6, end: 7 },
        QueryRequest::Intersect {
            nodes: vec![8, 9, 10],
        },
        QueryRequest::Related {
            node_id: 11,
            depth: 2,
        },
    ];

    for original in variants {
        let json = serde_json::to_string(&original).unwrap();
        let parsed: QueryRequest = serde_json::from_str(&json).unwrap();

        // Compare JSON representations (since QueryRequest doesn't impl PartialEq)
        let original_json = serde_json::to_value(&original).unwrap();
        let parsed_json = serde_json::to_value(&parsed).unwrap();
        assert_eq!(original_json, parsed_json);
    }
}

// =============================================================================
// DIAGNOSTIC FIELD TESTS
// =============================================================================

#[test]
fn test_query_response_diagnostic_is_none_by_default() {
    let response = QueryResponse::not_found();
    assert!(response.diagnostic.is_none());

    let response2 = QueryResponse::error("oops");
    assert!(response2.diagnostic.is_none());
}

#[test]
fn test_query_response_diagnostic_serialized_when_set() {
    let response = QueryResponse::not_found().with_diagnostic("node_not_found");
    assert_eq!(response.diagnostic, Some("node_not_found".to_string()));

    let json = serde_json::to_string(&response).unwrap();
    assert!(json.contains("\"diagnostic\":\"node_not_found\""));
}

#[test]
fn test_query_response_no_diagnostic_not_in_json() {
    let response = QueryResponse::not_found();
    let json = serde_json::to_string(&response).unwrap();
    // diagnostic field must be absent from JSON when None
    assert!(!json.contains("diagnostic"));
}

#[test]
fn test_query_response_diagnostic_deserializes_old_format() {
    // Old JSON without diagnostic field should deserialize with diagnostic = None
    let json =
        r#"{"success":true,"found":false,"path":[],"edges":[],"grounding":"unknown","error":null}"#;
    let response: QueryResponse = serde_json::from_str(json).unwrap();
    assert!(response.diagnostic.is_none());
}
