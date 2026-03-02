//! # MCP Server Configuration
//!
//! Minimal configuration loader for `kremis-mcp`.
//!
//! Reads the `[logging]`, `[security]`, and `[mcp]` sections from `kremis.toml`
//! (if present) and then applies environment variable overrides.
//!
//! Priority (highest to lowest):
//! 1. Environment variables
//! 2. `kremis.toml` in the current working directory
//! 3. Compiled-in defaults

use serde::Deserialize;

// =============================================================================
// CONFIG STRUCTS
// =============================================================================

/// Logging configuration for the MCP bridge.
#[derive(Debug, Clone, Deserialize)]
pub struct LoggingConfig {
    /// Log format: `"text"` (default) or `"json"`.
    #[serde(default = "LoggingConfig::default_format")]
    pub format: String,
}

impl LoggingConfig {
    fn default_format() -> String {
        "text".to_string()
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            format: Self::default_format(),
        }
    }
}

/// Security configuration for the MCP bridge.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct SecurityConfig {
    /// Bearer token forwarded to the Kremis HTTP API.
    #[serde(default)]
    pub api_key: Option<String>,
}

/// MCP target server configuration.
#[derive(Debug, Clone, Deserialize)]
pub struct McpConfig {
    /// URL of the Kremis HTTP server to proxy requests to.
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
// ROOT CONFIG
// =============================================================================

/// MCP bridge configuration.
///
/// Loaded via [`McpAppConfig::load`].
#[derive(Debug, Clone, Deserialize, Default)]
pub struct McpAppConfig {
    /// Logging settings.
    #[serde(default)]
    pub logging: LoggingConfig,

    /// Security settings.
    #[serde(default)]
    pub security: SecurityConfig,

    /// MCP target server settings.
    #[serde(default)]
    pub mcp: McpConfig,
}

impl McpAppConfig {
    /// Load configuration with priority: env vars > `kremis.toml` > defaults.
    #[must_use]
    pub fn load() -> Self {
        let mut config = Self::default();

        // Layer 1: kremis.toml (if present)
        if let Ok(raw) = std::fs::read_to_string("kremis.toml") {
            match toml::from_str::<McpAppConfig>(&raw) {
                Ok(file_cfg) => {
                    config = file_cfg;
                }
                Err(e) => {
                    eprintln!("kremis.toml parse error (using defaults): {e}");
                }
            }
        }

        // Layer 2: environment variable overrides
        if let Ok(v) = std::env::var("KREMIS_LOG_FORMAT")
            && !v.is_empty()
        {
            config.logging.format = v;
        }
        if let Ok(v) = std::env::var("KREMIS_API_KEY") {
            if !v.is_empty() {
                config.security.api_key = Some(v);
            } else {
                config.security.api_key = None;
            }
        }
        if let Ok(v) = std::env::var("KREMIS_URL")
            && !v.is_empty()
        {
            config.mcp.url = v;
        }

        config
    }
}
