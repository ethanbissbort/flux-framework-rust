pub mod certs;
pub mod firewall;
pub mod hostname;
pub mod motd;
pub mod netdata;
pub mod network;
pub mod ssh;
pub mod sysctl;
pub mod update;
pub mod user;
pub mod zsh;

use crate::config::Config;
use crate::error::{FluxError, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Module trait that all modules must implement
#[async_trait]
pub trait Module: Send + Sync {
    /// Get module name
    fn name(&self) -> &str;
    
    /// Get module description
    fn description(&self) -> &str;
    
    /// Get module version
    fn version(&self) -> &str;
    
    /// Check if module is available on this system
    fn is_available(&self) -> bool;
    
    /// Get module help text
    fn help(&self) -> String;
    
    /// Execute module with arguments
    async fn execute(&self, args: Vec<String>, config: &Config) -> Result<()>;
}

/// Module metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleInfo {
    pub name: String,
    pub description: String,
    pub version: String,
    pub author: String,
    pub tags: Vec<String>,
    pub requires_root: bool,
    pub supported_distros: Vec<String>,
}

/// Module manager for discovering and loading modules
pub struct ModuleManager {
    modules: HashMap<String, Box<dyn Module>>,
}

impl ModuleManager {
    /// Create new module manager
    pub fn new() -> Result<Self> {
        let mut modules: HashMap<String, Box<dyn Module>> = HashMap::new();
        
        // Register all modules
        let all_modules: Vec<Box<dyn Module>> = vec![
            Box::new(update::UpdateModule::new()),
            Box::new(network::NetworkModule::new()),
            Box::new(hostname::HostnameModule::new()),
            Box::new(user::UserModule::new()),
            Box::new(ssh::SshModule::new()),
            Box::new(firewall::FirewallModule::new()),
            Box::new(certs::CertsModule::new()),
            Box::new(sysctl::SysctlModule::new()),
            Box::new(zsh::ZshModule::new()),
            Box::new(motd::MotdModule::new()),
            Box::new(netdata::NetdataModule::new()),
        ];
        
        for module in all_modules {
            modules.insert(module.name().to_string(), module);
        }
        
        Ok(Self { modules })
    }
    
    /// Discover available modules
    pub fn discover_modules(&self) -> Result<Vec<ModuleDescriptor>> {
        let mut descriptors = Vec::new();
        
        for (name, module) in &self.modules {
            descriptors.push(ModuleDescriptor {
                name: name.clone(),
                description: module.description().to_string(),
                version: module.version().to_string(),
                available: module.is_available(),
            });
        }
        
        descriptors.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(descriptors)
    }
    
    /// Get module by name
    pub fn get_module(&self, name: &str) -> Result<&Box<dyn Module>> {
        self.modules
            .get(name)
            .ok_or_else(|| FluxError::module(format!("Module '{}' not found", name)))
    }
    
    /// Load and execute a module
    pub async fn load_module(
        &self,
        name: &str,
        args: Vec<String>,
        config: &Config,
    ) -> Result<()> {
        let module = self.get_module(name)?;
        
        if !module.is_available() {
            return Err(FluxError::module(format!(
                "Module '{}' is not available on this system",
                name
            )));
        }
        
        // Check for help flag
        if args.iter().any(|arg| arg == "--help" || arg == "-h") {
            println!("{}", module.help());
            return Ok(());
        }
        
        module.execute(args, config).await
    }
}

/// Module descriptor for listing
#[derive(Debug, Clone)]
pub struct ModuleDescriptor {
    pub name: String,
    pub description: String,
    pub version: String,
    pub available: bool,
}

impl ModuleDescriptor {
    pub fn is_executable(&self) -> bool {
        self.available
    }
}

/// Base implementation for modules
pub struct ModuleBase {
    pub info: ModuleInfo,
}

impl ModuleBase {
    /// Create argument parser with common options
    pub fn create_args_parser(&self) -> clap::Command {
        clap::Command::new(self.info.name.clone().leak() as &str)
            .about(self.info.description.clone().leak() as &str)
            .version(self.info.version.clone().leak() as &str)
            .arg(
                clap::Arg::new("help")
                    .short('h')
                    .long("help")
                    .help("Show module help")
                    .action(clap::ArgAction::SetTrue)
            )
    }
}

/// Module execution context
pub struct ModuleContext<'a> {
    pub config: &'a Config,
    pub args: Vec<String>,
    pub dry_run: bool,
    pub verbose: bool,
}

impl<'a> ModuleContext<'a> {
    pub fn new(config: &'a Config, args: Vec<String>) -> Self {
        let dry_run = args.iter().any(|arg| arg == "--dry-run");
        let verbose = args.iter().any(|arg| arg == "--verbose" || arg == "-v");
        
        Self {
            config,
            args,
            dry_run,
            verbose,
        }
    }
}