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
mod config;

use clap::Parser;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// =============================================================================
// APPLICATION ENTRY POINT
// =============================================================================

#[tokio::main]
async fn main() {
    // Load configuration: kremis.toml → env var overrides → defaults.
    let (cfg, config_report) = config::AppConfig::load();

    let filter = tracing_subscriber::EnvFilter::new(&cfg.logging.level);

    match cfg.logging.format.as_str() {
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

    tracing::info!(
        toml = config_report.toml_loaded,
        env_overrides = ?config_report.env_overrides,
        "configuration loaded"
    );

    // Parse CLI arguments
    let cli = cli::Cli::parse();

    // Display startup banner
    if !cli.quiet {
        print_banner();
    }

    // Execute command
    if let Err(e) = cli::execute(cli, cfg).await {
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
