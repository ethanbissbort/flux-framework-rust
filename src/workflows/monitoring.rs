use crate::config::Config;
use crate::error::Result;
use crate::workflows::{BaseWorkflow, Workflow};
use async_trait::async_trait;

/// Monitoring stack workflow
pub struct MonitoringWorkflow;

#[async_trait]
impl Workflow for MonitoringWorkflow {
    fn name(&self) -> &str {
        "monitoring"
    }

    fn description(&self) -> &str {
        "Monitoring stack deployment: Netdata and system monitoring setup"
    }

    fn modules(&self) -> Vec<String> {
        vec![
            "netdata".to_string(),
            "certs".to_string(),
            "motd".to_string(),
        ]
    }

    async fn execute(&self, config: &Config) -> Result<()> {
        let base = BaseWorkflow::new(
            self.name(),
            self.description(),
            vec!["netdata", "certs", "motd"],
        );

        base.execute_modules(config).await
    }
}
