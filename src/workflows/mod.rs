pub mod complete;
pub mod development;
pub mod essential;
pub mod monitoring;
pub mod security;

pub use system::{check_reboot_needed, is_service_active_enhanced};

use crate::config::Config;
use crate::error::{FluxError, Result};
use crate::helpers::{
    logging::{log_info, log_warn},
    user_input::prompt_yes_no,
};
use crate::modules::ModuleManager;
use async_trait::async_trait;
use colored::Colorize;
use std::collections::HashMap;

/// Workflow trait that all workflows must implement
#[async_trait]
pub trait Workflow: Send + Sync {
    /// Get workflow name
    fn name(&self) -> &str;
    
    /// Get workflow description
    fn description(&self) -> &str;
    
    /// Get list of modules in execution order
    fn modules(&self) -> Vec<String>;
    
    /// Execute the workflow
    async fn execute(&self, config: &Config) -> Result<()>;
}

/// Workflow manager
pub struct WorkflowManager {
    workflows: HashMap<String, Box<dyn Workflow>>,
}

impl WorkflowManager {
    /// Create new workflow manager
    pub fn new() -> Result<Self> {
        let mut workflows: HashMap<String, Box<dyn Workflow>> = HashMap::new();
        
        // Register all workflows
        workflows.insert("essential".to_string(), Box::new(essential::EssentialWorkflow));
        workflows.insert("complete".to_string(), Box::new(complete::CompleteWorkflow));
        workflows.insert("security".to_string(), Box::new(security::SecurityWorkflow));
        workflows.insert("development".to_string(), Box::new(development::DevelopmentWorkflow));
        workflows.insert("monitoring".to_string(), Box::new(monitoring::MonitoringWorkflow));
        
        Ok(Self { workflows })
    }
    
    /// Execute a workflow by name
    pub async fn execute_workflow(&self, name: &str, config: &Config) -> Result<()> {
        let workflow = self.workflows
            .get(name)
            .ok_or_else(|| FluxError::not_found(format!("Workflow '{}' not found", name)))?;
        
        log_info(format!("Executing workflow: {}", name));
        
        // Show workflow information
        println!("{}", format!("=== Workflow: {} ===", workflow.name()).cyan());
        println!("{}", workflow.description());
        println!();
        
        // Show modules that will be executed
        println!("{}", "This workflow will execute the following modules:".white());
        let modules = workflow.modules();
        for (i, module) in modules.iter().enumerate() {
            println!("  {}. {}", i + 1, module);
        }
        println!();
        
        if !prompt_yes_no("Continue with workflow execution?", true)? {
            log_info("Workflow execution cancelled by user");
            return Ok(());
        }
        
        // Execute the workflow
        workflow.execute(config).await?;
        
        // Check if reboot is needed
        crate::helpers::system::check_reboot_needed()?;
        
        Ok(())
    }
    
    /// List available workflows
    pub fn list_workflows(&self) -> Vec<(&str, &str)> {
        let mut workflows: Vec<(&str, &str)> = self.workflows
            .iter()
            .map(|(name, workflow)| (name.as_str(), workflow.description()))
            .collect();
        
        workflows.sort_by(|a, b| a.0.cmp(b.0));
        workflows
    }
}

/// Base workflow implementation
pub struct BaseWorkflow {
    name: String,
    description: String,
    modules: Vec<String>,
}

impl BaseWorkflow {
    pub fn new(name: &str, description: &str, modules: Vec<&str>) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            modules: modules.into_iter().map(String::from).collect(),
        }
    }
    
    /// Execute modules in sequence
    pub async fn execute_modules(&self, config: &Config) -> Result<()> {
        let manager = ModuleManager::new()?;
        
        let total = self.modules.len();
        let mut completed = 0;
        let mut failed = 0;
        let mut skipped = 0;
        
        for (i, module_name) in self.modules.iter().enumerate() {
            println!("\n{}", format!("[{}/{}] Module: {}", i + 1, total, module_name).white());
            
            // Check if module is available
            let module = manager.get_module(module_name)?;
            if !module.is_available() {
                log_warn(format!("Module {} is not available on this system", module_name));
                skipped += 1;
                continue;
            }
            
            // Ask user if they want to execute this module
            if prompt_yes_no(&format!("Execute {} module?", module_name), true)? {
                match manager.load_module(module_name, vec![], config).await {
                    Ok(_) => {
                        completed += 1;
                        log_info(format!("Module {} completed successfully", module_name));
                    }
                    Err(e) => {
                        failed += 1;
                        log_warn(format!("Module {} failed: {}", module_name, e));
                        
                        if !prompt_yes_no("Continue with remaining modules?", true)? {
                            break;
                        }
                    }
                }
            } else {
                skipped += 1;
                log_info(format!("Skipped module: {}", module_name));
            }
        }
        
        // Summary
        println!("\n{}", "=== Workflow Summary ===".cyan());
        println!("{}", format!("✓ Completed: {}", completed).green());
        if failed > 0 {
            println!("{}", format!("✗ Failed: {}", failed).red());
        }
        if skipped > 0 {
            println!("{}", format!("○ Skipped: {}", skipped).yellow());
        }
        
        Ok(())
    }
}