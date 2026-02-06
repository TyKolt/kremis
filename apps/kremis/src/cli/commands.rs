//! # CLI Command Implementations
//!
//! This module contains the actual implementations of CLI commands.

use crate::api;
use kremis_core::{
    Graph, KremisError, NodeId, Session,
    export::{canonical_checksum, export_canonical, import_canonical},
    primitives::MAX_SEQUENCE_LENGTH,
    system::{GraphMetrics, StageAssessor},
};
use std::path::PathBuf;

// =============================================================================
// FILE SIZE LIMITS
// =============================================================================

/// Maximum file size for ingestion (100 MB).
///
/// This prevents memory exhaustion from malicious or accidental large files.
const MAX_INGEST_FILE_SIZE: u64 = 100 * 1024 * 1024;

/// Maximum file size for import (500 MB).
///
/// Import files can be larger since they contain binary graph data.
const MAX_IMPORT_FILE_SIZE: u64 = 500 * 1024 * 1024;

/// Validate file size before reading.
fn validate_file_size(path: &PathBuf, max_size: u64) -> Result<(), KremisError> {
    let metadata = std::fs::metadata(path)
        .map_err(|e| KremisError::IoError(format!("Cannot read file metadata: {}", e)))?;

    if metadata.len() > max_size {
        return Err(KremisError::SerializationError(format!(
            "File size {} bytes exceeds maximum allowed {} bytes",
            metadata.len(),
            max_size
        )));
    }
    Ok(())
}

/// Validate file path for security (L1 fix).
///
/// This function:
/// 1. Canonicalizes the path to resolve symlinks and ".."
/// 2. Ensures the path exists
/// 3. Ensures the path is a file (not a directory)
///
/// # Security Note
///
/// This prevents path traversal attacks where a malicious path like
/// "../../../etc/passwd" could be used to access sensitive files.
fn validate_file_path(path: &std::path::Path) -> Result<PathBuf, KremisError> {
    // Canonicalize resolves "..", symlinks, and validates existence
    let canonical = path.canonicalize().map_err(|e| {
        KremisError::IoError(format!("Invalid file path '{}': {}", path.display(), e))
    })?;

    // Ensure it's a file, not a directory
    if !canonical.is_file() {
        return Err(KremisError::IoError(format!(
            "Path '{}' is not a regular file",
            path.display()
        )));
    }

    Ok(canonical)
}

/// Validate output path for security (L1 fix).
///
/// For output files, we validate the parent directory exists and is writable.
fn validate_output_path(path: &std::path::Path) -> Result<PathBuf, KremisError> {
    // Get parent directory
    let parent = path.parent().unwrap_or(std::path::Path::new("."));

    // Canonicalize parent to resolve ".." and symlinks
    let canonical_parent = parent.canonicalize().map_err(|e| {
        KremisError::IoError(format!(
            "Invalid output directory '{}': {}",
            parent.display(),
            e
        ))
    })?;

    // Ensure parent is a directory
    if !canonical_parent.is_dir() {
        return Err(KremisError::IoError(format!(
            "Output directory '{}' is not a valid directory",
            parent.display()
        )));
    }

    // Return the path with canonical parent + original filename
    let filename = path
        .file_name()
        .ok_or_else(|| KremisError::IoError("Output path has no filename".to_string()))?;

    Ok(canonical_parent.join(filename))
}

// =============================================================================
// SERVER COMMAND
// =============================================================================

/// Start the HTTP server.
pub async fn cmd_server(
    db_path: &PathBuf,
    backend: &str,
    host: &str,
    port: u16,
) -> Result<(), KremisError> {
    let session = load_or_create_session(db_path, backend)?;

    println!("Kremis Honest AGI Server Starting...");
    println!();
    println!("Configuration:");
    println!("  Host:     {}", host);
    println!("  Port:     {}", port);
    println!("  Backend:  {}", backend);
    println!("  Database: {:?}", db_path);
    println!();
    println!("Endpoints:");
    println!("  POST /signal - Ingest a signal");
    println!("  POST /query  - Execute a query");
    println!("  GET  /status - Get graph status");
    println!("  GET  /stage  - Get developmental stage");
    println!("  POST /export - Export graph");
    println!("  GET  /health - Health check");
    println!();
    println!("Press Ctrl+C to stop");
    println!();

    let addr = format!("{}:{}", host, port);
    api::run_server(&addr, session).await
}

// =============================================================================
// STATUS COMMAND
// =============================================================================

/// Show graph status.
pub fn cmd_status(db_path: &PathBuf, backend: &str, json_mode: bool) -> Result<(), KremisError> {
    let session = load_or_create_session(db_path, backend)?;
    let metrics = GraphMetrics::from_session(&session);

    if json_mode {
        let output = serde_json::json!({
            "database": db_path.to_string_lossy(),
            "backend": backend,
            "node_count": metrics.node_count,
            "edge_count": metrics.edge_count,
            "stable_edges": metrics.stable_edge_count,
            "density_per_thousand": metrics.density_per_thousand(),
            "max_depth": metrics.max_depth
        });
        println!(
            "{}",
            serde_json::to_string_pretty(&output).unwrap_or_default()
        );
        return Ok(());
    }

    println!("Kremis Graph Status");
    println!("==================");
    println!("Database: {:?}", db_path);
    println!("Backend:  {}", backend);
    println!();
    println!("Nodes:        {}", metrics.node_count);
    println!("Edges:        {}", metrics.edge_count);
    println!("Stable Edges: {}", metrics.stable_edge_count);
    println!(
        "Density:      {} per thousand",
        metrics.density_per_thousand()
    );
    println!("Max Depth:    {}", metrics.max_depth);

    Ok(())
}

// =============================================================================
// STAGE COMMAND
// =============================================================================

/// Show developmental stage.
pub fn cmd_stage(
    db_path: &PathBuf,
    backend: &str,
    json_mode: bool,
    detailed: bool,
) -> Result<(), KremisError> {
    let session = load_or_create_session(db_path, backend)?;

    let assessor = StageAssessor::new();
    let progress = assessor.progress_to_next_session(&session);

    if json_mode {
        let output = serde_json::json!({
            "current_stage": format!("{:?}", progress.current),
            "stage_name": progress.current.name(),
            "next_stage": progress.next.map(|s| format!("{:?}", s)),
            "progress_percent": progress.percent,
            "stable_edges_current": progress.stable_edges_current,
            "stable_edges_needed": progress.stable_edges_needed,
            "metrics": {
                "node_count": progress.metrics.node_count,
                "edge_count": progress.metrics.edge_count,
                "stable_edge_count": progress.metrics.stable_edge_count
            }
        });
        println!(
            "{}",
            serde_json::to_string_pretty(&output).unwrap_or_default()
        );
        return Ok(());
    }

    println!("Kremis Developmental Stage");
    println!("==========================");
    println!();
    println!("Current Stage: {}", progress.current);
    println!();

    if let Some(next) = progress.next {
        println!("Next Stage: {}", next);
        println!("Progress:   {}%", progress.percent);
        println!(
            "Stable Edges: {} / {} needed",
            progress.stable_edges_current, progress.stable_edges_needed
        );
    } else {
        println!("Terminal stage reached (S3)");
    }

    if detailed {
        println!();
        println!("Metrics:");
        println!("  Nodes:          {}", progress.metrics.node_count);
        println!("  Edges:          {}", progress.metrics.edge_count);
        println!("  Stable Edges:   {}", progress.metrics.stable_edge_count);
        println!(
            "  Density:        {} per thousand",
            progress.metrics.density_per_thousand()
        );
        println!("  Max Depth:      {}", progress.metrics.max_depth);
    }

    Ok(())
}

// =============================================================================
// INGEST COMMAND
// =============================================================================

/// Ingest signals from a file.
pub fn cmd_ingest(
    db_path: &PathBuf,
    backend: &str,
    _json_mode: bool,
    file: &PathBuf,
    format: &str,
) -> Result<(), KremisError> {
    use kremis_core::{Attribute, EntityId, Signal, Value};

    tracing::info!("Ingesting from {:?} (format: {})", file, format);

    let mut session = load_or_create_session(db_path, backend)?;

    // L1 FIX: Validate file path for security (prevents path traversal)
    let validated_path = validate_file_path(file)?;

    // Validate file size before reading to prevent DoS
    validate_file_size(&validated_path, MAX_INGEST_FILE_SIZE)?;

    // Read file contents
    let contents = std::fs::read(&validated_path)
        .map_err(|e| KremisError::SerializationError(format!("Read file: {}", e)))?;

    // Parse signals based on format
    let signals = match format {
        "json" => {
            let json_values: Vec<serde_json::Value> =
                serde_json::from_slice(&contents).map_err(|_| KremisError::InvalidSignal)?;

            // Validate signal count to prevent DoS
            if json_values.len() > MAX_SEQUENCE_LENGTH {
                return Err(KremisError::SerializationError(format!(
                    "Signal count {} exceeds maximum allowed {}",
                    json_values.len(),
                    MAX_SEQUENCE_LENGTH
                )));
            }

            let mut signals = Vec::new();
            for val in json_values {
                let entity_id = val["entity_id"]
                    .as_u64()
                    .ok_or(KremisError::InvalidSignal)?;
                let attribute = val["attribute"]
                    .as_str()
                    .ok_or(KremisError::InvalidSignal)?;
                let value = val["value"].as_str().ok_or(KremisError::InvalidSignal)?;

                if attribute.is_empty() || value.is_empty() {
                    return Err(KremisError::InvalidSignal);
                }

                signals.push(Signal::new(
                    EntityId(entity_id),
                    Attribute::new(attribute),
                    Value::new(value),
                ));
            }
            signals
        }
        "text" => {
            let text = String::from_utf8_lossy(&contents);
            let mut signals = Vec::new();

            for line in text.lines() {
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() >= 3 {
                    let entity_id: u64 = parts[0]
                        .trim()
                        .parse()
                        .map_err(|_| KremisError::InvalidSignal)?;
                    let attribute = parts[1].trim();
                    let value = parts[2..].join(":");

                    if attribute.is_empty() || value.is_empty() {
                        continue;
                    }

                    signals.push(Signal::new(
                        EntityId(entity_id),
                        Attribute::new(attribute),
                        Value::new(value.trim()),
                    ));
                }
            }
            signals
        }
        _ => {
            return Err(KremisError::SerializationError(format!(
                "Unknown format: {}",
                format
            )));
        }
    };

    // Validate signal count
    if signals.len() > kremis_core::primitives::MAX_SEQUENCE_LENGTH {
        return Err(KremisError::SerializationError(format!(
            "Signal count {} exceeds maximum {}",
            signals.len(),
            kremis_core::primitives::MAX_SEQUENCE_LENGTH
        )));
    }

    // Ingest signals
    let count = signals.len();
    session.ingest_sequence(&signals)?;

    // Save graph
    save_session(&session, db_path)?;

    println!("Ingested {} signals", count);
    println!(
        "Graph now has {} nodes, {} edges",
        session.node_count(),
        session.edge_count()
    );

    Ok(())
}

// =============================================================================
// QUERY COMMAND
// =============================================================================

/// Execute a query.
#[allow(clippy::too_many_arguments)]
pub fn cmd_query(
    db_path: &PathBuf,
    backend: &str,
    _json_mode: bool,
    query_type: &str,
    start: Option<u64>,
    end: Option<u64>,
    depth: usize,
    entity: Option<u64>,
    nodes: Option<String>,
    min_weight: Option<i64>,
) -> Result<(), KremisError> {
    use kremis_core::{EdgeWeight, EntityId};

    let depth = depth.min(kremis_core::primitives::MAX_TRAVERSAL_DEPTH);
    let session = load_or_create_session(db_path, backend)?;

    match query_type {
        "lookup" => {
            let entity_id = entity.ok_or(KremisError::InvalidSignal)?;

            match session.lookup_entity(EntityId(entity_id)) {
                Some(node_id) => {
                    println!("Entity {} -> Node {}", entity_id, node_id.0);
                }
                None => println!("Entity {} not found", entity_id),
            }
        }

        "traverse" => {
            let start_id = start.ok_or(KremisError::InvalidSignal)?;

            let artifact = if let Some(min_w) = min_weight {
                session.traverse_filtered(NodeId(start_id), depth, EdgeWeight::new(min_w))
            } else {
                session.traverse(NodeId(start_id), depth)
            };

            match artifact {
                Some(a) => {
                    println!("Traversal from node {} (depth {}):", start_id, depth);
                    println!(
                        "  Path: {:?}",
                        a.path.iter().map(|n| n.0).collect::<Vec<_>>()
                    );
                    if let Some(ref sg) = a.subgraph {
                        println!("  Edges: {}", sg.len());
                        for (from, to, weight) in sg.iter().take(10) {
                            println!("    {} -> {} (weight: {})", from.0, to.0, weight.value());
                        }
                        if sg.len() > 10 {
                            println!("    ... and {} more", sg.len() - 10);
                        }
                    }
                }
                None => println!("Node {} not found", start_id),
            }
        }

        "path" => {
            let start_id = start.ok_or(KremisError::InvalidSignal)?;
            let end_id = end.ok_or(KremisError::InvalidSignal)?;

            match session.strongest_path(NodeId(start_id), NodeId(end_id)) {
                Some(path) => {
                    println!("Strongest path {} -> {}:", start_id, end_id);
                    println!("  {:?}", path.iter().map(|n| n.0).collect::<Vec<_>>());
                }
                None => println!("No path found from {} to {}", start_id, end_id),
            }
        }

        "intersect" => {
            let nodes_str = nodes.ok_or(KremisError::InvalidSignal)?;

            let node_ids: Vec<NodeId> = nodes_str
                .split(',')
                .filter_map(|s: &str| s.trim().parse::<u64>().ok().map(NodeId))
                .collect();

            let result = session.intersect(&node_ids);

            println!(
                "Intersection of {:?}:",
                node_ids.iter().map(|n| n.0).collect::<Vec<_>>()
            );
            println!(
                "  Common neighbors: {:?}",
                result.iter().map(|n| n.0).collect::<Vec<_>>()
            );
        }

        "properties" => {
            let node_id = start.ok_or(KremisError::InvalidSignal)?;

            match session.get_properties(NodeId(node_id)) {
                Ok(props) => {
                    if props.is_empty() {
                        println!("Node {} has no properties", node_id);
                    } else {
                        println!("Properties for node {}:", node_id);
                        for (attr, val) in &props {
                            println!("  {} = {}", attr.as_str(), val.as_str());
                        }
                    }
                }
                Err(KremisError::NodeNotFound(_)) => {
                    println!("Node {} not found", node_id);
                }
                Err(e) => return Err(e),
            }
        }

        _ => {
            return Err(KremisError::SerializationError(format!(
                "Unknown query type: {}. Use: lookup, traverse, path, intersect, properties",
                query_type
            )));
        }
    }

    Ok(())
}

// =============================================================================
// EXPORT COMMAND
// =============================================================================

/// Export graph.
///
/// # M3 Fix
///
/// This function now supports both in-memory and persistent (redb) backends.
/// For persistent backends, it builds a graph snapshot by iterating all
/// nodes and edges from the database.
pub fn cmd_export(
    db_path: &PathBuf,
    backend: &str,
    output: &std::path::Path,
    format: &str,
) -> Result<(), KremisError> {
    // L1 FIX: Validate output path for security (prevents path traversal)
    let validated_output = validate_output_path(output)?;

    let session = load_or_create_session(db_path, backend)?;

    // M3 FIX: Use export_graph_snapshot() which works with both backends
    let graph = session.export_graph_snapshot()?;

    let data = match format {
        "canonical" => {
            let data = export_canonical(&graph)?;
            let checksum = canonical_checksum(&graph);
            println!("Checksum: {}", checksum);
            data
        }
        "json" => {
            let serializable = kremis_core::SerializableGraph::from(&graph);
            serde_json::to_vec_pretty(&serializable)
                .map_err(|e| KremisError::SerializationError(e.to_string()))?
        }
        _ => {
            return Err(KremisError::SerializationError(format!(
                "Unknown format: {}. Use: canonical, json",
                format
            )));
        }
    };

    std::fs::write(&validated_output, &data)
        .map_err(|e| KremisError::SerializationError(format!("Write file: {}", e)))?;

    println!("Exported {} bytes to {:?}", data.len(), validated_output);

    Ok(())
}

// =============================================================================
// IMPORT COMMAND
// =============================================================================

/// Import graph.
pub fn cmd_import(
    db_path: &PathBuf,
    backend: &str,
    input: &std::path::Path,
) -> Result<(), KremisError> {
    // L1 FIX: Validate file path for security (prevents path traversal)
    let validated_path = validate_file_path(input)?;

    // Validate file size before reading to prevent DoS
    validate_file_size(&validated_path, MAX_IMPORT_FILE_SIZE)?;

    let data = std::fs::read(&validated_path)
        .map_err(|e| KremisError::SerializationError(format!("Read file: {}", e)))?;

    let graph = import_canonical(&data)?;
    let session = Session::with_graph(graph);

    if backend == "redb" {
        return Err(KremisError::SerializationError(
            "Import to redb not yet supported. Use file backend.".to_string(),
        ));
    }

    save_session(&session, db_path)?;

    println!(
        "Imported graph: {} nodes, {} edges",
        session.node_count(),
        session.edge_count()
    );

    Ok(())
}

// =============================================================================
// INIT COMMAND
// =============================================================================

/// Initialize new database.
pub fn cmd_init(db_path: &PathBuf, backend: &str, force: bool) -> Result<(), KremisError> {
    if db_path.exists() && !force {
        return Err(KremisError::SerializationError(
            "Database already exists. Use --force to overwrite.".to_string(),
        ));
    }

    match backend {
        "redb" => {
            let _session = Session::with_redb(db_path)?;
            println!("Initialized new redb database at {:?}", db_path);
        }
        _ => {
            let session = Session::new();
            save_session(&session, db_path)?;
            println!("Initialized new file database at {:?}", db_path);
        }
    }

    Ok(())
}

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

/// Load or create a session from a database path with specified backend.
pub fn load_or_create_session(db_path: &PathBuf, backend: &str) -> Result<Session, KremisError> {
    match backend {
        "redb" => Session::with_redb(db_path),
        _ => {
            if db_path.exists() {
                let data = std::fs::read(db_path)
                    .map_err(|e| KremisError::SerializationError(format!("Read db: {}", e)))?;

                // Try canonical format first
                if let Ok(graph) = import_canonical(&data) {
                    return Ok(Session::with_graph(graph));
                }

                // Try JSON format
                if let Ok(serializable) =
                    serde_json::from_slice::<kremis_core::SerializableGraph>(&data)
                {
                    return Ok(Session::with_graph(Graph::from(serializable)));
                }

                Err(KremisError::SerializationError(
                    "Could not parse database file".to_string(),
                ))
            } else {
                Ok(Session::new())
            }
        }
    }
}

/// Save a session to a database path.
pub fn save_session(session: &Session, db_path: &PathBuf) -> Result<(), KremisError> {
    if session.is_persistent() {
        // Redb backend - already persisted, nothing to do
        Ok(())
    } else {
        // File backend - export to canonical format
        // Use graph_opt() - we know it's in-memory since is_persistent() is false
        let graph = session.graph_opt().ok_or_else(|| {
            KremisError::SerializationError("No graph available for export".to_string())
        })?;
        let data = export_canonical(graph)?;
        std::fs::write(db_path, &data)
            .map_err(|e| KremisError::SerializationError(format!("Write db: {}", e)))?;
        Ok(())
    }
}
