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
        "Basic system setup including network, certificates, updates, ZSH, and MOTD"
    }

    fn modules(&self) -> Vec<String> {
        vec![
            "network".to_string(),
            "certs".to_string(),
            "update".to_string(),
            "zsh".to_string(),
            "motd".to_string(),
        ]
    }

    async fn execute(&self, config: &Config) -> Result<()> {
        let base = BaseWorkflow::new(
            self.name(),
            self.description(),
            vec!["network", "certs", "update", "zsh", "motd"],
        );

        base.execute_modules(config).await
    }
}