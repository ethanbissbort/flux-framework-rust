// Integration tests for error handling

use flux_framework::error::{FluxError, Result};

#[test]
fn test_error_creation() {
    // Test config error
    let err = FluxError::config("test config error");
    assert!(matches!(err, FluxError::Config(_)));
    assert_eq!(err.to_string(), "Configuration error: test config error");

    // Test module error
    let err = FluxError::module("test module error");
    assert!(matches!(err, FluxError::Module(_)));
    assert_eq!(err.to_string(), "Module error: test module error");

    // Test validation error
    let err = FluxError::validation("test validation error");
    assert!(matches!(err, FluxError::Validation(_)));
    assert_eq!(err.to_string(), "Validation error: test validation error");

    // Test system error
    let err = FluxError::system("test system error");
    assert!(matches!(err, FluxError::System(_)));
    assert_eq!(err.to_string(), "System error: test system error");

    // Test network error
    let err = FluxError::network("test network error");
    assert!(matches!(err, FluxError::Network(_)));
    assert_eq!(err.to_string(), "Network error: test network error");

    // Test permission error
    let err = FluxError::permission("test permission error");
    assert!(matches!(err, FluxError::Permission(_)));
    assert_eq!(err.to_string(), "Permission denied: test permission error");

    // Test not found error
    let err = FluxError::not_found("test not found");
    assert!(matches!(err, FluxError::NotFound(_)));
    assert_eq!(err.to_string(), "Not found: test not found");

    // Test command failed error
    let err = FluxError::command_failed("test command failed");
    assert!(matches!(err, FluxError::CommandFailed(_)));
    assert_eq!(err.to_string(), "Command execution failed: test command failed");

    // Test parse error
    let err = FluxError::parse("test parse error");
    assert!(matches!(err, FluxError::Parse(_)));
    assert_eq!(err.to_string(), "Parse error: test parse error");

    // Test SSH error
    let err = FluxError::ssh("test ssh error");
    assert!(matches!(err, FluxError::Ssh(_)));
    assert_eq!(err.to_string(), "SSH error: test ssh error");

    // Test unsupported error
    let err = FluxError::unsupported("test unsupported");
    assert!(matches!(err, FluxError::Unsupported(_)));
    assert_eq!(err.to_string(), "Unsupported operation: test unsupported");

    // Test user cancelled
    let err = FluxError::UserCancelled;
    assert!(matches!(err, FluxError::UserCancelled));
    assert_eq!(err.to_string(), "User cancelled operation");
}

#[test]
fn test_io_error_conversion() {
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
    let flux_err: FluxError = io_err.into();
    assert!(matches!(flux_err, FluxError::Io(_)));
}

#[test]
fn test_result_type() {
    fn returns_ok() -> Result<String> {
        Ok("success".to_string())
    }

    fn returns_err() -> Result<String> {
        Err(FluxError::config("error"))
    }

    assert!(returns_ok().is_ok());
    assert_eq!(returns_ok().unwrap(), "success");

    assert!(returns_err().is_err());
    match returns_err() {
        Err(FluxError::Config(msg)) => assert_eq!(msg, "error"),
        _ => panic!("Expected Config error"),
    }
}

#[test]
fn test_error_propagation() {
    fn inner_function() -> Result<()> {
        Err(FluxError::validation("inner error"))
    }

    fn outer_function() -> Result<()> {
        inner_function()?;
        Ok(())
    }

    let result = outer_function();
    assert!(result.is_err());
    match result {
        Err(FluxError::Validation(msg)) => assert_eq!(msg, "inner error"),
        _ => panic!("Expected Validation error"),
    }
}
