//! # Middleware Module
//!
//! Rate limiting and other middleware for the Kremis HTTP API.
//!
//! ## Configuration
//!
//! Rate limiting is configured via environment variable:
//! - `KREMIS_RATE_LIMIT`: Requests per second (default: 100)

use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use governor::{
    Quota, RateLimiter,
    clock::DefaultClock,
    state::{InMemoryState, NotKeyed},
};
use std::num::NonZeroU32;
use std::sync::Arc;

/// Default rate limit: 100 requests per second.
const DEFAULT_RPS: NonZeroU32 = NonZeroU32::new(100).unwrap();

// =============================================================================
// RATE LIMITER
// =============================================================================

/// Global rate limiter type alias.
pub type GlobalRateLimiter = Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>;

/// Create a new global rate limiter.
///
/// # Arguments
/// * `requests_per_second` - Maximum requests per second
///
/// # Returns
/// A thread-safe rate limiter wrapped in Arc.
pub fn create_rate_limiter(requests_per_second: u32) -> GlobalRateLimiter {
    let rps = NonZeroU32::new(requests_per_second).unwrap_or(DEFAULT_RPS);
    let quota = Quota::per_second(rps);
    Arc::new(RateLimiter::direct(quota))
}

/// Get rate limit from environment variable.
///
/// Returns the value of `KREMIS_RATE_LIMIT` or 100 if not set.
pub fn get_rate_limit_from_env() -> u32 {
    std::env::var("KREMIS_RATE_LIMIT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(100)
}

/// Rate limiting middleware.
///
/// Checks the global rate limiter before allowing requests through.
/// Returns 429 Too Many Requests if the limit is exceeded.
pub async fn rate_limit_middleware(
    State(limiter): State<GlobalRateLimiter>,
    request: Request<Body>,
    next: Next,
) -> Result<Response, (StatusCode, &'static str)> {
    match limiter.check() {
        Ok(_) => Ok(next.run(request).await),
        Err(_) => {
            tracing::warn!("Rate limit exceeded");
            Err((StatusCode::TOO_MANY_REQUESTS, "Too Many Requests"))
        }
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_rate_limiter() {
        let limiter = create_rate_limiter(50);
        // Should allow first request
        assert!(limiter.check().is_ok());
    }

    #[test]
    fn test_create_rate_limiter_zero_defaults() {
        let limiter = create_rate_limiter(0);
        // Should use default of 100
        assert!(limiter.check().is_ok());
    }
}
