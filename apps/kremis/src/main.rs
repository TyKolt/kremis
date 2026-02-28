//! # Kremis - Honest AGI Server
//!
//! The main binary for the Kremis deterministic graph substrate.
//!
//! This application provides:
//! - HTTP REST API server (axum-based)
//! - CLI interface for graph operations
//! - Plugin process management
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                      apps/kremis (THE BINARY)                   │
//! │                                                                 │
//! │  ┌─────────────┐    ┌─────────────┐    ┌──────────────────┐   │
//! │  │   CLI       │    │   HTTP API  │    │  Plugin Manager  │   │
//! │  │  (clap)     │    │   (axum)    │    │  (process mgmt)  │   │
//! │  └──────┬──────┘    └──────┬──────┘    └────────┬─────────┘   │
//! │         │                  │                    │              │
//! │         └──────────────────┼────────────────────┘              │
//! │                            ▼                                   │
//! │                    ┌───────────────┐                           │
//! │                    │  kremis-core  │                           │
//! │                    │ (THE LOGIC)   │                           │
//! │                    └───────────────┘                           │
//! └─────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Usage
//!
//! ```bash
//! # Start the HTTP server
//! kremis server --host 0.0.0.0 --port 8080
//!
//! # CLI operations
//! kremis status
//! kremis ingest -f signals.json
//! kremis query -t traverse -s 1 -d 3
//! ```

mod api;
mod cli;

use clap::Parser;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// =============================================================================
// APPLICATION ENTRY POINT
// =============================================================================

#[tokio::main]
async fn main() {
    // Initialize tracing — KREMIS_LOG_FORMAT=json enables machine-parseable output.
    let log_format = std::env::var("KREMIS_LOG_FORMAT").unwrap_or_else(|_| "text".to_string());

    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "kremis=info,tower_http=debug".into());

    match log_format.as_str() {
        "json" => {
            tracing_subscriber::registry()
                .with(filter)
                .with(tracing_subscriber::fmt::layer().json())
                .init();
        }
        _ => {
            tracing_subscriber::registry()
                .with(filter)
                .with(tracing_subscriber::fmt::layer())
                .init();
        }
    }

    // Parse CLI arguments
    let cli = cli::Cli::parse();

    // Display startup banner
    if !cli.quiet {
        print_banner();
    }

    // Execute command
    if let Err(e) = cli::execute(cli).await {
        tracing::error!("Error: {}", e);
        std::process::exit(1);
    }
}

/// Print the Kremis startup banner.
fn print_banner() {
    println!(
        r#"
  ██╗  ██╗██████╗ ███████╗███╗   ███╗██╗███████╗
  ██║ ██╔╝██╔══██╗██╔════╝████╗ ████║██║██╔════╝
  █████╔╝ ██████╔╝█████╗  ██╔████╔██║██║███████╗
  ██╔═██╗ ██╔══██╗██╔══╝  ██║╚██╔╝██║██║╚════██║
  ██║  ██╗██║  ██║███████╗██║ ╚═╝ ██║██║███████║
  ╚═╝  ╚═╝╚═╝  ╚═╝╚══════╝╚═╝     ╚═╝╚═╝╚══════╝

  Honest AGI Server v{}

  Deterministic • Grounded • Verifiable
"#,
        env!("CARGO_PKG_VERSION")
    );
}
