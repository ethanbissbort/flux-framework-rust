// src/modules/netdata.rs
// Netdata monitoring agent installation and configuration module

use crate::config::Config;
use crate::error::{FluxError, Result};
use crate::helpers::{
    file_ops::safe_write_file,
    logging::{log_info, log_success, log_warn},
    system::{check_command, execute_command},
    user_input::{prompt_input, prompt_with_default, prompt_yes_no, select_from_menu},
};
use crate::modules::{Module, ModuleBase, ModuleInfo};
use async_trait::async_trait;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

const NETDATA_CONFIG_DIR: &str = "/etc/netdata";
const NETDATA_INSTALL_URL: &str = "https://get.netdata.cloud/kickstart.sh";

pub struct NetdataModule {
    base: ModuleBase,
}

impl NetdataModule {
    pub fn new() -> Self {
        let info = ModuleInfo {
            name: "netdata".to_string(),
            description: "Netdata monitoring agent installation and configuration".to_string(),
            version: "1.0.0".to_string(),
            author: "Flux Contributors".to_string(),
            tags: vec!["monitoring".to_string(), "observability".to_string()],
            requires_root: true,
            supported_distros: vec!["all".to_string()],
        };
        Self {
            base: ModuleBase { info },
        }
    }

    /// Check if Netdata is installed
    fn is_netdata_installed(&self) -> bool {
        check_command("netdata").is_ok() || PathBuf::from("/usr/sbin/netdata").exists()
    }

    /// Check system requirements
    async fn check_requirements(&self) -> Result<()> {
        log_info("Checking system requirements");

        // Check available memory (minimum 512MB)
        let output = Command::new("free")
            .arg("-m")
            .output()
            .ok();

        if let Some(out) = output {
            let mem_info = String::from_utf8_lossy(&out.stdout);
            if let Some(line) = mem_info.lines().nth(1) {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() > 1 {
                    if let Ok(total_mem) = parts[1].parse::<u64>() {
                        if total_mem < 512 {
                            log_warn("Low memory detected. Netdata may impact system performance.");
                        }
                    }
                }
            }
        }

        // Check disk space (minimum 100MB)
        let output = Command::new("df")
            .arg("-m")
            .arg("/")
            .output()
            .ok();

        if let Some(out) = output {
            let disk_info = String::from_utf8_lossy(&out.stdout);
            if let Some(line) = disk_info.lines().nth(1) {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() > 3 {
                    if let Ok(available) = parts[3].parse::<u64>() {
                        if available < 100 {
                            return Err(FluxError::Module(
                                "Insufficient disk space. At least 100MB required.".to_string(),
                            ));
                        }
                    }
                }
            }
        }

        log_success("System requirements check passed");
        Ok(())
    }

    /// Install Netdata
    async fn install_netdata(&self, disable_telemetry: bool, claim_token: Option<&str>) -> Result<()> {
        if self.is_netdata_installed() {
            log_info("Netdata is already installed");
            return Ok(());
        }

        log_info("Installing Netdata");

        // Check requirements
        self.check_requirements().await?;

        // Download installer
        log_info("Downloading Netdata installer");
        let temp_dir = std::env::temp_dir();
        let installer_path = temp_dir.join("netdata-kickstart.sh");

        let client = reqwest::Client::new();
        let response = client
            .get(NETDATA_INSTALL_URL)
            .send()
            .await
            .map_err(|e| FluxError::Network(format!("Failed to download installer: {}", e)))?;

        let installer_script = response
            .text()
            .await
            .map_err(|e| FluxError::Network(format!("Failed to read installer: {}", e)))?;

        fs::write(&installer_path, installer_script)?;

        // Build installation command
        let mut install_args = vec!["--dont-wait"];

        if disable_telemetry {
            install_args.push("--disable-telemetry");
        }

        // Execute installer
        log_info("Running Netdata installer (this may take a few minutes)");

        let mut cmd = Command::new("bash");
        cmd.arg(&installer_path);
        cmd.args(&install_args);

        if let Some(token) = claim_token {
            cmd.arg("--claim-token").arg(token);
        }

        let output = cmd
            .output()
            .map_err(|e| FluxError::command_failed(format!("Failed to execute installer: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(FluxError::command_failed(format!(
                "Netdata installation failed: {}",
                stderr
            )));
        }

        // Clean up installer
        fs::remove_file(&installer_path).ok();

        log_success("Netdata installed successfully");
        Ok(())
    }

    /// Configure Netdata
    async fn configure_netdata(&self, web_port: u16, enable_cloud: bool) -> Result<()> {
        log_info("Configuring Netdata");

        let config_path = PathBuf::from(NETDATA_CONFIG_DIR).join("netdata.conf");

        // Generate configuration
        let config = format!(
            r#"# Flux Framework - Netdata Configuration

[global]
    # Web server configuration
    bind to = *
    default port = {}

    # Performance tuning
    update every = 1
    history = 3600
    memory mode = ram

    # Security
    run as user = netdata

[web]
    enable gzip compression = yes

[plugins]
    # Enable/disable plugins
    proc = yes
    diskspace = yes
    cgroups = yes
    tc = no
    idlejitter = no

[health]
    enabled = yes
    default repeat warning = 300
    default repeat critical = 60
"#,
            web_port
        );

        let config_path_str = config_path
            .to_str()
            .ok_or_else(|| FluxError::system("Invalid UTF-8 in config path"))?;
        safe_write_file(config_path_str, &config, true)?;

        log_success("Netdata configured");
        Ok(())
    }

    /// Setup health alarms
    async fn setup_health_alarms(&self) -> Result<()> {
        log_info("Setting up health alarms");

        let health_dir = PathBuf::from(NETDATA_CONFIG_DIR).join("health.d");
        fs::create_dir_all(&health_dir)?;

        // CPU alarm
        let cpu_alarm = r#"
alarm: cpu_usage
on: system.cpu
lookup: average -3m percentage of user,system
units: %
every: 60s
warn: $this > 80
crit: $this > 95
info: CPU usage is high
"#;

        let cpu_alarm_path = health_dir.join("cpu_usage.conf");
        let cpu_alarm_path_str = cpu_alarm_path
            .to_str()
            .ok_or_else(|| FluxError::system("Invalid UTF-8 in alarm path"))?;
        safe_write_file(cpu_alarm_path_str, cpu_alarm, true)?;

        // Memory alarm
        let mem_alarm = r#"
alarm: ram_usage
on: system.ram
lookup: average -3m percentage of used
units: %
every: 60s
warn: $this > 80
crit: $this > 90
info: RAM usage is high
"#;

        let mem_alarm_path = health_dir.join("ram_usage.conf");
        let mem_alarm_path_str = mem_alarm_path
            .to_str()
            .ok_or_else(|| FluxError::system("Invalid UTF-8 in alarm path"))?;
        safe_write_file(mem_alarm_path_str, mem_alarm, true)?;

        // Disk alarm
        let disk_alarm = r#"
alarm: disk_space
on: disk.space
lookup: average -1m percentage of used
units: %
every: 60s
warn: $this > 80
crit: $this > 90
info: Disk space usage is high
"#;

        let disk_alarm_path = health_dir.join("disk_space.conf");
        let disk_alarm_path_str = disk_alarm_path
            .to_str()
            .ok_or_else(|| FluxError::system("Invalid UTF-8 in alarm path"))?;
        safe_write_file(disk_alarm_path_str, disk_alarm, true)?;

        log_success("Health alarms configured");
        Ok(())
    }

    /// Configure firewall for Netdata
    async fn configure_firewall(&self, port: u16) -> Result<()> {
        log_info(&format!("Configuring firewall for Netdata (port {})", port));

        // Check which firewall is active
        if check_command("ufw").is_ok() {
            let output = Command::new("ufw")
                .arg("status")
                .output()
                .ok();

            if let Some(out) = output {
                let status = String::from_utf8_lossy(&out.stdout);
                if status.contains("Status: active") {
                    execute_command("ufw", &["allow", &port.to_string()])?;
                    log_success(&format!("UFW rule added for port {}", port));
                }
            }
        } else if check_command("firewall-cmd").is_ok() {
            execute_command(
                "firewall-cmd",
                &["--permanent", &format!("--add-port={}/tcp", port)],
            )?;
            execute_command("firewall-cmd", &["--reload"])?;
            log_success(&format!("firewalld rule added for port {}", port));
        } else {
            log_warn("No supported firewall detected. You may need to manually configure firewall rules.");
        }

        Ok(())
    }

    /// Start Netdata service
    async fn start_service(&self) -> Result<()> {
        log_info("Starting Netdata service");

        if check_command("systemctl").is_ok() {
            execute_command("systemctl", &["start", "netdata"])?;
            execute_command("systemctl", &["enable", "netdata"])?;
            log_success("Netdata service started and enabled");
        } else {
            log_warn("systemctl not found. You may need to start Netdata manually.");
        }

        Ok(())
    }

    /// Stop Netdata service
    async fn stop_service(&self) -> Result<()> {
        log_info("Stopping Netdata service");

        if check_command("systemctl").is_ok() {
            execute_command("systemctl", &["stop", "netdata"])?;
            log_success("Netdata service stopped");
        }

        Ok(())
    }

    /// Uninstall Netdata
    async fn uninstall_netdata(&self) -> Result<()> {
        log_info("Uninstalling Netdata");

        let confirm = prompt_yes_no("Are you sure you want to uninstall Netdata?", false)?;
        if !confirm {
            log_info("Uninstall cancelled");
            return Ok(());
        }

        // Stop service
        self.stop_service().await.ok();

        // Run uninstaller if it exists
        let uninstaller_path = "/usr/libexec/netdata/netdata-uninstaller.sh";
        if PathBuf::from(uninstaller_path).exists() {
            execute_command("bash", &[uninstaller_path, "--yes", "--force"])?;
        } else {
            log_warn("Netdata uninstaller not found. Manual cleanup may be required.");
        }

        log_success("Netdata uninstalled");
        Ok(())
    }

    /// Show Netdata status
    async fn show_status(&self) -> Result<()> {
        if !self.is_netdata_installed() {
            log_warn("Netdata is not installed");
            return Ok(());
        }

        log_info("Netdata Status:");

        if check_command("systemctl").is_ok() {
            let output = Command::new("systemctl")
                .arg("status")
                .arg("netdata")
                .output()
                .ok();

            if let Some(out) = output {
                println!("\n{}", String::from_utf8_lossy(&out.stdout));
            }
        }

        // Show access URL
        let output = Command::new("hostname")
            .arg("-I")
            .output()
            .ok();

        if let Some(out) = output {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let ip = stdout.trim().split_whitespace().next().unwrap_or("localhost");
            println!("\n{}", "=".repeat(70));
            println!("Netdata Web Interface: http://{}:19999", ip);
            println!("{}", "=".repeat(70));
        }

        Ok(())
    }

    /// Full setup wizard
    async fn setup_wizard(&self) -> Result<()> {
        log_info("Netdata Setup Wizard");
        println!("\nThis wizard will guide you through Netdata installation and configuration.\n");

        // Install Netdata
        let disable_telemetry = prompt_yes_no("Disable telemetry?", true)?;
        let use_cloud = prompt_yes_no("Connect to Netdata Cloud?", false)?;

        let claim_token = if use_cloud {
            Some(prompt_input("Enter Netdata Cloud claim token")?)
        } else {
            None
        };

        self.install_netdata(disable_telemetry, claim_token.as_deref()).await?;

        // Configure
        let web_port_str = prompt_with_default("Enter web interface port", "19999")?;
        let web_port = web_port_str.parse::<u16>().unwrap_or(19999);

        self.configure_netdata(web_port, use_cloud).await?;

        // Setup health alarms
        let setup_alarms = prompt_yes_no("Setup health alarms?", true)?;
        if setup_alarms {
            self.setup_health_alarms().await?;
        }

        // Configure firewall
        let configure_fw = prompt_yes_no("Configure firewall?", true)?;
        if configure_fw {
            self.configure_firewall(web_port).await.ok();
        }

        // Start service
        self.start_service().await?;

        log_success("Netdata setup complete!");
        self.show_status().await?;

        Ok(())
    }

    /// Show interactive menu
    async fn show_menu(&self) -> Result<()> {
        loop {
            let options = vec![
                "Run setup wizard",
                "Install Netdata",
                "Configure Netdata",
                "Setup health alarms",
                "Show status",
                "Start service",
                "Stop service",
                "Uninstall Netdata",
                "Exit",
            ];

            let choice = select_from_menu("Netdata Management", &options)?;

            match choice {
                0 => {
                    self.setup_wizard().await?;
                }
                1 => {
                    let disable_telemetry = prompt_yes_no("Disable telemetry?", true)?;
                    self.install_netdata(disable_telemetry, None).await?;
                }
                2 => {
                    let port_str = prompt_with_default("Enter web port", "19999")?;
                    let port = port_str.parse::<u16>().unwrap_or(19999);
                    self.configure_netdata(port, false).await?;
                }
                3 => {
                    self.setup_health_alarms().await?;
                }
                4 => {
                    self.show_status().await?;
                }
                5 => {
                    self.start_service().await?;
                }
                6 => {
                    self.stop_service().await?;
                }
                7 => {
                    self.uninstall_netdata().await?;
                }
                8 => {
                    log_info("Exiting Netdata management");
                    break;
                }
                _ => {}
            }
        }

        Ok(())
    }
}

#[async_trait]
impl Module for NetdataModule {
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
        check_command("curl").is_ok() || check_command("wget").is_ok()
    }

    fn help(&self) -> String {
        format!(
            r#"Netdata Monitoring Module v{}

DESCRIPTION:
    {}

    This module installs and configures Netdata, a real-time performance
    and health monitoring system with beautiful visualizations.

USAGE:
    flux module {} [OPTIONS]

OPTIONS:
    --wizard                     Run interactive setup wizard
    --install                    Install Netdata
    --disable-telemetry          Disable telemetry during installation
    --claim-token <token>        Netdata Cloud claim token
    --configure                  Configure Netdata
    --port <port>                Set web interface port (default: 19999)
    --setup-alarms               Configure health alarms
    --status                     Show Netdata status
    --start                      Start Netdata service
    --stop                       Stop Netdata service
    --uninstall                  Uninstall Netdata
    --menu                       Show interactive menu

FEATURES:
    - Real-time system monitoring
    - CPU, memory, disk, network metrics
    - Health alarms and notifications
    - Web-based dashboard (port 19999)
    - Low resource footprint
    - Optional Netdata Cloud integration

EXAMPLES:
    flux module {} --menu
    flux module {} --wizard
    flux module {} --install --disable-telemetry
    flux module {} --configure --port 8080
    flux module {} --status

WEB INTERFACE:
    After installation, access Netdata at: http://your-server-ip:19999
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
                "--install" => {
                    let disable_telemetry = args.contains(&"--disable-telemetry".to_string());
                    let claim_token = if args.contains(&"--claim-token".to_string()) {
                        if let Some(token_idx) = args.iter().position(|s| s == "--claim-token") {
                            if token_idx + 1 < args.len() {
                                Some(args[token_idx + 1].as_str())
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    };

                    self.install_netdata(disable_telemetry, claim_token).await?;
                    i += 1;
                }
                "--configure" => {
                    let port = if args.contains(&"--port".to_string()) {
                        if let Some(port_idx) = args.iter().position(|s| s == "--port") {
                            if port_idx + 1 < args.len() {
                                args[port_idx + 1].parse::<u16>().unwrap_or(19999)
                            } else {
                                19999
                            }
                        } else {
                            19999
                        }
                    } else {
                        19999
                    };

                    self.configure_netdata(port, false).await?;
                    i += 1;
                }
                "--setup-alarms" => {
                    self.setup_health_alarms().await?;
                    i += 1;
                }
                "--status" => {
                    self.show_status().await?;
                    i += 1;
                }
                "--start" => {
                    self.start_service().await?;
                    i += 1;
                }
                "--stop" => {
                    self.stop_service().await?;
                    i += 1;
                }
                "--uninstall" => {
                    self.uninstall_netdata().await?;
                    i += 1;
                }
                "--disable-telemetry" | "--claim-token" | "--port" => {
                    i += 2;
                }
                _ => {
                    i += 1;
                }
            }
        }

        Ok(())
    }
}
