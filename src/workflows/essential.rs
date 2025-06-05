use crate::config::Config;
use crate::error::Result;
use crate::workflows::{BaseWorkflow, Workflow};
use async_trait::async_trait;

/// Essential system setup workflow
pub struct EssentialWorkflow;

#[async_trait]
impl Workflow for EssentialWorkflow {
    fn name(&self) -> &str {
        "essential"
    }
    
    fn description(&self) -> &str {
        "Basic system setup including updates, certificates, system hardening, and SSH configuration"
    }
    
    fn modules(&self) -> Vec<String> {
        vec![
            "update".to_string(),
            "certs".to_string(),
            "sysctl".to_string(),
            "ssh".to_string(),
        ]
    }
    
    async fn execute(&self, config: &Config) -> Result<()> {
        let base = BaseWorkflow::new(
            self.name(),
            self.description(),
            vec!["update", "certs", "sysctl", "ssh"],
        );
        
        base.execute_modules(config).await
    }
}