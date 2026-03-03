//! # Kremis MCP Server
//!
//! Entry point for the MCP (Model Context Protocol) bridge to Kremis.
//!
//! Configuration is loaded from `kremis.toml` (if present), with environment
//! variables as overrides:
//! - `KREMIS_URL`        — Kremis server URL (default: `http://localhost:8080`)
//! - `KREMIS_API_KEY`    — Optional Bearer token for authentication
//! - `KREMIS_LOG_FORMAT` — Log format: `"text"` (default) or `"json"`
//!
//! Communicates with AI clients (Claude, GPT) via MCP over stdio,
//! and forwards requests to the Kremis HTTP API.

mod client;
mod config;
mod server;

use client::KremisClient;
use config::McpAppConfig;
use rmcp::{ServiceExt, transport::stdio};
use server::KremisMcp;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration: kremis.toml → env var overrides → defaults.
    let (cfg, config_report) = McpAppConfig::load();

    // Logging to stderr only — stdout is reserved for MCP stdio transport.
    match cfg.logging.format.as_str() {
        "json" => {
            tracing_subscriber::fmt()
                .with_writer(std::io::stderr)
                .with_ansi(false)
                .json()
                .init();
        }
        _ => {
            tracing_subscriber::fmt()
                .with_writer(std::io::stderr)
                .with_ansi(false)
                .init();
        }
    }

    tracing::info!(
        toml = config_report.toml_loaded,
        env_overrides = ?config_report.env_overrides,
        "configuration loaded"
    );

    tracing::info!("Kremis MCP server starting, target: {}", cfg.mcp.url);

    let client = KremisClient::new(cfg.mcp.url, cfg.security.api_key);
    let mcp = KremisMcp::new(client);

    let service = mcp.serve(stdio()).await.inspect_err(|e| {
        tracing::error!("MCP serve error: {:?}", e);
    })?;

    service.waiting().await?;
    Ok(())
}
