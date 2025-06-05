use crate::config::Config;
use crate::error::{FluxError, Result};
use crate::helpers::{
    file_ops::{backup_file, safe_write_file},
    logging::{log_error, log_info, log_warn},
    network::{get_network_interfaces, NetworkInterface},
    system::{detect_distro, Distribution, execute_command},
    user_input::{prompt_input, prompt_ip, prompt_yes_no, prompt_select},
    validation::{validate_interface, validate_ip, validate_vlan},
};
use crate::modules::{Module, ModuleBase, ModuleContext, ModuleInfo};
use async_trait::async_trait;
use clap::{Arg, ArgAction, ArgMatches, Command};
use colored::Colorize;
use ipnetwork::IpNetwork;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Network configuration module
pub struct NetworkModule {
    base: ModuleBase,
}

/// Network configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub interface: String,
    pub dhcp: bool,
    pub address: Option<String>,
    pub netmask: Option<String>,
    pub gateway: Option<String>,
    pub dns: Vec<String>,
    pub mtu: Option<u32>,
    pub vlan_id: Option<u16>,
}

impl NetworkModule {
    pub fn new() -> Self {
        let info = ModuleInfo {
            name: "network".to_string(),
            description: "Network configuration and management".to_string(),
            version: "1.0.0".to_string(),
            author: "Flux Contributors".to_string(),
            tags: vec!["network".to_string(), "configuration".to_string()],
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
                Arg::new("list")
                    .short('l')
                    .long("list")
                    .help("List network interfaces")
                    .action(ArgAction::SetTrue)
            )
            .arg(
                Arg::new("configure")
                    .short('c')
                    .long("configure")
                    .help("Configure interface interactively")
                    .action(ArgAction::SetTrue)
            )
            .arg(
                Arg::new("diagnostics")
                    .short('d')
                    .long("diagnostics")
                    .help("Run network diagnostics")
                    .action(ArgAction::SetTrue)
            )
            .arg(
                Arg::new("restart")
                    .short('r')
                    .long("restart")
                    .help("Restart networking")
                    .action(ArgAction::SetTrue)
            )
            .arg(
                Arg::new("add-vlan")
                    .long("add-vlan")
                    .help("Add VLAN interface")
                    .num_args(2)
                    .value_names(&["INTERFACE", "VLAN_ID"])
            )
            .arg(
                Arg::new("dhcp")
                    .long("dhcp")
                    .help("Configure interface for DHCP")
                    .num_args(1)
                    .value_name("INTERFACE")
            )
            .arg(
                Arg::new("static")
                    .long("static")
                    .help("Configure static IP")
                    .num_args(2)
                    .value_names(&["INTERFACE", "IP_ADDRESS"])
            )
    }
    
    async fn execute_network(&self, matches: &ArgMatches, ctx: &ModuleContext<'_>) -> Result<()> {
        if matches.get_flag("list") {
            return self.list_interfaces().await;
        }
        
        if matches.get_flag("configure") {
            return self.configure_interactive().await;
        }
        
        if matches.get_flag("diagnostics") {
            return self.run_diagnostics().await;
        }
        
        if matches.get_flag("restart") {
            return self.restart_networking().await;
        }
        
        if let Some(values) = matches.get_many::<String>("add-vlan") {
            let vals: Vec<&String> = values.collect();
            if vals.len() == 2 {
                return self.add_vlan_interface(vals[0], vals[1]).await;
            }
        }
        
        if let Some(interface) = matches.get_one::<String>("dhcp") {
            return self.configure_dhcp(interface).await;
        }
        
        if let Some(values) = matches.get_many::<String>("static") {
            let vals: Vec<&String> = values.collect();
            if vals.len() == 2 {
                return self.configure_static(vals[0], vals[1]).await;
            }
        }
        
        // Default: show menu
        self.show_network_menu().await
    }
    
    async fn list_interfaces(&self) -> Result<()> {
        println!("{}", "=== Network Interfaces ===".cyan());
        println!();
        
        let interfaces = get_network_interfaces()?;
        
        // Physical interfaces
        println!("{}", "Physical Interfaces:".white());
        for iface in interfaces.iter().filter(|i| !i.is_loopback) {
            let status = if iface.is_up {
                "UP".green()
            } else {
                "DOWN".red()
            };
            
            println!("  {:<15} {}", format!("{}:", iface.name).white(), status);
            println!("    MAC: {}", iface.mac);
            
            if !iface.ips.is_empty() {
                let ips: Vec<String> = iface.ips.iter().map(|ip| ip.to_string()).collect();
                println!("    IPs: {}", ips.join(", "));
            }
            
            println!("    MTU: {}", iface.mtu);
            println!();
        }
        
        // Show routing table
        println!("{}", "Routing Table:".white());
        let routes = crate::helpers::network::get_routing_table()?;
        for route in routes.iter().take(10) {
            println!("  {} via {} dev {}", 
                route.destination,
                route.gateway.as_ref().unwrap_or(&"*".to_string()),
                route.interface
            );
        }
        
        // DNS configuration
        println!("\n{}", "DNS Configuration:".white());
        let dns_servers = crate::helpers::network::get_dns_servers()?;
        for dns in dns_servers {
            println!("  {}", dns);
        }
        
        Ok(())
    }
    
    async fn configure_interactive(&self) -> Result<()> {
        log_info("Starting interactive interface configuration");
        
        // Get available interfaces
        let interfaces = get_network_interfaces()?;
        let iface_names: Vec<String> = interfaces
            .iter()
            .filter(|i| !i.is_loopback)
            .map(|i| i.name.clone())
            .collect();
        
        if iface_names.is_empty() {
            return Err(FluxError::network("No network interfaces found"));
        }
        
        // Show interfaces
        println!("{}", "Available network interfaces:".cyan());
        for (i, name) in iface_names.iter().enumerate() {
            println!("  {}. {}", i + 1, name);
        }
        println!();
        
        // Select interface
        let selection = prompt_select("Select interface to configure", &iface_names, 0)?;
        let interface = &iface_names[selection];
        
        // VLAN configuration
        let vlan_id = if prompt_yes_no("Configure as VLAN interface?", false)? {
            let vlan = prompt_input("Enter VLAN ID (1-4094)")?;
            Some(validate_vlan(&vlan)?)
        } else {
            None
        };
        
        // DHCP or static
        let mut config = NetworkConfig {
            interface: interface.clone(),
            dhcp: false,
            address: None,
            netmask: None,
            gateway: None,
            dns: vec![],
            mtu: None,
            vlan_id,
        };
        
        if prompt_yes_no("Use DHCP?", false)? {
            config.dhcp = true;
        } else {
            // Static configuration
            config.address = Some(prompt_ip("Enter IP address", None)?);
            config.netmask = Some(prompt_ip("Enter netmask", Some("255.255.255.0"))?);
            
            // Get default gateway
            let default_gw = crate::helpers::network::get_default_gateway()
                .unwrap_or_else(|_| "".to_string());
            config.gateway = Some(prompt_ip("Enter gateway", Some(&default_gw))?);
            
            // DNS servers
            let dns1 = prompt_ip("Enter primary DNS", Some("1.1.1.1"))?;
            let dns2 = prompt_ip("Enter secondary DNS", Some("8.8.8.8"))?;
            config.dns = vec![dns1, dns2];
            
            // MTU
            let mtu = prompt_input("Enter MTU (or press Enter for default)")?;
            if !mtu.is_empty() {
                config.mtu = Some(mtu.parse().map_err(|_| FluxError::validation("Invalid MTU"))?);
            }
        }
        
        // Apply configuration
        self.apply_network_config(&config).await?;
        
        // Restart networking
        if prompt_yes_no("Restart networking to apply changes?", true)? {
            self.restart_networking().await?;
        }
        
        Ok(())
    }
    
    async fn apply_network_config(&self, config: &NetworkConfig) -> Result<()> {
        let distro = detect_distro()?;
        let net_manager = self.detect_network_manager()?;
        
        log_info(format!("Applying network configuration using {}", net_manager));
        
        match net_manager.as_str() {
            "interfaces" => self.configure_interface_debian(config).await,
            "netplan" => self.configure_interface_netplan(config).await,
            "networkmanager" => self.configure_interface_networkmanager(config).await,
            _ => Err(FluxError::unsupported("Unknown network management system")),
        }
    }
    
    async fn configure_interface_debian(&self, config: &NetworkConfig) -> Result<()> {
        let interfaces_file = "/etc/network/interfaces";
        
        // Backup interfaces file
        backup_file(interfaces_file)?;
        
        // Read existing content
        let mut content = std::fs::read_to_string(interfaces_file).unwrap_or_default();
        
        // Add new configuration
        content.push_str(&format!("\n\nauto {}\n", config.interface));
        
        if config.dhcp {
            content.push_str(&format!("iface {} inet dhcp\n", config.interface));
        } else {
            content.push_str(&format!("iface {} inet static\n", config.interface));
            if let Some(addr) = &config.address {
                content.push_str(&format!("    address {}\n", addr));
            }
            if let Some(mask) = &config.netmask {
                content.push_str(&format!("    netmask {}\n", mask));
            }
            if let Some(gw) = &config.gateway {
                content.push_str(&format!("    gateway {}\n", gw));
            }
            if !config.dns.is_empty() {
                content.push_str(&format!("    dns-nameservers {}\n", config.dns.join(" ")));
            }
        }
        
        safe_write_file(interfaces_file, &content, false)?;
        log_info("Network configuration updated in /etc/network/interfaces");
        
        Ok(())
    }
    
    async fn configure_interface_netplan(&self, config: &NetworkConfig) -> Result<()> {
        let netplan_file = format!("/etc/netplan/50-flux-{}.yaml", config.interface);
        
        // Generate netplan configuration
        let mut yaml = String::from("network:\n  version: 2\n  renderer: networkd\n");
        
        if let Some(vlan_id) = config.vlan_id {
            yaml.push_str(&format!("  vlans:\n    {}.{}:\n", config.interface, vlan_id));
            yaml.push_str(&format!("      id: {}\n", vlan_id));
            yaml.push_str(&format!("      link: {}\n", config.interface));
        } else {
            yaml.push_str(&format!("  ethernets:\n    {}:\n", config.interface));
        }
        
        if config.dhcp {
            yaml.push_str("      dhcp4: true\n");
            yaml.push_str("      dhcp6: false\n");
        } else {
            yaml.push_str("      dhcp4: false\n");
            yaml.push_str("      dhcp6: false\n");
            
            if let Some(addr) = &config.address {
                let prefix = self.netmask_to_prefix(config.netmask.as_deref().unwrap_or("255.255.255.0"));
                yaml.push_str(&format!("      addresses:\n        - {}/{}\n", addr, prefix));
            }
            
            if let Some(gw) = &config.gateway {
                yaml.push_str("      routes:\n");
                yaml.push_str(&format!("        - to: default\n          via: {}\n", gw));
            }
            
            if !config.dns.is_empty() {
                yaml.push_str(&format!("      nameservers:\n        addresses: [{}]\n", 
                    config.dns.join(", ")));
            }
        }
        
        safe_write_file(&netplan_file, &yaml, true)?;
        log_info(format!("Network configuration written to {}", netplan_file));
        
        // Apply netplan configuration
        execute_command("netplan", &["apply"])?;
        
        Ok(())
    }
    
    async fn configure_interface_networkmanager(&self, config: &NetworkConfig) -> Result<()> {
        // NetworkManager configuration would use nmcli commands
        log_warn("NetworkManager configuration not yet implemented");
        Err(FluxError::unsupported("NetworkManager support coming soon"))
    }
    
    async fn run_diagnostics(&self) -> Result<()> {
        println!("{}", "=== Network Diagnostics ===".cyan());
        
        let connectivity = crate::helpers::network::test_connectivity();
        
        println!("\n{}", "Connectivity Tests:".white());
        
        // Gateway
        print!("  Gateway ({}): ", connectivity.gateway);
        if connectivity.gateway_reachable {
            println!("{}", "OK".green());
        } else {
            println!("{}", "FAILED".red());
        }
        
        // DNS
        print!("  DNS ({}): ", connectivity.dns_server);
        if connectivity.dns_reachable {
            println!("{}", "OK".green());
        } else {
            println!("{}", "FAILED".red());
        }
        
        // DNS resolution
        print!("  DNS Resolution: ");
        if connectivity.dns_resolution_working {
            println!("{}", "OK".green());
        } else {
            println!("{}", "FAILED".red());
        }
        
        // Internet
        print!("  Internet (8.8.8.8): ");
        if connectivity.internet_reachable {
            println!("{}", "OK".green());
        } else {
            println!("{}", "FAILED".red());
        }
        
        // Port checks
        println!("\n{}", "Common Ports:".white());
        let ports = vec![
            (22, "SSH"),
            (80, "HTTP"),
            (443, "HTTPS"),
            (53, "DNS"),
        ];
        
        for (port, name) in ports {
            print!("  {} (port {}): ", name, port);
            if crate::helpers::network::is_port_open("localhost", port, 1) {
                println!("{}", "LISTENING".green());
            } else {
                println!("{}", "NOT LISTENING".yellow());
            }
        }
        
        // Network manager
        let net_manager = self.detect_network_manager()?;
        println!("\n{}: {}", "Network Manager".white(), net_manager);
        
        Ok(())
    }
    
    async fn restart_networking(&self) -> Result<()> {
        let net_manager = self.detect_network_manager()?;
        
        log_info(format!("Restarting networking ({})", net_manager));
        
        match net_manager.as_str() {
            "interfaces" => {
                execute_command("systemctl", &["restart", "networking"])?;
            }
            "netplan" => {
                execute_command("netplan", &["apply"])?;
            }
            "networkmanager" => {
                execute_command("systemctl", &["restart", "NetworkManager"])?;
            }
            _ => return Err(FluxError::unsupported("Unknown network system")),
        }
        
        log_info("Networking restarted");
        Ok(())
    }
    
    async fn add_vlan_interface(&self, interface: &str, vlan_id: &str) -> Result<()> {
        // Validate inputs
        validate_interface(interface)?;
        let vlan_id = validate_vlan(vlan_id)?;
        
        // Check if 802.1Q module is loaded
        if execute_command("lsmod", &[]).is_ok() {
            if !execute_command("lsmod", &[])?.contains("8021q") {
                log_info("Loading 802.1Q VLAN module");
                execute_command("modprobe", &["8021q"])?;
                
                // Make it persistent
                let modules_content = "8021q\n";
                crate::helpers::file_ops::safe_append_file("/etc/modules", modules_content, true)?;
            }
        }
        
        let config = NetworkConfig {
            interface: interface.to_string(),
            dhcp: true,
            address: None,
            netmask: None,
            gateway: None,
            dns: vec![],
            mtu: None,
            vlan_id: Some(vlan_id),
        };
        
        self.apply_network_config(&config).await?;
        
        log_info(format!("VLAN interface {}.{} created", interface, vlan_id));
        Ok(())
    }
    
    async fn configure_dhcp(&self, interface: &str) -> Result<()> {
        validate_interface(interface)?;
        
        let config = NetworkConfig {
            interface: interface.to_string(),
            dhcp: true,
            address: None,
            netmask: None,
            gateway: None,
            dns: vec![],
            mtu: None,
            vlan_id: None,
        };
        
        self.apply_network_config(&config).await
    }
    
    async fn configure_static(&self, interface: &str, ip_address: &str) -> Result<()> {
        validate_interface(interface)?;
        validate_ip(ip_address)?;
        
        let config = NetworkConfig {
            interface: interface.to_string(),
            dhcp: false,
            address: Some(ip_address.to_string()),
            netmask: Some("255.255.255.0".to_string()),
            gateway: crate::helpers::network::get_default_gateway().ok(),
            dns: vec!["1.1.1.1".to_string(), "8.8.8.8".to_string()],
            mtu: None,
            vlan_id: None,
        };
        
        self.apply_network_config(&config).await
    }
    
    async fn show_network_menu(&self) -> Result<()> {
        use crate::helpers::user_input::Menu;
        
        loop {
            let menu = Menu::new("Network Configuration Menu")
                .add_item("list", "List network interfaces")
                .add_item("configure", "Configure interface")
                .add_item("dhcp", "Configure DHCP")
                .add_item("static", "Configure static IP")
                .add_item("vlan", "Add VLAN interface")
                .add_item("diagnostics", "Run diagnostics")
                .add_item("restart", "Restart networking");
            
            match menu.show() {
                Ok(choice) => match choice.as_str() {
                    "list" => self.list_interfaces().await?,
                    "configure" => self.configure_interactive().await?,
                    "dhcp" => {
                        let iface = prompt_input("Enter interface name")?;
                        self.configure_dhcp(&iface).await?;
                    }
                    "static" => {
                        let iface = prompt_input("Enter interface name")?;
                        let ip = prompt_ip("Enter IP address", None)?;
                        self.configure_static(&iface, &ip).await?;
                    }
                    "vlan" => {
                        let iface = prompt_input("Enter parent interface")?;
                        let vlan = prompt_input("Enter VLAN ID")?;
                        self.add_vlan_interface(&iface, &vlan).await?;
                    }
                    "diagnostics" => self.run_diagnostics().await?,
                    "restart" => self.restart_networking().await?,
                    _ => break,
                },
                Err(_) => break,
            }
        }
        
        Ok(())
    }
    
    fn detect_network_manager(&self) -> Result<String> {
        if Path::new("/etc/netplan").exists() && crate::helpers::system::command_exists("netplan") {
            Ok("netplan".to_string())
        } else if Path::new("/etc/network/interfaces").exists() {
            Ok("interfaces".to_string())
        } else if Path::new("/etc/NetworkManager").exists() 
            && crate::helpers::system::is_service_active("NetworkManager").unwrap_or(false) {
            Ok("networkmanager".to_string())
        } else {
            Ok("unknown".to_string())
        }
    }
    
    fn netmask_to_prefix(&self, netmask: &str) -> u8 {
        match netmask {
            "255.255.255.0" => 24,
            "255.255.0.0" => 16,
            "255.0.0.0" => 8,
            "255.255.255.128" => 25,
            "255.255.255.192" => 26,
            "255.255.255.224" => 27,
            "255.255.255.240" => 28,
            "255.255.255.248" => 29,
            "255.255.255.252" => 30,
            _ => 24, // Default
        }
    }
}

#[async_trait]
impl Module for NetworkModule {
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
        
        let matches = self.create_cli()
            .try_get_matches_from(["network"].iter().chain(args.iter()))?;
        
        self.execute_network(&matches, &ctx).await
    }
}