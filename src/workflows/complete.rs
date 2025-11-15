use crate::config::Config;
use crate::error::Result;
use crate::workflows::{BaseWorkflow, Workflow};
use async_trait::async_trait;

/// Complete system provisioning workflow
pub struct CompleteWorkflow;

#[async_trait]
impl Workflow for CompleteWorkflow {
    fn name(&self) -> &str {
        "complete"
    }

    fn description(&self) -> &str {
        "Complete system provisioning with all modules (updates, network, security, user tools)"
    }

    fn modules(&self) -> Vec<String> {
        vec![
            "update".to_string(),
            "hostname".to_string(),
            "network".to_string(),
            "firewall".to_string(),
            "ssh".to_string(),
            "sysctl".to_string(),
            "certs".to_string(),
            "user".to_string(),
            "zsh".to_string(),
            "motd".to_string(),
            "netdata".to_string(),
        ]
    }

    async fn execute(&self, config: &Config) -> Result<()> {
        let base = BaseWorkflow::new(
            self.name(),
            self.description(),
            vec![
                "update",
                "hostname",
                "network",
                "firewall",
                "ssh",
                "sysctl",
                "certs",
                "user",
                "zsh",
                "motd",
                "netdata",
            ],
        );

        base.execute_modules(config).await
    }
}
