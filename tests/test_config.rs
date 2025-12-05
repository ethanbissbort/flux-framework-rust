// Integration tests for configuration

use flux_framework::config::{Config, GeneralConfig};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_default_config() {
    let config = Config::default();

    // Check general config defaults
    assert_eq!(config.general.default_ssh_port, 22);
    assert_eq!(config.general.default_admin_user, "fluxadmin");
    assert!(config.general.default_admin_groups.contains(&"sudo".to_string()));
    assert_eq!(config.general.log_level, "info");
    assert_eq!(config.general.colored_output, true);
    assert_eq!(config.general.default_dns, vec!["1.1.1.1", "8.8.8.8"]);
}

#[test]
fn test_general_config_default() {
    let general = GeneralConfig::default();

    assert_eq!(general.default_ssh_port, 22);
    assert_eq!(general.default_admin_user, "fluxadmin");
    assert_eq!(general.log_level, "info");
    assert_eq!(general.colored_output, true);
}

#[test]
fn test_config_get_set() {
    let mut config = Config::default();

    // Test getting non-existent key
    assert!(config.get("nonexistent").is_none());

    // Test getting existing general config
    assert_eq!(config.get("default_ssh_port").unwrap(), "22");
    assert_eq!(config.get("default_admin_user").unwrap(), "fluxadmin");
    assert_eq!(config.get("log_level").unwrap(), "info");

    // Test setting general config
    config.set("default_ssh_port", "2222").unwrap();
    assert_eq!(config.get("default_ssh_port").unwrap(), "2222");
    assert_eq!(config.general.default_ssh_port, 2222);

    config.set("default_admin_user", "admin").unwrap();
    assert_eq!(config.get("default_admin_user").unwrap(), "admin");

    config.set("log_level", "debug").unwrap();
    assert_eq!(config.get("log_level").unwrap(), "debug");

    config.set("colored_output", "false").unwrap();
    assert_eq!(config.get("colored_output").unwrap(), "false");
    assert_eq!(config.general.colored_output, false);

    // Test setting custom values
    config.set("custom_key", "custom_value").unwrap();
    assert_eq!(config.get("custom_key").unwrap(), "custom_value");
}

#[test]
fn test_config_all() {
    let config = Config::default();
    let all_values = config.all();

    // Should have at least the general config values
    assert!(all_values.len() >= 4);

    // Should be sorted
    let keys: Vec<_> = all_values.iter().map(|(k, _)| k.as_str()).collect();
    let mut sorted_keys = keys.clone();
    sorted_keys.sort();
    assert_eq!(keys, sorted_keys);
}

#[test]
fn test_config_save_load() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("test_config.toml");

    // Create and save config
    let mut config = Config::default();
    config.set("default_ssh_port", "2222").unwrap();
    config.set("custom_key", "custom_value").unwrap();

    // Manually set config path for testing
    let config_str = toml::to_string_pretty(&config).unwrap();
    fs::write(&config_path, config_str).unwrap();

    // Load config
    let loaded_config = Config::from_file(&config_path).unwrap();

    assert_eq!(loaded_config.general.default_ssh_port, 2222);
    assert_eq!(loaded_config.get("custom_key").unwrap(), "custom_value");
}

#[test]
fn test_config_invalid_port() {
    let mut config = Config::default();

    // Invalid port number
    let result = config.set("default_ssh_port", "invalid");
    assert!(result.is_err());

    // Port unchanged
    assert_eq!(config.general.default_ssh_port, 22);
}

#[test]
fn test_config_invalid_boolean() {
    let mut config = Config::default();

    // Invalid boolean
    let result = config.set("colored_output", "invalid");
    assert!(result.is_err());

    // Value unchanged
    assert_eq!(config.general.colored_output, true);
}

#[test]
fn test_config_from_nonexistent_file() {
    let result = Config::from_file("/nonexistent/path/config.toml");
    assert!(result.is_err());
}

#[test]
fn test_config_module_config() {
    let mut config = Config::default();

    // Get non-existent module config
    assert!(config.get_module_config("nonexistent").is_none());

    // Set module config
    let module_config = toml::Value::String("test_value".to_string());
    config.set_module_config("test_module", module_config.clone());

    // Get module config
    let retrieved = config.get_module_config("test_module");
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap(), &module_config);
}
