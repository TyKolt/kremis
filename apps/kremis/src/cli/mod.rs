//! # Kremis CLI Module
//!
//! This module implements the CLI interface for Kremis.
//!
//! ## Available Commands
//!
//! - `server` - Start the HTTP server
//! - `status` - Show graph status
//! - `stage` - Show developmental stage
//! - `ingest` - Ingest signals from a file
//! - `query` - Execute a query on the graph
//! - `export` - Export graph to file
//! - `import` - Import graph from file
//! - `init` - Initialize new database
//! - `hash` - Compute BLAKE3 cryptographic hash of graph

mod commands;

use clap::{Parser, Subcommand};
use kremis_core::KremisError;
use std::path::PathBuf;

pub use commands::*;

// =============================================================================
// CLI STRUCTURE
// =============================================================================

/// Kremis - Honest AGI Server
///
/// A minimal, deterministic, grounded cognitive core.
/// The system contains only the structure of the signals it has processed.
#[derive(Parser, Debug)]
#[command(name = "kremis")]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// Enable verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Suppress banner output
    #[arg(short, long, global = true)]
    pub quiet: bool,

    /// Path to the graph database
    #[arg(short = 'D', long, global = true, default_value = "kremis.db")]
    pub database: PathBuf,

    /// Storage backend: "file" (canonical file) or "redb" (ACID database)
    #[arg(short = 'B', long, global = true, default_value = "redb")]
    pub backend: String,

    /// Output in JSON format (for programmatic access)
    #[arg(long, global = true)]
    pub json_mode: bool,

    /// Subcommand to execute
    #[command(subcommand)]
    pub command: Option<Commands>,
}

/// Available CLI commands.
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Start HTTP server
    Server {
        /// Host to bind to
        #[arg(short = 'H', long, default_value = "127.0.0.1")]
        host: String,

        /// Port to bind to
        #[arg(short, long, default_value = "8080")]
        port: u16,
    },

    /// Show graph status
    Status,

    /// Show current developmental stage
    Stage {
        /// Show detailed progress information
        #[arg(short, long)]
        detailed: bool,
    },

    /// Ingest signals from a file
    Ingest {
        /// Path to the input file (JSON or text)
        #[arg(short, long)]
        file: PathBuf,

        /// Input format (json, text)
        #[arg(short = 't', long, default_value = "json")]
        format: String,
    },

    /// Execute a query on the graph
    Query {
        /// Query type (lookup, traverse, path, intersect)
        #[arg(short = 't', long)]
        query_type: String,

        /// Start node ID
        #[arg(short, long)]
        start: Option<u64>,

        /// End node ID (for path queries)
        #[arg(short, long)]
        end: Option<u64>,

        /// Traversal depth
        #[arg(short, long, default_value = "3")]
        depth: usize,

        /// Entity ID (for lookup queries)
        #[arg(long)]
        entity: Option<u64>,

        /// Node IDs for intersection (comma-separated)
        #[arg(long)]
        nodes: Option<String>,

        /// Minimum edge weight filter
        #[arg(long)]
        min_weight: Option<i64>,
    },

    /// Export graph in canonical format
    Export {
        /// Output file path
        #[arg(short, long)]
        output: PathBuf,

        /// Export format (canonical, json)
        #[arg(short = 't', long, default_value = "canonical")]
        format: String,
    },

    /// Import graph from canonical format (file backend only)
    Import {
        /// Input file path
        #[arg(short, long)]
        input: PathBuf,
    },

    /// Initialize a new empty database
    Init {
        /// Force initialization even if database exists
        #[arg(short, long)]
        force: bool,
    },

    /// Compute BLAKE3 cryptographic hash of graph
    Hash,
}

// =============================================================================
// COMMAND EXECUTION
// =============================================================================

/// Execute the CLI with parsed arguments.
pub async fn execute(cli: Cli) -> Result<(), KremisError> {
    let backend = cli.backend.as_str();
    let json_mode = cli.json_mode;

    match cli.command {
        Some(Commands::Server { host, port }) => {
            cmd_server(&cli.database, backend, &host, port).await
        }
        Some(Commands::Status) => cmd_status(&cli.database, backend, json_mode),
        Some(Commands::Stage { detailed }) => {
            cmd_stage(&cli.database, backend, json_mode, detailed)
        }
        Some(Commands::Ingest { file, format }) => {
            cmd_ingest(&cli.database, backend, json_mode, &file, &format)
        }
        Some(Commands::Query {
            query_type,
            start,
            end,
            depth,
            entity,
            nodes,
            min_weight,
        }) => cmd_query(
            &cli.database,
            backend,
            json_mode,
            &query_type,
            start,
            end,
            depth,
            entity,
            nodes,
            min_weight,
        ),
        Some(Commands::Export { output, format }) => {
            cmd_export(&cli.database, backend, &output, &format)
        }
        Some(Commands::Import { input }) => cmd_import(&cli.database, backend, &input),
        Some(Commands::Init { force }) => cmd_init(&cli.database, backend, force),
        Some(Commands::Hash) => cmd_hash(&cli.database, backend, json_mode),
        None => {
            // No subcommand - show status by default
            cmd_status(&cli.database, backend, json_mode)
        }
    }
}
