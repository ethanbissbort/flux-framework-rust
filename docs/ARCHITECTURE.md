# 🏗️ Flux Framework - Architecture

> **Technical design and implementation details**

---

## 📖 Table of Contents

- [Overview](#-overview)
- [Design Philosophy](#-design-philosophy)
- [System Architecture](#-system-architecture)
- [Module System](#-module-system)
- [Workflow Engine](#-workflow-engine)
- [Helper Functions](#-helper-functions)
- [Error Handling](#-error-handling)
- [Configuration System](#-configuration-system)
- [Logging & Telemetry](#-logging--telemetry)
- [Security Architecture](#-security-architecture)
- [Future Improvements](#-future-improvements)

---

## 🌟 Overview

Flux Framework is built on **modern Rust principles** with a focus on:

- 🦀 **Type Safety** - Leveraging Rust's type system
- ⚡ **Performance** - Native binary, zero-cost abstractions
- 🔒 **Security** - Memory safety, no undefined behavior
- 🧩 **Modularity** - Composable, independent modules
- 📦 **Portability** - Single binary, minimal dependencies

### Technology Stack

| Component | Technology | Purpose |
|-----------|------------|---------|
| **Language** | Rust 1.77+ | Core implementation |
| **Async Runtime** | Tokio | Asynchronous operations |
| **CLI Framework** | Clap 4.5 | Command-line parsing |
| **Serialization** | Serde + TOML | Configuration handling |
| **Logging** | Tracing | Structured logging |
| **HTTP** | Reqwest | Network operations |
| **Terminal** | Dialoguer + Colored | Interactive UI |

---

## 💭 Design Philosophy

### Core Principles

#### 1. **Idempotency**
Operations can be run multiple times safely:

```rust
// Running twice produces same result
flux module ssh --port 2222
flux module ssh --port 2222  // Safe, no-op if already set
```

#### 2. **Fail-Safe**
Never leave system in broken state:

```rust
async fn apply_configuration() -> Result<()> {
    // Always backup before changes
    backup_config()?;

    match apply_changes() {
        Ok(_) => Ok(()),
        Err(e) => {
            // Restore on failure
            restore_backup()?;
            Err(e)
        }
    }
}
```

#### 3. **Explicit > Implicit**
User confirms important operations:

```rust
if !prompt_yes_no("This will disable password auth. Continue?", false)? {
    return Ok(());
}
```

#### 4. **Composability**
Modules work independently or together:

```rust
// Each module is self-contained
flux module ssh --harden
flux module firewall --preset web-server

// Or combined in workflows
flux workflow security  // Runs multiple modules
```

---

## 🏛️ System Architecture

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────┐
│                      CLI Entry Point                     │
│                      (src/main.rs)                       │
└────────────────────┬────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────┐
│                    Command Parser                        │
│                     (src/cli.rs)                         │
│   • Argument parsing (Clap)                             │
│   • Command routing                                      │
│   • Help text generation                                 │
└────────────┬───────────────────────────┬─────────────────┘
             │                           │
     ┌───────▼────────┐          ┌───────▼────────┐
     │  Module System │          │ Workflow Engine │
     │ (src/modules/) │          │(src/workflows/) │
     └───────┬────────┘          └───────┬─────────┘
             │                           │
             └─────────┬─────────────────┘
                       │
                       ▼
        ┌──────────────────────────────┐
        │      Helper Functions        │
        │     (src/helpers/)           │
        │  • Logging                   │
        │  • System detection          │
        │  • File operations           │
        │  • User input                │
        │  • Validation                │
        └──────────────────────────────┘
```

### Directory Structure

```
flux-framework/
├── src/
│   ├── main.rs                 # Binary entry point
│   ├── lib.rs                  # Library root
│   ├── cli.rs                  # CLI implementation
│   ├── config.rs               # Configuration types
│   ├── error.rs                # Error handling
│   │
│   ├── modules/                # Module implementations
│   │   ├── mod.rs              # Module trait & manager
│   │   ├── ssh.rs              # SSH module
│   │   ├── firewall.rs         # Firewall module
│   │   ├── user.rs             # User module
│   │   ├── network.rs          # Network module
│   │   ├── hostname.rs         # Hostname module
│   │   ├── update.rs           # Update module
│   │   ├── sysctl.rs           # Sysctl module
│   │   ├── certs.rs            # Certificates module
│   │   ├── zsh.rs              # ZSH module
│   │   ├── motd.rs             # MOTD module
│   │   └── netdata.rs          # Netdata module
│   │
│   ├── workflows/              # Workflow definitions
│   │   ├── mod.rs              # Workflow trait & manager
│   │   ├── essential.rs        # Essential workflow
│   │   ├── security.rs         # Security workflow
│   │   ├── complete.rs         # Complete workflow
│   │   ├── development.rs      # Development workflow
│   │   └── monitoring.rs       # Monitoring workflow
│   │
│   └── helpers/                # Utility functions
│       ├── mod.rs              # Helpers module root
│       ├── logging.rs          # Logging utilities
│       ├── system.rs           # System detection
│       ├── file_ops.rs         # File operations
│       ├── user_input.rs       # Interactive prompts
│       ├── validation.rs       # Input validation
│       └── network.rs          # Network utilities
│
├── tests/                      # Integration tests
├── docs/                       # Documentation
├── config/                     # Sample configs
└── Cargo.toml                  # Dependencies
```

---

## 🧩 Module System

### Module Trait

All modules implement the `Module` trait:

```rust
#[async_trait]
pub trait Module: Send + Sync {
    /// Module name (e.g., "ssh")
    fn name(&self) -> &str;

    /// Module description
    fn description(&self) -> &str;

    /// Module version
    fn version(&self) -> &str;

    /// Check if module is available on this system
    fn is_available(&self) -> bool;

    /// Get help text
    fn help(&self) -> String;

    /// Execute module with arguments
    async fn execute(&self, args: Vec<String>, config: &Config) -> Result<()>;
}
```

### Module Lifecycle

```
┌──────────────┐
│ User Command │
└──────┬───────┘
       │
       ▼
┌──────────────────┐
│  Module Manager  │
│  • Discover      │
│  • Validate      │
│  • Load          │
└──────┬───────────┘
       │
       ▼
┌──────────────────┐
│ Module Instance  │
│  • Parse args    │
│  • Check avail.  │
│  • Execute       │
└──────┬───────────┘
       │
       ▼
┌──────────────────┐
│    Execution     │
│  • Backup config │
│  • Apply changes │
│  • Validate      │
│  • Log results   │
└──────────────────┘
```

### Module Registration

Modules are registered in `ModuleManager`:

```rust
impl ModuleManager {
    pub fn new() -> Result<Self> {
        let mut modules: HashMap<String, Box<dyn Module>> = HashMap::new();

        // Register all modules
        let all_modules: Vec<Box<dyn Module>> = vec![
            Box::new(update::UpdateModule::new()),
            Box::new(network::NetworkModule::new()),
            Box::new(hostname::HostnameModule::new()),
            Box::new(user::UserModule::new()),
            Box::new(ssh::SshModule::new()),
            Box::new(firewall::FirewallModule::new()),
            Box::new(certs::CertsModule::new()),
            Box::new(sysctl::SysctlModule::new()),
            Box::new(zsh::ZshModule::new()),
            Box::new(motd::MotdModule::new()),
            Box::new(netdata::NetdataModule::new()),
        ];

        for module in all_modules {
            modules.insert(module.name().to_string(), module);
        }

        Ok(Self { modules })
    }
}
```

### Module Example: SSH Module

```rust
pub struct SshModule {
    base: ModuleBase,
}

impl SshModule {
    pub fn new() -> Self {
        let info = ModuleInfo {
            name: "ssh".to_string(),
            description: "SSH server hardening".to_string(),
            version: "1.0.0".to_string(),
            author: "Flux Contributors".to_string(),
            tags: vec!["security".to_string()],
            requires_root: true,
            supported_distros: vec!["all".to_string()],
        };
        Self { base: ModuleBase { info } }
    }

    async fn harden_ssh(&self, port: Option<u16>) -> Result<()> {
        // Implementation
    }
}

#[async_trait]
impl Module for SshModule {
    fn name(&self) -> &str { &self.base.info.name }
    fn description(&self) -> &str { &self.base.info.description }
    fn version(&self) -> &str { &self.base.info.version }
    fn is_available(&self) -> bool { true }
    fn help(&self) -> String { "..." }

    async fn execute(&self, args: Vec<String>, config: &Config) -> Result<()> {
        // Parse arguments and execute
    }
}
```

---

## 🔗 Workflow Engine

### Workflow Trait

```rust
#[async_trait]
pub trait Workflow: Send + Sync {
    /// Workflow name
    fn name(&self) -> &str;

    /// Workflow description
    fn description(&self) -> &str;

    /// List of modules in execution order
    fn modules(&self) -> Vec<String>;

    /// Execute the workflow
    async fn execute(&self, config: &Config) -> Result<()>;
}
```

### Workflow Execution

```
┌─────────────────┐
│ Workflow Start  │
└────────┬────────┘
         │
         ▼
┌──────────────────────┐
│ Display Info         │
│ • Name & description │
│ • Module list        │
└────────┬─────────────┘
         │
         ▼
┌──────────────────────┐
│ User Confirmation    │
│ Continue? [Y/n]      │
└────────┬─────────────┘
         │
         ▼
┌──────────────────────┐
│ For each module:     │
│ ┌──────────────────┐ │
│ │ Show module name │ │
│ │ Confirm execute  │ │
│ │ Run module       │ │
│ │ Handle errors    │ │
│ └──────────────────┘ │
└────────┬─────────────┘
         │
         ▼
┌──────────────────────┐
│ Display Summary      │
│ • Completed          │
│ • Failed             │
│ • Skipped            │
└──────────────────────┘
```

### Workflow Implementation

```rust
pub struct SecurityWorkflow;

#[async_trait]
impl Workflow for SecurityWorkflow {
    fn name(&self) -> &str {
        "security"
    }

    fn description(&self) -> &str {
        "Security hardening: firewall, SSH, and kernel parameters"
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
```

---

## 🛠️ Helper Functions

### Logging System

```rust
// src/helpers/logging.rs

use tracing::{info, warn, error};

pub fn log_info(msg: String) {
    info!("{}", msg);
    println!("ℹ️  {}", msg);
}

pub fn log_success(msg: String) {
    info!("SUCCESS: {}", msg);
    println!("✅ {}", msg.green());
}

pub fn log_warn(msg: String) {
    warn!("{}", msg);
    println!("⚠️  {}", msg.yellow());
}

pub fn log_error(msg: String) {
    error!("{}", msg);
    eprintln!("❌ {}", msg.red());
}
```

### System Detection

```rust
// src/helpers/system.rs

pub fn detect_distro() -> Result<Distro> {
    let os_release = fs::read_to_string("/etc/os-release")?;

    if os_release.contains("ubuntu") {
        Ok(Distro::Ubuntu)
    } else if os_release.contains("debian") {
        Ok(Distro::Debian)
    } else if os_release.contains("rhel") || os_release.contains("redhat") {
        Ok(Distro::RHEL)
    } else if os_release.contains("centos") {
        Ok(Distro::CentOS)
    } else if os_release.contains("rocky") {
        Ok(Distro::Rocky)
    } else if os_release.contains("fedora") {
        Ok(Distro::Fedora)
    } else {
        Err(FluxError::UnsupportedDistro)
    }
}

pub struct Distro;

impl Distro {
    pub fn is_debian_based(&self) -> bool {
        matches!(self, Distro::Ubuntu | Distro::Debian)
    }

    pub fn is_redhat_based(&self) -> bool {
        matches!(self, Distro::RHEL | Distro::CentOS | Distro::Rocky | Distro::Fedora)
    }
}
```

### File Operations

```rust
// src/helpers/file_ops.rs

pub fn backup_file(path: &str) -> Result<String> {
    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
    let backup_path = format!("{}.backup-{}", path, timestamp);

    fs::copy(path, &backup_path)?;
    log_info(format!("Backed up {} to {}", path, backup_path));

    Ok(backup_path)
}

pub fn safe_write_file(path: &str, content: &str, backup: bool) -> Result<()> {
    if backup && Path::new(path).exists() {
        backup_file(path)?;
    }

    // Write to temp file first
    let temp_path = format!("{}.tmp", path);
    fs::write(&temp_path, content)?;

    // Atomic rename
    fs::rename(&temp_path, path)?;

    Ok(())
}
```

### User Input

```rust
// src/helpers/user_input.rs

use dialoguer::{Confirm, Input, Select};

pub fn prompt_yes_no(msg: &str, default: bool) -> Result<bool> {
    Ok(Confirm::new()
        .with_prompt(msg)
        .default(default)
        .interact()?)
}

pub fn prompt_input(msg: &str) -> Result<String> {
    Ok(Input::<String>::new()
        .with_prompt(msg)
        .interact()?)
}

pub fn select_from_menu(title: &str, items: &[&str]) -> Result<usize> {
    Ok(Select::new()
        .with_prompt(title)
        .items(items)
        .interact()?)
}
```

---

## ❗ Error Handling

### Error Types

```rust
// src/error.rs

use thiserror::Error;

#[derive(Error, Debug)]
pub enum FluxError {
    #[error("System error: {0}")]
    System(String),

    #[error("Module error: {0}")]
    Module(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Command failed: {0}")]
    CommandFailed(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Parse error: {0}")]
    Parse(String),
}

pub type Result<T> = std::result::Result<T, FluxError>;
```

### Error Handling Pattern

```rust
// Propagate errors with context
pub async fn execute_module(name: &str) -> Result<()> {
    let module = load_module(name)
        .map_err(|e| FluxError::module(format!("Failed to load {}: {}", name, e)))?;

    module.execute()
        .await
        .map_err(|e| FluxError::module(format!("Execution failed: {}", e)))?;

    Ok(())
}

// Handle errors gracefully
match execute_module("ssh").await {
    Ok(_) => log_success("Module executed successfully"),
    Err(e) => {
        log_error(format!("Module failed: {}", e));
        // Don't panic, continue or return error
        return Err(e);
    }
}
```

---

## ⚙️ Configuration System

### Configuration Structure (Future v3.1)

```rust
// src/config.rs

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub global: GlobalConfig,
    pub modules: HashMap<String, ModuleConfig>,
    pub workflows: HashMap<String, WorkflowConfig>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GlobalConfig {
    pub log_level: String,
    pub log_file: Option<String>,
    pub dry_run: bool,
    pub interactive: bool,
    pub backup_dir: String,
}

impl Config {
    pub fn load(path: &str) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        Ok(toml::from_str(&content)?)
    }

    pub fn save(&self, path: &str) -> Result<()> {
        let content = toml::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }
}
```

---

## 📊 Logging & Telemetry

### Structured Logging

```rust
use tracing::{info, warn, error, instrument};
use tracing_subscriber::{EnvFilter, fmt};

// Initialize logging
pub fn init_logging() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_target(false)
        .init();
}

// Instrumented function
#[instrument]
async fn execute_ssh_hardening(port: u16) -> Result<()> {
    info!(port = port, "Starting SSH hardening");

    // Implementation...

    info!("SSH hardening completed");
    Ok(())
}
```

### Log Levels

- **TRACE** - Very detailed, debug-level information
- **DEBUG** - Detailed information for debugging
- **INFO** - General information about execution
- **WARN** - Warning messages, non-fatal issues
- **ERROR** - Error messages, operation failures

---

## 🔒 Security Architecture

### Privilege Handling

```rust
pub fn require_root() -> Result<()> {
    if !nix::unistd::Uid::effective().is_root() {
        return Err(FluxError::permission_denied(
            "This operation requires root privileges. Run with sudo."
        ));
    }
    Ok(())
}
```

### Input Validation

```rust
pub fn validate_port(port_str: &str) -> Result<u16> {
    let port: u16 = port_str.parse()
        .map_err(|_| FluxError::validation("Invalid port number"))?;

    if port < 1024 || port > 65535 {
        return Err(FluxError::validation(
            "Port must be between 1024-65535"
        ));
    }

    Ok(port)
}
```

### Secure Defaults

- 🔒 All operations require explicit confirmation
- 🔒 Backup before destructive changes
- 🔒 Validate input before execution
- 🔒 Use secure communication (HTTPS)
- 🔒 No passwords in logs or output

---

## 🚀 Future Improvements

### Planned Features

**v3.1:**
- Configuration file support (TOML)
- Dry-run mode for all operations
- Enhanced logging with JSON output
- Module dependency resolution
- Automatic rollback on failure

**v3.2:**
- Plugin system for custom modules
- Remote execution support
- Multi-server orchestration
- Web UI dashboard
- API server mode

**v4.0:**
- Alpine & Arch Linux support
- Container-based testing
- Integration with Ansible/Terraform
- Cloud provider integrations
- Compliance reporting (CIS, NIST)

### Performance Optimizations

- Parallel module execution (where safe)
- Caching for repeated operations
- Binary size reduction
- Startup time optimization

---

## 📚 Additional Resources

- 📖 [Modules Reference](MODULES.md)
- 📖 [Workflows Guide](WORKFLOWS.md)
- 📖 [Contributing Guide](CONTRIBUTING.md)
- 📖 [Roadmap](ROADMAP.md)

---

<div align="center">

**🏗️ Built with Rust, Designed for Production**

[GitHub](https://github.com/ethanbissbort/flux-framework-rust) •
[Documentation](../README.md) •
[Contributing](CONTRIBUTING.md)

</div>
