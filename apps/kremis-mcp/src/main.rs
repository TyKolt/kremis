//! # Kremis MCP Server
//!
//! Entry point for the MCP (Model Context Protocol) bridge to Kremis.
//!
//! Reads configuration from environment variables:
//! - `KREMIS_URL` — Kremis server URL (default: `http://localhost:8080`)
//! - `KREMIS_API_KEY` — Optional Bearer token for authentication
//!
//! Communicates with AI clients (Claude, GPT) via MCP over stdio,
//! and forwards requests to the Kremis HTTP API.

mod client;
mod server;

use client::KremisClient;
use rmcp::{ServiceExt, transport::stdio};
use server::KremisMcp;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Logging to stderr only — stdout is reserved for MCP stdio transport.
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    let url = std::env::var("KREMIS_URL").unwrap_or_else(|_| "http://localhost:8080".into());
    let api_key = std::env::var("KREMIS_API_KEY").ok();

    tracing::info!("Kremis MCP server starting, target: {}", url);

    let client = KremisClient::new(url, api_key);
    let mcp = KremisMcp::new(client);

    let service = mcp.serve(stdio()).await.inspect_err(|e| {
        tracing::error!("MCP serve error: {:?}", e);
    })?;

    service.waiting().await?;
    Ok(())
}
