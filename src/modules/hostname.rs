use crate::config::Config;
use crate::error::{FluxError, Result};
use crate::helpers::{
    file_ops::{backup_file, safe_write_file},
    logging::{log_error, log_info, log_warn},
    system::{execute_command, get_hostname},
    user_input::{prompt_input, prompt_yes_no},
    validation::validate_hostname,
};
use crate::modules::{Module, ModuleBase, ModuleContext, ModuleInfo};
use async_trait::async_trait;
use clap::{Arg, ArgAction, ArgMatches, Command};
use colored::Colorize;
use std::collections::HashMap;

/// Hostname configuration module
pub struct HostnameModule {
    base: ModuleBase,
}

impl HostnameModule {
    pub fn new() -> Self {
        let info = ModuleInfo {
            name: "hostname".to_string(),
            description: "Hostname and FQDN configuration".to_string(),
            version: "1.0.0".to_string(),
            author: "Flux Contributors".to_string(),
            tags: vec!["system".to_string(), "network".to_string()],
            requires_root: true,
            supported_distros: vec!["all".to_string()],
        };
        
        Self {
            base: ModuleBase { info },
        }
    }
    
    fn create_cli(&self) -> Command {
        self.base
            .create_args_parser()
            .arg(
                Arg::new("show")
                    .short('s')
                    .long("show")
                    .help("Show current hostname configuration")
                    .action(ArgAction::SetTrue)
            )
            .arg(
                Arg::new("hostname")
                    .short('n')
                    .long("hostname")
                    .help("Set hostname")
                    .num_args(1)
                    .value_name("NAME")
            )
            .arg(
                Arg::new("fqdn")
                    .short('f')
                    .long("fqdn")
                    .help("Set fully qualified domain name")
                    .num_args(1)
                    .value_name("FQDN")
            )
            .arg(
                Arg::new("interactive")
                    .short('i')
                    .long("interactive")
                    .help("Interactive configuration")
                    .action(ArgAction::SetTrue)
            )
            .arg(
                Arg::new("verify")
                    .short('v')
                    .long("verify")
                    .help("Verify hostname configuration")
                    .action(ArgAction::SetTrue)
            )
    }
    
    async fn execute_hostname(&self, matches: &ArgMatches, ctx: &ModuleContext<'_>) -> Result<()> {
        if matches.get_flag("show") {
            return self.show_hostname_config().await;
        }
        
        if let Some(hostname) = matches.get_one::<String>("hostname") {
            return self.set_hostname(hostname).await;
        }
        
        if let Some(fqdn) = matches.get_one::<String>("fqdn") {
            return self.set_fqdn(fqdn).await;
        }
        
        if matches.get_flag("interactive") {
            return self.configure_interactive().await;
        }
        
        if matches.get_flag("verify") {
            return self.verify_configuration().await;
        }
        
        // Default: show current configuration
        self.show_hostname_config().await
    }
    
    async fn show_hostname_config(&self) -> Result<()> {
        println!("{}", "=== Current Hostname Configuration ===".cyan());
        
        let info = self.get_hostname_info()?;
        
        println!("Hostname: {}", info.hostname);
        println!("FQDN: {}", info.fqdn.as_ref().unwrap_or(&"Not set".to_string()));
        println!("Domain: {}", info.domain.as_ref().unwrap_or(&"Not set".to_string()));
        println!("Short: {}", info.short_name);
        
        // Show hostnamectl output if available
        if crate::helpers::system::command_exists("hostnamectl") {
            println!("\n{}", "Hostnamectl output:".white());
            let output = execute_command("hostnamectl", &["status"])?;
            println!("{}", output);
        }
        
        Ok(())
    }
    
    async fn set_hostname(&self, new_hostname: &str) -> Result<()> {
        log_info(format!("Setting hostname to: {}", new_hostname));
        
        // Validate hostname
        validate_hostname(new_hostname)?;
        
        // Backup configuration files
        backup_file("/etc/hostname")?;
        backup_file("/etc/hosts")?;
        
        // Set hostname using hostnamectl if available
        if crate::helpers::system::command_exists("hostnamectl") {
            log_info("Using hostnamectl to set hostname");
            
            execute_command("hostnamectl", &["set-hostname", new_hostname, "--static"])?;
            execute_command("hostnamectl", &["set-hostname", new_hostname, "--transient"])?;
            
            // Also set pretty hostname if it's a simple name
            if !new_hostname.contains('.') {
                execute_command("hostnamectl", &["set-hostname", new_hostname, "--pretty"])?;
            }
        } else {
            // Fallback method
            log_info("Using traditional method to set hostname");
            
            safe_write_file("/etc/hostname", new_hostname, false)?;
            execute_command("hostname", &[new_hostname])?;
        }
        
        // Update hosts file
        self.update_hosts_file(new_hostname, None).await?;
        
        // Update machine-info if it exists
        let machine_info_path = "/etc/machine-info";
        if std::path::Path::new(machine_info_path).exists() {
            let content = std::fs::read_to_string(machine_info_path).unwrap_or_default();
            let updated = if content.contains("PRETTY_HOSTNAME=") {
                content
                    .lines()
                    .map(|line| {
                        if line.starts_with("PRETTY_HOSTNAME=") {
                            format!("PRETTY_HOSTNAME=\"{}\"", new_hostname)
                        } else {
                            line.to_string()
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            } else {
                format!("{}\nPRETTY_HOSTNAME=\"{}\"", content, new_hostname)
            };
            
            safe_write_file(machine_info_path, &updated, true)?;
        }
        
        log_info("Hostname set successfully");
        
        // Check if services need restart
        self.check_services_needing_restart();
        
        Ok(())
    }
    
    async fn set_fqdn(&self, new_fqdn: &str) -> Result<()> {
        log_info(format!("Setting FQDN to: {}", new_fqdn));
        
        // Validate FQDN
        validate_hostname(new_fqdn)?;
        
        // FQDN must contain at least one dot
        if !new_fqdn.contains('.') {
            return Err(FluxError::validation(
                "FQDN must contain a domain (e.g., host.domain.com)"
            ));
        }
        
        // Extract hostname and domain
        let parts: Vec<&str> = new_fqdn.splitn(2, '.').collect();
        let new_hostname = parts[0];
        let new_domain = parts[1];
        
        log_info(format!("Extracted hostname: {}", new_hostname));
        log_info(format!("Extracted domain: {}", new_domain));
        
        // Set the hostname
        self.set_hostname(new_hostname).await?;
        
        // Update hosts file with FQDN
        self.update_hosts_file(new_hostname, Some(new_fqdn)).await?;
        
        // Set domain in resolv.conf if not already set
        let resolv_conf = "/etc/resolv.conf";
        if std::path::Path::new(resolv_conf).exists() {
            let content = std::fs::read_to_string(resolv_conf)?;
            if !content.contains("domain ") && !content.contains("search ") {
                let updated = format!("{}\ndomain {}", content, new_domain);
                safe_write_file(resolv_conf, &updated, true)?;
            }
        }
        
        log_info("FQDN set successfully");
        Ok(())
    }
    
    async fn configure_interactive(&self) -> Result<()> {
        println!("{}", "=== Hostname Configuration ===".cyan());
        
        // Show current configuration
        let info = self.get_hostname_info()?;
        println!("\n{}", "Current Configuration:".white());
        println!("  Hostname: {}", info.hostname);
        println!("  FQDN: {}", info.fqdn.as_ref().unwrap_or(&"Not set".to_string()));
        println!("  Domain: {}", info.domain.as_ref().unwrap_or(&"Not set".to_string()));
        println!();
        
        // Ask what to configure
        println!("What would you like to configure?");
        println!("  1) Simple hostname only");
        println!("  2) Fully Qualified Domain Name (FQDN)");
        println!("  3) Cancel");
        println!();
        
        let choice = prompt_input("Select option [1-3]")?;
        
        match choice.as_str() {
            "1" => {
                // Simple hostname
                let new_hostname = loop {
                    let input = prompt_input("Enter new hostname")?;
                    match validate_hostname(&input) {
                        Ok(_) => break input,
                        Err(e) => eprintln!("{} {}", "[ERROR]".red(), e),
                    }
                };
                
                self.set_hostname(&new_hostname).await?;
                println!("\n{}", format!("Hostname set to: {}", new_hostname).green());
            }
            
            "2" => {
                // FQDN
                println!("\n{}", "Note: FQDN should be in format: hostname.domain.tld".yellow());
                println!("{}", "Example: server01.example.com".yellow());
                println!();
                
                let new_fqdn = loop {
                    let input = prompt_input("Enter new FQDN")?;
                    
                    // Validate as hostname first
                    if let Err(e) = validate_hostname(&input) {
                        eprintln!("{} {}", "[ERROR]".red(), e);
                        continue;
                    }
                    
                    // Check for domain
                    if !input.contains('.') {
                        eprintln!("{} FQDN must contain a domain (e.g., host.domain.com)", "[ERROR]".red());
                        continue;
                    }
                    
                    break input;
                };
                
                self.set_fqdn(&new_fqdn).await?;
                println!("\n{}", format!("FQDN set to: {}", new_fqdn).green());
            }
            
            "3" => {
                log_info("Configuration cancelled");
                return Ok(());
            }
            
            _ => {
                return Err(FluxError::validation("Invalid option"));
            }
        }
        
        // Show new configuration
        println!("\n{}", "New Configuration:".white());
        let new_info = self.get_hostname_info()?;
        println!("  Hostname: {}", new_info.hostname);
        println!("  FQDN: {}", new_info.fqdn.as_ref().unwrap_or(&"Not set".to_string()));
        println!("  Domain: {}", new_info.domain.as_ref().unwrap_or(&"Not set".to_string()));
        
        println!("\n{}", "Note: You may need to reconnect SSH sessions for changes to take effect".yellow());
        
        Ok(())
    }
    
    async fn verify_configuration(&self) -> Result<()> {
        println!("{}", "=== Hostname Configuration Verification ===".cyan());
        
        let mut all_good = true;
        let info = self.get_hostname_info()?;
        
        // Check hostname command
        print!("Hostname command: ");
        if !info.hostname.is_empty() && info.hostname != "localhost" {
            println!("{}", info.hostname.green());
        } else {
            println!("{}", "Not set properly".red());
            all_good = false;
        }
        
        // Check /etc/hostname
        print!("/etc/hostname: ");
        if let Ok(file_hostname) = std::fs::read_to_string("/etc/hostname") {
            let file_hostname = file_hostname.trim();
            if file_hostname == info.hostname {
                println!("{}", file_hostname.green());
            } else {
                println!("{} (mismatch)", file_hostname.yellow());
                all_good = false;
            }
        } else {
            println!("{}", "File not found".red());
            all_good = false;
        }
        
        // Check FQDN resolution
        print!("FQDN resolution: ");
        if let Some(fqdn) = &info.fqdn {
            println!("{}", fqdn.green());
        } else {
            println!("{}", "Not set".yellow());
        }
        
        // Check /etc/hosts
        print!("/etc/hosts entries: ");
        let hosts_content = std::fs::read_to_string("/etc/hosts").unwrap_or_default();
        if hosts_content.contains(&info.hostname) {
            println!("{}", "Found".green());
            
            // Show relevant entries
            println!("  Entries containing hostname:");
            for line in hosts_content.lines() {
                if line.contains(&info.hostname) {
                    println!("    {}", line);
                }
            }
        } else {
            println!("{}", "Not found".red());
            all_good = false;
        }
        
        // DNS resolution test
        print!("DNS resolution test: ");
        if let Ok(output) = execute_command("host", &[&info.hostname]) {
            if output.contains("has address") {
                let ip = output
                    .lines()
                    .find(|l| l.contains("has address"))
                    .and_then(|l| l.split_whitespace().last())
                    .unwrap_or("unknown");
                println!("{}", format!("OK ({})", ip).green());
            } else {
                println!("{}", "Cannot resolve (this is normal for local hostnames)".yellow());
            }
        } else {
            println!("{}", "host command not available".yellow());
        }
        
        // Overall status
        println!();
        if all_good {
            println!("{}", "✓ Hostname configuration is correct".green());
        } else {
            println!("{}", "⚠ Some issues detected with hostname configuration".yellow());
        }
        
        Ok(())
    }
    
    async fn update_hosts_file(&self, hostname: &str, fqdn: Option<&str>) -> Result<()> {
        log_info("Updating /etc/hosts");
        
        let hosts_path = "/etc/hosts";
        let content = std::fs::read_to_string(hosts_path).unwrap_or_default();
        
        // Get primary IP address
        let primary_ip = crate::helpers::network::get_network_interfaces()?
            .into_iter()
            .find(|iface| !iface.is_loopback && iface.is_up)
            .and_then(|iface| iface.ips.into_iter().find(|ip| ip.is_ipv4()))
            .map(|ip| ip.to_string())
            .unwrap_or_default();
        
        // Filter out old hostname entries
        let mut new_lines: Vec<String> = content
            .lines()
            .filter(|line| {
                !line.contains(&format!("127.0.1.1.*{}", hostname)) &&
                !line.contains(&format!("{}.*{}", primary_ip, hostname))
            })
            .map(String::from)
            .collect();
        
        // Ensure localhost entries exist
        if !new_lines.iter().any(|l| l.contains("127.0.0.1") && l.contains("localhost")) {
            new_lines.insert(0, "127.0.0.1 localhost".to_string());
        }
        
        // Add new entries
        if let Some(fqdn) = fqdn {
            // Add both FQDN and short hostname
            new_lines.push(format!("127.0.1.1 {} {}", fqdn, hostname));
            if !primary_ip.is_empty() {
                new_lines.push(format!("{} {} {}", primary_ip, fqdn, hostname));
            }
        } else {
            // Just hostname
            new_lines.push(format!("127.0.1.1 {}", hostname));
            if !primary_ip.is_empty() {
                new_lines.push(format!("{} {}", primary_ip, hostname));
            }
        }
        
        // Ensure IPv6 localhost
        if !new_lines.iter().any(|l| l.contains("::1") && l.contains("localhost")) {
            new_lines.push("::1 localhost ip6-localhost ip6-loopback".to_string());
        }
        
        let new_content = new_lines.join("\n") + "\n";
        safe_write_file(hosts_path, &new_content, false)?;
        
        log_info("Hosts file updated");
        Ok(())
    }
    
    fn get_hostname_info(&self) -> Result<HostnameInfo> {
        let hostname = get_hostname()?;
        
        // Try to get FQDN
        let fqdn = execute_command("hostname", &["-f"])
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty() && s != &hostname);
        
        // Try to get domain
        let domain = execute_command("hostname", &["-d"])
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
        
        // Get short name
        let short_name = execute_command("hostname", &["-s"])
            .ok()
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| hostname.clone());
        
        Ok(HostnameInfo {
            hostname,
            fqdn,
            domain,
            short_name,
        })
    }
    
    fn check_services_needing_restart(&self) {
        let services_to_check = vec![
            "ssh", "rsyslog", "postfix", "nginx", "apache2", 
            "mysql", "postgresql", "docker",
        ];
        
        let mut services_needing_restart = Vec::new();
        
        for service in services_to_check {
            if crate::helpers::system::is_service_active(service).unwrap_or(false) {
                services_needing_restart.push(service);
            }
        }
        
        if !services_needing_restart.is_empty() {
            println!("\n{}", "The following services may need to be restarted:".yellow());
            for service in &services_needing_restart {
                println!("  - {}", service);
            }
            
            if let Ok(true) = prompt_yes_no("Restart these services now?", false) {
                for service in services_needing_restart {
                    log_info(format!("Restarting {}", service));
                    let _ = execute_command("systemctl", &["restart", service]);
                }
            }
        }
    }
}

#[derive(Debug)]
struct HostnameInfo {
    hostname: String,
    fqdn: Option<String>,
    domain: Option<String>,
    short_name: String,
}

#[async_trait]
impl Module for HostnameModule {
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
        true
    }
    
    fn help(&self) -> String {
        self.create_cli().render_help().to_string()
    }
    
    async fn execute(&self, args: Vec<String>, config: &Config) -> Result<()> {
        let ctx = ModuleContext::new(config, args.clone());

        let args_strs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        let mut all_args = vec!["hostname"];
        all_args.extend(args_strs);

        let matches = self.create_cli()
            .try_get_matches_from(all_args)
            .map_err(|e| FluxError::validation(format!("Invalid arguments: {}", e)))?;

        self.execute_hostname(&matches, &ctx).await
    }
}