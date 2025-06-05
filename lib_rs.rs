//! Flux System Administration Framework
//! 
//! A modular, enterprise-grade Linux system configuration and hardening framework.

pub mod cli;
pub mod config;
pub mod error;
pub mod helpers;
pub mod modules;
pub mod workflows;

// Re-export commonly used types
pub use config::Config;
pub use error::{FluxError, Result};

// Framework metadata
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const RELEASE: &str = "2025.05";

/// Initialize the Flux framework
pub fn init() -> Result<()> {
    helpers::logging::init_logging(helpers::logging::LogLevel::Info)?;
    Ok(())
}