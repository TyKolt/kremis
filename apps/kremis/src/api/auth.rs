//! # Authentication Module
//!
//! Simple API key authentication for the Kremis HTTP API.
//!
//! ## Configuration
//!
//! Authentication is configured via environment variable:
//! - `KREMIS_API_KEY`: If set, all requests (except /health) require this key
//!
//! ## Usage
//!
//! Send the API key in the Authorization header:
//! ```text
//! Authorization: Bearer <your-api-key>
//! ```

use axum::{
    body::Body,
    http::{Request, StatusCode, header},
    middleware::Next,
    response::Response,
};
use subtle::ConstantTimeEq;

// =============================================================================
// API KEY AUTHENTICATION
// =============================================================================

/// Get API key from environment variable.
///
/// Returns `Some(key)` if `KREMIS_API_KEY` is set and non-empty,
/// `None` otherwise (disabling authentication).
pub fn get_api_key_from_env() -> Option<String> {
    std::env::var("KREMIS_API_KEY")
        .ok()
        .filter(|k| !k.is_empty())
}

/// API key authentication middleware.
///
/// If `KREMIS_API_KEY` is set:
/// - `/health` endpoint is always allowed (for load balancer health checks)
/// - All other endpoints require `Authorization: Bearer <key>` header
///
/// If `KREMIS_API_KEY` is not set, all requests are allowed.
pub async fn api_key_auth_middleware(
    request: Request<Body>,
    next: Next,
) -> Result<Response, (StatusCode, &'static str)> {
    let expected_key = get_api_key_from_env();

    // If no API key configured, allow all requests
    let Some(expected) = expected_key else {
        return Ok(next.run(request).await);
    };

    // Always allow health endpoint (for load balancer checks)
    if request.uri().path() == "/health" {
        return Ok(next.run(request).await);
    }

    // Extract API key from Authorization header
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok());

    match auth_header {
        Some(header_value) => {
            // Support both "Bearer <key>" and raw "<key>" formats
            let provided_key = header_value.strip_prefix("Bearer ").unwrap_or(header_value);

            // Constant-time comparison to prevent timing attacks.
            // Pad both keys to the same length so ct_eq always runs over
            // the same number of bytes, preventing length-leaking side channels.
            let provided_bytes = provided_key.as_bytes();
            let expected_bytes = expected.as_bytes();

            let max_len = provided_bytes.len().max(expected_bytes.len());
            let mut padded_provided = vec![0u8; max_len];
            let mut padded_expected = vec![0u8; max_len];
            padded_provided[..provided_bytes.len()].copy_from_slice(provided_bytes);
            padded_expected[..expected_bytes.len()].copy_from_slice(expected_bytes);

            let bytes_match: bool = padded_provided.ct_eq(&padded_expected).into();
            let is_valid = bytes_match && provided_bytes.len() == expected_bytes.len();

            if is_valid {
                Ok(next.run(request).await)
            } else {
                tracing::warn!(
                    event = "auth_failure",
                    reason = "invalid_api_key",
                    "Authentication failed: invalid API key"
                );
                Err((StatusCode::UNAUTHORIZED, "Unauthorized"))
            }
        }
        None => {
            tracing::warn!(
                event = "auth_failure",
                reason = "missing_authorization_header",
                "Missing Authorization header"
            );
            Err((StatusCode::UNAUTHORIZED, "Unauthorized"))
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
    fn test_get_api_key_empty_returns_none() {
        // Clear the env var if set
        // SAFETY: This is a unit test running in isolation.
        unsafe { std::env::remove_var("KREMIS_API_KEY") };
        assert!(get_api_key_from_env().is_none());
    }
}
