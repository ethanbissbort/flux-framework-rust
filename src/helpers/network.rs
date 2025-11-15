use crate::error::{FluxError, Result};
use crate::helpers::system::execute_command;
use ipnetwork::IpNetwork;
use pnet::datalink;
use std::collections::HashMap;
use std::net::{IpAddr, TcpStream, ToSocketAddrs};
use std::time::Duration;

/// Network interface information
#[derive(Debug, Clone)]
pub struct NetworkInterface {
    pub name: String,
    pub mac: String,
    pub ips: Vec<IpAddr>,
    pub is_up: bool,
    pub is_loopback: bool,
    pub mtu: u32,
}

/// Get all network interfaces
pub fn get_network_interfaces() -> Result<Vec<NetworkInterface>> {
    let mut interfaces = Vec::new();
    
    for interface in datalink::interfaces() {
        let mut ips = Vec::new();
        
        for ip in &interface.ips {
            match ip {
                ipnetwork::IpNetwork::V4(addr) => ips.push(IpAddr::V4(addr.ip())),
                ipnetwork::IpNetwork::V6(addr) => ips.push(IpAddr::V6(addr.ip())),
            }
        }
        
        // MTU is not available in pnet's NetworkInterface, try to read from sysfs
        let mtu = get_interface_mtu(&interface.name).unwrap_or(0);

        interfaces.push(NetworkInterface {
            name: interface.name.clone(),
            mac: interface.mac.map(|m| m.to_string()).unwrap_or_default(),
            ips,
            is_up: interface.is_up(),
            is_loopback: interface.is_loopback(),
            mtu,
        });
    }
    
    Ok(interfaces)
}

/// Get MTU for a network interface from sysfs
fn get_interface_mtu(name: &str) -> Result<u32> {
    let mtu_path = format!("/sys/class/net/{}/mtu", name);
    let mtu_str = std::fs::read_to_string(mtu_path)
        .map_err(|e| FluxError::system(format!("Failed to read MTU: {}", e)))?;
    mtu_str.trim().parse::<u32>()
        .map_err(|e| FluxError::parse(format!("Failed to parse MTU: {}", e)))
}

/// Get specific network interface
pub fn get_interface(name: &str) -> Result<NetworkInterface> {
    get_network_interfaces()?
        .into_iter()
        .find(|iface| iface.name == name)
        .ok_or_else(|| FluxError::not_found(format!("Interface {} not found", name)))
}

/// Check if port is open
pub fn is_port_open(host: &str, port: u16, timeout_secs: u64) -> bool {
    let addr = format!("{}:{}", host, port);
    
    if let Ok(addrs) = addr.to_socket_addrs() {
        for addr in addrs {
            if TcpStream::connect_timeout(&addr, Duration::from_secs(timeout_secs)).is_ok() {
                return true;
            }
        }
    }
    
    false
}

/// Get DNS servers from resolv.conf
pub fn get_dns_servers() -> Result<Vec<String>> {
    let content = std::fs::read_to_string("/etc/resolv.conf")?;
    let mut servers = Vec::new();
    
    for line in content.lines() {
        if line.trim_start().starts_with("nameserver") {
            if let Some(server) = line.split_whitespace().nth(1) {
                servers.push(server.to_string());
            }
        }
    }
    
    Ok(servers)
}

/// Get default gateway
pub fn get_default_gateway() -> Result<String> {
    let output = execute_command("ip", &["route", "show", "default"])?;
    
    // Parse: default via X.X.X.X dev ...
    for line in output.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 3 && parts[0] == "default" && parts[1] == "via" {
            return Ok(parts[2].to_string());
        }
    }
    
    Err(FluxError::network("No default gateway found"))
}

/// Get routing table
pub fn get_routing_table() -> Result<Vec<RouteEntry>> {
    let output = execute_command("ip", &["route", "show"])?;
    let mut routes = Vec::new();
    
    for line in output.lines() {
        if let Some(route) = parse_route_line(line) {
            routes.push(route);
        }
    }
    
    Ok(routes)
}

/// Routing table entry
#[derive(Debug, Clone)]
pub struct RouteEntry {
    pub destination: String,
    pub gateway: Option<String>,
    pub interface: String,
    pub metric: Option<u32>,
}

fn parse_route_line(line: &str) -> Option<RouteEntry> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    
    if parts.is_empty() {
        return None;
    }
    
    let destination = parts[0].to_string();
    let mut gateway = None;
    let mut interface = String::new();
    let mut metric = None;
    
    let mut i = 1;
    while i < parts.len() {
        match parts[i] {
            "via" => {
                if i + 1 < parts.len() {
                    gateway = Some(parts[i + 1].to_string());
                    i += 2;
                } else {
                    i += 1;
                }
            }
            "dev" => {
                if i + 1 < parts.len() {
                    interface = parts[i + 1].to_string();
                    i += 2;
                } else {
                    i += 1;
                }
            }
            "metric" => {
                if i + 1 < parts.len() {
                    metric = parts[i + 1].parse().ok();
                    i += 2;
                } else {
                    i += 1;
                }
            }
            _ => i += 1,
        }
    }
    
    if interface.is_empty() {
        return None;
    }
    
    Some(RouteEntry {
        destination,
        gateway,
        interface,
        metric,
    })
}

/// Test network connectivity
pub fn test_connectivity() -> NetworkConnectivity {
    let mut connectivity = NetworkConnectivity::default();
    
    // Test gateway
    if let Ok(gateway) = get_default_gateway() {
        connectivity.gateway = gateway.clone();
        connectivity.gateway_reachable = ping_host(&gateway, 1);
    }
    
    // Test DNS
    if let Ok(servers) = get_dns_servers() {
        if let Some(dns) = servers.first() {
            connectivity.dns_server = dns.clone();
            connectivity.dns_reachable = ping_host(dns, 1);
        }
    }
    
    // Test DNS resolution
    connectivity.dns_resolution_working = resolve_hostname("google.com").is_ok();
    
    // Test internet connectivity
    connectivity.internet_reachable = ping_host("8.8.8.8", 1);
    
    connectivity
}

/// Network connectivity status
#[derive(Debug, Default)]
pub struct NetworkConnectivity {
    pub gateway: String,
    pub gateway_reachable: bool,
    pub dns_server: String,
    pub dns_reachable: bool,
    pub dns_resolution_working: bool,
    pub internet_reachable: bool,
}

/// Ping a host (simplified)
fn ping_host(host: &str, timeout_secs: u64) -> bool {
    // Try to connect to common ports
    is_port_open(host, 80, timeout_secs) || 
    is_port_open(host, 443, timeout_secs) ||
    is_port_open(host, 22, timeout_secs)
}

/// Resolve hostname to IP
pub fn resolve_hostname(hostname: &str) -> Result<Vec<IpAddr>> {
    use std::net::ToSocketAddrs;
    
    let addrs = format!("{}:0", hostname)
        .to_socket_addrs()
        .map_err(|e| FluxError::network(format!("Failed to resolve {}: {}", hostname, e)))?;
    
    Ok(addrs.map(|addr| addr.ip()).collect())
}

/// Download file from URL
pub async fn download_file(url: &str, dest: &std::path::Path) -> Result<()> {
    let response = reqwest::get(url)
        .await
        .map_err(|e| FluxError::network(format!("Failed to download {}: {}", url, e)))?;
    
    if !response.status().is_success() {
        return Err(FluxError::network(format!(
            "HTTP error {}: {}",
            response.status(),
            url
        )));
    }
    
    let bytes = response.bytes()
        .await
        .map_err(|e| FluxError::network(format!("Failed to read response: {}", e)))?;
    
    std::fs::write(dest, bytes)?;
    
    Ok(())
}

/// Get network statistics
pub fn get_network_stats(interface: &str) -> Result<NetworkStats> {
    let rx_bytes_path = format!("/sys/class/net/{}/statistics/rx_bytes", interface);
    let tx_bytes_path = format!("/sys/class/net/{}/statistics/tx_bytes", interface);
    let rx_packets_path = format!("/sys/class/net/{}/statistics/rx_packets", interface);
    let tx_packets_path = format!("/sys/class/net/{}/statistics/tx_packets", interface);
    let rx_errors_path = format!("/sys/class/net/{}/statistics/rx_errors", interface);
    let tx_errors_path = format!("/sys/class/net/{}/statistics/tx_errors", interface);
    
    Ok(NetworkStats {
        rx_bytes: std::fs::read_to_string(&rx_bytes_path)?.trim().parse().unwrap_or(0),
        tx_bytes: std::fs::read_to_string(&tx_bytes_path)?.trim().parse().unwrap_or(0),
        rx_packets: std::fs::read_to_string(&rx_packets_path)?.trim().parse().unwrap_or(0),
        tx_packets: std::fs::read_to_string(&tx_packets_path)?.trim().parse().unwrap_or(0),
        rx_errors: std::fs::read_to_string(&rx_errors_path)?.trim().parse().unwrap_or(0),
        tx_errors: std::fs::read_to_string(&tx_errors_path)?.trim().parse().unwrap_or(0),
    })
}

/// Network statistics
#[derive(Debug, Default)]
pub struct NetworkStats {
    pub rx_bytes: u64,
    pub tx_bytes: u64,
    pub rx_packets: u64,
    pub tx_packets: u64,
    pub rx_errors: u64,
    pub tx_errors: u64,
}

/// Check if IPv6 is enabled
pub fn is_ipv6_enabled() -> bool {
    std::path::Path::new("/proc/sys/net/ipv6").exists()
}

/// Get active connections
pub fn get_active_connections() -> Result<Vec<Connection>> {
    let output = execute_command("ss", &["-tupn"])?;
    let mut connections = Vec::new();
    
    for line in output.lines().skip(1) {
        if let Some(conn) = parse_connection_line(line) {
            connections.push(conn);
        }
    }
    
    Ok(connections)
}

/// Network connection info
#[derive(Debug)]
pub struct Connection {
    pub protocol: String,
    pub local_addr: String,
    pub remote_addr: String,
    pub state: String,
    pub process: Option<String>,
}

fn parse_connection_line(line: &str) -> Option<Connection> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    
    if parts.len() < 5 {
        return None;
    }
    
    Some(Connection {
        protocol: parts[0].to_string(),
        local_addr: parts[3].to_string(),
        remote_addr: parts[4].to_string(),
        state: if parts.len() > 5 { parts[5].to_string() } else { String::new() },
        process: if parts.len() > 6 { Some(parts[6].to_string()) } else { None },
    })
}