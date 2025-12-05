use crate::error::{FluxError, Result};
use ipnetwork::IpNetwork;
use regex::Regex;
use std::net::IpAddr;
use std::str::FromStr;
use base64::{Engine as _, engine::general_purpose};

lazy_static::lazy_static! {
    /// Hostname validation regex
    static ref HOSTNAME_REGEX: Regex = Regex::new(
        r"^[a-zA-Z0-9]([a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(\.[a-zA-Z0-9]([a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*$"
    ).unwrap();
    
    /// Username validation regex
    static ref USERNAME_REGEX: Regex = Regex::new(
        r"^[a-z_][a-z0-9_-]*$"
    ).unwrap();
    
    /// Email validation regex
    static ref EMAIL_REGEX: Regex = Regex::new(
        r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$"
    ).unwrap();
}

/// Validate IP address
pub fn validate_ip(ip: &str) -> Result<IpAddr> {
    IpAddr::from_str(ip)
        .map_err(|_| FluxError::validation(format!("Invalid IP address: {}", ip)))
}

/// Validate IP address with CIDR notation
pub fn validate_ip_network(network: &str) -> Result<IpNetwork> {
    IpNetwork::from_str(network)
        .map_err(|_| FluxError::validation(format!("Invalid IP network: {}", network)))
}

/// Validate hostname/FQDN
pub fn validate_hostname(hostname: &str) -> Result<()> {
    // Check length
    if hostname.is_empty() || hostname.len() > 253 {
        return Err(FluxError::validation(
            "Hostname must be between 1 and 253 characters",
        ));
    }
    
    // Check format
    if !HOSTNAME_REGEX.is_match(hostname) {
        return Err(FluxError::validation(
            "Invalid hostname format. Use only letters, numbers, hyphens, and dots",
        ));
    }
    
    // Check individual labels
    for label in hostname.split('.') {
        if label.len() > 63 {
            return Err(FluxError::validation(
                "Each hostname label must be 63 characters or less",
            ));
        }
    }
    
    Ok(())
}

/// Validate port number
pub fn validate_port(port: &str) -> Result<u16> {
    let port_num = port
        .parse::<u16>()
        .map_err(|_| FluxError::validation("Port must be a number"))?;
    
    if port_num == 0 || port_num > 65535 {
        return Err(FluxError::validation("Port must be between 1 and 65535"));
    }
    
    Ok(port_num)
}

/// Validate VLAN ID
pub fn validate_vlan(vlan: &str) -> Result<u16> {
    let vlan_id = vlan
        .parse::<u16>()
        .map_err(|_| FluxError::validation("VLAN ID must be a number"))?;
    
    if vlan_id < 1 || vlan_id > 4094 {
        return Err(FluxError::validation("VLAN ID must be between 1 and 4094"));
    }
    
    Ok(vlan_id)
}

/// Validate username
pub fn validate_username(username: &str) -> Result<()> {
    // Check length
    if username.is_empty() || username.len() > 32 {
        return Err(FluxError::validation(
            "Username must be between 1 and 32 characters",
        ));
    }
    
    // Check format
    if !USERNAME_REGEX.is_match(username) {
        return Err(FluxError::validation(
            "Username must start with lowercase letter or underscore, \
             followed by lowercase letters, numbers, dash, or underscore",
        ));
    }
    
    // Check for reserved names
    let reserved = [
        "root", "daemon", "bin", "sys", "sync", "games", "man", "lp",
        "mail", "news", "uucp", "proxy", "www-data", "backup", "list",
        "irc", "gnats", "nobody", "systemd-network", "systemd-resolve",
    ];
    
    if reserved.contains(&username) {
        return Err(FluxError::validation(format!(
            "Username '{}' is reserved",
            username
        )));
    }
    
    Ok(())
}

/// Validate email address
pub fn validate_email(email: &str) -> Result<()> {
    if !EMAIL_REGEX.is_match(email) {
        return Err(FluxError::validation("Invalid email address format"));
    }
    
    Ok(())
}

/// Validate network interface name
pub fn validate_interface(interface: &str) -> Result<()> {
    // Check if interface exists
    let sys_path = format!("/sys/class/net/{}", interface);
    if !std::path::Path::new(&sys_path).exists() {
        return Err(FluxError::validation(format!(
            "Network interface '{}' does not exist",
            interface
        )));
    }
    
    Ok(())
}

/// Validate file path
pub fn validate_path(path: &str) -> Result<std::path::PathBuf> {
    let path = std::path::PathBuf::from(path);
    
    // Check if path contains invalid characters
    if path.to_string_lossy().contains('\0') {
        return Err(FluxError::validation("Path contains null characters"));
    }
    
    Ok(path)
}

/// Validate SSH public key
pub fn validate_ssh_key(key: &str) -> Result<()> {
    // Basic SSH key format validation
    let parts: Vec<&str> = key.split_whitespace().collect();
    
    if parts.is_empty() {
        return Err(FluxError::validation("Empty SSH key"));
    }
    
    // Check key type
    let valid_types = [
        "ssh-rsa",
        "ssh-dss",
        "ssh-ed25519",
        "ecdsa-sha2-nistp256",
        "ecdsa-sha2-nistp384",
        "ecdsa-sha2-nistp521",
    ];
    
    if !valid_types.contains(&parts[0]) {
        return Err(FluxError::validation(format!(
            "Invalid SSH key type: {}",
            parts[0]
        )));
    }
    
    // Check if key data is present
    if parts.len() < 2 {
        return Err(FluxError::validation("SSH key missing key data"));
    }
    
    // Try to decode base64 key data
    if general_purpose::STANDARD.decode(parts[1]).is_err() {
        return Err(FluxError::validation("Invalid SSH key data (not valid base64)"));
    }
    
    Ok(())
}

/// Validate URL
pub fn validate_url(url: &str) -> Result<reqwest::Url> {
    reqwest::Url::parse(url)
        .map_err(|e| FluxError::validation(format!("Invalid URL: {}", e)))
}

/// Validate GitHub username
pub fn validate_github_username(username: &str) -> Result<()> {
    // GitHub username rules:
    // - May only contain alphanumeric characters or hyphens
    // - Cannot have multiple consecutive hyphens
    // - Cannot begin or end with a hyphen
    // - Maximum 39 characters
    
    if username.is_empty() || username.len() > 39 {
        return Err(FluxError::validation(
            "GitHub username must be between 1 and 39 characters",
        ));
    }
    
    if username.starts_with('-') || username.ends_with('-') {
        return Err(FluxError::validation(
            "GitHub username cannot start or end with a hyphen",
        ));
    }
    
    if username.contains("--") {
        return Err(FluxError::validation(
            "GitHub username cannot contain consecutive hyphens",
        ));
    }
    
    let github_regex = Regex::new(r"^[a-zA-Z0-9-]+$").unwrap();
    if !github_regex.is_match(username) {
        return Err(FluxError::validation(
            "GitHub username can only contain alphanumeric characters and hyphens",
        ));
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_validate_ip() {
        assert!(validate_ip("192.168.1.1").is_ok());
        assert!(validate_ip("10.0.0.1").is_ok());
        assert!(validate_ip("::1").is_ok());
        assert!(validate_ip("2001:db8::1").is_ok());
        
        assert!(validate_ip("256.1.1.1").is_err());
        assert!(validate_ip("192.168.1").is_err());
        assert!(validate_ip("not-an-ip").is_err());
    }
    
    #[test]
    fn test_validate_hostname() {
        assert!(validate_hostname("localhost").is_ok());
        assert!(validate_hostname("server01").is_ok());
        assert!(validate_hostname("web.example.com").is_ok());
        assert!(validate_hostname("test-server-01.example.com").is_ok());
        
        assert!(validate_hostname("").is_err());
        assert!(validate_hostname("-invalid").is_err());
        assert!(validate_hostname("invalid-").is_err());
        assert!(validate_hostname("invalid..com").is_err());
    }
    
    #[test]
    fn test_validate_username() {
        assert!(validate_username("john").is_ok());
        assert!(validate_username("john_doe").is_ok());
        assert!(validate_username("user123").is_ok());
        assert!(validate_username("_service").is_ok());
        
        assert!(validate_username("").is_err());
        assert!(validate_username("root").is_err());
        assert!(validate_username("John").is_err());
        assert!(validate_username("user@domain").is_err());
    }
}