use crate::config::Config;
use crate::error::Result;
use crate::workflows::{BaseWorkflow, Workflow};
use async_trait::async_trait;

/// Security hardening workflow
pub struct SecurityWorkflow;

#[async_trait]
impl Workflow for SecurityWorkflow {
    fn name(&self) -> &str {
        "security"
    }

    fn description(&self) -> &str {
        "Security hardening: firewall setup, SSH hardening, and kernel parameters"
    }

    fn modules(&self) -> Vec<String> {
        vec![
            "firewall".to_string(),
            "ssh".to_string(),
            "sysctl".to_string(),
        ]
    }

    async fn execute(&self, config: &Config) -> Result<()> {
        let base = BaseWorkflow::new(
            self.name(),
            self.description(),
            vec!["firewall", "ssh", "sysctl"],
        );

        base.execute_modules(config).await
    }
}
