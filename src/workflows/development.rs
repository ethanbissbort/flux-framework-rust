use crate::config::Config;
use crate::error::Result;
use crate::workflows::{BaseWorkflow, Workflow};
use async_trait::async_trait;

/// Development environment workflow
pub struct DevelopmentWorkflow;

#[async_trait]
impl Workflow for DevelopmentWorkflow {
    fn name(&self) -> &str {
        "development"
    }

    fn description(&self) -> &str {
        "Development environment setup: user creation, ZSH, and development tools"
    }

    fn modules(&self) -> Vec<String> {
        vec![
            "user".to_string(),
            "zsh".to_string(),
            "certs".to_string(),
        ]
    }

    async fn execute(&self, config: &Config) -> Result<()> {
        let base = BaseWorkflow::new(
            self.name(),
            self.description(),
            vec!["user", "zsh", "certs"],
        );

        base.execute_modules(config).await
    }
}
