// src/workflows/security.rs
// Security hardening chain

pub struct SecurityWorkflow {
    base: WorkflowBase,
}

impl SecurityWorkflow {
    pub fn new() -> Self {
        let info = WorkflowInfo {
            name: "security".to_string(),
            description: "Firewall, SSH, sysctl hardening".to_string(),
            version: "0.1.0".to_string(),
            author: "Flux Contributors".to_string(),
        };
        Self { base: WorkflowBase { info } }
    }
}

impl Workflow for SecurityWorkflow {
    fn name(&self) -> &str { &self.base.info.name }
    fn description(&self) -> &str { &self.base.info.description }
    fn version(&self) -> &str { &self.base.info.version }
    fn help(&self) -> String {
        format!("{} workflow is not yet implemented.", self.name())
    }

    fn execute(&self, _config: &Config) -> Result<()> {
        Err(FluxError::Workflow(format!("{} workflow not implemented", self.name())))
    }
}

// ----------------------------------------------------------