use crate::error::{FluxError, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// General settings
    pub general: GeneralConfig,
    
    /// Module-specific configurations
    pub modules: HashMap<String, toml::Value>,
    
    /// Custom key-value pairs
    pub custom: HashMap<String, String>,
    
    #[serde(skip)]
    config_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    /// Default SSH port
    pub default_ssh_port: u16,
    
    /// Default admin username
    pub default_admin_user: String,
    
    /// Default admin groups
    pub default_admin_groups: Vec<String>,
    
    /// GitHub username for SSH key imports
    pub github_user: Option<String>,
    
    /// Log level
    pub log_level: String,
    
    /// Module directory
    pub modules_dir: Option<PathBuf>,
    
    /// Enable colored output
    pub colored_output: bool,
    
    /// Default gateway
    pub default_gateway: Option<String>,
    
    /// Default DNS servers
    pub default_dns: Vec<String>,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            default_ssh_port: 22,
            default_admin_user: "fluxadmin".to_string(),
            default_admin_groups: vec![
                "sudo".to_string(),
                "adm".to_string(),
                "systemd-journal".to_string(),
            ],
            github_user: None,
            log_level: "info".to_string(),
            modules_dir: None,
            colored_output: true,
            default_gateway: None,
            default_dns: vec!["1.1.1.1".to_string(), "8.8.8.8".to_string()],
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            general: GeneralConfig::default(),
            modules: HashMap::new(),
            custom: HashMap::new(),
            config_path: None,
        }
    }
}

impl Config {
    /// Get the default config directory path
    pub fn default_dir() -> Result<PathBuf> {
        if let Some(proj_dirs) = ProjectDirs::from("com", "flux", "flux-framework") {
            Ok(proj_dirs.config_dir().to_path_buf())
        } else {
            // Fallback to ~/.config/flux
            let home = dirs::home_dir()
                .ok_or_else(|| FluxError::config("Could not determine home directory"))?;
            Ok(home.join(".config").join("flux"))
        }
    }

    /// Get the default config file path
    pub fn default_path() -> Result<PathBuf> {
        Ok(Self::default_dir()?.join("flux.toml"))
    }

    /// Load configuration from the default location or create default
    pub fn load_or_default() -> Result<Self> {
        let config_path = Self::default_path()?;
        
        if config_path.exists() {
            Self::from_file(&config_path)
        } else {
            let mut config = Self::default();
            config.config_path = Some(config_path);
            Ok(config)
        }
    }

    /// Load configuration from a specific file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let contents = fs::read_to_string(path)
            .map_err(|e| FluxError::config(format!("Failed to read config file: {}", e)))?;
        
        let mut config: Config = toml::from_str(&contents)
            .map_err(|e| FluxError::config(format!("Failed to parse config file: {}", e)))?;
        
        config.config_path = Some(path.to_path_buf());
        Ok(config)
    }

    /// Save configuration to file
    pub fn save(&self) -> Result<()> {
        let path = self.config_path.as_ref()
            .ok_or_else(|| FluxError::config("No config path set"))?;
        
        // Ensure directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| FluxError::config(format!("Failed to create config directory: {}", e)))?;
        }
        
        let contents = toml::to_string_pretty(self)
            .map_err(|e| FluxError::config(format!("Failed to serialize config: {}", e)))?;
        
        fs::write(path, contents)
            .map_err(|e| FluxError::config(format!("Failed to write config file: {}", e)))?;
        
        Ok(())
    }

    /// Get a configuration value
    pub fn get(&self, key: &str) -> Option<String> {
        // Check custom values first
        if let Some(value) = self.custom.get(key) {
            return Some(value.clone());
        }
        
        // Check if it's a general config key
        match key {
            "default_ssh_port" => Some(self.general.default_ssh_port.to_string()),
            "default_admin_user" => Some(self.general.default_admin_user.clone()),
            "github_user" => self.general.github_user.clone(),
            "log_level" => Some(self.general.log_level.clone()),
            "colored_output" => Some(self.general.colored_output.to_string()),
            _ => None,
        }
    }

    /// Set a configuration value
    pub fn set(&mut self, key: &str, value: &str) -> Result<()> {
        match key {
            "default_ssh_port" => {
                self.general.default_ssh_port = value.parse()
                    .map_err(|_| FluxError::validation("Invalid port number"))?;
            }
            "default_admin_user" => {
                self.general.default_admin_user = value.to_string();
            }
            "github_user" => {
                self.general.github_user = Some(value.to_string());
            }
            "log_level" => {
                self.general.log_level = value.to_string();
            }
            "colored_output" => {
                self.general.colored_output = value.parse()
                    .map_err(|_| FluxError::validation("Invalid boolean value"))?;
            }
            _ => {
                // Store in custom values
                self.custom.insert(key.to_string(), value.to_string());
            }
        }
        
        Ok(())
    }

    /// Get all configuration values
    pub fn all(&self) -> Vec<(String, String)> {
        let mut values = vec![
            ("default_ssh_port".to_string(), self.general.default_ssh_port.to_string()),
            ("default_admin_user".to_string(), self.general.default_admin_user.clone()),
            ("log_level".to_string(), self.general.log_level.clone()),
            ("colored_output".to_string(), self.general.colored_output.to_string()),
        ];
        
        if let Some(github_user) = &self.general.github_user {
            values.push(("github_user".to_string(), github_user.clone()));
        }
        
        // Add custom values
        for (k, v) in &self.custom {
            values.push((k.clone(), v.clone()));
        }
        
        values.sort_by(|a, b| a.0.cmp(&b.0));
        values
    }

    /// Get module-specific configuration
    pub fn get_module_config(&self, module: &str) -> Option<&toml::Value> {
        self.modules.get(module)
    }

    /// Set module-specific configuration
    pub fn set_module_config(&mut self, module: &str, config: toml::Value) {
        self.modules.insert(module.to_string(), config);
    }
}