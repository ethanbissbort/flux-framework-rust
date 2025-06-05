use thiserror::Error;

/// Result type for Flux operations
pub type Result<T> = std::result::Result<T, FluxError>;

/// Main error type for Flux framework
#[derive(Error, Debug)]
pub enum FluxError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Module error: {0}")]
    Module(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("System error: {0}")]
    System(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Permission denied: {0}")]
    Permission(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Command execution failed: {0}")]
    CommandFailed(String),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("SSH error: {0}")]
    Ssh(String),

    #[error("User cancelled operation")]
    UserCancelled,

    #[error("Unsupported operation: {0}")]
    Unsupported(String),

    #[error("External error: {0}")]
    External(#[from] anyhow::Error),
}

impl FluxError {
    /// Create a new configuration error
    pub fn config<S: Into<String>>(msg: S) -> Self {
        FluxError::Config(msg.into())
    }

    /// Create a new module error
    pub fn module<S: Into<String>>(msg: S) -> Self {
        FluxError::Module(msg.into())
    }

    /// Create a new validation error
    pub fn validation<S: Into<String>>(msg: S) -> Self {
        FluxError::Validation(msg.into())
    }

    /// Create a new system error
    pub fn system<S: Into<String>>(msg: S) -> Self {
        FluxError::System(msg.into())
    }

    /// Create a new network error
    pub fn network<S: Into<String>>(msg: S) -> Self {
        FluxError::Network(msg.into())
    }

    /// Create a new permission error
    pub fn permission<S: Into<String>>(msg: S) -> Self {
        FluxError::Permission(msg.into())
    }

    /// Create a new not found error
    pub fn not_found<S: Into<String>>(msg: S) -> Self {
        FluxError::NotFound(msg.into())
    }

    /// Create a new command failed error
    pub fn command_failed<S: Into<String>>(msg: S) -> Self {
        FluxError::CommandFailed(msg.into())
    }

    /// Create a new parse error
    pub fn parse<S: Into<String>>(msg: S) -> Self {
        FluxError::Parse(msg.into())
    }

    /// Create a new SSH error
    pub fn ssh<S: Into<String>>(msg: S) -> Self {
        FluxError::Ssh(msg.into())
    }

    /// Create a new unsupported error
    pub fn unsupported<S: Into<String>>(msg: S) -> Self {
        FluxError::Unsupported(msg.into())
    }
}