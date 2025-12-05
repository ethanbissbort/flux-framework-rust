// src/modules/sysctl.rs
// Kernel sysctl hardening parameters module

use crate::config::Config;
use crate::error::{FluxError, Result};
use crate::helpers::{
    file_ops::safe_write_file,
    logging::{log_info, log_success, log_warn},
    system::check_command,
    user_input::{prompt_yes_no, select_from_menu},
};
use crate::modules::{Module, ModuleBase, ModuleInfo};
use async_trait::async_trait;
use std::collections::HashMap;
use std::fs;
use std::process::Command;

const SYSCTL_CONFIG_PATH: &str = "/etc/sysctl.d/99-flux-hardening.conf";
const SYSCTL_BACKUP_DIR: &str = "/var/backups/flux/sysctl";

pub struct SysctlModule {
    base: ModuleBase,
}

impl SysctlModule {
    pub fn new() -> Self {
        let info = ModuleInfo {
            name: "sysctl".to_string(),
            description: "Kernel sysctl hardening parameters".to_string(),
            version: "1.0.0".to_string(),
            author: "Flux Contributors".to_string(),
            tags: vec!["security".to_string(), "kernel".to_string()],
            requires_root: true,
            supported_distros: vec!["all".to_string()],
        };
        Self {
            base: ModuleBase { info },
        }
    }

    /// Get hardening parameters
    fn get_hardening_params(&self) -> HashMap<String, (String, String)> {
        let mut params = HashMap::new();

        // Kernel hardening
        params.insert(
            "kernel.dmesg_restrict".to_string(),
            ("1".to_string(), "Restrict kernel log access".to_string()),
        );
        params.insert(
            "kernel.kptr_restrict".to_string(),
            ("2".to_string(), "Hide kernel pointers".to_string()),
        );
        params.insert(
            "kernel.randomize_va_space".to_string(),
            ("2".to_string(), "Enable full ASLR".to_string()),
        );
        params.insert(
            "kernel.panic".to_string(),
            ("10".to_string(), "Reboot 10s after panic".to_string()),
        );
        params.insert(
            "kernel.panic_on_oops".to_string(),
            ("1".to_string(), "Panic on oops".to_string()),
        );
        params.insert(
            "kernel.yama.ptrace_scope".to_string(),
            ("1".to_string(), "Restrict ptrace".to_string()),
        );

        // Network hardening - IPv4
        params.insert(
            "net.ipv4.ip_forward".to_string(),
            ("0".to_string(), "Disable IP forwarding".to_string()),
        );
        params.insert(
            "net.ipv4.conf.all.forwarding".to_string(),
            ("0".to_string(), "Disable forwarding (all)".to_string()),
        );
        params.insert(
            "net.ipv4.conf.default.forwarding".to_string(),
            ("0".to_string(), "Disable forwarding (default)".to_string()),
        );
        params.insert(
            "net.ipv4.conf.all.send_redirects".to_string(),
            ("0".to_string(), "Disable ICMP redirects".to_string()),
        );
        params.insert(
            "net.ipv4.conf.default.send_redirects".to_string(),
            ("0".to_string(), "Disable ICMP redirects (default)".to_string()),
        );
        params.insert(
            "net.ipv4.conf.all.accept_redirects".to_string(),
            ("0".to_string(), "Do not accept ICMP redirects".to_string()),
        );
        params.insert(
            "net.ipv4.conf.default.accept_redirects".to_string(),
            ("0".to_string(), "Do not accept ICMP redirects (default)".to_string()),
        );
        params.insert(
            "net.ipv4.conf.all.secure_redirects".to_string(),
            ("0".to_string(), "Do not accept secure ICMP redirects".to_string()),
        );
        params.insert(
            "net.ipv4.conf.default.secure_redirects".to_string(),
            ("0".to_string(), "Do not accept secure ICMP redirects (default)".to_string()),
        );
        params.insert(
            "net.ipv4.conf.all.accept_source_route".to_string(),
            ("0".to_string(), "Disable source routing".to_string()),
        );
        params.insert(
            "net.ipv4.conf.default.accept_source_route".to_string(),
            ("0".to_string(), "Disable source routing (default)".to_string()),
        );
        params.insert(
            "net.ipv4.conf.all.log_martians".to_string(),
            ("1".to_string(), "Log martian packets".to_string()),
        );
        params.insert(
            "net.ipv4.conf.default.log_martians".to_string(),
            ("1".to_string(), "Log martian packets (default)".to_string()),
        );
        params.insert(
            "net.ipv4.conf.all.rp_filter".to_string(),
            ("1".to_string(), "Enable reverse path filtering".to_string()),
        );
        params.insert(
            "net.ipv4.conf.default.rp_filter".to_string(),
            ("1".to_string(), "Enable reverse path filtering (default)".to_string()),
        );
        params.insert(
            "net.ipv4.icmp_echo_ignore_broadcasts".to_string(),
            ("1".to_string(), "Ignore ICMP broadcast".to_string()),
        );
        params.insert(
            "net.ipv4.icmp_ignore_bogus_error_responses".to_string(),
            ("1".to_string(), "Ignore bogus ICMP errors".to_string()),
        );
        params.insert(
            "net.ipv4.tcp_syncookies".to_string(),
            ("1".to_string(), "Enable SYN flood protection".to_string()),
        );
        params.insert(
            "net.ipv4.tcp_timestamps".to_string(),
            ("1".to_string(), "Enable TCP timestamps".to_string()),
        );

        // Network hardening - IPv6
        params.insert(
            "net.ipv6.conf.all.forwarding".to_string(),
            ("0".to_string(), "Disable IPv6 forwarding".to_string()),
        );
        params.insert(
            "net.ipv6.conf.default.forwarding".to_string(),
            ("0".to_string(), "Disable IPv6 forwarding (default)".to_string()),
        );
        params.insert(
            "net.ipv6.conf.all.accept_redirects".to_string(),
            ("0".to_string(), "Do not accept IPv6 redirects".to_string()),
        );
        params.insert(
            "net.ipv6.conf.default.accept_redirects".to_string(),
            ("0".to_string(), "Do not accept IPv6 redirects (default)".to_string()),
        );
        params.insert(
            "net.ipv6.conf.all.accept_source_route".to_string(),
            ("0".to_string(), "Disable IPv6 source routing".to_string()),
        );
        params.insert(
            "net.ipv6.conf.default.accept_source_route".to_string(),
            ("0".to_string(), "Disable IPv6 source routing (default)".to_string()),
        );
        params.insert(
            "net.ipv6.conf.all.accept_ra".to_string(),
            ("0".to_string(), "Do not accept router advertisements".to_string()),
        );
        params.insert(
            "net.ipv6.conf.default.accept_ra".to_string(),
            ("0".to_string(), "Do not accept router advertisements (default)".to_string()),
        );

        // Filesystem hardening
        params.insert(
            "fs.protected_hardlinks".to_string(),
            ("1".to_string(), "Enable hardlink protection".to_string()),
        );
        params.insert(
            "fs.protected_symlinks".to_string(),
            ("1".to_string(), "Enable symlink protection".to_string()),
        );
        params.insert(
            "fs.suid_dumpable".to_string(),
            ("0".to_string(), "Disable SUID core dumps".to_string()),
        );

        // Performance tuning
        params.insert(
            "net.core.default_qdisc".to_string(),
            ("fq".to_string(), "Fair queue packet scheduler".to_string()),
        );
        params.insert(
            "net.ipv4.tcp_congestion_control".to_string(),
            ("bbr".to_string(), "BBR congestion control".to_string()),
        );
        params.insert(
            "net.core.rmem_max".to_string(),
            ("134217728".to_string(), "Max socket receive buffer (128MB)".to_string()),
        );
        params.insert(
            "net.core.wmem_max".to_string(),
            ("134217728".to_string(), "Max socket send buffer (128MB)".to_string()),
        );
        params.insert(
            "net.ipv4.tcp_rmem".to_string(),
            ("4096 87380 67108864".to_string(), "TCP read buffer sizes".to_string()),
        );
        params.insert(
            "net.ipv4.tcp_wmem".to_string(),
            ("4096 65536 67108864".to_string(), "TCP write buffer sizes".to_string()),
        );

        params
    }

    /// Get performance-focused parameters
    fn get_performance_params(&self) -> HashMap<String, (String, String)> {
        let mut params = HashMap::new();

        params.insert(
            "net.core.netdev_max_backlog".to_string(),
            ("5000".to_string(), "Max network device backlog".to_string()),
        );
        params.insert(
            "net.ipv4.tcp_max_syn_backlog".to_string(),
            ("8192".to_string(), "Max SYN backlog".to_string()),
        );
        params.insert(
            "net.core.somaxconn".to_string(),
            ("1024".to_string(), "Max socket connections".to_string()),
        );
        params.insert(
            "vm.swappiness".to_string(),
            ("10".to_string(), "Reduce swappiness".to_string()),
        );
        params.insert(
            "vm.dirty_ratio".to_string(),
            ("15".to_string(), "Dirty page ratio".to_string()),
        );
        params.insert(
            "vm.dirty_background_ratio".to_string(),
            ("5".to_string(), "Background dirty ratio".to_string()),
        );

        params
    }

    /// Generate sysctl configuration file
    fn generate_config(&self, include_performance: bool) -> String {
        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
        let mut config = format!(
            r#"# Flux Framework - Sysctl Hardening Configuration
# Generated: {}
# This file contains kernel security hardening parameters

"#,
            timestamp
        );

        // Add hardening parameters
        config.push_str("# =========================================\n");
        config.push_str("# KERNEL HARDENING\n");
        config.push_str("# =========================================\n\n");

        let hardening_params = self.get_hardening_params();
        let mut kernel_params: Vec<_> = hardening_params
            .iter()
            .filter(|(k, _)| k.starts_with("kernel."))
            .collect();
        kernel_params.sort_by_key(|(k, _)| k.as_str());

        for (key, (value, description)) in kernel_params {
            config.push_str(&format!("# {}\n", description));
            config.push_str(&format!("{} = {}\n\n", key, value));
        }

        // Network hardening
        config.push_str("# =========================================\n");
        config.push_str("# NETWORK HARDENING\n");
        config.push_str("# =========================================\n\n");

        let mut net_params: Vec<_> = hardening_params
            .iter()
            .filter(|(k, _)| k.starts_with("net."))
            .collect();
        net_params.sort_by_key(|(k, _)| k.as_str());

        for (key, (value, description)) in net_params {
            config.push_str(&format!("# {}\n", description));
            config.push_str(&format!("{} = {}\n\n", key, value));
        }

        // Filesystem hardening
        config.push_str("# =========================================\n");
        config.push_str("# FILESYSTEM HARDENING\n");
        config.push_str("# =========================================\n\n");

        let mut fs_params: Vec<_> = hardening_params
            .iter()
            .filter(|(k, _)| k.starts_with("fs."))
            .collect();
        fs_params.sort_by_key(|(k, _)| k.as_str());

        for (key, (value, description)) in fs_params {
            config.push_str(&format!("# {}\n", description));
            config.push_str(&format!("{} = {}\n\n", key, value));
        }

        // Performance tuning (optional)
        if include_performance {
            config.push_str("# =========================================\n");
            config.push_str("# PERFORMANCE TUNING\n");
            config.push_str("# =========================================\n\n");

            let perf_params = self.get_performance_params();
            let mut perf_vec: Vec<_> = perf_params.iter().collect();
            perf_vec.sort_by_key(|(k, _)| k.as_str());

            for (key, (value, description)) in perf_vec {
                config.push_str(&format!("# {}\n", description));
                config.push_str(&format!("{} = {}\n\n", key, value));
            }
        }

        config
    }

    /// Apply sysctl hardening
    async fn apply_hardening(&self, include_performance: bool, force: bool) -> Result<()> {
        log_info("Applying sysctl hardening configuration");

        // Check if config already exists
        if fs::metadata(SYSCTL_CONFIG_PATH).is_ok() && !force {
            let overwrite = prompt_yes_no(
                "Sysctl configuration already exists. Overwrite?",
                false,
            )?;
            if !overwrite {
                log_info("Sysctl hardening cancelled");
                return Ok(());
            }
        }

        // Backup existing configuration
        if fs::metadata(SYSCTL_CONFIG_PATH).is_ok() {
            fs::create_dir_all(SYSCTL_BACKUP_DIR)?;
            let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
            let backup_path = format!("{}/sysctl-{}.conf.bak", SYSCTL_BACKUP_DIR, timestamp);
            fs::copy(SYSCTL_CONFIG_PATH, &backup_path)?;
            log_info(&format!("Backed up existing config to: {}", backup_path));
        }

        // Generate and write configuration
        let config = self.generate_config(include_performance);
        safe_write_file(SYSCTL_CONFIG_PATH, &config, true)?;

        log_success(&format!(
            "Sysctl configuration written to: {}",
            SYSCTL_CONFIG_PATH
        ));

        // Apply the configuration
        log_info("Applying sysctl settings...");
        let output = Command::new("sysctl")
            .arg("-p")
            .arg(SYSCTL_CONFIG_PATH)
            .output()
            .map_err(|e| FluxError::command_failed(format!("Failed to apply sysctl: {}", e)))?;

        if output.status.success() {
            log_success("Sysctl hardening applied successfully");
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            log_warn(&format!("Some sysctl parameters failed to apply: {}", stderr));
            log_info("This is normal for parameters not supported by your kernel");
        }

        Ok(())
    }

    /// Show current sysctl configuration
    async fn show_config(&self) -> Result<()> {
        if !fs::metadata(SYSCTL_CONFIG_PATH).is_ok() {
            log_warn("Flux sysctl configuration not found");
            log_info(&format!("Run 'flux module {} --apply' to create it", self.name()));
            return Ok(());
        }

        log_info("Current Flux sysctl configuration:");
        println!("\n{}", "=".repeat(70));

        let config = fs::read_to_string(SYSCTL_CONFIG_PATH)
            .map_err(|e| FluxError::system(format!("Failed to read config: {}", e)))?;

        println!("{}", config);
        println!("{}", "=".repeat(70));

        Ok(())
    }

    /// Verify sysctl settings
    async fn verify_settings(&self) -> Result<()> {
        log_info("Verifying sysctl hardening settings");

        if !fs::metadata(SYSCTL_CONFIG_PATH).is_ok() {
            return Err(FluxError::Module(
                "Flux sysctl configuration not found. Apply hardening first.".to_string(),
            ));
        }

        let params = self.get_hardening_params();
        let mut success_count = 0;
        let mut fail_count = 0;

        println!("\n{:<50} {:<15} {:<15} {}", "Parameter", "Expected", "Current", "Status");
        println!("{}", "-".repeat(95));

        for (key, (expected_value, _)) in params.iter() {
            // Get current value
            let output = Command::new("sysctl")
                .arg("-n")
                .arg(key)
                .output()
                .ok();

            if let Some(out) = output {
                let current_value = String::from_utf8_lossy(&out.stdout).trim().to_string();

                if current_value == *expected_value {
                    println!("{:<50} {:<15} {:<15} ✓", key, expected_value, current_value);
                    success_count += 1;
                } else {
                    println!("{:<50} {:<15} {:<15} ✗", key, expected_value, current_value);
                    fail_count += 1;
                }
            } else {
                println!("{:<50} {:<15} {:<15} N/A", key, expected_value, "not available");
            }
        }

        println!("{}", "-".repeat(95));
        println!(
            "\nVerification complete: {} passed, {} failed",
            success_count, fail_count
        );

        if fail_count > 0 {
            log_warn("Some parameters don't match expected values");
            log_info("This may be normal if your kernel doesn't support all parameters");
        } else {
            log_success("All sysctl parameters verified successfully");
        }

        Ok(())
    }

    /// Remove sysctl hardening
    async fn remove_hardening(&self) -> Result<()> {
        log_info("Removing sysctl hardening configuration");

        if !fs::metadata(SYSCTL_CONFIG_PATH).is_ok() {
            log_warn("Flux sysctl configuration not found");
            return Ok(());
        }

        let confirm = prompt_yes_no(
            "Are you sure you want to remove sysctl hardening?",
            false,
        )?;

        if !confirm {
            log_info("Removal cancelled");
            return Ok(());
        }

        // Backup before removal
        fs::create_dir_all(SYSCTL_BACKUP_DIR)?;
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
        let backup_path = format!("{}/sysctl-removed-{}.conf", SYSCTL_BACKUP_DIR, timestamp);
        fs::copy(SYSCTL_CONFIG_PATH, &backup_path)?;
        log_info(&format!("Backed up config to: {}", backup_path));

        // Remove configuration file
        fs::remove_file(SYSCTL_CONFIG_PATH)?;
        log_success("Sysctl hardening configuration removed");

        log_warn("System defaults will be used after reboot");
        log_info("To restore original values now, run: sysctl --system");

        Ok(())
    }

    /// Show interactive menu
    async fn show_menu(&self) -> Result<()> {
        loop {
            let options = vec![
                "Apply hardening (security only)",
                "Apply hardening (security + performance)",
                "Show current configuration",
                "Verify settings",
                "Remove hardening",
                "Exit",
            ];

            let choice = select_from_menu("Sysctl Management", &options)?;

            match choice {
                0 => {
                    self.apply_hardening(false, false).await?;
                }
                1 => {
                    self.apply_hardening(true, false).await?;
                }
                2 => {
                    self.show_config().await?;
                }
                3 => {
                    self.verify_settings().await?;
                }
                4 => {
                    self.remove_hardening().await?;
                }
                5 => {
                    log_info("Exiting sysctl management");
                    break;
                }
                _ => {}
            }
        }

        Ok(())
    }
}

#[async_trait]
impl Module for SysctlModule {
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
        check_command("sysctl").is_ok()
    }

    fn help(&self) -> String {
        format!(
            r#"Sysctl Hardening Module v{}

DESCRIPTION:
    {}

    This module applies kernel security hardening through sysctl parameters,
    including network security, kernel protections, and filesystem hardening.

USAGE:
    flux module {} [OPTIONS]

OPTIONS:
    --apply                      Apply security hardening
    --apply-performance          Apply security + performance tuning
    --force                      Force overwrite existing configuration
    --show                       Show current configuration
    --verify                     Verify applied settings
    --remove                     Remove hardening configuration
    --menu                       Show interactive menu

HARDENING AREAS:
    - Kernel: ASLR, ptrace restrictions, panic handling
    - Network: IP forwarding, ICMP, SYN flood protection
    - IPv6: Disable forwarding, router advertisements
    - Filesystem: Hardlink/symlink protection

EXAMPLES:
    flux module {} --menu
    flux module {} --apply
    flux module {} --apply-performance
    flux module {} --verify
    flux module {} --show
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
                "--apply" => {
                    let force = args.contains(&"--force".to_string());
                    self.apply_hardening(false, force).await?;
                    i += 1;
                }
                "--apply-performance" => {
                    let force = args.contains(&"--force".to_string());
                    self.apply_hardening(true, force).await?;
                    i += 1;
                }
                "--show" => {
                    self.show_config().await?;
                    i += 1;
                }
                "--verify" => {
                    self.verify_settings().await?;
                    i += 1;
                }
                "--remove" => {
                    self.remove_hardening().await?;
                    i += 1;
                }
                "--force" => {
                    // Flag handled in --apply
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
