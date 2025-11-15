use crate::error::{FluxError, Result};
use crate::helpers::logging::{log_info, log_warn};
use crate::helpers::user_input::prompt_yes_no;
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::process::Command;
use sysinfo::System;

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
    let hostname = hostname::get()
        .map_err(|e| FluxError::system(format!("Failed to get hostname: {}", e)))?
        .to_string_lossy()
        .into_owned();
    Ok(hostname)
}

/// Get system status
pub fn get_system_status() -> Result<SystemStatus> {
    let mut sys = System::new_all();
    sys.refresh_all();

    // OS information
    let os_info = format!(
        "{} {}",
        System::name().unwrap_or_else(|| "Unknown".to_string()),
        System::os_version().unwrap_or_else(|| "".to_string())
    );

    let kernel_version = System::kernel_version().unwrap_or_else(|| "Unknown".to_string());
    let architecture = std::env::consts::ARCH.to_string();
    let hostname = get_hostname()?;

    // CPU load
    let load_avg = System::load_average();
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
    use sysinfo::Disks;
    let mut disk_usage = "Unknown".to_string();
    let disks = Disks::new_with_refreshed_list();
    for disk in &disks {
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

/// Check if a command exists and return Result
pub fn check_command(command: &str) -> Result<bool> {
    Ok(which::which(command).is_ok())
}

/// Get OS information as a string
pub fn get_os_info() -> Result<String> {
    let os_name = System::name().unwrap_or_else(|| "Unknown".to_string());
    let os_version = System::os_version().unwrap_or_else(|| "".to_string());
    let kernel_version = System::kernel_version().unwrap_or_else(|| "Unknown".to_string());

    Ok(format!("{} {} (kernel {})", os_name, os_version, kernel_version))
}

/// Restart a systemd service
pub fn restart_service(service: &str) -> Result<()> {
    if !has_systemd() {
        return Err(FluxError::unsupported("systemd not available"));
    }

    log_info(&format!("Restarting service: {}", service));

    let output = Command::new("systemctl")
        .args(&["restart", service])
        .output()
        .map_err(|e| FluxError::command_failed(format!("Failed to restart {}: {}", service, e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(FluxError::command_failed(format!(
            "Failed to restart {}: {}",
            service, stderr
        )));
    }

    Ok(())
}

/// Get system uptime
pub fn get_uptime() -> Result<String> {
    let uptime_seconds = System::uptime();
    
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


// =============================================================================
// MISSING FUNCTIONS IN src/helpers/system.rs
// =============================================================================

/// Check if system reboot is required and optionally prompt user
/// Called from: src/workflows/mod.rs, src/modules/update.rs
pub fn check_reboot_needed() -> Result<()> {
    log_info("Checking if system reboot is required");
    
    let reboot_required = is_reboot_required()?;
    
    if reboot_required {
        log_warn("System reboot is required");
        println!("{}", "⚠ System reboot is required".yellow());
        
        if prompt_yes_no("Reboot system now?", false)? {
            log_info("Initiating system reboot");
            execute_reboot()?;
        } else {
            log_info("Reboot postponed by user");
            println!("{}", "Remember to reboot later to complete the configuration".yellow());
        }
    } else {
        log_info("No reboot required");
    }
    
    Ok(())
}

/// Check if reboot is required by examining system files
fn is_reboot_required() -> Result<bool> {
    // Check for Debian/Ubuntu reboot-required file
    if std::path::Path::new("/var/run/reboot-required").exists() {
        return Ok(true);
    }
    
    // Check for Red Hat/CentOS kernel updates
    if let Ok(output) = Command::new("needs-restarting").arg("-r").output() {
        if output.status.code() == Some(1) {
            return Ok(true);
        }
    }
    
    // Check if running kernel differs from installed kernel
    if let Ok(running_kernel) = std::fs::read_to_string("/proc/version") {
        if let Ok(installed_kernel) = get_installed_kernel_version() {
            if !running_kernel.contains(&installed_kernel) {
                return Ok(true);
            }
        }
    }
    
    Ok(false)
}

/// Get the version of the installed kernel
fn get_installed_kernel_version() -> Result<String> {
    // Try different methods based on distribution
    
    // Method 1: dpkg (Debian/Ubuntu)
    if let Ok(output) = Command::new("dpkg")
        .args(&["-l", "linux-image-*"])
        .output() 
    {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            // Parse dpkg output for latest kernel version
            // This is a simplified implementation
            for line in stdout.lines() {
                if line.contains("linux-image-") && line.contains("ii") {
                    if let Some(version) = extract_kernel_version_from_dpkg_line(line) {
                        return Ok(version);
                    }
                }
            }
        }
    }
    
    // Method 2: rpm (Red Hat/CentOS)
    if let Ok(output) = Command::new("rpm")
        .args(&["-q", "kernel", "--last"])
        .output()
    {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if let Some(first_line) = stdout.lines().next() {
                if let Some(version) = extract_kernel_version_from_rpm_line(first_line) {
                    return Ok(version);
                }
            }
        }
    }
    
    Err(FluxError::system("Could not determine installed kernel version"))
}

/// Extract kernel version from dpkg output line
fn extract_kernel_version_from_dpkg_line(line: &str) -> Option<String> {
    // Example line: "ii  linux-image-5.4.0-84-generic  5.4.0-84.94  amd64"
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() >= 2 {
        let package_name = parts[1];
        if let Some(version_start) = package_name.find("linux-image-") {
            let version = &package_name[version_start + "linux-image-".len()..];
            return Some(version.to_string());
        }
    }
    None
}

/// Extract kernel version from rpm output line
fn extract_kernel_version_from_rpm_line(line: &str) -> Option<String> {
    // Example line: "kernel-5.4.17-2102.201.3.el8uek.x86_64"
    if let Some(version_start) = line.find("kernel-") {
        let rest = &line[version_start + "kernel-".len()..];
        if let Some(arch_pos) = rest.rfind(".x86_64") {
            let version = &rest[..arch_pos];
            return Some(version.to_string());
        }
    }
    None
}

/// Execute system reboot
fn execute_reboot() -> Result<()> {
    log_info("Executing system reboot in 10 seconds...");
    println!("{}", "System will reboot in 10 seconds...".red());
    
    // Give users a chance to cancel
    std::thread::sleep(std::time::Duration::from_secs(10));
    
    Command::new("reboot")
        .output()
        .map_err(|e| FluxError::command_failed(format!("Failed to execute reboot: {}", e)))?;
    
    Ok(())
}

/// Enhanced service status check with better error handling
/// Improves existing is_service_active function
pub fn is_service_active_enhanced(service: &str) -> Result<bool> {
    // First check if systemd is available
    if !has_systemd() {
        return check_service_sysvinit(service);
    }
    
    // Use systemctl to check service status
    match Command::new("systemctl")
        .args(&["is-active", service])
        .output()
    {
        Ok(output) => {
            let status = String::from_utf8_lossy(&output.stdout).trim().to_lowercase();
            Ok(status == "active")
        }
        Err(e) => Err(FluxError::system(format!(
            "Failed to check service status for {}: {}", service, e
        )))
    }
}

/// Check service status using SysV init (fallback for non-systemd systems)
fn check_service_sysvinit(service: &str) -> Result<bool> {
    // Try service command
    if let Ok(output) = Command::new("service")
        .args(&[service, "status"])
        .output()
    {
        return Ok(output.status.success());
    }
    
    // Try init.d script directly
    let init_script = format!("/etc/init.d/{}", service);
    if std::path::Path::new(&init_script).exists() {
        if let Ok(output) = Command::new(&init_script)
            .arg("status")
            .output()
        {
            return Ok(output.status.success());
        }
    }
    
    Err(FluxError::system(format!(
        "Could not determine status of service: {}", service
    )))
}

/// Get comprehensive system status with additional metrics
/// Enhances existing get_system_status function
pub fn get_system_status_enhanced() -> Result<EnhancedSystemStatus> {
    let basic_status = crate::helpers::system::get_system_status()?;
    
    Ok(EnhancedSystemStatus {
        basic: basic_status,
        kernel_version: get_running_kernel_version()?,
        installed_kernel: get_installed_kernel_version().unwrap_or_else(|_| "unknown".to_string()),
        reboot_required: is_reboot_required()?,
        security_updates: count_security_updates()?,
        failed_services: get_failed_services()?,
        system_load_1min: get_system_load_1min()?,
        swap_usage: get_swap_usage()?,
        inodes_usage: get_inodes_usage()?,
        zombie_processes: count_zombie_processes()?,
    })
}

/// Enhanced system status structure
#[derive(Debug)]
pub struct EnhancedSystemStatus {
    pub basic: crate::helpers::system::SystemStatus,
    pub kernel_version: String,
    pub installed_kernel: String,
    pub reboot_required: bool,
    pub security_updates: u32,
    pub failed_services: Vec<String>,
    pub system_load_1min: f64,
    pub swap_usage: SwapUsage,
    pub inodes_usage: HashMap<String, InodesUsage>,
    pub zombie_processes: u32,
}

#[derive(Debug)]
pub struct SwapUsage {
    pub total: u64,
    pub used: u64,
    pub free: u64,
    pub percentage: f64,
}

#[derive(Debug)]
pub struct InodesUsage {
    pub total: u64,
    pub used: u64,
    pub free: u64,
    pub percentage: f64,
}

/// Get running kernel version
fn get_running_kernel_version() -> Result<String> {
    let version = std::fs::read_to_string("/proc/version")?;
    // Extract version from: "Linux version 5.4.0-84-generic ..."
    if let Some(start) = version.find("Linux version ") {
        let rest = &version[start + "Linux version ".len()..];
        if let Some(end) = rest.find(' ') {
            return Ok(rest[..end].to_string());
        }
    }
    Err(FluxError::system("Could not parse kernel version"))
}

/// Count available security updates
fn count_security_updates() -> Result<u32> {
    let distro = crate::helpers::system::detect_distro()?;
    
    if distro.is_debian_based() {
        // Check for security updates in apt
        if let Ok(output) = Command::new("apt")
            .args(&["list", "--upgradable"])
            .output()
        {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let count = stdout.lines()
                    .filter(|line| line.contains("-security"))
                    .count();
                return Ok(count as u32);
            }
        }
    } else if distro.is_redhat_based() {
        // Check for security updates with yum/dnf
        let pkg_manager = if which::which("dnf").is_ok() { "dnf" } else { "yum" };
        
        if let Ok(output) = Command::new(pkg_manager)
            .args(&["updateinfo", "list", "security"])
            .output()
        {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let count = stdout.lines()
                    .filter(|line| !line.trim().is_empty() && !line.starts_with("Last metadata"))
                    .count();
                return Ok(count as u32);
            }
        }
    }
    
    Ok(0)
}

/// Get list of failed systemd services
fn get_failed_services() -> Result<Vec<String>> {
    if !has_systemd() {
        return Ok(Vec::new());
    }
    
    let output = Command::new("systemctl")
        .args(&["--failed", "--no-legend", "--no-pager"])
        .output()
        .map_err(|e| FluxError::system(format!("Failed to get failed services: {}", e)))?;
    
    if !output.status.success() {
        return Ok(Vec::new());
    }
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let failed_services: Vec<String> = stdout
        .lines()
        .filter_map(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if !parts.is_empty() {
                Some(parts[0].to_string())
            } else {
                None
            }
        })
        .collect();
    
    Ok(failed_services)
}

/// Get 1-minute system load average
fn get_system_load_1min() -> Result<f64> {
    let loadavg = std::fs::read_to_string("/proc/loadavg")?;
    let parts: Vec<&str> = loadavg.split_whitespace().collect();
    
    if !parts.is_empty() {
        parts[0].parse::<f64>()
            .map_err(|e| FluxError::parse(format!("Failed to parse load average: {}", e)))
    } else {
        Err(FluxError::system("Could not read load average"))
    }
}

/// Get swap usage information
fn get_swap_usage() -> Result<SwapUsage> {
    let meminfo = std::fs::read_to_string("/proc/meminfo")?;
    
    let mut swap_total = 0u64;
    let mut swap_free = 0u64;
    
    for line in meminfo.lines() {
        if line.starts_with("SwapTotal:") {
            swap_total = parse_meminfo_value(line)?;
        } else if line.starts_with("SwapFree:") {
            swap_free = parse_meminfo_value(line)?;
        }
    }
    
    let swap_used = swap_total.saturating_sub(swap_free);
    let percentage = if swap_total > 0 {
        (swap_used as f64 / swap_total as f64) * 100.0
    } else {
        0.0
    };
    
    Ok(SwapUsage {
        total: swap_total * 1024, // Convert from KB to bytes
        used: swap_used * 1024,
        free: swap_free * 1024,
        percentage,
    })
}

/// Parse memory info value from /proc/meminfo
fn parse_meminfo_value(line: &str) -> Result<u64> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() >= 2 {
        parts[1].parse::<u64>()
            .map_err(|e| FluxError::parse(format!("Failed to parse meminfo value: {}", e)))
    } else {
        Err(FluxError::parse("Invalid meminfo line format"))
    }
}

/// Get inode usage for all mounted filesystems
fn get_inodes_usage() -> Result<HashMap<String, InodesUsage>> {
    let output = Command::new("df")
        .args(&["-i"])
        .output()
        .map_err(|e| FluxError::system(format!("Failed to get inode usage: {}", e)))?;
    
    if !output.status.success() {
        return Err(FluxError::command_failed("df command failed"));
    }
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut usage_map = HashMap::new();
    
    for line in stdout.lines().skip(1) { // Skip header
        if let Some(inode_usage) = parse_df_inode_line(line) {
            usage_map.insert(inode_usage.0, inode_usage.1);
        }
    }
    
    Ok(usage_map)
}

/// Parse df -i output line
fn parse_df_inode_line(line: &str) -> Option<(String, InodesUsage)> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    
    if parts.len() >= 6 {
        let filesystem = parts[0];
        if let (Ok(total), Ok(used), Ok(free)) = (
            parts[1].parse::<u64>(),
            parts[2].parse::<u64>(), 
            parts[3].parse::<u64>()
        ) {
            let percentage = if total > 0 {
                (used as f64 / total as f64) * 100.0
            } else {
                0.0
            };
            
            let usage = InodesUsage {
                total,
                used,
                free,
                percentage,
            };
            
            return Some((filesystem.to_string(), usage));
        }
    }
    
    None
}

/// Count zombie processes
fn count_zombie_processes() -> Result<u32> {
    let stat = std::fs::read_to_string("/proc/stat")?;
    
    for line in stat.lines() {
        if line.starts_with("processes") {
            // This is a simplified implementation
            // In reality, you'd need to count processes in 'Z' state from /proc/*/stat
            return count_zombie_processes_detailed();
        }
    }
    
    Ok(0)
}

/// Count zombie processes by examining /proc/*/stat
fn count_zombie_processes_detailed() -> Result<u32> {
    let mut zombie_count = 0;
    
    if let Ok(proc_entries) = std::fs::read_dir("/proc") {
        for entry in proc_entries.flatten() {
            if let Ok(file_name) = entry.file_name().into_string() {
                if file_name.chars().all(|c| c.is_ascii_digit()) {
                    // This is a PID directory
                    let stat_path = format!("/proc/{}/stat", file_name);
                    if let Ok(stat_content) = std::fs::read_to_string(&stat_path) {
                        if is_zombie_process(&stat_content) {
                            zombie_count += 1;
                        }
                    }
                }
            }
        }
    }
    
    Ok(zombie_count)
}

/// Check if process is a zombie based on /proc/PID/stat content
fn is_zombie_process(stat_content: &str) -> bool {
    let parts: Vec<&str> = stat_content.split_whitespace().collect();
    // Process state is the third field (after PID and command)
    if parts.len() >= 3 {
        return parts[2] == "Z";
    }
    false
}