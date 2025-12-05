// Integration tests for validation module

use flux_framework::helpers::validation::*;

#[test]
fn test_ip_validation() {
    // Valid IPv4
    assert!(validate_ip("192.168.1.1").is_ok());
    assert!(validate_ip("10.0.0.1").is_ok());
    assert!(validate_ip("8.8.8.8").is_ok());

    // Valid IPv6
    assert!(validate_ip("::1").is_ok());
    assert!(validate_ip("2001:db8::1").is_ok());
    assert!(validate_ip("fe80::1").is_ok());

    // Invalid IPs
    assert!(validate_ip("256.1.1.1").is_err());
    assert!(validate_ip("192.168.1").is_err());
    assert!(validate_ip("not-an-ip").is_err());
    assert!(validate_ip("").is_err());
}

#[test]
fn test_ip_network_validation() {
    // Valid CIDR notation
    assert!(validate_ip_network("192.168.1.0/24").is_ok());
    assert!(validate_ip_network("10.0.0.0/8").is_ok());
    assert!(validate_ip_network("2001:db8::/32").is_ok());

    // Invalid CIDR
    assert!(validate_ip_network("192.168.1.1/33").is_err());
    assert!(validate_ip_network("invalid/24").is_err());
    assert!(validate_ip_network("not-an-ip").is_err());
}

#[test]
fn test_hostname_validation() {
    // Valid hostnames
    assert!(validate_hostname("localhost").is_ok());
    assert!(validate_hostname("server01").is_ok());
    assert!(validate_hostname("web.example.com").is_ok());
    assert!(validate_hostname("test-server-01.example.com").is_ok());
    assert!(validate_hostname("a").is_ok());

    // Invalid hostnames
    assert!(validate_hostname("").is_err());
    assert!(validate_hostname("-invalid").is_err());
    assert!(validate_hostname("invalid-").is_err());
    assert!(validate_hostname("invalid..com").is_err());
    assert!(validate_hostname(".invalid").is_err());
    assert!(validate_hostname("invalid.").is_err());

    // Too long
    let too_long = "a".repeat(254);
    assert!(validate_hostname(&too_long).is_err());

    // Label too long
    let long_label = format!("{}.com", "a".repeat(64));
    assert!(validate_hostname(&long_label).is_err());
}

#[test]
fn test_port_validation() {
    // Valid ports
    assert_eq!(validate_port("80").unwrap(), 80);
    assert_eq!(validate_port("443").unwrap(), 443);
    assert_eq!(validate_port("8080").unwrap(), 8080);
    assert_eq!(validate_port("65535").unwrap(), 65535);
    assert_eq!(validate_port("1").unwrap(), 1);

    // Invalid ports
    assert!(validate_port("0").is_err());
    assert!(validate_port("65536").is_err());
    assert!(validate_port("-1").is_err());
    assert!(validate_port("not-a-port").is_err());
    assert!(validate_port("").is_err());
}

#[test]
fn test_vlan_validation() {
    // Valid VLANs
    assert_eq!(validate_vlan("1").unwrap(), 1);
    assert_eq!(validate_vlan("100").unwrap(), 100);
    assert_eq!(validate_vlan("4094").unwrap(), 4094);

    // Invalid VLANs
    assert!(validate_vlan("0").is_err());
    assert!(validate_vlan("4095").is_err());
    assert!(validate_vlan("-1").is_err());
    assert!(validate_vlan("not-a-vlan").is_err());
}

#[test]
fn test_username_validation() {
    // Valid usernames
    assert!(validate_username("john").is_ok());
    assert!(validate_username("john_doe").is_ok());
    assert!(validate_username("user123").is_ok());
    assert!(validate_username("_service").is_ok());
    assert!(validate_username("test-user").is_ok());

    // Invalid usernames
    assert!(validate_username("").is_err());
    assert!(validate_username("John").is_err()); // Uppercase
    assert!(validate_username("user@domain").is_err()); // Special char
    assert!(validate_username("123user").is_err()); // Starts with number

    // Reserved usernames
    assert!(validate_username("root").is_err());
    assert!(validate_username("daemon").is_err());
    assert!(validate_username("nobody").is_err());

    // Too long
    let too_long = "a".repeat(33);
    assert!(validate_username(&too_long).is_err());
}

#[test]
fn test_email_validation() {
    // Valid emails
    assert!(validate_email("user@example.com").is_ok());
    assert!(validate_email("john.doe@example.com").is_ok());
    assert!(validate_email("user+tag@example.co.uk").is_ok());
    assert!(validate_email("test_user@sub.example.com").is_ok());

    // Invalid emails
    assert!(validate_email("").is_err());
    assert!(validate_email("not-an-email").is_err());
    assert!(validate_email("@example.com").is_err());
    assert!(validate_email("user@").is_err());
    assert!(validate_email("user").is_err());
}

#[test]
fn test_ssh_key_validation() {
    // Real example keys (truncated for brevity but valid base64)
    let valid_rsa = "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABgQDN0eH7Cq6LKzW/1x/xPqSxU6QoN9k= user@host";
    let valid_ed25519 = "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIGJW5P+7nPdPQ8EMMSfP7wEGV5Tz/E4= user@host";
    let valid_ecdsa = "ecdsa-sha2-nistp256 AAAAE2VjZHNhLXNoYTItbmlzdHAyNTYAAAAIbmlzdHAyNTY= user@host";

    assert!(validate_ssh_key(valid_rsa).is_ok());
    assert!(validate_ssh_key(valid_ed25519).is_ok());
    assert!(validate_ssh_key(valid_ecdsa).is_ok());

    // Invalid SSH keys
    assert!(validate_ssh_key("").is_err());
    assert!(validate_ssh_key("invalid-key").is_err());
    assert!(validate_ssh_key("ssh-rsa").is_err()); // Missing key data
    assert!(validate_ssh_key("ssh-rsa invalid-base64! user@host").is_err());
    assert!(validate_ssh_key("unknown-type AAAAB3NzaC1 user@host").is_err());
}

#[test]
fn test_github_username_validation() {
    // Valid GitHub usernames
    assert!(validate_github_username("torvalds").is_ok());
    assert!(validate_github_username("john-doe").is_ok());
    assert!(validate_github_username("user123").is_ok());
    assert!(validate_github_username("a").is_ok());

    // Invalid GitHub usernames
    assert!(validate_github_username("").is_err());
    assert!(validate_github_username("-user").is_err()); // Starts with hyphen
    assert!(validate_github_username("user-").is_err()); // Ends with hyphen
    assert!(validate_github_username("user--name").is_err()); // Consecutive hyphens
    assert!(validate_github_username("user_name").is_err()); // Underscore not allowed
    assert!(validate_github_username("user@name").is_err()); // Special char

    // Too long
    let too_long = "a".repeat(40);
    assert!(validate_github_username(&too_long).is_err());
}

#[test]
fn test_url_validation() {
    // Valid URLs
    assert!(validate_url("https://example.com").is_ok());
    assert!(validate_url("http://example.com").is_ok());
    assert!(validate_url("https://example.com/path").is_ok());
    assert!(validate_url("https://example.com:8080").is_ok());

    // Invalid URLs
    assert!(validate_url("").is_err());
    assert!(validate_url("not-a-url").is_err());
    assert!(validate_url("example.com").is_err()); // Missing scheme
}

#[test]
fn test_path_validation() {
    // Valid paths
    assert!(validate_path("/home/user").is_ok());
    assert!(validate_path("/etc/config").is_ok());
    assert!(validate_path("relative/path").is_ok());
    assert!(validate_path(".").is_ok());

    // Invalid paths
    assert!(validate_path("\0invalid").is_err()); // Null byte
}
