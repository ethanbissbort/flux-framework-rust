use crate::error::Result;
use clap::{ArgMatches, Command};
use colored::Colorize;

/// Flux CLI builder and handler
pub struct FluxCli;

impl FluxCli {
    /// Build the CLI command structure
    pub fn build() -> Command {
        Command::new("flux")
            .about("Flux System Administration Framework")
            .version(crate::VERSION)
            .author("Flux Contributors")
            .arg_required_else_help(true)
            .subcommand_required(true)
            .subcommand(
                Command::new("list")
                    .about("List available modules")
                    .alias("ls")
            )
            .subcommand(
                Command::new("status")
                    .about("Show system status")
                    .alias("st")
            )
            .subcommand(
                Command::new("load")
                    .about("Load and execute a specific module")
                    .arg(
                        clap::Arg::new("module")
                            .help("Module name")
                            .required(true)
                            .index(1)
                    )
                    .arg(
                        clap::Arg::new("args")
                            .help("Module arguments")
                            .num_args(0..)
                            .last(true)
                    )
            )
            .subcommand(
                Command::new("workflow")
                    .about("Execute a predefined workflow")
                    .alias("wf")
                    .arg(
                        clap::Arg::new("name")
                            .help("Workflow name")
                            .required(true)
                            .value_parser([
                                "essential",
                                "complete",
                                "security",
                                "development",
                                "monitoring",
                            ])
                    )
            )
            .subcommand(
                Command::new("config")
                    .about("Get or set configuration values")
                    .arg(
                        clap::Arg::new("key")
                            .help("Configuration key")
                            .index(1)
                    )
                    .arg(
                        clap::Arg::new("value")
                            .help("Configuration value (if setting)")
                            .index(2)
                    )
            )
    }

    /// Print colored banner
    pub fn print_banner() {
        println!();
        println!("{}", r#"
 ███████╗██╗     ██╗   ██╗██╗  ██╗
 ██╔════╝██║     ██║   ██║╚██╗██╔╝
 █████╗  ██║     ██║   ██║ ╚███╔╝ 
 ██╔══╝  ██║     ██║   ██║ ██╔██╗ 
 ██║     ███████╗╚██████╔╝██╔╝ ██╗
 ╚═╝     ╚══════╝ ╚═════╝ ╚═╝  ╚═╝
"#.cyan());
        println!("{}", "    System Administration Framework".white());
        println!("{}", format!("    Version {}", crate::VERSION).bright_black());
        println!();
    }

    /// Print help text with examples
    pub fn print_help() {
        Self::print_banner();
        
        println!("{}", "USAGE:".yellow());
        println!("    flux [COMMAND] [OPTIONS]");
        println!();
        
        println!("{}", "COMMANDS:".yellow());
        println!("    {}    List available modules", "list".green());
        println!("    {}  Show system status", "status".green());
        println!("    {}    Load and execute a module", "load".green());
        println!("    {} Execute a workflow", "workflow".green());
        println!("    {}  Manage configuration", "config".green());
        println!();
        
        println!("{}", "EXAMPLES:".yellow());
        println!("    # List all modules");
        println!("    flux list");
        println!();
        println!("    # Run essential setup workflow");
        println!("    flux workflow essential");
        println!();
        println!("    # Configure SSH");
        println!("    flux load ssh --wizard");
        println!();
        println!("    # Set configuration value");
        println!("    flux config default_ssh_port 2222");
        println!();
        
        println!("{}", "For more help on a command, use:".bright_black());
        println!("    flux [COMMAND] --help");
    }

    /// Interactive module selection
    pub fn select_module(modules: &[String]) -> Result<String> {
        use dialoguer::{theme::ColorfulTheme, Select};
        
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Select a module to load")
            .items(modules)
            .default(0)
            .interact()?;
        
        Ok(modules[selection].clone())
    }

    /// Interactive workflow selection
    pub fn select_workflow() -> Result<String> {
        use dialoguer::{theme::ColorfulTheme, Select};
        
        let workflows = vec![
            ("essential", "Basic system setup (update, certs, sysctl, ssh)"),
            ("complete", "Full system configuration"),
            ("security", "Security hardening workflow"),
            ("development", "Development environment setup"),
            ("monitoring", "Monitoring tools installation"),
        ];
        
        let items: Vec<String> = workflows
            .iter()
            .map(|(name, desc)| format!("{:<12} - {}", name, desc))
            .collect();
        
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Select a workflow to execute")
            .items(&items)
            .default(0)
            .interact()?;
        
        Ok(workflows[selection].0.to_string())
    }

    /// Print module help
    pub fn print_module_help(module_name: &str, help_text: &str) {
        println!("{}", format!("=== {} Module Help ===", module_name).cyan());
        println!();
        println!("{}", help_text);
    }

    /// Print workflow description
    pub fn print_workflow_info(name: &str, modules: &[String]) {
        println!("{}", format!("=== Workflow: {} ===", name).cyan());
        println!();
        println!("{}", "This workflow will execute the following modules:".white());
        
        for (i, module) in modules.iter().enumerate() {
            println!("  {}. {}", i + 1, module);
        }
        
        println!();
    }
}