//! Integration tests for Kremis CLI commands.
//!
//! Uses tempfile for testing file-based operations.

// Allow unwrap and panic in tests - these are standard for test code
#![allow(clippy::unwrap_used, clippy::panic)]

use kremis::cli::{
    cmd_export, cmd_import, cmd_ingest, cmd_init, cmd_query, cmd_stage, cmd_status,
    load_or_create_session, save_session,
};
use kremis_core::{Attribute, EntityId, Session, Signal, Value};
use std::path::PathBuf;
use tempfile::TempDir;

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

/// Create a temporary directory for tests.
fn create_temp_dir() -> TempDir {
    tempfile::tempdir().expect("Failed to create temp dir")
}

/// Create a sample signals JSON file.
fn create_signals_json(dir: &TempDir) -> PathBuf {
    let path = dir.path().join("signals.json");
    let content = r#"[
        {"entity_id": 1, "attribute": "name", "value": "Alice"},
        {"entity_id": 2, "attribute": "name", "value": "Bob"},
        {"entity_id": 1, "attribute": "knows", "value": "Bob"}
    ]"#;
    std::fs::write(&path, content).unwrap();
    path
}

/// Create a sample signals text file.
fn create_signals_text(dir: &TempDir) -> PathBuf {
    let path = dir.path().join("signals.txt");
    let content = "1:name:Alice\n2:name:Bob\n1:knows:Bob";
    std::fs::write(&path, content).unwrap();
    path
}

// =============================================================================
// INIT COMMAND TESTS
// =============================================================================

#[test]
fn test_init_creates_file_database() {
    let temp = create_temp_dir();
    let db_path = temp.path().join("test.db");

    let result = cmd_init(&db_path, "file", false);
    assert!(result.is_ok());
    assert!(db_path.exists());
}

#[test]
fn test_init_creates_redb_database() {
    let temp = create_temp_dir();
    let db_path = temp.path().join("test.redb");

    let result = cmd_init(&db_path, "redb", false);
    assert!(result.is_ok());
    assert!(db_path.exists());
}

#[test]
fn test_init_fails_if_exists_without_force() {
    let temp = create_temp_dir();
    let db_path = temp.path().join("test.db");

    // First init
    cmd_init(&db_path, "file", false).unwrap();

    // Second init should fail
    let result = cmd_init(&db_path, "file", false);
    assert!(result.is_err());
}

#[test]
fn test_init_succeeds_with_force() {
    let temp = create_temp_dir();
    let db_path = temp.path().join("test.db");

    // First init
    cmd_init(&db_path, "file", false).unwrap();

    // Second init with force should succeed
    let result = cmd_init(&db_path, "file", true);
    assert!(result.is_ok());
}

// =============================================================================
// LOAD/SAVE SESSION TESTS
// =============================================================================

#[test]
fn test_load_nonexistent_creates_new() {
    let temp = create_temp_dir();
    let db_path = temp.path().join("nonexistent.db");

    let session = load_or_create_session(&db_path, "file");
    assert!(session.is_ok());
    let session = session.unwrap();
    assert_eq!(session.node_count().expect("node_count"), 0);
}

#[test]
fn test_save_and_load_session() {
    let temp = create_temp_dir();
    let db_path = temp.path().join("test.db");

    // Create and save session with data
    let mut session = Session::new();
    let signals = vec![
        Signal::new(EntityId(1), Attribute::new("name"), Value::new("Alice")),
        Signal::new(EntityId(2), Attribute::new("name"), Value::new("Bob")),
    ];
    session.ingest_sequence(&signals).unwrap();
    let node_count = session.node_count().expect("node_count");

    save_session(&session, &db_path).unwrap();

    // Load session back
    let loaded = load_or_create_session(&db_path, "file").unwrap();
    assert_eq!(loaded.node_count().expect("node_count"), node_count);
}

#[test]
fn test_load_redb_session() {
    let temp = create_temp_dir();
    let db_path = temp.path().join("test.redb");

    // Initialize redb
    cmd_init(&db_path, "redb", false).unwrap();

    // Load should work
    let session = load_or_create_session(&db_path, "redb");
    assert!(session.is_ok());
}

// =============================================================================
// STATUS COMMAND TESTS
// =============================================================================

#[test]
fn test_status_empty_graph() {
    let temp = create_temp_dir();
    let db_path = temp.path().join("test.db");
    cmd_init(&db_path, "file", false).unwrap();

    let result = cmd_status(&db_path, "file", false);
    assert!(result.is_ok());
}

#[test]
fn test_status_json_mode() {
    let temp = create_temp_dir();
    let db_path = temp.path().join("test.db");
    cmd_init(&db_path, "file", false).unwrap();

    let result = cmd_status(&db_path, "file", true);
    assert!(result.is_ok());
}

// =============================================================================
// STAGE COMMAND TESTS
// =============================================================================

#[test]
fn test_stage_empty_graph() {
    let temp = create_temp_dir();
    let db_path = temp.path().join("test.db");
    cmd_init(&db_path, "file", false).unwrap();

    let result = cmd_stage(&db_path, "file", false, false);
    assert!(result.is_ok());
}

#[test]
fn test_stage_json_mode() {
    let temp = create_temp_dir();
    let db_path = temp.path().join("test.db");
    cmd_init(&db_path, "file", false).unwrap();

    let result = cmd_stage(&db_path, "file", true, false);
    assert!(result.is_ok());
}

#[test]
fn test_stage_detailed_mode() {
    let temp = create_temp_dir();
    let db_path = temp.path().join("test.db");
    cmd_init(&db_path, "file", false).unwrap();

    let result = cmd_stage(&db_path, "file", false, true);
    assert!(result.is_ok());
}

// =============================================================================
// INGEST COMMAND TESTS
// =============================================================================

#[test]
fn test_ingest_json_format() {
    let temp = create_temp_dir();
    let db_path = temp.path().join("test.db");
    let signals_file = create_signals_json(&temp);

    cmd_init(&db_path, "file", false).unwrap();
    let result = cmd_ingest(
        &db_path,
        "file",
        false,
        Some(&signals_file),
        "json",
        false,
        false,
    );
    assert!(result.is_ok());

    // Verify data was ingested
    let session = load_or_create_session(&db_path, "file").unwrap();
    assert!(session.node_count().expect("node_count") > 0);
}

#[test]
fn test_ingest_text_format() {
    let temp = create_temp_dir();
    let db_path = temp.path().join("test.db");
    let signals_file = create_signals_text(&temp);

    cmd_init(&db_path, "file", false).unwrap();
    let result = cmd_ingest(
        &db_path,
        "file",
        false,
        Some(&signals_file),
        "text",
        false,
        false,
    );
    assert!(result.is_ok());

    // Verify data was ingested
    let session = load_or_create_session(&db_path, "file").unwrap();
    assert!(session.node_count().expect("node_count") > 0);
}

#[test]
fn test_ingest_invalid_format() {
    let temp = create_temp_dir();
    let db_path = temp.path().join("test.db");
    let signals_file = create_signals_json(&temp);

    cmd_init(&db_path, "file", false).unwrap();
    let result = cmd_ingest(
        &db_path,
        "file",
        false,
        Some(&signals_file),
        "unknown",
        false,
        false,
    );
    assert!(result.is_err());
}

#[test]
fn test_ingest_invalid_json() {
    let temp = create_temp_dir();
    let db_path = temp.path().join("test.db");
    let bad_file = temp.path().join("bad.json");
    std::fs::write(&bad_file, "not valid json").unwrap();

    cmd_init(&db_path, "file", false).unwrap();
    let result = cmd_ingest(
        &db_path,
        "file",
        false,
        Some(&bad_file),
        "json",
        false,
        false,
    );
    assert!(result.is_err());
}

#[test]
fn test_ingest_json_with_bom() {
    let temp = create_temp_dir();
    let db_path = temp.path().join("test.db");
    let bom_file = temp.path().join("bom.json");

    // UTF-8 BOM (EF BB BF) followed by valid JSON
    let mut bom_content = vec![0xEF, 0xBB, 0xBF];
    bom_content.extend_from_slice(br#"[{"entity_id":1,"attribute":"name","value":"Alice"}]"#);
    std::fs::write(&bom_file, bom_content).unwrap();

    cmd_init(&db_path, "file", false).unwrap();
    let result = cmd_ingest(
        &db_path,
        "file",
        false,
        Some(&bom_file),
        "json",
        false,
        false,
    );
    assert!(result.is_ok(), "ingest should accept JSON with UTF-8 BOM");

    let session = load_or_create_session(&db_path, "file").unwrap();
    assert!(session.node_count().expect("node_count") > 0);
}

#[test]
fn ingest_text_strict_fails_on_malformed_lines() {
    let temp = create_temp_dir();
    let db_path = temp.path().join("test.db");
    let bad_file = temp.path().join("mixed.txt");
    std::fs::write(&bad_file, "1:name:Alice\nbad line\n2:role:Engineer").unwrap();

    cmd_init(&db_path, "file", false).unwrap();
    let result = cmd_ingest(
        &db_path,
        "file",
        false,
        Some(&bad_file),
        "text",
        false,
        true,
    );
    assert!(result.is_err());
}

#[test]
fn ingest_text_strict_ok_on_all_valid_lines() {
    let temp = create_temp_dir();
    let db_path = temp.path().join("test.db");
    let signals_file = create_signals_text(&temp);

    cmd_init(&db_path, "file", false).unwrap();
    let result = cmd_ingest(
        &db_path,
        "file",
        false,
        Some(&signals_file),
        "text",
        false,
        true,
    );
    assert!(result.is_ok());
}

// =============================================================================
// QUERY COMMAND TESTS
// =============================================================================

#[test]
fn test_query_lookup_not_found() {
    let temp = create_temp_dir();
    let db_path = temp.path().join("test.db");

    cmd_init(&db_path, "file", false).unwrap();

    let result = cmd_query(
        &db_path,
        "file",
        false,
        "lookup",
        None,
        None,
        2,
        Some(999),
        None,
        None,
    );
    assert!(result.is_ok());
}

#[test]
fn test_query_lookup_found() {
    let temp = create_temp_dir();
    let db_path = temp.path().join("test.db");
    let signals_file = create_signals_json(&temp);

    cmd_init(&db_path, "file", false).unwrap();
    cmd_ingest(
        &db_path,
        "file",
        false,
        Some(&signals_file),
        "json",
        false,
        false,
    )
    .unwrap();

    let result = cmd_query(
        &db_path,
        "file",
        false,
        "lookup",
        None,
        None,
        2,
        Some(1),
        None,
        None,
    );
    assert!(result.is_ok());
}

#[test]
fn test_query_traverse() {
    let temp = create_temp_dir();
    let db_path = temp.path().join("test.db");
    let signals_file = create_signals_json(&temp);

    cmd_init(&db_path, "file", false).unwrap();
    cmd_ingest(
        &db_path,
        "file",
        false,
        Some(&signals_file),
        "json",
        false,
        false,
    )
    .unwrap();

    let result = cmd_query(
        &db_path,
        "file",
        false,
        "traverse",
        Some(1),
        None,
        3,
        None,
        None,
        None,
    );
    assert!(result.is_ok());
}

#[test]
fn test_query_traverse_filtered() {
    let temp = create_temp_dir();
    let db_path = temp.path().join("test.db");
    let signals_file = create_signals_json(&temp);

    cmd_init(&db_path, "file", false).unwrap();
    cmd_ingest(
        &db_path,
        "file",
        false,
        Some(&signals_file),
        "json",
        false,
        false,
    )
    .unwrap();

    let result = cmd_query(
        &db_path,
        "file",
        false,
        "traverse",
        Some(1),
        None,
        3,
        None,
        None,
        Some(10),
    );
    assert!(result.is_ok());
}

#[test]
fn test_query_path() {
    let temp = create_temp_dir();
    let db_path = temp.path().join("test.db");
    let signals_file = create_signals_json(&temp);

    cmd_init(&db_path, "file", false).unwrap();
    cmd_ingest(
        &db_path,
        "file",
        false,
        Some(&signals_file),
        "json",
        false,
        false,
    )
    .unwrap();

    let result = cmd_query(
        &db_path,
        "file",
        false,
        "path",
        Some(1),
        Some(2),
        2,
        None,
        None,
        None,
    );
    assert!(result.is_ok());
}

#[test]
fn test_query_intersect() {
    let temp = create_temp_dir();
    let db_path = temp.path().join("test.db");
    let signals_file = create_signals_json(&temp);

    cmd_init(&db_path, "file", false).unwrap();
    cmd_ingest(
        &db_path,
        "file",
        false,
        Some(&signals_file),
        "json",
        false,
        false,
    )
    .unwrap();

    let result = cmd_query(
        &db_path,
        "file",
        false,
        "intersect",
        None,
        None,
        2,
        None,
        Some("1,2".to_string()),
        None,
    );
    assert!(result.is_ok());
}

#[test]
fn test_query_unknown_type() {
    let temp = create_temp_dir();
    let db_path = temp.path().join("test.db");

    cmd_init(&db_path, "file", false).unwrap();

    let result = cmd_query(
        &db_path,
        "file",
        false,
        "unknown_query_type",
        None,
        None,
        2,
        None,
        None,
        None,
    );
    assert!(result.is_err());
}

// =============================================================================
// QUERY JSON MODE TESTS
// =============================================================================

#[test]
fn test_query_lookup_json_mode() {
    let temp = create_temp_dir();
    let db_path = temp.path().join("test.db");
    let signals_file = create_signals_json(&temp);

    cmd_init(&db_path, "file", false).unwrap();
    cmd_ingest(
        &db_path,
        "file",
        false,
        Some(&signals_file),
        "json",
        false,
        false,
    )
    .unwrap();

    let result = cmd_query(
        &db_path,
        "file",
        true,
        "lookup",
        None,
        None,
        2,
        Some(1),
        None,
        None,
    );
    assert!(result.is_ok());
}

#[test]
fn test_query_traverse_json_mode() {
    let temp = create_temp_dir();
    let db_path = temp.path().join("test.db");
    let signals_file = create_signals_json(&temp);

    cmd_init(&db_path, "file", false).unwrap();
    cmd_ingest(
        &db_path,
        "file",
        false,
        Some(&signals_file),
        "json",
        false,
        false,
    )
    .unwrap();

    let result = cmd_query(
        &db_path,
        "file",
        true,
        "traverse",
        Some(1),
        None,
        3,
        None,
        None,
        None,
    );
    assert!(result.is_ok());
}

#[test]
fn test_query_intersect_json_mode() {
    let temp = create_temp_dir();
    let db_path = temp.path().join("test.db");
    let signals_file = create_signals_json(&temp);

    cmd_init(&db_path, "file", false).unwrap();
    cmd_ingest(
        &db_path,
        "file",
        false,
        Some(&signals_file),
        "json",
        false,
        false,
    )
    .unwrap();

    let result = cmd_query(
        &db_path,
        "file",
        true,
        "intersect",
        None,
        None,
        2,
        None,
        Some("1,2".to_string()),
        None,
    );
    assert!(result.is_ok());
}

#[test]
fn test_query_properties_json_mode() {
    let temp = create_temp_dir();
    let db_path = temp.path().join("test.db");
    let signals_file = create_signals_json(&temp);

    cmd_init(&db_path, "file", false).unwrap();
    cmd_ingest(
        &db_path,
        "file",
        false,
        Some(&signals_file),
        "json",
        false,
        false,
    )
    .unwrap();

    let result = cmd_query(
        &db_path,
        "file",
        true,
        "properties",
        Some(1),
        None,
        2,
        None,
        None,
        None,
    );
    assert!(result.is_ok());
}

// =============================================================================
// EXPORT COMMAND TESTS
// =============================================================================

#[test]
fn test_export_canonical_format() {
    let temp = create_temp_dir();
    let db_path = temp.path().join("test.db");
    let signals_file = create_signals_json(&temp);
    let output_path = temp.path().join("export.bin");

    cmd_init(&db_path, "file", false).unwrap();
    cmd_ingest(
        &db_path,
        "file",
        false,
        Some(&signals_file),
        "json",
        false,
        false,
    )
    .unwrap();

    let result = cmd_export(&db_path, "file", &output_path, "canonical");
    assert!(result.is_ok());
    assert!(output_path.exists());
}

#[test]
fn test_export_json_format() {
    let temp = create_temp_dir();
    let db_path = temp.path().join("test.db");
    let signals_file = create_signals_json(&temp);
    let output_path = temp.path().join("export.json");

    cmd_init(&db_path, "file", false).unwrap();
    cmd_ingest(
        &db_path,
        "file",
        false,
        Some(&signals_file),
        "json",
        false,
        false,
    )
    .unwrap();

    let result = cmd_export(&db_path, "file", &output_path, "json");
    assert!(result.is_ok());
    assert!(output_path.exists());

    // Verify it's valid JSON
    let content = std::fs::read_to_string(&output_path).unwrap();
    let _: serde_json::Value = serde_json::from_str(&content).unwrap();
}

#[test]
fn test_export_unknown_format() {
    let temp = create_temp_dir();
    let db_path = temp.path().join("test.db");
    let output_path = temp.path().join("export.bin");

    cmd_init(&db_path, "file", false).unwrap();

    let result = cmd_export(&db_path, "file", &output_path, "unknown");
    assert!(result.is_err());
}

// =============================================================================
// IMPORT COMMAND TESTS
// =============================================================================

#[test]
fn test_import_canonical() {
    let temp = create_temp_dir();
    let db_path = temp.path().join("test.db");
    let signals_file = create_signals_json(&temp);
    let export_path = temp.path().join("export.bin");
    let import_db_path = temp.path().join("imported.db");

    // Create and export
    cmd_init(&db_path, "file", false).unwrap();
    cmd_ingest(
        &db_path,
        "file",
        false,
        Some(&signals_file),
        "json",
        false,
        false,
    )
    .unwrap();
    cmd_export(&db_path, "file", &export_path, "canonical").unwrap();

    // Import
    let result = cmd_import(&import_db_path, "file", &export_path);
    assert!(result.is_ok());

    // Verify imported data matches
    let original = load_or_create_session(&db_path, "file").unwrap();
    let imported = load_or_create_session(&import_db_path, "file").unwrap();
    assert_eq!(
        original.node_count().expect("node_count"),
        imported.node_count().expect("node_count")
    );
    assert_eq!(
        original.edge_count().expect("edge_count"),
        imported.edge_count().expect("edge_count")
    );
}

#[test]
fn test_import_to_redb_fails() {
    let temp = create_temp_dir();
    let export_path = temp.path().join("export.bin");
    let import_db_path = temp.path().join("imported.redb");

    // Create a minimal canonical export
    let session = Session::new();
    let graph = session
        .graph_opt()
        .expect("in-memory session should have graph");
    let data = kremis_core::export::export_canonical(graph).unwrap();
    std::fs::write(&export_path, &data).unwrap();

    // Import to redb should fail (not supported)
    let result = cmd_import(&import_db_path, "redb", &export_path);
    assert!(result.is_err());
}

// =============================================================================
// ROUNDTRIP TESTS
// =============================================================================

#[test]
fn test_export_import_roundtrip_preserves_data() {
    let temp = create_temp_dir();
    let db1_path = temp.path().join("db1.db");
    let db2_path = temp.path().join("db2.db");
    let export_path = temp.path().join("export.bin");

    // Create session with data
    let mut session = Session::new();
    let signals = vec![
        Signal::new(EntityId(1), Attribute::new("name"), Value::new("Alice")),
        Signal::new(EntityId(2), Attribute::new("name"), Value::new("Bob")),
        Signal::new(EntityId(3), Attribute::new("name"), Value::new("Carol")),
        Signal::new(EntityId(1), Attribute::new("knows"), Value::new("Bob")),
        Signal::new(EntityId(2), Attribute::new("knows"), Value::new("Carol")),
    ];
    session.ingest_sequence(&signals).unwrap();
    save_session(&session, &db1_path).unwrap();

    let original_node_count = session.node_count().expect("node_count");
    let original_edge_count = session.edge_count().expect("edge_count");

    // Export
    cmd_export(&db1_path, "file", &export_path, "canonical").unwrap();

    // Import to new database
    cmd_import(&db2_path, "file", &export_path).unwrap();

    // Verify
    let imported = load_or_create_session(&db2_path, "file").unwrap();
    assert_eq!(
        imported.node_count().expect("node_count"),
        original_node_count
    );
    assert_eq!(
        imported.edge_count().expect("edge_count"),
        original_edge_count
    );
}

#[test]
fn test_deterministic_export() {
    let temp = create_temp_dir();
    let db_path = temp.path().join("test.db");
    let export1_path = temp.path().join("export1.bin");
    let export2_path = temp.path().join("export2.bin");

    // Create session with deterministic data
    let mut session = Session::new();
    let signals = vec![
        Signal::new(EntityId(1), Attribute::new("a"), Value::new("1")),
        Signal::new(EntityId(2), Attribute::new("b"), Value::new("2")),
    ];
    session.ingest_sequence(&signals).unwrap();
    save_session(&session, &db_path).unwrap();

    // Export twice
    cmd_export(&db_path, "file", &export1_path, "canonical").unwrap();
    cmd_export(&db_path, "file", &export2_path, "canonical").unwrap();

    // Both exports should be identical (deterministic)
    let data1 = std::fs::read(&export1_path).unwrap();
    let data2 = std::fs::read(&export2_path).unwrap();
    assert_eq!(data1, data2, "Canonical export should be deterministic");
}
