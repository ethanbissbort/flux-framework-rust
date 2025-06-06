// ----------------------------------------------------------
// src/modules/zsh.rs – COMPLETE STUB (compiles, no-op logic)
// ----------------------------------------------------------

use crate::config::Config;
use crate::error::{FluxError, Result};
use crate::modules::{Module, ModuleBase, ModuleInfo};
use async_trait::async_trait;

/// ZshModule installs the `zsh` shell, sets up Oh‑My‑Zsh, and
/// switches the default shell for the target user.  This is a
/// **placeholder implementation**: it compiles and returns an
/// “unimplemented” error so workflows can proceed without failing
/// the linker. Replace `install_oh_my_zsh()` with real logic.
pub struct ZshModule {
    base: ModuleBase,
}

impl ZshModule {
    pub fn new() -> Self {
        let info = ModuleInfo {
            name: "zsh".to_string(),
            description: "Oh‑My‑Zsh installation and configuration".to_string(),
            version: "0.1.0".to_string(),
            author: "Flux Contributors".to_string(),
            tags: vec!["ux".to_string()],
            requires_root: true,
            supported_distros: vec!["all".to_string()],
        };
        Self { base: ModuleBase { info } }
    }

    /// Dummy placeholder. Implement the actual installation steps here.
    async fn install_oh_my_zsh(&self) -> Result<()> {
        Err(FluxError::Module("install_oh_my_zsh() not implemented".into()))
    }
}

#[async_trait]
impl Module for ZshModule {
    fn name(&self) -> &str { &self.base.info.name }
    fn description(&self) -> &str { &self.base.info.description }
    fn version(&self) -> &str { &self.base.info.version }
    fn is_available(&self) -> bool { true }

    fn help(&self) -> String {
        "Installs zsh, sets Oh‑My‑Zsh, and optionally Powerlevel10k.".into()
    }

    async fn execute(&self, _args: Vec<String>, _config: &Config) -> Result<()> {
        self.install_oh_my_zsh().await
    }
}