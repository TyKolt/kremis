//! # Application Configuration
//!
//! Centralised configuration loader for Kremis.
//!
//! Priority (highest to lowest):
//! 1. Environment variables
//! 2. `kremis.toml` in the current working directory
//! 3. Compiled-in defaults
//!
//! ## Supported environment variable overrides
//!
//! | Env Var             | Config key              |
//! |---------------------|-------------------------|
//! | `KREMIS_LOG_FORMAT` | `[logging] format`      |
//! | `RUST_LOG`          | `[logging] level`       |
//! | `KREMIS_RATE_LIMIT` | `[api] rate_limit`      |
//! | `KREMIS_API_KEY`    | `[security] api_key`    |
//! | `KREMIS_CORS_ORIGINS` | `[cors] origins`      |
//! | `KREMIS_URL`        | `[mcp] url`             |

use serde::Deserialize;

// =============================================================================
// CONFIG STRUCTS
// =============================================================================

/// Logging configuration.
#[derive(Debug, Clone, Deserialize)]
pub struct LoggingConfig {
    /// Log format: `"text"` (default) or `"json"`.
    #[serde(default = "LoggingConfig::default_format")]
    pub format: String,

    /// `tracing_subscriber` filter string (mirrors `RUST_LOG`).
    #[serde(default = "LoggingConfig::default_level")]
    pub level: String,
}

impl LoggingConfig {
    fn default_format() -> String {
        "text".to_string()
    }
    fn default_level() -> String {
        "kremis=info,tower_http=debug".to_string()
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            format: Self::default_format(),
            level: Self::default_level(),
        }
    }
}

/// HTTP API configuration.
#[derive(Debug, Clone, Deserialize)]
pub struct ApiConfig {
    /// Requests per second. `0` disables rate limiting.
    #[serde(default = "ApiConfig::default_rate_limit")]
    pub rate_limit: u32,
}

impl ApiConfig {
    fn default_rate_limit() -> u32 {
        100
    }
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            rate_limit: Self::default_rate_limit(),
        }
    }
}

/// Security configuration.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct SecurityConfig {
    /// Bearer token for API key authentication. `None` disables auth.
    #[serde(default)]
    pub api_key: Option<String>,
}

/// CORS configuration.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct CorsConfig {
    /// Allowed origins. Empty list defaults to localhost-only. `["*"]` allows all.
    #[serde(default)]
    pub origins: Vec<String>,
}

/// MCP client configuration (used by both kremis and kremis-mcp).
#[derive(Debug, Clone, Deserialize)]
pub struct McpConfig {
    /// Kremis server URL used by the MCP bridge.
    #[serde(default = "McpConfig::default_url")]
    pub url: String,
}

impl McpConfig {
    fn default_url() -> String {
        "http://localhost:8080".to_string()
    }
}

impl Default for McpConfig {
    fn default() -> Self {
        Self {
            url: Self::default_url(),
        }
    }
}

// =============================================================================
// CONFIG REPORT
// =============================================================================

/// Provenance report produced by [`AppConfig::load`].
///
/// Reports *which* configuration sources were active, never the values.
/// Sensitive fields (e.g. `api_key`) are only reported as present/absent.
#[derive(Debug, Clone)]
pub struct ConfigReport {
    /// `true` if `kremis.toml` was found and parsed successfully.
    pub toml_loaded: bool,
    /// Names of environment variables that overrode a config value.
    /// Values are NEVER included.
    pub env_overrides: Vec<&'static str>,
}

// =============================================================================
// ROOT CONFIG
// =============================================================================

/// Full application configuration.
///
/// Loaded via [`AppConfig::load`].
#[derive(Debug, Clone, Deserialize, Default)]
pub struct AppConfig {
    /// Logging settings.
    #[serde(default)]
    pub logging: LoggingConfig,

    /// HTTP API settings.
    #[serde(default)]
    pub api: ApiConfig,

    /// Security settings.
    #[serde(default)]
    pub security: SecurityConfig,

    /// CORS settings.
    #[serde(default)]
    pub cors: CorsConfig,

    /// MCP bridge settings.
    #[serde(default)]
    pub mcp: McpConfig,
}

impl AppConfig {
    /// Load configuration with priority: env vars > `kremis.toml` > defaults.
    ///
    /// This function never fails: if `kremis.toml` is absent or malformed it
    /// logs a warning and falls back to defaults.
    ///
    /// Returns the loaded config together with a [`ConfigReport`] that records
    /// which sources were active (file present, env overrides applied).
    #[must_use]
    pub fn load() -> (Self, ConfigReport) {
        let mut config = Self::default();
        let mut report = ConfigReport {
            toml_loaded: false,
            env_overrides: vec![],
        };

        // Layer 1: kremis.toml (if present)
        if let Ok(raw) = std::fs::read_to_string("kremis.toml") {
            match toml::from_str::<AppConfig>(&raw) {
                Ok(file_cfg) => {
                    config = file_cfg;
                    report.toml_loaded = true;
                }
                Err(e) => {
                    // Can't use tracing here (not yet initialised) — print to stderr.
                    eprintln!("kremis.toml parse error (using defaults): {e}");
                }
            }
        }

        // Layer 2: environment variable overrides (track each applied override)
        if let Ok(v) = std::env::var("KREMIS_LOG_FORMAT")
            && !v.is_empty()
        {
            config.logging.format = v;
            report.env_overrides.push("KREMIS_LOG_FORMAT");
        }
        if let Ok(v) = std::env::var("RUST_LOG")
            && !v.is_empty()
        {
            config.logging.level = v;
            report.env_overrides.push("RUST_LOG");
        }
        if let Ok(v) = std::env::var("KREMIS_RATE_LIMIT")
            && let Ok(n) = v.parse::<u32>()
        {
            config.api.rate_limit = n;
            report.env_overrides.push("KREMIS_RATE_LIMIT");
        }
        if let Ok(v) = std::env::var("KREMIS_API_KEY") {
            if !v.is_empty() {
                config.security.api_key = Some(v);
            } else {
                // Explicit empty string clears the key (auth disabled)
                config.security.api_key = None;
            }
            report.env_overrides.push("KREMIS_API_KEY");
        }
        if let Ok(v) = std::env::var("KREMIS_CORS_ORIGINS")
            && !v.is_empty()
        {
            config.cors.origins = v
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            report.env_overrides.push("KREMIS_CORS_ORIGINS");
        }
        if let Ok(v) = std::env::var("KREMIS_URL")
            && !v.is_empty()
        {
            config.mcp.url = v;
            report.env_overrides.push("KREMIS_URL");
        }

        (config, report)
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = AppConfig::default();
        assert_eq!(cfg.logging.format, "text");
        assert_eq!(cfg.logging.level, "kremis=info,tower_http=debug");
        assert_eq!(cfg.api.rate_limit, 100);
        assert!(cfg.security.api_key.is_none());
        assert!(cfg.cors.origins.is_empty());
        assert_eq!(cfg.mcp.url, "http://localhost:8080");
    }

    #[test]
    fn test_toml_parse_partial() {
        let raw = r#"
[logging]
format = "json"
"#;
        let cfg: AppConfig = toml::from_str(raw).expect("hardcoded TOML must parse");
        assert_eq!(cfg.logging.format, "json");
        // Unset fields keep their defaults
        assert_eq!(cfg.logging.level, "kremis=info,tower_http=debug");
        assert_eq!(cfg.api.rate_limit, 100);
    }

    #[test]
    fn test_cors_origins_csv_parsing() {
        // Verify that KREMIS_CORS_ORIGINS is split by comma and trimmed.
        // SAFETY: no other test in this file sets KREMIS_CORS_ORIGINS.
        unsafe {
            std::env::set_var(
                "KREMIS_CORS_ORIGINS",
                "https://a.example.com, https://b.example.com , https://c.example.com",
            );
        }
        let (cfg, report) = AppConfig::load();
        unsafe {
            std::env::remove_var("KREMIS_CORS_ORIGINS");
        }
        assert!(report.env_overrides.contains(&"KREMIS_CORS_ORIGINS"));
        assert_eq!(cfg.cors.origins.len(), 3);
        assert_eq!(cfg.cors.origins[0], "https://a.example.com");
        assert_eq!(cfg.cors.origins[1], "https://b.example.com");
        assert_eq!(cfg.cors.origins[2], "https://c.example.com");
    }

    #[test]
    fn test_toml_parse_full() {
        let raw = r#"
[logging]
format = "json"
level  = "debug"

[api]
rate_limit = 50

[security]
api_key = "secret"

[cors]
origins = ["https://example.com"]

[mcp]
url = "http://kremis:9090"
"#;
        let cfg: AppConfig = toml::from_str(raw).expect("hardcoded TOML must parse");
        assert_eq!(cfg.logging.format, "json");
        assert_eq!(cfg.logging.level, "debug");
        assert_eq!(cfg.api.rate_limit, 50);
        assert_eq!(cfg.security.api_key.as_deref(), Some("secret"));
        assert_eq!(cfg.cors.origins, vec!["https://example.com"]);
        assert_eq!(cfg.mcp.url, "http://kremis:9090");
    }
}
