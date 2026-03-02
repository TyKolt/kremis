//! # Kremis Library
//!
//! This library exposes the Kremis modules for testing and integration.
//!
//! The main binary uses these modules through the `main.rs` entry point.

pub mod api;
pub mod cli;
pub mod config;

// Re-export kremis_core for convenience
pub use kremis_core;
