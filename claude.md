# Flux System Administration Framework (Rust Edition) - Complete Reference

## Project Summary

Flux is a **modular, enterprise-grade Linux configuration framework** written in **Rust**. It standardizes server setup, security hardening, and maintenance across Ubuntu, Debian, CentOS, RHEL, and Fedora distributions through automated workflows and independent functional modules. This is a complete rewrite of the original Bash-based framework with improved performance, safety, and type checking.

**Repository**: https://github.com/ethanbissbort/flux-framework-rust

## Core Architecture

The framework consists of:

- **src/main.rs** - Central orchestrator handling CLI interface and workflow routing
- **src/cli.rs** - Clap-based argument parser with shell completion support
- **src/config.rs** - TOML configuration loader and validator
- **src/error.rs** - Unified error type using thiserror
- **src/helpers/** - Reusable functions covering logging, validation, system operations, and user input
- **src/modules/** - Self-contained functional units implementing the Module trait
- **src/workflows/** - High-level orchestration combining multiple modules
- **config/** - Configuration templates (flux.toml)
- **certs/** - SSL/TLS certificate storage location

## Available Modules (11 Total)

| Module | Status | Purpose |
|--------|--------|---------|
| network | ✅ | Static IPs, VLANs, network diagnostics |
| hostname | ✅ | FQDN and hostname configuration |
| update | ✅ | System patching and package management |
| user | ✅ | User/group creation with SSH key management |
| ssh | ✅ | SSH hardening and fail2ban setup |
| firewall | ✅ | Unified firewall rules (UFW/firewalld/nftables) |
| sysctl | ✅ | Kernel parameter hardening |
| certs | ✅ | Certificate installation and management |
| zsh | ✅ | ZSH shell with Oh-My-Zsh framework |
| motd | ✅ | Custom dynamic message of the day |
| netdata | ✅ | System monitoring agent installation |

## Predefined Workflows

```bash
sudo flux apply essential      # Updates, certs, hardening
sudo flux apply complete       # All modules in sequence
sudo flux apply security       # Security-focused setup
sudo flux apply development    # Dev environment (user, zsh, certs)
sudo flux apply monitoring     # Monitoring stack (netdata, certs)
```

## Development Standards

**Code Quality Requirements:**
- Comprehensive error handling using Result<T, FluxError>
- Structured logging with tracing crate (DEBUG through SUCCESS levels)
- Input validation using dedicated validator helpers
- Support for both interactive and automated modes via config
- Idempotent operations safe for repeated execution
- Async/await patterns for I/O operations
- Type-safe configuration with serde

**Testing Protocol:**
- Unit tests for all validation functions
- Integration tests on clean VMs across multiple distributions
- Root privilege verification for system operations
- Multiple executions to verify idempotence
- Log file inspection at `/var/log/flux-setup.log`
- Pre-change backup creation before destructive operations

## Module Trait Interface

All modules implement the `Module` trait:

```rust
pub trait Module {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn version(&self) -> &str;
    fn is_available(&self) -> bool;
    fn help(&self) -> String;
    async fn execute(&self, args: Vec<String>, config: &Config) -> Result<()>;
}
```

## Key Helper Functions

**Logging**: `init_logging()`, `log_info()`, `log_warn()`, `log_error()`, `log_debug()`, `log_success()`, `show_progress()`

**Validation**: `validate_ip()`, `validate_hostname()`, `validate_port()`, `validate_email()`, `validate_username()`, `validate_interface()`, `validate_vlan_id()`, `validate_ssh_public_key()`, `validate_url()`

**System Checks**: `check_root()`, `detect_os()`, `check_command()`, `is_systemd_available()`, `get_system_status()`, `is_reboot_required()`, `get_failed_services()`, `check_security_updates()`

**File Operations**: `backup_file()`, `safe_write_file()`, `safe_append_to_file()`, `create_directory()`, `copy_file_with_permissions()`, `calculate_checksum()`

**Network Utilities**: `get_network_interfaces()`, `scan_ports()`, `get_dns_servers()`, `get_default_gateway()`, `test_connectivity()`, `resolve_hostname()`, `download_file()`

**User Input**: `prompt_yes_no()`, `prompt_input()`, `prompt_password()`, `prompt_ip()`, `prompt_hostname()`, `prompt_port()`, `select_from_menu()`, `multi_select_menu()`, `show_warning_prompt()`

## Security Framework

The system implements defense-in-depth:
- SSH port modification with key-only authentication
- Firewall with default-deny ingress policy
- Certificate chain validation before installation
- Comprehensive input validation across all modules
- Structured logging for audit trails
- Kernel hardening via sysctl parameters (net.ipv4.*, fs.*, kernel.*)
- Fail2ban integration for SSH protection
- Automatic security update detection and installation

## Module Development Pattern

New modules must:
1. Implement the `Module` trait in `src/modules/`
2. Register in `src/modules/mod.rs` exports
3. Use helper functions from `src/helpers/`
4. Support both interactive and non-interactive modes
5. Validate all user inputs using validation helpers
6. Create backups before modifications using `backup_file()`
7. Log operations using the tracing framework
8. Handle errors gracefully with proper Result types
9. Be idempotent (safe to run multiple times)
10. Check OS compatibility using `detect_os()`

## Environment Configuration

Configuration is managed via TOML files:

**Primary config**: `/etc/flux/flux.toml`

Key sections:
- `[global]` - mode (interactive/auto), logging level
- `[update]` - auto_security_updates, reboot settings
- `[network]` - DNS, static IP, VLAN configuration
- `[hostname]` - hostname and FQDN settings
- `[user]` - user creation, sudo rights, SSH key deployment
- `[ssh]` - port, authentication methods, hardening options
- `[firewall]` - backend (ufw/firewalld/nftables), rules
- `[sysctl]` - kernel parameter overrides
- `[certs]` - certificate paths and installation
- `[zsh]` - Oh-My-Zsh theme and plugins
- `[motd]` - message customization
- `[netdata]` - monitoring configuration

## CLI Usage Examples

```bash
# Show help
sudo flux --help

# List available modules
sudo flux list modules

# List available workflows
sudo flux list workflows

# Run a single module interactively
sudo flux module network

# Run a module in non-interactive mode
sudo flux module hostname --auto

# Apply a workflow
sudo flux apply essential

# Apply with dry-run
sudo flux apply security --dry-run

# Show system status
sudo flux status

# Verify configuration
sudo flux verify-config
```

## Operational Notes

- Most operations require elevated privileges (sudo/root)
- SSH hardening disables root login and password authentication by default
- Firewall defaults to deny-all with explicit allow rules
- Multi-distribution support via OS detection (detect_os helper)
- Package managers: apt (Debian/Ubuntu), dnf/yum (RHEL/CentOS/Fedora)
- Configuration changes are logged to `/var/log/flux-setup.log`
- Backups are created at `{original_path}.backup.{timestamp}`
- Reboot detection: framework warns when kernel/system updates require restart
- All file operations use safe write patterns (temp file + atomic move)

## Differences from Bash Version

**Improvements:**
- **Type Safety**: Compile-time guarantees vs runtime errors
- **Performance**: Native binary vs interpreted shell
- **Async I/O**: Non-blocking operations for network tasks
- **Better Error Handling**: Result types vs exit codes
- **Enhanced Validation**: Comprehensive input validation library
- **Structured Logging**: JSON-capable logs with tracing
- **Memory Safety**: Rust ownership prevents common bugs
- **Dependency Management**: Cargo vs manual script sourcing
- **Testing**: Built-in unit and integration test framework
- **Code Organization**: Module system vs script sourcing

**Maintained Features:**
- Same module names and purposes
- Compatible workflow definitions
- Similar CLI interface
- Equivalent functionality for all operations
- Cross-distribution support

## Building from Source

```bash
# Install Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone repository
git clone https://github.com/ethanbissbort/flux-framework-rust.git
cd flux-framework-rust

# Build release binary
cargo build --release

# Install system-wide
sudo install -m755 target/release/flux /usr/local/bin/flux

# Generate shell completions
flux completions bash > /etc/bash_completion.d/flux
```

## Dependencies

**Runtime Requirements:**
- Linux kernel 3.10+ (for network features)
- systemd (recommended, SysV init supported)
- sudo or root access
- Package manager: apt, dnf, or yum

**Rust Dependencies** (managed by Cargo):
- tokio - Async runtime
- clap - CLI parsing
- serde/toml - Configuration
- tracing - Logging
- anyhow/thiserror - Error handling
- nix/sysinfo - System interaction
- dialoguer/indicatif - User interaction
- See Cargo.toml for complete list

## Troubleshooting

**Permission Denied:**
```bash
# Run with sudo
sudo flux apply essential
```

**Module Not Found:**
```bash
# List available modules
sudo flux list modules
```

**Configuration Errors:**
```bash
# Verify config syntax
sudo flux verify-config

# Use default config
sudo cp config/default.toml /etc/flux/flux.toml
```

**Log Analysis:**
```bash
# View recent logs
sudo tail -f /var/log/flux-setup.log

# Search for errors
sudo grep ERROR /var/log/flux-setup.log
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Implement changes with tests
4. Run: `cargo fmt`, `cargo clippy`, `cargo test`
5. Submit pull request with description

All contributions must:
- Pass clippy lints with `-D warnings`
- Include unit tests for new functions
- Follow existing code style
- Update documentation as needed
- Sign commits with DCO (`git commit -s`)

## License

Dual-licensed under **MIT** or **Apache-2.0** (choose either).

---

*Flux Framework Rust Edition - Enterprise-grade system administration automation*
