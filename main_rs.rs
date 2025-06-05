use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;
use flux_framework::{
    cli::FluxCli,
    config::Config,
    error::FluxError,
    helpers::logging::{init_logging, LogLevel},
    modules::ModuleManager,
    workflows::WorkflowManager,
};
use std::process;
use tracing::{error, info};

/// Flux System Administration Framework
/// A modular, enterprise-grade Linux system configuration and hardening framework
#[derive(Parser)]
#[command(name = "flux")]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    /// Set log level
    #[arg(short = 'L', long, value_enum, default_value = "info")]
    log_level: LogLevel,

    /// Configuration file path
    #[arg(short, long, value_name = "FILE")]
    config: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List available modules
    List,

    /// Show system status
    Status,

    /// Load and execute a specific module
    Load {
        /// Module name
        module: String,
        /// Module arguments
        args: Vec<String>,
    },

    /// Execute a predefined workflow
    Workflow {
        /// Workflow name (essential, complete, security, development, monitoring)
        name: String,
    },

    /// Get or set configuration values
    Config {
        /// Configuration key
        key: Option<String>,
        /// Configuration value (if setting)
        value: Option<String>,
    },

    /// Generate shell completions
    Completions {
        /// Shell type
        #[arg(value_enum)]
        shell: clap_complete::Shell,
    },
}

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        error!("Fatal error: {}", e);
        process::exit(1);
    }
}

async fn run() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    init_logging(cli.log_level)?;

    // Load configuration
    let config = match cli.config {
        Some(path) => Config::from_file(&path)?,
        None => Config::load_or_default()?,
    };

    info!(
        "Flux Framework v{} initialized",
        env!("CARGO_PKG_VERSION")
    );

    // Check if running with appropriate privileges
    check_privileges(&cli.command)?;

    // Execute command
    match cli.command {
        Commands::List => {
            list_modules().await?;
        }
        Commands::Status => {
            show_system_status().await?;
        }
        Commands::Load { module, args } => {
            load_module(&module, args, &config).await?;
        }
        Commands::Workflow { name } => {
            execute_workflow(&name, &config).await?;
        }
        Commands::Config { key, value } => {
            handle_config(key, value, &config)?;
        }
        Commands::Completions { shell } => {
            generate_completions(shell);
        }
    }

    Ok(())
}

fn check_privileges(command: &Commands) -> Result<()> {
    // Some commands require root privileges
    let requires_root = matches!(
        command,
        Commands::Load { .. } | Commands::Workflow { .. } | Commands::Status
    );

    if requires_root && !nix::unistd::Uid::effective().is_root() {
        eprintln!(
            "{}",
            "This command requires root privileges. Please run with sudo.".red()
        );
        process::exit(1);
    }

    Ok(())
}

async fn list_modules() -> Result<()> {
    println!("{}", "=== Available Flux Modules ===".cyan());
    println!();

    let manager = ModuleManager::new()?;
    let modules = manager.discover_modules()?;

    if modules.is_empty() {
        println!("{}", "No modules found".red());
    } else {
        for module in modules {
            let status = if module.is_executable() {
                "✓".green()
            } else {
                "○".yellow()
            };

            println!(
                "{:<20} {} {}",
                module.name().white(),
                status,
                module.description()
            );
        }

        println!();
        println!("{}", format!("Total modules: {}", modules.len()).white());
    }

    Ok(())
}

async fn show_system_status() -> Result<()> {
    println!("{}", "=== System Status Check ===".cyan());
    println!();

    let status = flux_framework::helpers::system::get_system_status()?;

    println!("{}", "System Information:".white());
    println!("  OS: {}", status.os_info);
    println!("  Kernel: {}", status.kernel_version);
    println!("  Architecture: {}", status.architecture);
    println!("  Hostname: {}", status.hostname);

    println!();
    println!("{}", "Resource Usage:".white());
    println!("  CPU Load: {}", status.cpu_load);
    println!("  Memory: {}", status.memory_usage);
    println!("  Disk (/): {}", status.disk_usage);

    println!();
    println!("{}", "Network:".white());
    println!("  Primary IP: {}", status.primary_ip);
    println!("  Gateway: {}", status.gateway);

    println!();
    println!("{}", "Key Services:".white());
    for (service, active) in &status.services {
        let status_text = if *active {
            "active".green()
        } else {
            "inactive".red()
        };
        println!("  {}: {}", service, status_text);
    }

    println!();
    println!("{}", "System Updates:".white());
    if status.reboot_required {
        println!("  {}", "Reboot required".yellow());
    }
    if status.updates_available > 0 {
        println!("  {} updates available", status.updates_available);
    } else {
        println!("  {}", "System up to date".green());
    }

    Ok(())
}

async fn load_module(name: &str, args: Vec<String>, config: &Config) -> Result<()> {
    info!("Loading module: {}", name);

    let manager = ModuleManager::new()?;
    manager.load_module(name, args, config).await?;

    Ok(())
}

async fn execute_workflow(name: &str, config: &Config) -> Result<()> {
    info!("Executing workflow: {}", name);

    let manager = WorkflowManager::new()?;
    manager.execute_workflow(name, config).await?;

    Ok(())
}

fn handle_config(key: Option<String>, value: Option<String>, config: &Config) -> Result<()> {
    match (key, value) {
        (Some(k), Some(v)) => {
            // Set configuration value
            let mut config = config.clone();
            config.set(&k, &v)?;
            config.save()?;
            println!("Set {} = {}", k, v);
        }
        (Some(k), None) => {
            // Get configuration value
            match config.get(&k) {
                Some(v) => println!("{} = {}", k, v),
                None => println!("Key '{}' not found", k),
            }
        }
        (None, _) => {
            // Show all configuration
            println!("{}", "Current configuration:".white());
            for (k, v) in config.all() {
                println!("  {} = {}", k, v);
            }
        }
    }

    Ok(())
}

fn generate_completions(shell: clap_complete::Shell) {
    use clap::CommandFactory;
    use clap_complete::generate;
    use std::io;

    let mut cmd = Cli::command();
    let bin_name = cmd.get_name().to_string();
    generate(shell, &mut cmd, bin_name, &mut io::stdout());
}