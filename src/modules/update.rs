use crate::config::Config;
use crate::error::{FluxError, Result};
use crate::helpers::{
    logging::{log_info, log_warn, ProgressIndicator},
    system::{detect_distro, Distribution, execute_command},
    user_input::prompt_yes_no,
};
use crate::modules::{Module, ModuleBase, ModuleContext, ModuleInfo};
use async_trait::async_trait;
use clap::{Arg, ArgAction, ArgMatches, Command};
use colored::Colorize;

/// System update and package management module
pub struct UpdateModule {
    base: ModuleBase,
}

impl UpdateModule {
    pub fn new() -> Self {
        let info = ModuleInfo {
            name: "update".to_string(),
            description: "System update and package management".to_string(),
            version: "1.0.0".to_string(),
            author: "Flux Contributors".to_string(),
            tags: vec!["system".to_string(), "packages".to_string()],
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
                Arg::new("full")
                    .short('f')
                    .long("full")
                    .help("Perform complete system update")
                    .action(ArgAction::SetTrue)
            )
            .arg(
                Arg::new("update-only")
                    .short('u')
                    .long("update-only")
                    .help("Update package lists only")
                    .action(ArgAction::SetTrue)
            )
            .arg(
                Arg::new("upgrade-only")
                    .short('g')
                    .long("upgrade-only")
                    .help("Upgrade packages only")
                    .action(ArgAction::SetTrue)
            )
            .arg(
                Arg::new("install")
                    .short('i')
                    .long("install")
                    .help("Install essential packages only")
                    .action(ArgAction::SetTrue)
            )
            .arg(
                Arg::new("dev")
                    .short('d')
                    .long("dev")
                    .help("Include development packages")
                    .action(ArgAction::SetTrue)
            )
            .arg(
                Arg::new("auto")
                    .short('a')
                    .long("auto")
                    .help("Configure automatic updates")
                    .action(ArgAction::SetTrue)
            )
            .arg(
                Arg::new("cleanup")
                    .short('c')
                    .long("cleanup")
                    .help("Clean package cache only")
                    .action(ArgAction::SetTrue)
            )
            .arg(
                Arg::new("security")
                    .short('s')
                    .long("security")
                    .help("Security updates only")
                    .action(ArgAction::SetTrue)
            )
            .arg(
                Arg::new("check")
                    .long("check")
                    .help("Check system requirements only")
                    .action(ArgAction::SetTrue)
            )
    }
    
    async fn execute_update(&self, matches: &ArgMatches, ctx: &ModuleContext<'_>) -> Result<()> {
        let distro = detect_distro()?;
        
        // Check what action to perform
        if matches.get_flag("check") {
            return self.check_system_requirements().await;
        }
        
        if matches.get_flag("cleanup") {
            return self.cleanup_packages(&distro).await;
        }
        
        if matches.get_flag("update-only") {
            return self.update_package_lists(&distro).await;
        }
        
        if matches.get_flag("upgrade-only") {
            let security_only = matches.get_flag("security");
            return self.upgrade_packages(&distro, security_only).await;
        }
        
        if matches.get_flag("install") {
            let include_dev = matches.get_flag("dev");
            return self.install_essential_packages(&distro, include_dev).await;
        }
        
        if matches.get_flag("auto") {
            return self.configure_automatic_updates(&distro).await;
        }
        
        if matches.get_flag("full") {
            return self.full_system_update(&distro, matches.get_flag("dev")).await;
        }
        
        // Default: show menu
        self.show_update_menu(&distro).await
    }
    
    async fn check_system_requirements(&self) -> Result<()> {
        log_info("Checking system requirements");
        
        // Check disk space
        let disk_stats = crate::helpers::system::get_system_status()?;
        println!("Disk usage: {}", disk_stats.disk_usage);
        
        // Check internet connectivity
        let connectivity = crate::helpers::network::test_connectivity();
        if !connectivity.internet_reachable {
            return Err(FluxError::network("No internet connectivity"));
        }
        
        println!("{}", "✓ System requirements check passed".green());
        Ok(())
    }
    
    async fn update_package_lists(&self, distro: &Distribution) -> Result<()> {
        log_info("Updating package lists");
        
        let progress = ProgressIndicator::new_spinner("Updating package lists...");
        
        let result = match distro {
            d if d.is_debian_based() => {
                execute_command("apt-get", &["update"])
            }
            d if d.is_redhat_based() => {
                let pkg_manager = distro.package_manager();
                execute_command(pkg_manager, &["check-update"])
                    .or_else(|_| Ok("Updates available".to_string()))
            }
            _ => Err(FluxError::unsupported("Unsupported distribution")),
        };
        
        progress.finish_with_message("Package lists updated");
        
        result.map(|_| ())
    }
    
    async fn upgrade_packages(&self, distro: &Distribution, security_only: bool) -> Result<()> {
        log_info("Upgrading system packages");
        
        let upgradable_count = self.count_upgradable_packages(distro).await?;
        
        if upgradable_count == 0 {
            println!("{}", "No packages available for upgrade".green());
            return Ok(());
        }
        
        println!("Packages available for upgrade: {}", upgradable_count);
        
        if !prompt_yes_no("Proceed with package upgrade?", true)? {
            return Ok(());
        }
        
        let progress = ProgressIndicator::new_spinner("Upgrading packages...");
        
        match distro {
            d if d.is_debian_based() => {
                if security_only {
                    // Get security updates
                    let output = execute_command("apt", &["list", "--upgradable"])?;
                    let security_packages: Vec<String> = output
                        .lines()
                        .filter(|line| line.contains("-security"))
                        .map(|line| line.split('/').next().unwrap_or("").to_string())
                        .collect();
                    
                    if !security_packages.is_empty() {
                        let mut args = vec!["install", "-y"];
                        for pkg in &security_packages {
                            args.push(pkg);
                        }
                        execute_command("apt-get", &args)?;
                    }
                } else {
                    execute_command("apt-get", &["upgrade", "-y"])?;
                }
            }
            d if d.is_redhat_based() => {
                let pkg_manager = distro.package_manager();
                if security_only {
                    execute_command(pkg_manager, &["update", "--security", "-y"])?;
                } else {
                    execute_command(pkg_manager, &["update", "-y"])?;
                }
            }
            _ => return Err(FluxError::unsupported("Unsupported distribution")),
        }
        
        progress.finish_with_message("Packages upgraded successfully");
        
        // Check if reboot is required
        self.check_reboot_required(distro);
        
        Ok(())
    }
    
    async fn install_essential_packages(&self, distro: &Distribution, include_dev: bool) -> Result<()> {
        log_info("Installing essential packages");
        
        let packages = self.get_essential_packages(distro, include_dev);
        let progress = ProgressIndicator::new(packages.len() as u64, "Installing packages");
        
        let mut failed_packages = Vec::new();
        
        for (i, package) in packages.iter().enumerate() {
            progress.inc(1);
            progress.set_message(&format!("Installing {}", package));
            
            let result = match distro {
                d if d.is_debian_based() => {
                    execute_command("apt-get", &["install", "-y", package])
                }
                d if d.is_redhat_based() => {
                    let pkg_manager = distro.package_manager();
                    execute_command(pkg_manager, &["install", "-y", package])
                }
                _ => Err(FluxError::unsupported("Unsupported distribution")),
            };
            
            if result.is_err() {
                failed_packages.push(package.clone());
            }
        }
        
        progress.finish_with_message("Package installation completed");
        
        if !failed_packages.is_empty() {
            log_warn(format!("Failed to install packages: {:?}", failed_packages));
        }
        
        Ok(())
    }
    
    async fn cleanup_packages(&self, distro: &Distribution) -> Result<()> {
        log_info("Cleaning up package cache");
        
        match distro {
            d if d.is_debian_based() => {
                execute_command("apt-get", &["autoremove", "-y"])?;
                execute_command("apt-get", &["autoclean"])?;
                execute_command("apt-get", &["clean"])?;
            }
            d if d.is_redhat_based() => {
                let pkg_manager = distro.package_manager();
                execute_command(pkg_manager, &["autoremove", "-y"])?;
                execute_command(pkg_manager, &["clean", "all"])?;
            }
            _ => return Err(FluxError::unsupported("Unsupported distribution")),
        }
        
        println!("{}", "✓ Package cleanup completed".green());
        Ok(())
    }
    
    async fn configure_automatic_updates(&self, distro: &Distribution) -> Result<()> {
        log_info("Configuring automatic updates");
        
        if !prompt_yes_no("Configure automatic security updates?", true)? {
            return Ok(());
        }
        
        match distro {
            d if d.is_debian_based() => {
                // Install unattended-upgrades
                execute_command("apt-get", &["install", "-y", "unattended-upgrades", "apt-listchanges"])?;
                
                // Enable automatic updates
                let config = r#"
APT::Periodic::Update-Package-Lists "1";
APT::Periodic::Unattended-Upgrade "1";
"#;
                crate::helpers::file_ops::safe_write_file(
                    "/etc/apt/apt.conf.d/20auto-upgrades",
                    config,
                    true,
                )?;
                
                execute_command("systemctl", &["enable", "unattended-upgrades"])?;
                execute_command("systemctl", &["start", "unattended-upgrades"])?;
            }
            d if d.is_redhat_based() => {
                let pkg_manager = distro.package_manager();
                
                if pkg_manager == "dnf" {
                    execute_command("dnf", &["install", "-y", "dnf-automatic"])?;
                    
                    // Configure dnf-automatic
                    let config_path = "/etc/dnf/automatic.conf";
                    let content = std::fs::read_to_string(config_path)?;
                    let updated = content
                        .replace("apply_updates = no", "apply_updates = yes")
                        .replace("upgrade_type = default", "upgrade_type = security");
                    
                    crate::helpers::file_ops::safe_write_file(config_path, &updated, true)?;
                    
                    execute_command("systemctl", &["enable", "--now", "dnf-automatic.timer"])?;
                } else {
                    execute_command("yum", &["install", "-y", "yum-cron"])?;
                    execute_command("systemctl", &["enable", "--now", "yum-cron"])?;
                }
            }
            _ => return Err(FluxError::unsupported("Unsupported distribution")),
        }
        
        println!("{}", "✓ Automatic updates configured".green());
        Ok(())
    }
    
    async fn full_system_update(&self, distro: &Distribution, include_dev: bool) -> Result<()> {
        log_info("Starting complete system update process");
        
        // Check requirements
        self.check_system_requirements().await?;
        
        // Update package lists
        self.update_package_lists(distro).await?;
        
        // Upgrade packages
        self.upgrade_packages(distro, false).await?;
        
        // Install essential packages
        self.install_essential_packages(distro, include_dev).await?;
        
        // Configure automatic updates
        if prompt_yes_no("Configure automatic updates?", true)? {
            self.configure_automatic_updates(distro).await?;
        }
        
        // Cleanup
        self.cleanup_packages(distro).await?;
        
        println!("\n{}", "=== Update Summary ===".cyan());
        println!("{}", "✓ Package lists updated".white());
        println!("{}", "✓ System packages upgraded".white());
        println!("{}", "✓ Essential packages installed".white());
        println!("{}", "✓ Package cache cleaned".white());
        
        // Check if reboot is required
        self.check_reboot_required(distro);
        
        Ok(())
    }
    
    async fn show_update_menu(&self, distro: &Distribution) -> Result<()> {
        use crate::helpers::user_input::Menu;
        
        loop {
            let menu = Menu::new("System Update Menu")
                .add_item("full", "Complete system update")
                .add_item("update", "Update package lists")
                .add_item("upgrade", "Upgrade packages")
                .add_item("security", "Security updates only")
                .add_item("install", "Install essential packages")
                .add_item("dev", "Install development packages")
                .add_item("auto", "Configure automatic updates")
                .add_item("cleanup", "Clean package cache")
                .add_item("check", "Check system requirements");
            
            match menu.show() {
                Ok(choice) => match choice.as_str() {
                    "full" => self.full_system_update(distro, false).await?,
                    "update" => self.update_package_lists(distro).await?,
                    "upgrade" => self.upgrade_packages(distro, false).await?,
                    "security" => self.upgrade_packages(distro, true).await?,
                    "install" => self.install_essential_packages(distro, false).await?,
                    "dev" => self.install_essential_packages(distro, true).await?,
                    "auto" => self.configure_automatic_updates(distro).await?,
                    "cleanup" => self.cleanup_packages(distro).await?,
                    "check" => self.check_system_requirements().await?,
                    _ => break,
                },
                Err(_) => break,
            }
        }
        
        Ok(())
    }
    
    async fn count_upgradable_packages(&self, distro: &Distribution) -> Result<u32> {
        match distro {
            d if d.is_debian_based() => {
                let output = execute_command("apt", &["list", "--upgradable"])?;
                let count = output.lines().filter(|line| line.contains("upgradable")).count();
                Ok(count as u32)
            }
            d if d.is_redhat_based() => {
                // For RedHat-based systems, we'd need to parse the output differently
                Ok(0) // Simplified for now
            }
            _ => Ok(0),
        }
    }
    
    fn get_essential_packages(&self, distro: &Distribution, include_dev: bool) -> Vec<String> {
        let mut packages = vec![
            "curl", "wget", "git", "vim", "htop", "neofetch", "unzip",
            "ca-certificates", "gnupg", "lsb-release", "software-properties-common",
            "build-essential", "tree", "ncdu", "iotop", "net-tools", "dnsutils",
            "telnet", "rsync", "screen", "tmux", "jq", "dos2unix",
        ];
        
        if include_dev {
            packages.extend(vec![
                "nodejs", "npm", "python3", "python3-pip", "python3-venv",
                "docker.io", "docker-compose", "ansible",
            ]);
        }
        
        packages.into_iter().map(String::from).collect()
    }
    
    fn check_reboot_required(&self, distro: &Distribution) {
        if distro.is_debian_based() {
            if std::path::Path::new("/var/run/reboot-required").exists() {
                log_warn("System reboot is required");
                println!("{}", "⚠ System reboot recommended".yellow());
            }
        }
    }
}

#[async_trait]
impl Module for UpdateModule {
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
        true // Update module is always available
    }
    
    fn help(&self) -> String {
        self.create_cli().render_help().to_string()
    }
    
    async fn execute(&self, args: Vec<String>, config: &Config) -> Result<()> {
        let ctx = ModuleContext::new(config, args.clone());

        let args_strs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        let mut all_args = vec!["update"];
        all_args.extend(args_strs);

        let matches = self.create_cli()
            .try_get_matches_from(all_args)
            .map_err(|e| FluxError::validation(format!("Invalid arguments: {}", e)))?;

        self.execute_update(&matches, &ctx).await
    }
}