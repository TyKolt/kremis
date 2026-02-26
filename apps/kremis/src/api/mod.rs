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
//! ## Security Configuration (Environment Variables)
//!
//! - `KREMIS_CORS_ORIGINS`: Comma-separated list of allowed origins, or "*" for all (default: localhost only)
//! - `KREMIS_RATE_LIMIT`: Requests per second (default: 100, 0 to disable)
//! - `KREMIS_API_KEY`: If set, requires Bearer token authentication

mod auth;
mod handlers;
mod middleware;
mod types;

// Re-exports for external use
pub use auth::get_api_key_from_env;
pub use middleware::{create_rate_limiter, get_rate_limit_from_env};
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

// =============================================================================
// SERVER STATE
// =============================================================================

/// Shared server state containing the graph session.
#[derive(Clone)]
pub struct AppState {
    /// The session containing the graph.
    pub session: Arc<RwLock<Session>>,
}

impl AppState {
    /// Create new app state with a session.
    #[must_use]
    pub fn new(session: Session) -> Self {
        Self {
            session: Arc::new(RwLock::new(session)),
        }
    }
}

// =============================================================================
// CORS CONFIGURATION
// =============================================================================

/// Build CORS layer from environment configuration.
///
/// Reads `KREMIS_CORS_ORIGINS` environment variable:
/// - If "*": allows all origins (development mode - use with caution!)
/// - If not set: defaults to localhost only (restrictive default)
/// - Otherwise: parses comma-separated list of allowed origins
///
/// # Security Note
///
/// The default is restrictive (localhost only). Set `KREMIS_CORS_ORIGINS=*`
/// explicitly only for development or if you understand the security implications.
fn build_cors_layer() -> CorsLayer {
    let origins_env = std::env::var("KREMIS_CORS_ORIGINS").ok();

    match origins_env.as_deref() {
        Some("*") => {
            // Explicit wildcard - warn about security implications
            tracing::warn!(
                "CORS: Allowing ALL origins (KREMIS_CORS_ORIGINS=*). This is insecure for production!"
            );
            CorsLayer::permissive()
        }
        Some(origins) => {
            // Parse comma-separated origins
            let allowed_origins: Vec<HeaderValue> = origins
                .split(',')
                .filter_map(|s| {
                    let trimmed = s.trim();
                    match trimmed.parse::<HeaderValue>() {
                        Ok(hv) => {
                            tracing::info!("CORS: Allowing origin: {}", trimmed);
                            Some(hv)
                        }
                        Err(e) => {
                            tracing::warn!("CORS: Invalid origin '{}': {}", trimmed, e);
                            None
                        }
                    }
                })
                .collect();

            if allowed_origins.is_empty() {
                tracing::warn!(
                    "CORS: No valid origins in KREMIS_CORS_ORIGINS, defaulting to localhost only"
                );
                build_localhost_cors()
            } else {
                CorsLayer::new()
                    .allow_origin(allowed_origins)
                    .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
                    .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION])
            }
        }
        None => {
            // No configuration - default to localhost only (restrictive)
            tracing::info!("CORS: No KREMIS_CORS_ORIGINS set, defaulting to localhost only");
            build_localhost_cors()
        }
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
/// Middleware stack (outer to inner):
/// 1. CORS - handles preflight requests
/// 2. Tracing - logs all requests
/// 3. Rate Limiting - protects against DoS (if enabled)
/// 4. Authentication - validates API key (if configured)
pub fn create_router(state: AppState) -> Router {
    let cors = build_cors_layer();

    // Check if rate limiting is enabled
    let rate_limit = get_rate_limit_from_env();
    let rate_limiter = if rate_limit > 0 {
        tracing::info!("Rate limiting enabled: {} requests/second", rate_limit);
        Some(create_rate_limiter(rate_limit))
    } else {
        tracing::info!("Rate limiting disabled");
        None
    };

    // Check if authentication is enabled (M6 FIX: explicit warning for disabled auth)
    let has_auth = get_api_key_from_env().is_some();
    if has_auth {
        tracing::info!("API key authentication enabled");
    } else {
        // M6 FIX: Warn users about disabled authentication
        tracing::warn!(
            "⚠️  API key authentication DISABLED - all endpoints are publicly accessible! \
             Set KREMIS_API_KEY environment variable to enable authentication."
        );
    }

    // Build base router with routes
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

    // Apply authentication middleware (innermost - runs last on request)
    if has_auth {
        router = router.layer(axum_middleware::from_fn(auth::api_key_auth_middleware));
    }

    // Apply rate limiting middleware
    if let Some(limiter) = rate_limiter {
        router = router.layer(axum_middleware::from_fn_with_state(
            limiter,
            middleware::rate_limit_middleware,
        ));
    }

    // Apply CORS, body limit, and tracing (outermost layers)
    router
        .layer(axum::extract::DefaultBodyLimit::max(2 * 1024 * 1024))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

// =============================================================================
// SERVER STARTUP
// =============================================================================

/// Start the HTTP server.
pub async fn run_server(addr: &str, session: Session) -> Result<(), KremisError> {
    let state = AppState::new(session);
    let router = create_router(state);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|e| KremisError::IoError(format!("Bind failed: {}", e)))?;

    tracing::info!("Kremis HTTP server listening on {}", addr);

    axum::serve(listener, router)
        .await
        .map_err(|e| KremisError::IoError(format!("Server error: {}", e)))
}
