// src/modules/motd.rs
// Dynamic Message of the Day (MOTD) module

use crate::config::Config;
use crate::error::{FluxError, Result};
use crate::helpers::{
    file_ops::safe_write_file,
    logging::{log_debug, log_error, log_info, log_success, log_warn},
    system::{check_command, execute_command},
    user_input::{prompt_input, prompt_yes_no, select_from_menu},
};
use crate::modules::{Module, ModuleBase, ModuleInfo};
use async_trait::async_trait;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

const MOTD_DIR: &str = "/etc/update-motd.d";
const MOTD_STATIC: &str = "/etc/motd";

pub struct MotdModule {
    base: ModuleBase,
}

impl MotdModule {
    pub fn new() -> Self {
        let info = ModuleInfo {
            name: "motd".to_string(),
            description: "Dynamic Message of the Day configuration".to_string(),
            version: "1.0.0".to_string(),
            author: "Flux Contributors".to_string(),
            tags: vec!["system".to_string(), "ux".to_string()],
            requires_root: true,
            supported_distros: vec!["debian".to_string(), "ubuntu".to_string()],
        };
        Self {
            base: ModuleBase { info },
        }
    }

    /// Get ASCII art banner
    fn get_banner(&self, style: &str) -> String {
        match style {
            "flux-large" => {
                r#"
  ███████╗██╗     ██╗   ██╗██╗  ██╗
  ██╔════╝██║     ██║   ██║╚██╗██╔╝
  █████╗  ██║     ██║   ██║ ╚███╔╝
  ██╔══╝  ██║     ██║   ██║ ██╔██╗
  ██║     ███████╗╚██████╔╝██╔╝ ██╗
  ╚═╝     ╚══════╝ ╚═════╝ ╚═╝  ╚═╝
     ⚡ Flux System Administration ⚡
"#.to_string()
            }
            "simple" => {
                r#"
  ╔═══════════════════════════════╗
  ║      FLUX FRAMEWORK v1.0      ║
  ╚═══════════════════════════════╝
"#.to_string()
            }
            "minimal" => {
                r#"
  ┌─────────────────────────┐
  │   Flux Administration   │
  └─────────────────────────┘
"#.to_string()
            }
            _ => {
                // Default flux banner
                r#"
  ███████╗██╗     ██╗   ██╗██╗  ██╗
  ██╔════╝██║     ██║   ██║╚██╗██╔╝
  █████╗  ██║     ██║   ██║ ╚███╔╝
  ██╔══╝  ██║     ██║   ██║ ██╔██╗
  ██║     ███████╗╚██████╔╝██╔╝ ██╗
  ╚═╝     ╚══════╝ ╚═════╝ ╚═╝  ╚═╝
"#.to_string()
            }
        }
    }

    /// Create header script
    async fn create_header_script(&self, banner_style: &str, organization: Option<&str>) -> Result<()> {
        log_info("Creating MOTD header script");

        let banner = self.get_banner(banner_style);
        let org_text = if let Some(org) = organization {
            format!("  Organization: {}\n", org)
        } else {
            String::new()
        };

        let script = format!(
            r#"#!/bin/bash
# Flux Framework - MOTD Header
# 10-flux-header

CYAN='\033[0;36m'
RESET='\033[0m'

cat << 'EOF'
{}
EOF

echo -e "${{CYAN}}{}  Generated: $(date)${{RESET}}"
echo
"#,
            banner, org_text
        );

        let script_path = PathBuf::from(MOTD_DIR).join("10-flux-header");
        safe_write_file(script_path.to_str().unwrap(), &script, true)?;

        // Make executable
        let mut perms = fs::metadata(&script_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&script_path, perms)?;

        log_success("Header script created");
        Ok(())
    }

    /// Create system info script
    async fn create_sysinfo_script(&self) -> Result<()> {
        log_info("Creating system info script");

        let script = r#"#!/bin/bash
# Flux Framework - System Information
# 20-flux-sysinfo

GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
RESET='\033[0m'

# System info
HOSTNAME=$(hostname)
KERNEL=$(uname -r)
UPTIME=$(uptime -p)
USERS=$(who | wc -l)

# CPU info
CPU_LOAD=$(uptime | awk -F'load average:' '{print $2}' | cut -d, -f1 | xargs)

# Memory info
MEM_TOTAL=$(free -m | awk 'NR==2{print $2}')
MEM_USED=$(free -m | awk 'NR==2{print $3}')
MEM_PERCENT=$(free | awk 'NR==2{printf "%.0f", $3*100/$2}')

# Disk info
DISK_TOTAL=$(df -h / | awk 'NR==2{print $2}')
DISK_USED=$(df -h / | awk 'NR==2{print $3}')
DISK_PERCENT=$(df -h / | awk 'NR==2{print $5}')

# Network
IP_ADDR=$(hostname -I | awk '{print $1}')

echo -e "${GREEN}System Information:${RESET}"
echo "─────────────────────────────────────────"
echo -e "  ${BLUE}Hostname:${RESET}    $HOSTNAME"
echo -e "  ${BLUE}Kernel:${RESET}      $KERNEL"
echo -e "  ${BLUE}Uptime:${RESET}      $UPTIME"
echo -e "  ${BLUE}Users:${RESET}       $USERS active"
echo

echo -e "${GREEN}Resource Usage:${RESET}"
echo "─────────────────────────────────────────"
echo -e "  ${BLUE}CPU Load:${RESET}    $CPU_LOAD"
echo -e "  ${BLUE}Memory:${RESET}      ${MEM_USED}MB / ${MEM_TOTAL}MB (${MEM_PERCENT}%)"
echo -e "  ${BLUE}Disk:${RESET}        ${DISK_USED} / ${DISK_TOTAL} (${DISK_PERCENT})"
echo -e "  ${BLUE}IP Address:${RESET}  $IP_ADDR"
echo
"#;

        let script_path = PathBuf::from(MOTD_DIR).join("20-flux-sysinfo");
        safe_write_file(script_path.to_str().unwrap(), script, true)?;

        // Make executable
        let mut perms = fs::metadata(&script_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&script_path, perms)?;

        log_success("System info script created");
        Ok(())
    }

    /// Create security status script
    async fn create_security_script(&self) -> Result<()> {
        log_info("Creating security status script");

        let script = r#"#!/bin/bash
# Flux Framework - Security Status
# 30-flux-security

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
RESET='\033[0m'

echo -e "${GREEN}Security Status:${RESET}"
echo "─────────────────────────────────────────"

# Check firewall
if systemctl is-active --quiet ufw; then
    echo -e "  ${GREEN}✓${RESET} UFW Firewall: Active"
elif systemctl is-active --quiet firewalld; then
    echo -e "  ${GREEN}✓${RESET} firewalld: Active"
else
    echo -e "  ${RED}✗${RESET} Firewall: Inactive"
fi

# Check fail2ban
if systemctl is-active --quiet fail2ban; then
    BANNED=$(fail2ban-client status sshd 2>/dev/null | grep "Currently banned" | awk '{print $NF}' || echo "0")
    echo -e "  ${GREEN}✓${RESET} fail2ban: Active (${BANNED} banned)"
else
    echo -e "  ${YELLOW}○${RESET} fail2ban: Not installed"
fi

# Check for updates
if command -v apt-get &> /dev/null; then
    UPDATES=$(apt-get -s upgrade 2>/dev/null | grep -P '^\d+ upgraded' | cut -d" " -f1)
    if [ "$UPDATES" -gt 0 ]; then
        echo -e "  ${YELLOW}⚠${RESET}  System updates: $UPDATES available"
    else
        echo -e "  ${GREEN}✓${RESET} System updates: Up to date"
    fi
fi

echo
"#;

        let script_path = PathBuf::from(MOTD_DIR).join("30-flux-security");
        safe_write_file(script_path.to_str().unwrap(), script, true)?;

        // Make executable
        let mut perms = fs::metadata(&script_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&script_path, perms)?;

        log_success("Security status script created");
        Ok(())
    }

    /// Create footer script
    async fn create_footer_script(&self, message: Option<&str>) -> Result<()> {
        log_info("Creating footer script");

        let custom_msg = if let Some(msg) = message {
            format!("echo -e \"  {}\"\necho", msg)
        } else {
            String::new()
        };

        let script = format!(
            r#"#!/bin/bash
# Flux Framework - MOTD Footer
# 90-flux-footer

CYAN='\033[0;36m'
RESET='\033[0m'

{}

echo -e "${{CYAN}}For help, type: flux help${{RESET}}"
echo
"#,
            custom_msg
        );

        let script_path = PathBuf::from(MOTD_DIR).join("90-flux-footer");
        safe_write_file(script_path.to_str().unwrap(), &script, true)?;

        // Make executable
        let mut perms = fs::metadata(&script_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&script_path, perms)?;

        log_success("Footer script created");
        Ok(())
    }

    /// Disable default Ubuntu MOTD scripts
    async fn disable_default_scripts(&self) -> Result<()> {
        log_info("Disabling default MOTD scripts");

        let motd_dir = PathBuf::from(MOTD_DIR);
        if !motd_dir.exists() {
            log_warn("MOTD directory not found");
            return Ok(());
        }

        // Disable default scripts by removing execute permission
        let default_scripts = vec![
            "00-header",
            "10-help-text",
            "50-motd-news",
            "80-esm",
            "80-livepatch",
            "90-updates-available",
            "91-release-upgrade",
            "95-hwe-eol",
        ];

        for script in default_scripts {
            let script_path = motd_dir.join(script);
            if script_path.exists() {
                let mut perms = fs::metadata(&script_path)?.permissions();
                perms.set_mode(0o644); // Remove execute permission
                fs::set_permissions(&script_path, perms)?;
                log_debug(&format!("Disabled: {}", script));
            }
        }

        log_success("Default scripts disabled");
        Ok(())
    }

    /// Enable default Ubuntu MOTD scripts
    async fn enable_default_scripts(&self) -> Result<()> {
        log_info("Enabling default MOTD scripts");

        let motd_dir = PathBuf::from(MOTD_DIR);
        if !motd_dir.exists() {
            return Ok(());
        }

        let default_scripts = vec![
            "00-header",
            "10-help-text",
            "50-motd-news",
            "90-updates-available",
        ];

        for script in default_scripts {
            let script_path = motd_dir.join(script);
            if script_path.exists() {
                let mut perms = fs::metadata(&script_path)?.permissions();
                perms.set_mode(0o755); // Add execute permission
                fs::set_permissions(&script_path, perms)?;
                log_debug(&format!("Enabled: {}", script));
            }
        }

        log_success("Default scripts enabled");
        Ok(())
    }

    /// Install dynamic MOTD
    async fn install_motd(&self, banner_style: &str, organization: Option<&str>, message: Option<&str>) -> Result<()> {
        log_info("Installing dynamic MOTD");

        // Create MOTD directory if it doesn't exist
        fs::create_dir_all(MOTD_DIR)?;

        // Disable default scripts
        self.disable_default_scripts().await?;

        // Create Flux MOTD scripts
        self.create_header_script(banner_style, organization).await?;
        self.create_sysinfo_script().await?;
        self.create_security_script().await?;
        self.create_footer_script(message).await?;

        // Clear static MOTD
        safe_write_file(MOTD_STATIC, "", true)?;

        log_success("Dynamic MOTD installed successfully");
        log_info("MOTD will be displayed on next login");

        Ok(())
    }

    /// Remove Flux MOTD
    async fn remove_motd(&self) -> Result<()> {
        log_info("Removing Flux MOTD");

        let flux_scripts = vec![
            "10-flux-header",
            "20-flux-sysinfo",
            "30-flux-security",
            "90-flux-footer",
        ];

        let motd_dir = PathBuf::from(MOTD_DIR);
        for script in flux_scripts {
            let script_path = motd_dir.join(script);
            if script_path.exists() {
                fs::remove_file(&script_path)?;
                log_debug(&format!("Removed: {}", script));
            }
        }

        // Re-enable default scripts
        self.enable_default_scripts().await?;

        log_success("Flux MOTD removed");
        Ok(())
    }

    /// Preview MOTD
    async fn preview_motd(&self) -> Result<()> {
        log_info("Previewing MOTD:");

        println!("\n{}", "=".repeat(70));

        // Execute MOTD scripts
        let motd_dir = PathBuf::from(MOTD_DIR);
        if motd_dir.exists() {
            let mut scripts: Vec<_> = fs::read_dir(&motd_dir)?
                .filter_map(|e| e.ok())
                .filter(|e| {
                    let path = e.path();
                    path.is_file()
                        && path.file_name()
                            .and_then(|n| n.to_str())
                            .map(|s| s.starts_with("10-flux") || s.starts_with("20-flux") || s.starts_with("30-flux") || s.starts_with("90-flux"))
                            .unwrap_or(false)
                })
                .collect();

            scripts.sort_by_key(|e| e.file_name());

            for script in scripts {
                let output = Command::new("bash")
                    .arg(script.path())
                    .output()
                    .ok();

                if let Some(out) = output {
                    print!("{}", String::from_utf8_lossy(&out.stdout));
                }
            }
        }

        println!("{}", "=".repeat(70));
        Ok(())
    }

    /// Show interactive menu
    async fn show_menu(&self) -> Result<()> {
        loop {
            let options = vec![
                "Install dynamic MOTD",
                "Preview MOTD",
                "Remove Flux MOTD",
                "Restore default MOTD",
                "Exit",
            ];

            let choice = select_from_menu("MOTD Management", &options)?;

            match choice {
                0 => {
                    // Install MOTD
                    let banner_styles = vec!["default", "flux-large", "simple", "minimal"];
                    let banner_choice = select_from_menu("Select banner style", &banner_styles)?;

                    let org = prompt_input("Enter organization name (optional)").ok();
                    let message = prompt_input("Enter custom message (optional)").ok();

                    if let Err(e) = self.install_motd(
                        banner_styles[banner_choice],
                        org.as_deref(),
                        message.as_deref(),
                    ).await {
                        log_error(&format!("Failed to install MOTD: {}", e));
                    }
                }
                1 => {
                    // Preview MOTD
                    self.preview_motd().await?;
                }
                2 => {
                    // Remove Flux MOTD
                    let confirm = prompt_yes_no("Remove Flux MOTD?", false)?;
                    if confirm {
                        self.remove_motd().await?;
                    }
                }
                3 => {
                    // Restore default
                    self.enable_default_scripts().await?;
                }
                4 => {
                    log_info("Exiting MOTD management");
                    break;
                }
                _ => {}
            }
        }

        Ok(())
    }
}

#[async_trait]
impl Module for MotdModule {
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
        PathBuf::from(MOTD_DIR).exists()
    }

    fn help(&self) -> String {
        format!(
            r#"MOTD Management Module v{}

DESCRIPTION:
    {}

    This module creates dynamic Message of the Day (MOTD) displays with
    system information, security status, and customizable banners.

USAGE:
    flux module {} [OPTIONS]

OPTIONS:
    --install                    Install dynamic MOTD
    --banner <style>             Set banner style
    --org <name>                 Set organization name
    --message <text>             Set custom message
    --preview                    Preview MOTD
    --remove                     Remove Flux MOTD
    --restore                    Restore default MOTD
    --menu                       Show interactive menu

BANNER STYLES:
    default                      Standard Flux banner
    flux-large                   Large Flux banner with tagline
    simple                       Simple boxed design
    minimal                      Minimal bordered layout

EXAMPLES:
    flux module {} --menu
    flux module {} --install
    flux module {} --install --banner flux-large --org "MyCompany"
    flux module {} --preview
    flux module {} --remove
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
        let mut banner_style = "default";
        let mut org: Option<String> = None;
        let mut message: Option<String> = None;

        // Parse arguments first
        while i < args.len() {
            match args[i].as_str() {
                "--banner" => {
                    if i + 1 < args.len() {
                        banner_style = Box::leak(args[i + 1].clone().into_boxed_str());
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                "--org" => {
                    if i + 1 < args.len() {
                        org = Some(args[i + 1].clone());
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                "--message" => {
                    if i + 1 < args.len() {
                        message = Some(args[i + 1].clone());
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                _ => {
                    i += 1;
                }
            }
        }

        // Execute commands
        i = 0;
        while i < args.len() {
            match args[i].as_str() {
                "--install" => {
                    self.install_motd(banner_style, org.as_deref(), message.as_deref()).await?;
                    i += 1;
                }
                "--preview" => {
                    self.preview_motd().await?;
                    i += 1;
                }
                "--remove" => {
                    self.remove_motd().await?;
                    i += 1;
                }
                "--restore" => {
                    self.enable_default_scripts().await?;
                    i += 1;
                }
                "--banner" | "--org" | "--message" => {
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
