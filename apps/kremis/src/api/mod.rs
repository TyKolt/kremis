//! # Kremis HTTP API Module
//!
//! This module implements the HTTP REST API server using axum.
//!
//! ## Endpoints
//!
//! - `POST /signal` - Ingest a new signal
//! - `POST /query` - Execute a query
//! - `GET /status` - Get graph status
//! - `GET /stage` - Get current developmental stage
//! - `POST /export` - Export graph in canonical format
//! - `GET /health` - Health check
//! - `GET /hash` - BLAKE3 cryptographic hash of graph
//! - `GET /metrics` - Prometheus metrics
//!
//! ## Security Configuration
//!
//! Security settings are loaded from `kremis.toml` or environment variables
//! (see [`crate::config::AppConfig`]).

mod auth;
mod handlers;
mod middleware;
mod types;

// Re-export handlers and types for integration tests (via `kremis::api::*`)
#[allow(unused_imports)]
pub use handlers::{
    export_handler, hash_handler, health_handler, ingest_handler, metrics_handler, query_handler,
    retract_handler, stage_handler, status_handler,
};
#[allow(unused_imports)]
pub use types::{
    EdgeJson, ExportResponse, HealthResponse, IngestRequest, IngestResponse, QueryRequest,
    QueryResponse, RetractRequest, RetractResponse, StageResponse, StatusResponse,
};

use axum::{
    Router,
    http::{HeaderValue, Method, header},
    middleware as axum_middleware,
    routing::{get, post},
};
use kremis_core::{KremisError, Session};
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

use crate::config::AppConfig;
use middleware::create_rate_limiter;

// =============================================================================
// SERVER STATE
// =============================================================================

/// Shared server state containing the graph session and security config.
#[derive(Clone)]
pub struct AppState {
    /// The session containing the graph.
    pub session: Arc<RwLock<Session>>,
    /// API key for Bearer token authentication. `None` disables auth.
    pub api_key: Option<String>,
}

impl AppState {
    /// Create new app state with a session (no authentication).
    /// Used by integration tests.
    #[must_use]
    #[allow(dead_code)]
    pub fn new(session: Session) -> Self {
        Self {
            session: Arc::new(RwLock::new(session)),
            api_key: None,
        }
    }

    /// Create new app state with a session and optional API key.
    #[must_use]
    pub fn with_api_key(session: Session, api_key: Option<String>) -> Self {
        Self {
            session: Arc::new(RwLock::new(session)),
            api_key,
        }
    }
}

// =============================================================================
// CORS CONFIGURATION
// =============================================================================

/// Build CORS layer from the provided origins list.
///
/// - `["*"]` or a list containing `"*"` enables permissive CORS (dev only).
/// - Empty list defaults to localhost only (restrictive default).
/// - Otherwise parses each entry as an allowed origin.
fn build_cors_layer(origins: &[String]) -> CorsLayer {
    // Single wildcard entry → permissive
    if origins.iter().any(|o| o == "*") {
        tracing::warn!(
            event = "cors_insecure",
            "CORS: Allowing ALL origins. This is insecure for production!"
        );
        return CorsLayer::permissive();
    }

    if origins.is_empty() {
        tracing::info!("CORS: No origins configured, defaulting to localhost only");
        return build_localhost_cors();
    }

    // Parse each entry as a HeaderValue
    let allowed_origins: Vec<HeaderValue> = origins
        .iter()
        .filter_map(|s| match s.parse::<HeaderValue>() {
            Ok(hv) => {
                tracing::info!("CORS: Allowing origin: {}", s);
                Some(hv)
            }
            Err(e) => {
                tracing::warn!("CORS: Invalid origin '{}': {}", s, e);
                None
            }
        })
        .collect();

    if allowed_origins.is_empty() {
        tracing::warn!("CORS: No valid origins parsed, defaulting to localhost only");
        build_localhost_cors()
    } else {
        CorsLayer::new()
            .allow_origin(allowed_origins)
            .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
            .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION])
    }
}

/// Build a restrictive CORS layer that only allows localhost origins.
fn build_localhost_cors() -> CorsLayer {
    let localhost_origins = vec![
        "http://localhost:3000".parse::<HeaderValue>().ok(),
        "http://localhost:8080".parse::<HeaderValue>().ok(),
        "http://127.0.0.1:3000".parse::<HeaderValue>().ok(),
        "http://127.0.0.1:8080".parse::<HeaderValue>().ok(),
    ];
    let origins: Vec<HeaderValue> = localhost_origins.into_iter().flatten().collect();

    CorsLayer::new()
        .allow_origin(origins)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION])
}

// =============================================================================
// ROUTER CREATION
// =============================================================================

/// Create the axum router with all endpoints and middleware.
///
/// Loads configuration via [`AppConfig::load`] so env var overrides work.
/// Middleware stack (outer to inner):
/// 1. CORS - handles preflight requests
/// 2. Tracing - logs all requests
/// 3. Rate Limiting - protects against DoS (if enabled)
/// 4. Authentication - validates API key (if configured)
///
/// Used by integration tests.
#[allow(dead_code)]
pub fn create_router(state: AppState) -> Router {
    let config = AppConfig::load();
    // Merge api_key from state (explicitly set) with config (env/file).
    // State takes priority if already populated; otherwise use config.
    let merged_key = state
        .api_key
        .clone()
        .or_else(|| config.security.api_key.clone());
    let merged_state = AppState {
        session: state.session.clone(),
        api_key: merged_key,
    };
    create_router_with_config(merged_state, &config)
}

/// Create the axum router using an explicit [`AppConfig`].
pub fn create_router_with_config(state: AppState, config: &AppConfig) -> Router {
    let cors = build_cors_layer(&config.cors.origins);

    let rate_limit = config.api.rate_limit;
    let rate_limiter = if rate_limit > 0 {
        tracing::info!("Rate limiting enabled: {} requests/second", rate_limit);
        Some(create_rate_limiter(rate_limit))
    } else {
        tracing::info!("Rate limiting disabled");
        None
    };

    let has_auth = state.api_key.is_some();
    if has_auth {
        tracing::info!("API key authentication enabled");
    } else {
        tracing::warn!(
            "API key authentication DISABLED - all endpoints are publicly accessible! \
             Set KREMIS_API_KEY environment variable or kremis.toml [security] api_key to enable."
        );
    }

    let mut router = Router::new()
        .route("/health", get(handlers::health_handler))
        .route("/status", get(handlers::status_handler))
        .route("/stage", get(handlers::stage_handler))
        .route("/signal", post(handlers::ingest_handler))
        .route("/signal/retract", post(handlers::retract_handler))
        .route("/query", post(handlers::query_handler))
        .route("/export", post(handlers::export_handler))
        .route("/hash", get(handlers::hash_handler))
        .route("/metrics", get(handlers::metrics_handler));

    if has_auth {
        router = router.layer(axum_middleware::from_fn_with_state(
            state.api_key.clone(),
            auth::api_key_auth_middleware,
        ));
    }

    if let Some(limiter) = rate_limiter {
        router = router.layer(axum_middleware::from_fn_with_state(
            limiter,
            middleware::rate_limit_middleware,
        ));
    }

    router
        .layer(axum::extract::DefaultBodyLimit::max(2 * 1024 * 1024))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

// =============================================================================
// SERVER STARTUP
// =============================================================================

/// Start the HTTP server using the provided [`AppConfig`].
pub async fn run_server(
    addr: &str,
    session: Session,
    config: &AppConfig,
) -> Result<(), KremisError> {
    let state = AppState::with_api_key(session, config.security.api_key.clone());
    let router = create_router_with_config(state, config);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|e| KremisError::IoError(format!("Bind failed: {}", e)))?;

    tracing::info!(
        event = "server_start",
        addr = addr,
        "Kremis HTTP server listening on {}",
        addr
    );

    axum::serve(listener, router)
        .await
        .map_err(|e| KremisError::IoError(format!("Server error: {}", e)))
}
