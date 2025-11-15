// src/modules/firewall.rs
// Firewall configuration and management module

use crate::config::Config;
use crate::error::{FluxError, Result};
use crate::helpers::{
    file_ops::{backup_file, safe_write_file},
    logging::{log_debug, log_error, log_info, log_success, log_warn},
    system::{check_command, execute_command, get_os_info, restart_service},
    user_input::{prompt_input, prompt_with_default, prompt_yes_no, select_from_menu, multi_select_menu},
    validation::validate_port,
};
use crate::modules::{Module, ModuleBase, ModuleInfo};
use async_trait::async_trait;
use std::collections::HashMap;
use std::fs;
use std::process::Command;

const UFW_BACKUP_DIR: &str = "/var/backups/flux/firewall";
const FIREWALLD_BACKUP_DIR: &str = "/var/backups/flux/firewall";

#[derive(Debug, Clone, PartialEq)]
pub enum FirewallType {
    UFW,
    Firewalld,
    Iptables,
    None,
}

pub struct FirewallModule {
    base: ModuleBase,
}

impl FirewallModule {
    pub fn new() -> Self {
        let info = ModuleInfo {
            name: "firewall".to_string(),
            description: "Host firewall configuration and management".to_string(),
            version: "1.0.0".to_string(),
            author: "Flux Contributors".to_string(),
            tags: vec!["security".to_string(), "network".to_string()],
            requires_root: true,
            supported_distros: vec!["all".to_string()],
        };
        Self {
            base: ModuleBase { info },
        }
    }

    /// Detect which firewall is active on the system
    async fn detect_firewall(&self) -> Result<FirewallType> {
        log_debug("Detecting active firewall");

        // Check UFW
        if check_command("ufw").is_ok() {
            let output = Command::new("ufw")
                .arg("status")
                .output()
                .ok();

            if let Some(out) = output {
                let status = String::from_utf8_lossy(&out.stdout);
                if status.contains("Status: active") {
                    log_info("Detected active UFW firewall");
                    return Ok(FirewallType::UFW);
                }
            }
        }

        // Check firewalld
        if check_command("firewall-cmd").is_ok() {
            let output = Command::new("firewall-cmd")
                .arg("--state")
                .output()
                .ok();

            if let Some(out) = output {
                let status = String::from_utf8_lossy(&out.stdout);
                if status.trim() == "running" {
                    log_info("Detected active firewalld");
                    return Ok(FirewallType::Firewalld);
                }
            }
        }

        // Check iptables
        if check_command("iptables").is_ok() {
            log_info("Detected iptables (manual configuration)");
            return Ok(FirewallType::Iptables);
        }

        log_warn("No firewall detected");
        Ok(FirewallType::None)
    }

    /// Install firewall based on distribution
    async fn install_firewall(&self, fw_type: &FirewallType) -> Result<()> {
        let distro = crate::helpers::system::detect_distro()?;

        match fw_type {
            FirewallType::UFW => {
                log_info("Installing UFW firewall");

                if distro.is_debian_based() {
                    execute_command("apt-get", &["update"])?;
                    execute_command("apt-get", &["install", "-y", "ufw"])?;
                } else if distro.is_redhat_based() {
                    execute_command("yum", &["install", "-y", "ufw"])?;
                } else {
                    return Err(FluxError::Module("Unsupported distribution for UFW".to_string()));
                }

                log_success("UFW installed successfully");
                Ok(())
            }
            FirewallType::Firewalld => {
                log_info("Installing firewalld");

                if distro.is_debian_based() {
                    execute_command("apt-get", &["update"])?;
                    execute_command("apt-get", &["install", "-y", "firewalld"])?;
                } else if distro.is_redhat_based() {
                    execute_command("yum", &["install", "-y", "firewalld"])?;
                } else {
                    return Err(FluxError::Module("Unsupported distribution for firewalld".to_string()));
                }

                log_success("firewalld installed successfully");
                Ok(())
            }
            _ => Err(FluxError::Module("Invalid firewall type for installation".to_string()))
        }
    }

    /// Enable UFW with safety checks
    async fn enable_ufw(&self) -> Result<()> {
        log_info("Enabling UFW firewall");

        // Ensure SSH is allowed before enabling
        log_info("Ensuring SSH access is allowed before enabling firewall");
        execute_command("ufw", &["allow", "ssh"])?;

        // Enable UFW
        let output = Command::new("ufw")
            .arg("--force")
            .arg("enable")
            .output()
            .map_err(|e| FluxError::command_failed(format!("Failed to enable UFW: {}", e)))?;

        if !output.status.success() {
            return Err(FluxError::command_failed("Failed to enable UFW".to_string()));
        }

        // Enable UFW service
        if check_command("systemctl").is_ok() {
            execute_command("systemctl", &["enable", "ufw"])?;
            execute_command("systemctl", &["start", "ufw"])?;
        }

        log_success("UFW enabled successfully");
        Ok(())
    }

    /// Enable firewalld with safety checks
    async fn enable_firewalld(&self) -> Result<()> {
        log_info("Enabling firewalld");

        // Start and enable firewalld
        execute_command("systemctl", &["start", "firewalld"])?;
        execute_command("systemctl", &["enable", "firewalld"])?;

        // Ensure SSH is allowed
        log_info("Ensuring SSH access is allowed");
        execute_command("firewall-cmd", &["--permanent", "--add-service=ssh"])?;
        execute_command("firewall-cmd", &["--reload"])?;

        log_success("firewalld enabled successfully");
        Ok(())
    }

    /// Add UFW rule
    async fn add_ufw_rule(&self, port: u16, protocol: &str, comment: Option<&str>) -> Result<()> {
        log_info(&format!("Adding UFW rule: {}/{}", port, protocol));

        let mut args = vec!["allow".to_string()];
        if let Some(cmt) = comment {
            args.push("comment".to_string());
            args.push(cmt.to_string());
        }
        args.push(format!("{}/{}", port, protocol));

        let args_str: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        execute_command("ufw", &args_str)?;

        log_success(&format!("UFW rule added: {}/{}", port, protocol));
        Ok(())
    }

    /// Add firewalld rule
    async fn add_firewalld_rule(&self, port: u16, protocol: &str, zone: &str) -> Result<()> {
        log_info(&format!("Adding firewalld rule: {}/{} to zone {}", port, protocol, zone));

        execute_command(
            "firewall-cmd",
            &["--permanent", &format!("--zone={}", zone), &format!("--add-port={}/{}", port, protocol)]
        )?;
        execute_command("firewall-cmd", &["--reload"])?;

        log_success(&format!("firewalld rule added: {}/{}", port, protocol));
        Ok(())
    }

    /// List UFW rules
    async fn list_ufw_rules(&self, verbose: bool) -> Result<()> {
        log_info("Listing UFW rules:");

        let args = if verbose {
            vec!["status", "verbose"]
        } else {
            vec!["status", "numbered"]
        };

        let output = Command::new("ufw")
            .args(&args)
            .output()
            .map_err(|e| FluxError::command_failed(format!("Failed to list UFW rules: {}", e)))?;

        println!("\n{}", String::from_utf8_lossy(&output.stdout));
        Ok(())
    }

    /// List firewalld rules
    async fn list_firewalld_rules(&self, zone: &str) -> Result<()> {
        log_info(&format!("Listing firewalld rules for zone: {}", zone));

        let output = Command::new("firewall-cmd")
            .arg(&format!("--zone={}", zone))
            .arg("--list-all")
            .output()
            .map_err(|e| FluxError::command_failed(format!("Failed to list firewalld rules: {}", e)))?;

        println!("\n{}", String::from_utf8_lossy(&output.stdout));
        Ok(())
    }

    /// Get service port mappings
    fn get_service_ports(&self) -> HashMap<String, (u16, String)> {
        let mut services = HashMap::new();

        services.insert("ssh".to_string(), (22, "tcp".to_string()));
        services.insert("http".to_string(), (80, "tcp".to_string()));
        services.insert("https".to_string(), (443, "tcp".to_string()));
        services.insert("mysql".to_string(), (3306, "tcp".to_string()));
        services.insert("postgresql".to_string(), (5432, "tcp".to_string()));
        services.insert("redis".to_string(), (6379, "tcp".to_string()));
        services.insert("mongodb".to_string(), (27017, "tcp".to_string()));
        services.insert("docker".to_string(), (2376, "tcp".to_string()));
        services.insert("kubernetes".to_string(), (6443, "tcp".to_string()));
        services.insert("smtp".to_string(), (25, "tcp".to_string()));
        services.insert("smtps".to_string(), (465, "tcp".to_string()));
        services.insert("imap".to_string(), (143, "tcp".to_string()));
        services.insert("imaps".to_string(), (993, "tcp".to_string()));
        services.insert("pop3".to_string(), (110, "tcp".to_string()));
        services.insert("pop3s".to_string(), (995, "tcp".to_string()));
        services.insert("dns".to_string(), (53, "udp".to_string()));
        services.insert("ntp".to_string(), (123, "udp".to_string()));
        services.insert("netdata".to_string(), (19999, "tcp".to_string()));

        services
    }

    /// Apply security preset
    async fn apply_preset(&self, preset: &str) -> Result<()> {
        log_info(&format!("Applying firewall preset: {}", preset));

        let fw_type = self.detect_firewall().await?;

        let ports_to_open: Vec<(u16, &str)> = match preset {
            "web-server" => vec![
                (80, "tcp"),
                (443, "tcp"),
            ],
            "database" => vec![
                (3306, "tcp"),  // MySQL
                (5432, "tcp"),  // PostgreSQL
            ],
            "mail-server" => vec![
                (25, "tcp"),    // SMTP
                (465, "tcp"),   // SMTPS
                (587, "tcp"),   // Submission
                (143, "tcp"),   // IMAP
                (993, "tcp"),   // IMAPS
                (110, "tcp"),   // POP3
                (995, "tcp"),   // POP3S
            ],
            "docker-host" => vec![
                (2376, "tcp"),  // Docker daemon
                (2377, "tcp"),  // Swarm management
                (7946, "tcp"),  // Container network discovery
                (7946, "udp"),
                (4789, "udp"),  // Overlay network
            ],
            "kubernetes" => vec![
                (6443, "tcp"),  // API server
                (2379, "tcp"),  // etcd
                (2380, "tcp"),  // etcd
                (10250, "tcp"), // Kubelet
                (10251, "tcp"), // Scheduler
                (10252, "tcp"), // Controller
            ],
            "minimal" => vec![
                (22, "tcp"),    // SSH only
            ],
            _ => {
                return Err(FluxError::Module(format!("Unknown preset: {}", preset)));
            }
        };

        for (port, protocol) in ports_to_open {
            match fw_type {
                FirewallType::UFW => {
                    self.add_ufw_rule(port, protocol, Some(&format!("{} preset", preset))).await?;
                }
                FirewallType::Firewalld => {
                    self.add_firewalld_rule(port, protocol, "public").await?;
                }
                _ => {
                    log_warn(&format!("Cannot apply preset with firewall type: {:?}", fw_type));
                    break;
                }
            }
        }

        log_success(&format!("Preset '{}' applied successfully", preset));
        Ok(())
    }

    /// Backup firewall configuration
    async fn backup_config(&self) -> Result<String> {
        log_info("Backing up firewall configuration");

        let fw_type = self.detect_firewall().await?;
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");

        match fw_type {
            FirewallType::UFW => {
                fs::create_dir_all(UFW_BACKUP_DIR)?;
                let backup_path = format!("{}/ufw-rules-{}.backup", UFW_BACKUP_DIR, timestamp);

                // Export UFW rules
                let output = Command::new("ufw")
                    .arg("status")
                    .arg("numbered")
                    .output()
                    .map_err(|e| FluxError::command_failed(format!("Failed to backup UFW: {}", e)))?;

                fs::write(&backup_path, output.stdout)?;

                log_success(&format!("UFW configuration backed up to: {}", backup_path));
                Ok(backup_path)
            }
            FirewallType::Firewalld => {
                fs::create_dir_all(FIREWALLD_BACKUP_DIR)?;
                let backup_path = format!("{}/firewalld-{}.backup", FIREWALLD_BACKUP_DIR, timestamp);

                // Copy firewalld configuration
                execute_command("cp", &["-r", "/etc/firewalld", &backup_path])?;

                log_success(&format!("firewalld configuration backed up to: {}", backup_path));
                Ok(backup_path)
            }
            _ => Err(FluxError::Module("No supported firewall to backup".to_string()))
        }
    }

    /// Show firewall status
    async fn show_status(&self) -> Result<()> {
        let fw_type = self.detect_firewall().await?;

        match fw_type {
            FirewallType::UFW => {
                log_info("UFW Firewall Status:");
                self.list_ufw_rules(true).await?;
            }
            FirewallType::Firewalld => {
                log_info("firewalld Status:");
                self.list_firewalld_rules("public").await?;
            }
            FirewallType::Iptables => {
                log_info("iptables Status:");
                let output = Command::new("iptables")
                    .arg("-L")
                    .arg("-n")
                    .arg("-v")
                    .output()
                    .map_err(|e| FluxError::command_failed(format!("Failed to list iptables: {}", e)))?;
                println!("\n{}", String::from_utf8_lossy(&output.stdout));
            }
            FirewallType::None => {
                log_warn("No firewall is currently active");
            }
        }

        Ok(())
    }

    /// Interactive setup wizard
    async fn setup_wizard(&self) -> Result<()> {
        log_info("Firewall Setup Wizard");
        println!("\nThis wizard will help you configure your firewall.\n");

        let fw_type = self.detect_firewall().await?;

        // Install firewall if needed
        if fw_type == FirewallType::None {
            let install = prompt_yes_no("No firewall detected. Install one?", true)?;
            if !install {
                log_info("Firewall setup cancelled");
                return Ok(());
            }

            let distro = crate::helpers::system::detect_distro()?;
            let default_fw = if distro.is_debian_based() {
                "UFW"
            } else {
                "firewalld"
            };

            let fw_options = vec!["UFW", "firewalld"];
            let fw_choice = select_from_menu("Select firewall to install", &fw_options)?;

            let selected_fw = if fw_choice == 0 {
                FirewallType::UFW
            } else {
                FirewallType::Firewalld
            };

            self.install_firewall(&selected_fw).await?;

            // Enable the firewall
            match selected_fw {
                FirewallType::UFW => self.enable_ufw().await?,
                FirewallType::Firewalld => self.enable_firewalld().await?,
                _ => {}
            }
        }

        // Apply security preset
        let apply_preset = prompt_yes_no("Apply a security preset?", true)?;
        if apply_preset {
            let presets = vec![
                "minimal (SSH only)",
                "web-server (HTTP/HTTPS)",
                "database (MySQL/PostgreSQL)",
                "mail-server (SMTP/IMAP/POP3)",
                "docker-host",
                "kubernetes",
                "Skip preset",
            ];

            let preset_choice = select_from_menu("Select security preset", &presets)?;

            if preset_choice < 6 {
                let preset_name = match preset_choice {
                    0 => "minimal",
                    1 => "web-server",
                    2 => "database",
                    3 => "mail-server",
                    4 => "docker-host",
                    5 => "kubernetes",
                    _ => "minimal",
                };

                self.apply_preset(preset_name).await?;
            }
        }

        // Custom rules
        let add_custom = prompt_yes_no("Add custom firewall rules?", false)?;
        if add_custom {
            loop {
                let port_str = match prompt_input("Enter port number (or 'done' to finish)") {
                    Ok(s) if s.to_lowercase() == "done" => break,
                    Ok(s) => s,
                    Err(_) => break,
                };

                if let Ok(port) = port_str.parse::<u16>() {
                    let protocol = prompt_with_default("Enter protocol (tcp/udp)", "tcp")?;

                    let current_fw = self.detect_firewall().await?;
                    match current_fw {
                        FirewallType::UFW => {
                            self.add_ufw_rule(port, &protocol, Some("Custom rule")).await?;
                        }
                        FirewallType::Firewalld => {
                            self.add_firewalld_rule(port, &protocol, "public").await?;
                        }
                        _ => {
                            log_warn("Firewall not properly configured");
                            break;
                        }
                    }
                } else {
                    log_error("Invalid port number");
                }
            }
        }

        log_success("Firewall configuration complete!");
        self.show_status().await?;

        Ok(())
    }

    /// Show interactive menu
    async fn show_menu(&self) -> Result<()> {
        loop {
            let options = vec![
                "Run setup wizard",
                "Show firewall status",
                "Apply security preset",
                "Add custom rule",
                "List rules",
                "Backup configuration",
                "Enable firewall",
                "Exit",
            ];

            let choice = select_from_menu("Firewall Management", &options)?;

            match choice {
                0 => {
                    self.setup_wizard().await?;
                }
                1 => {
                    self.show_status().await?;
                }
                2 => {
                    let presets = vec!["minimal", "web-server", "database", "mail-server", "docker-host", "kubernetes"];
                    let preset_choice = select_from_menu("Select preset", &presets)?;
                    self.apply_preset(presets[preset_choice]).await?;
                }
                3 => {
                    let port_str = prompt_input("Enter port number")?;
                    if let Ok(port) = port_str.parse::<u16>() {
                        let protocol = prompt_with_default("Enter protocol (tcp/udp)", "tcp")?;
                        let fw_type = self.detect_firewall().await?;

                        match fw_type {
                            FirewallType::UFW => {
                                self.add_ufw_rule(port, &protocol, Some("Custom rule")).await?;
                            }
                            FirewallType::Firewalld => {
                                self.add_firewalld_rule(port, &protocol, "public").await?;
                            }
                            _ => {
                                log_warn("No supported firewall active");
                            }
                        }
                    }
                }
                4 => {
                    let fw_type = self.detect_firewall().await?;
                    match fw_type {
                        FirewallType::UFW => self.list_ufw_rules(true).await?,
                        FirewallType::Firewalld => self.list_firewalld_rules("public").await?,
                        _ => log_warn("No supported firewall active"),
                    }
                }
                5 => {
                    self.backup_config().await?;
                }
                6 => {
                    let fw_type = self.detect_firewall().await?;
                    match fw_type {
                        FirewallType::UFW => self.enable_ufw().await?,
                        FirewallType::Firewalld => self.enable_firewalld().await?,
                        FirewallType::None => {
                            log_warn("No firewall installed. Run setup wizard first.");
                        }
                        _ => {
                            log_warn("Firewall type not supported for this operation");
                        }
                    }
                }
                7 => {
                    log_info("Exiting firewall management");
                    break;
                }
                _ => {}
            }
        }

        Ok(())
    }
}

#[async_trait]
impl Module for FirewallModule {
    fn name(&self) -> &str {
        &self.base.info.name
    }

    fn description(&self) -> &str {
        &self.base.info.description
    }

    fn version(&self) -> &str {
        &self.base.info.version
    }

    fn is_available(&self) -> bool {
        check_command("ufw").is_ok()
            || check_command("firewall-cmd").is_ok()
            || check_command("iptables").is_ok()
    }

    fn help(&self) -> String {
        format!(
            r#"Firewall Management Module v{}

DESCRIPTION:
    {}

USAGE:
    flux module {} [OPTIONS]

OPTIONS:
    --status                     Show firewall status
    --enable                     Enable firewall with safety checks
    --preset <name>              Apply security preset
    --allow <port>/<protocol>    Allow port through firewall
    --list                       List firewall rules
    --backup                     Backup firewall configuration
    --wizard                     Run interactive setup wizard
    --menu                       Show interactive menu

PRESETS:
    minimal        - SSH only (port 22)
    web-server     - HTTP/HTTPS (ports 80, 443)
    database       - MySQL/PostgreSQL (ports 3306, 5432)
    mail-server    - SMTP/IMAP/POP3
    docker-host    - Docker daemon and Swarm
    kubernetes     - Kubernetes cluster ports

EXAMPLES:
    flux module {} --menu
    flux module {} --wizard
    flux module {} --preset web-server
    flux module {} --allow 8080/tcp
    flux module {} --status
"#,
            self.version(),
            self.description(),
            self.name(),
            self.name(),
            self.name(),
            self.name(),
            self.name(),
            self.name()
        )
    }

    async fn execute(&self, args: Vec<String>, _config: &Config) -> Result<()> {
        if args.is_empty() || args.contains(&"--menu".to_string()) {
            return self.show_menu().await;
        }

        let mut i = 0;
        while i < args.len() {
            match args[i].as_str() {
                "--wizard" => {
                    self.setup_wizard().await?;
                    i += 1;
                }
                "--status" => {
                    self.show_status().await?;
                    i += 1;
                }
                "--enable" => {
                    let fw_type = self.detect_firewall().await?;
                    match fw_type {
                        FirewallType::UFW => self.enable_ufw().await?,
                        FirewallType::Firewalld => self.enable_firewalld().await?,
                        FirewallType::None => {
                            log_error("No firewall installed");
                        }
                        _ => {
                            log_warn("Unsupported firewall type");
                        }
                    }
                    i += 1;
                }
                "--preset" => {
                    if i + 1 < args.len() {
                        self.apply_preset(&args[i + 1]).await?;
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                "--allow" => {
                    if i + 1 < args.len() {
                        let parts: Vec<&str> = args[i + 1].split('/').collect();
                        if parts.len() == 2 {
                            if let Ok(port) = parts[0].parse::<u16>() {
                                let fw_type = self.detect_firewall().await?;
                                match fw_type {
                                    FirewallType::UFW => {
                                        self.add_ufw_rule(port, parts[1], Some("CLI rule")).await?;
                                    }
                                    FirewallType::Firewalld => {
                                        self.add_firewalld_rule(port, parts[1], "public").await?;
                                    }
                                    _ => {
                                        log_warn("No supported firewall active");
                                    }
                                }
                            }
                        }
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                "--list" => {
                    let fw_type = self.detect_firewall().await?;
                    match fw_type {
                        FirewallType::UFW => self.list_ufw_rules(true).await?,
                        FirewallType::Firewalld => self.list_firewalld_rules("public").await?,
                        _ => log_warn("No supported firewall active"),
                    }
                    i += 1;
                }
                "--backup" => {
                    self.backup_config().await?;
                    i += 1;
                }
                _ => {
                    i += 1;
                }
            }
        }

        Ok(())
    }
}
