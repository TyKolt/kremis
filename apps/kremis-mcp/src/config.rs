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
// CONFIG REPORT
// =============================================================================

/// Provenance report produced by [`McpAppConfig::load`].
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
            match toml::from_str::<McpAppConfig>(&raw) {
                Ok(file_cfg) => {
                    config = file_cfg;
                    report.toml_loaded = true;
                }
                Err(e) => {
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
        if let Ok(v) = std::env::var("KREMIS_API_KEY") {
            if !v.is_empty() {
                config.security.api_key = Some(v);
            } else {
                config.security.api_key = None;
            }
            report.env_overrides.push("KREMIS_API_KEY");
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
