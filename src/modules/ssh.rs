// src/modules/ssh.rs
// SSH hardening and configuration module

use crate::config::Config;
use crate::error::{FluxError, Result};
use crate::helpers::{
    file_ops::{backup_file, safe_write_file},
    logging::{log_error, log_info, log_success, log_warn},
    system::{check_command, execute_command, restart_service},
    user_input::{prompt_with_default, prompt_yes_no, select_from_menu},
};
use crate::modules::{Module, ModuleBase, ModuleInfo};
use async_trait::async_trait;
use std::fs;
use std::path::Path;
use std::process::Command;

const SSH_CONFIG_PATH: &str = "/etc/ssh/sshd_config";
const SSH_CONFIG_DIR: &str = "/etc/ssh/sshd_config.d";

pub struct SshModule {
    base: ModuleBase,
}

impl SshModule {
    pub fn new() -> Self {
        let info = ModuleInfo {
            name: "ssh".to_string(),
            description: "SSH server hardening and configuration".to_string(),
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

    /// Apply SSH hardening configuration
    async fn harden_ssh(&self, port: Option<u16>) -> Result<()> {
        log_info("Applying SSH hardening configuration");

        // Backup existing config
        backup_file(SSH_CONFIG_PATH)?;

        let ssh_port = port.unwrap_or(22);

        // Build hardened SSH config
        let hardened_config = format!(
            r#"
# Flux Framework - SSH Hardening Configuration
# Generated: {}

# Network Configuration
Port {}
AddressFamily any
ListenAddress 0.0.0.0
ListenAddress ::

# Protocol Configuration
Protocol 2

# Host Keys (prefer modern algorithms)
HostKey /etc/ssh/ssh_host_ed25519_key
HostKey /etc/ssh/ssh_host_rsa_key
HostKey /etc/ssh/ssh_host_ecdsa_key

# Ciphers and Key Exchange
Ciphers chacha20-poly1305@openssh.com,aes256-gcm@openssh.com,aes128-gcm@openssh.com,aes256-ctr,aes192-ctr,aes128-ctr
MACs hmac-sha2-512-etm@openssh.com,hmac-sha2-256-etm@openssh.com,hmac-sha2-512,hmac-sha2-256
KexAlgorithms curve25519-sha256,curve25519-sha256@libssh.org,diffie-hellman-group16-sha512,diffie-hellman-group18-sha512,diffie-hellman-group-exchange-sha256

# Authentication
PermitRootLogin no
PubkeyAuthentication yes
PasswordAuthentication no
PermitEmptyPasswords no
ChallengeResponseAuthentication no
KerberosAuthentication no
GSSAPIAuthentication no
HostbasedAuthentication no

# Security Settings
StrictModes yes
MaxAuthTries 3
MaxSessions 10
LoginGraceTime 30
ClientAliveInterval 300
ClientAliveCountMax 2

# Access Control
AllowAgentForwarding no
AllowTcpForwarding no
X11Forwarding no
PermitTunnel no
PermitUserEnvironment no

# Logging
SyslogFacility AUTH
LogLevel VERBOSE

# Subsystems
Subsystem sftp /usr/lib/openssh/sftp-server -f AUTHPRIV -l INFO

# Banner
Banner /etc/ssh/banner.txt

# Include additional configurations
Include {}/*.conf
"#,
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
            ssh_port,
            SSH_CONFIG_DIR
        );

        // Write hardened config
        safe_write_file(SSH_CONFIG_PATH, &hardened_config, true)?;
        log_success("SSH hardening configuration applied");

        // Create banner
        self.create_ssh_banner().await?;

        // Validate configuration
        self.validate_ssh_config().await?;

        log_info("SSH hardening complete. Remember to restart SSH service.");
        Ok(())
    }

    /// Create SSH banner
    async fn create_ssh_banner(&self) -> Result<()> {
        let banner_path = "/etc/ssh/banner.txt";

        let banner_content = r#"
********************************************************************************
*                             AUTHORIZED ACCESS ONLY                           *
********************************************************************************
*                                                                              *
*  This system is for authorized users only. All activity is monitored and    *
*  logged. Unauthorized access attempts will be prosecuted to the fullest     *
*  extent of the law.                                                          *
*                                                                              *
*  By accessing this system, you consent to monitoring and recording.         *
*                                                                              *
********************************************************************************
"#;

        safe_write_file(banner_path, banner_content, true)?;
        log_success("SSH banner created");
        Ok(())
    }

    /// Validate SSH configuration
    async fn validate_ssh_config(&self) -> Result<()> {
        log_info("Validating SSH configuration");

        let output = Command::new("sshd")
            .arg("-t")
            .output()
            .map_err(|e| FluxError::command_failed(format!("Failed to validate SSH config: {}", e)))?;

        if output.status.success() {
            log_success("SSH configuration is valid");
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(FluxError::command_failed(format!(
                "SSH configuration validation failed: {}",
                stderr
            )))
        }
    }

    /// Change SSH port
    async fn change_port(&self, new_port: u16) -> Result<()> {
        if let Err(e) = crate::helpers::validation::validate_port(&new_port.to_string()) {
            return Err(e);
        }

        log_info(&format!("Changing SSH port to {}", new_port));

        // Backup config
        backup_file(SSH_CONFIG_PATH)?;

        // Read current config
        let config = fs::read_to_string(SSH_CONFIG_PATH)
            .map_err(|e| FluxError::system(format!("Failed to read SSH config: {}", e)))?;

        // Replace port
        let new_config = if config.contains("Port ") {
            // Replace existing Port directive
            config
                .lines()
                .map(|line| {
                    if line.trim().starts_with("Port ") {
                        format!("Port {}", new_port)
                    } else {
                        line.to_string()
                    }
                })
                .collect::<Vec<_>>()
                .join("\n")
        } else {
            // Add Port directive
            format!("Port {}\n{}", new_port, config)
        };

        // Write updated config
        safe_write_file(SSH_CONFIG_PATH, &new_config, true)?;

        // Validate
        self.validate_ssh_config().await?;

        log_success(&format!("SSH port changed to {}", new_port));
        log_warn(&format!(
            "Update your firewall rules to allow port {}",
            new_port
        ));
        log_warn("Restart SSH service to apply changes");

        Ok(())
    }

    /// Disable password authentication
    async fn disable_password_auth(&self) -> Result<()> {
        log_info("Disabling password authentication");

        // Backup config
        backup_file(SSH_CONFIG_PATH)?;

        // Read current config
        let config = fs::read_to_string(SSH_CONFIG_PATH)
            .map_err(|e| FluxError::system(format!("Failed to read SSH config: {}", e)))?;

        // Update password authentication settings
        let new_config = config
            .lines()
            .map(|line| {
                let trimmed = line.trim();
                if trimmed.starts_with("PasswordAuthentication ") {
                    "PasswordAuthentication no".to_string()
                } else if trimmed.starts_with("ChallengeResponseAuthentication ") {
                    "ChallengeResponseAuthentication no".to_string()
                } else if trimmed.starts_with("PermitEmptyPasswords ") {
                    "PermitEmptyPasswords no".to_string()
                } else {
                    line.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join("\n");

        // Add directives if not present
        let mut final_config = new_config;
        if !final_config.contains("PasswordAuthentication no") {
            final_config.push_str("\nPasswordAuthentication no\n");
        }
        if !final_config.contains("ChallengeResponseAuthentication no") {
            final_config.push_str("ChallengeResponseAuthentication no\n");
        }
        if !final_config.contains("PermitEmptyPasswords no") {
            final_config.push_str("PermitEmptyPasswords no\n");
        }

        // Write updated config
        safe_write_file(SSH_CONFIG_PATH, &final_config, true)?;

        // Validate
        self.validate_ssh_config().await?;

        log_success("Password authentication disabled");
        log_warn("Ensure you have SSH key access configured before restarting SSH!");

        Ok(())
    }

    /// Setup fail2ban for SSH protection
    async fn setup_fail2ban(&self) -> Result<()> {
        log_info("Setting up fail2ban for SSH protection");

        // Check if fail2ban is installed
        if check_command("fail2ban-client").is_err() {
            log_info("Installing fail2ban");
            let distro = crate::helpers::system::detect_distro()?;

            if distro.is_debian_based() {
                execute_command("apt-get", &["update"])?;
                execute_command("apt-get", &["install", "-y", "fail2ban"])?;
            } else if distro.is_redhat_based() {
                execute_command("yum", &["install", "-y", "fail2ban"])?;
            } else {
                return Err(FluxError::Module(
                    "Unsupported distribution for fail2ban installation".to_string(),
                ));
            }
        }

        // Create fail2ban jail for SSH
        let jail_config = r#"
[sshd]
enabled = true
port = ssh
filter = sshd
logpath = /var/log/auth.log
maxretry = 3
findtime = 600
bantime = 3600
banaction = iptables-multiport

[sshd-ddos]
enabled = true
port = ssh
filter = sshd-ddos
logpath = /var/log/auth.log
maxretry = 2
findtime = 600
bantime = 86400
"#;

        let jail_path = "/etc/fail2ban/jail.d/sshd.conf";
        safe_write_file(jail_path, jail_config, true)?;

        // Start and enable fail2ban
        restart_service("fail2ban")?;

        log_success("fail2ban configured for SSH protection");
        Ok(())
    }

    /// Generate SSH host keys
    async fn generate_host_keys(&self) -> Result<()> {
        log_info("Generating SSH host keys");

        let key_types = vec![
            ("ed25519", "/etc/ssh/ssh_host_ed25519_key"),
            ("rsa", "/etc/ssh/ssh_host_rsa_key"),
            ("ecdsa", "/etc/ssh/ssh_host_ecdsa_key"),
        ];

        for (key_type, key_path) in key_types {
            log_info(&format!("Generating {} key", key_type));

            // Backup existing key
            if Path::new(key_path).exists() {
                backup_file(key_path)?;
            }

            // Generate new key
            let mut cmd = Command::new("ssh-keygen");
            cmd.arg("-t").arg(key_type);
            cmd.arg("-f").arg(key_path);
            cmd.arg("-N").arg(""); // No passphrase

            if key_type == "rsa" {
                cmd.arg("-b").arg("4096");
            } else if key_type == "ecdsa" {
                cmd.arg("-b").arg("521");
            }

            let output = cmd
                .output()
                .map_err(|e| FluxError::command_failed(format!("Failed to generate {} key: {}", key_type, e)))?;

            if output.status.success() {
                log_success(&format!("{} host key generated", key_type));
            } else {
                log_warn(&format!("Failed to generate {} host key", key_type));
            }
        }

        Ok(())
    }

    /// Show SSH status and information
    async fn show_status(&self) -> Result<()> {
        log_info("SSH Server Status:");

        // Check if SSH is running
        let ssh_service = if check_command("systemctl").is_ok() {
            "sshd"
        } else {
            "ssh"
        };

        let output = Command::new("systemctl")
            .arg("status")
            .arg(ssh_service)
            .output();

        if let Ok(output) = output {
            if output.status.success() {
                println!("\n{}", String::from_utf8_lossy(&output.stdout));
            }
        }

        // Show current SSH configuration
        if let Ok(config) = fs::read_to_string(SSH_CONFIG_PATH) {
            println!("\nKey SSH Configuration Settings:");
            println!("{}", "=".repeat(70));

            for line in config.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("Port ")
                    || trimmed.starts_with("PermitRootLogin ")
                    || trimmed.starts_with("PasswordAuthentication ")
                    || trimmed.starts_with("PubkeyAuthentication ")
                {
                    println!("{}", trimmed);
                }
            }
        }

        // Show active connections
        println!("\nActive SSH Connections:");
        println!("{}", "=".repeat(70));

        let output = Command::new("ss")
            .arg("-tn")
            .arg("state")
            .arg("established")
            .arg("'( dport = :ssh or sport = :ssh )'")
            .output();

        if let Ok(output) = output {
            println!("{}", String::from_utf8_lossy(&output.stdout));
        }

        Ok(())
    }

    /// Interactive hardening wizard
    async fn hardening_wizard(&self) -> Result<()> {
        log_info("SSH Hardening Wizard");

        println!("\nThis wizard will guide you through hardening your SSH configuration.");
        println!("Current configuration will be backed up before any changes.\n");

        // Port configuration
        let change_port = prompt_yes_no("Change SSH port from default (22)?", true)?;
        let port = if change_port {
            let port_str = prompt_with_default("Enter new SSH port (1024-65535)", "2222")?;
            port_str.parse::<u16>().unwrap_or(2222)
        } else {
            22
        };

        // Password authentication
        let disable_passwords = prompt_yes_no(
            "Disable password authentication (key-only)?",
            true,
        )?;

        if disable_passwords {
            log_warn("Make sure you have SSH key access configured before proceeding!");
            let confirm = prompt_yes_no("Are you sure you want to continue?", false)?;
            if !confirm {
                log_info("Aborting SSH hardening");
                return Ok(());
            }
        }

        // Fail2ban
        let setup_fail2ban = prompt_yes_no("Setup fail2ban for SSH protection?", true)?;

        // Apply hardening
        log_info("Applying SSH hardening...");
        self.harden_ssh(Some(port)).await?;

        if disable_passwords {
            self.disable_password_auth().await?;
        }

        if setup_fail2ban {
            self.setup_fail2ban().await?;
        }

        // Restart SSH
        let restart = prompt_yes_no("Restart SSH service now?", false)?;
        if restart {
            log_warn("Restarting SSH service...");
            let ssh_service = if check_command("systemctl").is_ok() {
                "sshd"
            } else {
                "ssh"
            };
            restart_service(ssh_service)?;
            log_success("SSH service restarted");
        } else {
            log_warn("Remember to restart SSH service to apply changes:");
            log_warn("  sudo systemctl restart sshd");
        }

        log_success("SSH hardening complete!");

        if change_port {
            log_warn(&format!(
                "\nIMPORTANT: Update your firewall to allow port {}",
                port
            ));
            log_warn(&format!("Next SSH connection: ssh -p {} user@host", port));
        }

        Ok(())
    }

    /// Show interactive menu
    async fn show_menu(&self) -> Result<()> {
        loop {
            let options = vec![
                "Run hardening wizard",
                "Change SSH port",
                "Disable password authentication",
                "Setup fail2ban",
                "Generate host keys",
                "Validate configuration",
                "Show SSH status",
                "Exit",
            ];

            let choice = select_from_menu("SSH Management", &options)?;

            match choice {
                0 => {
                    self.hardening_wizard().await?;
                }
                1 => {
                    let port_str = prompt_with_default("Enter new SSH port", "2222")?;
                    if let Ok(port) = port_str.parse::<u16>() {
                        self.change_port(port).await?;
                    } else {
                        log_error("Invalid port number");
                    }
                }
                2 => {
                    self.disable_password_auth().await?;
                }
                3 => {
                    self.setup_fail2ban().await?;
                }
                4 => {
                    self.generate_host_keys().await?;
                }
                5 => {
                    self.validate_ssh_config().await?;
                }
                6 => {
                    self.show_status().await?;
                }
                7 => {
                    log_info("Exiting SSH management");
                    break;
                }
                _ => {}
            }
        }

        Ok(())
    }
}

#[async_trait]
impl Module for SshModule {
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
        check_command("sshd").is_ok() || check_command("ssh").is_ok()
    }

    fn help(&self) -> String {
        format!(
            r#"SSH Hardening Module v{}

DESCRIPTION:
    {}

USAGE:
    flux module {} [OPTIONS]

OPTIONS:
    --harden                     Apply full SSH hardening
    --port <port>                Change SSH port
    --disable-passwords          Disable password authentication
    --fail2ban                   Setup fail2ban protection
    --generate-keys              Generate new host keys
    --status                     Show SSH status
    --menu                       Show interactive menu

EXAMPLES:
    flux module {} --menu
    flux module {} --harden
    flux module {} --port 2222
    flux module {} --disable-passwords --fail2ban
"#,
            self.version(),
            self.description(),
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

        // Parse arguments
        let mut i = 0;
        while i < args.len() {
            match args[i].as_str() {
                "--harden" => {
                    self.hardening_wizard().await?;
                    i += 1;
                }
                "--port" => {
                    if i + 1 < args.len() {
                        if let Ok(port) = args[i + 1].parse::<u16>() {
                            self.change_port(port).await?;
                        }
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                "--disable-passwords" => {
                    self.disable_password_auth().await?;
                    i += 1;
                }
                "--fail2ban" => {
                    self.setup_fail2ban().await?;
                    i += 1;
                }
                "--generate-keys" => {
                    self.generate_host_keys().await?;
                    i += 1;
                }
                "--status" => {
                    self.show_status().await?;
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
