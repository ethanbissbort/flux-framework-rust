use crate::error::{FluxError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::process::Command;
use sysinfo::{DiskExt, System, SystemExt};

/// Linux distribution types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Distribution {
    Ubuntu,
    Debian,
    Mint,
    Pop,
    CentOS,
    Fedora,
    RHEL,
    Rocky,
    AlmaLinux,
    Unknown(String),
}

impl Distribution {
    /// Check if this is a Debian-based distribution
    pub fn is_debian_based(&self) -> bool {
        matches!(
            self,
            Distribution::Ubuntu
                | Distribution::Debian
                | Distribution::Mint
                | Distribution::Pop
        )
    }
    
    /// Check if this is a RedHat-based distribution
    pub fn is_redhat_based(&self) -> bool {
        matches!(
            self,
            Distribution::CentOS
                | Distribution::Fedora
                | Distribution::RHEL
                | Distribution::Rocky
                | Distribution::AlmaLinux
        )
    }
    
    /// Get the package manager for this distribution
    pub fn package_manager(&self) -> &str {
        if self.is_debian_based() {
            "apt"
        } else if self.is_redhat_based() {
            if which::which("dnf").is_ok() {
                "dnf"
            } else {
                "yum"
            }
        } else {
            "unknown"
        }
    }
}

/// System status information
#[derive(Debug, Serialize, Deserialize)]
pub struct SystemStatus {
    pub os_info: String,
    pub kernel_version: String,
    pub architecture: String,
    pub hostname: String,
    pub cpu_load: String,
    pub memory_usage: String,
    pub disk_usage: String,
    pub primary_ip: String,
    pub gateway: String,
    pub services: HashMap<String, bool>,
    pub reboot_required: bool,
    pub updates_available: u32,
}

/// Detect the Linux distribution
pub fn detect_distro() -> Result<Distribution> {
    // Try /etc/os-release first (most modern systems)
    if let Ok(contents) = fs::read_to_string("/etc/os-release") {
        for line in contents.lines() {
            if line.starts_with("ID=") {
                let id = line.trim_start_matches("ID=").trim_matches('"').to_lowercase();
                return Ok(match id.as_str() {
                    "ubuntu" => Distribution::Ubuntu,
                    "debian" => Distribution::Debian,
                    "linuxmint" => Distribution::Mint,
                    "pop" => Distribution::Pop,
                    "centos" => Distribution::CentOS,
                    "fedora" => Distribution::Fedora,
                    "rhel" => Distribution::RHEL,
                    "rocky" => Distribution::Rocky,
                    "almalinux" => Distribution::AlmaLinux,
                    other => Distribution::Unknown(other.to_string()),
                });
            }
        }
    }
    
    // Fallback to other methods
    if fs::read_to_string("/etc/debian_version").is_ok() {
        return Ok(Distribution::Debian);
    }
    
    if fs::read_to_string("/etc/redhat-release").is_ok() {
        return Ok(Distribution::RHEL);
    }
    
    Ok(Distribution::Unknown("unknown".to_string()))
}

/// Check if running as root
pub fn is_root() -> bool {
    nix::unistd::Uid::effective().is_root()
}

/// Check if systemd is available
pub fn has_systemd() -> bool {
    which::which("systemctl").is_ok()
}

/// Get system information
pub fn get_system_info() -> Result<sysinfo::System> {
    let mut sys = System::new_all();
    sys.refresh_all();
    Ok(sys)
}

/// Get hostname
pub fn get_hostname() -> Result<String> {
    hostname::get()
        .map_err(|e| FluxError::system(format!("Failed to get hostname: {}", e)))?
        .to_string_lossy()
        .into_owned()
        .into()
}

/// Get system status
pub fn get_system_status() -> Result<SystemStatus> {
    let mut sys = System::new_all();
    sys.refresh_all();
    
    // OS information
    let os_info = format!(
        "{} {}",
        sys.name().unwrap_or_else(|| "Unknown".to_string()),
        sys.os_version().unwrap_or_else(|| "".to_string())
    );
    
    let kernel_version = sys.kernel_version().unwrap_or_else(|| "Unknown".to_string());
    let architecture = std::env::consts::ARCH.to_string();
    let hostname = get_hostname()?;
    
    // CPU load
    let load_avg = sys.load_average();
    let cpu_load = format!("{:.2} {:.2} {:.2}", load_avg.one, load_avg.five, load_avg.fifteen);
    
    // Memory usage
    let total_mem = sys.total_memory();
    let used_mem = sys.used_memory();
    let mem_percent = (used_mem as f64 / total_mem as f64) * 100.0;
    let memory_usage = format!(
        "{:.1}GB/{:.1}GB ({:.0}%)",
        used_mem as f64 / 1024.0 / 1024.0 / 1024.0,
        total_mem as f64 / 1024.0 / 1024.0 / 1024.0,
        mem_percent
    );
    
    // Disk usage for root partition
    let mut disk_usage = "Unknown".to_string();
    for disk in sys.disks() {
        if disk.mount_point() == std::path::Path::new("/") {
            let total = disk.total_space();
            let available = disk.available_space();
            let used = total - available;
            let percent = (used as f64 / total as f64) * 100.0;
            disk_usage = format!(
                "{:.1}GB/{:.1}GB ({:.0}%)",
                used as f64 / 1024.0 / 1024.0 / 1024.0,
                total as f64 / 1024.0 / 1024.0 / 1024.0,
                percent
            );
            break;
        }
    }
    
    // Network information
    let primary_ip = get_primary_ip().unwrap_or_else(|_| "Unknown".to_string());
    let gateway = get_default_gateway().unwrap_or_else(|_| "Unknown".to_string());
    
    // Check services
    let mut services = HashMap::new();
    for service in &["ssh", "ufw", "fail2ban", "docker", "netdata"] {
        services.insert(
            service.to_string(),
            is_service_active(service).unwrap_or(false),
        );
    }
    
    // Check if reboot is required
    let reboot_required = fs::metadata("/var/run/reboot-required").is_ok();
    
    // Check for updates (simplified)
    let updates_available = check_updates_available().unwrap_or(0);
    
    Ok(SystemStatus {
        os_info,
        kernel_version,
        architecture,
        hostname,
        cpu_load,
        memory_usage,
        disk_usage,
        primary_ip,
        gateway,
        services,
        reboot_required,
        updates_available,
    })
}

/// Get primary IP address
pub fn get_primary_ip() -> Result<String> {
    // Try to get IP from hostname command first
    if let Ok(output) = Command::new("hostname").arg("-I").output() {
        if output.status.success() {
            if let Ok(ips) = String::from_utf8(output.stdout) {
                if let Some(ip) = ips.split_whitespace().next() {
                    return Ok(ip.to_string());
                }
            }
        }
    }
    
    // Fallback: get IP from default route
    if let Ok(output) = Command::new("ip").args(&["route", "get", "1"]).output() {
        if output.status.success() {
            if let Ok(route) = String::from_utf8(output.stdout) {
                for part in route.split_whitespace() {
                    if let Ok(addr) = part.parse::<std::net::IpAddr>() {
                        if !addr.is_loopback() {
                            return Ok(addr.to_string());
                        }
                    }
                }
            }
        }
    }
    
    Err(FluxError::network("Could not determine primary IP address"))
}

/// Get default gateway
pub fn get_default_gateway() -> Result<String> {
    if let Ok(output) = Command::new("ip").args(&["route", "show", "default"]).output() {
        if output.status.success() {
            if let Ok(route) = String::from_utf8(output.stdout) {
                // Parse: default via X.X.X.X dev ...
                let parts: Vec<&str> = route.split_whitespace().collect();
                if parts.len() >= 3 && parts[0] == "default" && parts[1] == "via" {
                    return Ok(parts[2].to_string());
                }
            }
        }
    }
    
    Err(FluxError::network("Could not determine default gateway"))
}

/// Check if a service is active
pub fn is_service_active(service: &str) -> Result<bool> {
    if !has_systemd() {
        return Err(FluxError::unsupported("systemd not available"));
    }
    
    let output = Command::new("systemctl")
        .args(&["is-active", service])
        .output()?;
    
    Ok(output.status.success())
}

/// Check for available updates
pub fn check_updates_available() -> Result<u32> {
    let distro = detect_distro()?;
    
    if distro.is_debian_based() {
        // Check apt for updates
        if let Ok(output) = Command::new("apt")
            .args(&["list", "--upgradable"])
            .output()
        {
            if output.status.success() {
                if let Ok(list) = String::from_utf8(output.stdout) {
                    // Count lines that contain "upgradable"
                    let count = list.lines()
                        .filter(|line| line.contains("upgradable"))
                        .count();
                    return Ok(count as u32);
                }
            }
        }
    } else if distro.is_redhat_based() {
        // Check yum/dnf for updates
        let pkg_manager = if which::which("dnf").is_ok() { "dnf" } else { "yum" };
        
        if let Ok(output) = Command::new(pkg_manager)
            .args(&["check-update"])
            .output()
        {
            // yum/dnf returns 100 when updates are available
            if output.status.code() == Some(100) {
                if let Ok(list) = String::from_utf8(output.stdout) {
                    // Count non-empty lines after the header
                    let count = list.lines()
                        .skip_while(|line| !line.is_empty())
                        .skip(1)
                        .filter(|line| !line.is_empty() && !line.starts_with(' '))
                        .count();
                    return Ok(count as u32);
                }
            }
        }
    }
    
    Ok(0)
}

/// Execute a command with error handling
pub fn execute_command(command: &str, args: &[&str]) -> Result<String> {
    let output = Command::new(command)
        .args(args)
        .output()
        .map_err(|e| FluxError::command_failed(format!("Failed to execute {}: {}", command, e)))?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(FluxError::command_failed(format!(
            "{} failed: {}",
            command, stderr
        )));
    }
    
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

/// Check if a command exists
pub fn command_exists(command: &str) -> bool {
    which::which(command).is_ok()
}

/// Get system uptime
pub fn get_uptime() -> Result<String> {
    let sys = System::new();
    let uptime_seconds = sys.uptime();
    
    let days = uptime_seconds / 86400;
    let hours = (uptime_seconds % 86400) / 3600;
    let minutes = (uptime_seconds % 3600) / 60;
    
    if days > 0 {
        Ok(format!("{} days, {} hours, {} minutes", days, hours, minutes))
    } else if hours > 0 {
        Ok(format!("{} hours, {} minutes", hours, minutes))
    } else {
        Ok(format!("{} minutes", minutes))
    }
}