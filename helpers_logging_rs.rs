use crate::error::Result;
use clap::ValueEnum;
use colored::Colorize;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;
use tracing::{debug, error, info, warn};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

/// Log levels
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

impl From<LogLevel> for tracing::Level {
    fn from(level: LogLevel) -> Self {
        match level {
            LogLevel::Debug => tracing::Level::DEBUG,
            LogLevel::Info => tracing::Level::INFO,
            LogLevel::Warn => tracing::Level::WARN,
            LogLevel::Error => tracing::Level::ERROR,
        }
    }
}

/// Initialize logging system
pub fn init_logging(level: LogLevel) -> Result<()> {
    let env_filter = EnvFilter::new(format!("flux_framework={}", tracing::Level::from(level)));
    
    tracing_subscriber::registry()
        .with(fmt::layer().with_target(false).with_thread_ids(false))
        .with(env_filter)
        .init();
    
    Ok(())
}

/// Log directory path
pub fn log_dir() -> Result<std::path::PathBuf> {
    Ok(Path::new("/var/log").to_path_buf())
}

/// Log file path
pub fn log_file() -> Result<std::path::PathBuf> {
    Ok(log_dir()?.join("flux-setup.log"))
}

/// Write to log file
pub fn write_to_log_file(message: &str) -> Result<()> {
    let log_path = log_file()?;
    
    // Ensure log directory exists
    if let Some(parent) = log_path.parent() {
        fs::create_dir_all(parent)?;
    }
    
    // Open log file in append mode
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)?;
    
    // Write timestamped message
    let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
    writeln!(file, "[{}] {}", timestamp, message)?;
    
    Ok(())
}

/// Log debug message
pub fn log_debug<S: AsRef<str>>(message: S) {
    let msg = message.as_ref();
    debug!("{}", msg);
    println!("{} {}", "[DEBUG]".cyan(), msg);
    let _ = write_to_log_file(&format!("[DEBUG] {}", msg));
}

/// Log info message
pub fn log_info<S: AsRef<str>>(message: S) {
    let msg = message.as_ref();
    info!("{}", msg);
    println!("{} {}", "[INFO]".green(), msg);
    let _ = write_to_log_file(&format!("[INFO] {}", msg));
}

/// Log warning message
pub fn log_warn<S: AsRef<str>>(message: S) {
    let msg = message.as_ref();
    warn!("{}", msg);
    println!("{} {}", "[WARN]".yellow(), msg);
    let _ = write_to_log_file(&format!("[WARN] {}", msg));
}

/// Log error message
pub fn log_error<S: AsRef<str>>(message: S) {
    let msg = message.as_ref();
    error!("{}", msg);
    eprintln!("{} {}", "[ERROR]".red(), msg);
    let _ = write_to_log_file(&format!("[ERROR] {}", msg));
}

/// Progress indicator
pub struct ProgressIndicator {
    pb: indicatif::ProgressBar,
}

impl ProgressIndicator {
    /// Create a new progress indicator
    pub fn new(len: u64, message: &str) -> Self {
        let pb = indicatif::ProgressBar::new(len);
        pb.set_style(
            indicatif::ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
                .unwrap()
                .progress_chars("#>-"),
        );
        pb.set_message(message.to_string());
        
        Self { pb }
    }
    
    /// Create a spinner for indeterminate progress
    pub fn new_spinner(message: &str) -> Self {
        let pb = indicatif::ProgressBar::new_spinner();
        pb.set_style(
            indicatif::ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg}")
                .unwrap(),
        );
        pb.set_message(message.to_string());
        
        Self { pb }
    }
    
    /// Update progress
    pub fn inc(&self, delta: u64) {
        self.pb.inc(delta);
    }
    
    /// Set message
    pub fn set_message(&self, message: &str) {
        self.pb.set_message(message.to_string());
    }
    
    /// Finish with message
    pub fn finish_with_message(&self, message: &str) {
        self.pb.finish_with_message(message.to_string());
    }
    
    /// Finish and clear
    pub fn finish_and_clear(&self) {
        self.pb.finish_and_clear();
    }
}