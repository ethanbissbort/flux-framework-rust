# ⚙️ Flux Framework - Configuration Reference

> **Configuration file format and options (Future: v3.1+)**

---

## 📖 Table of Contents

- [Overview](#-overview)
- [Configuration File](#-configuration-file)
- [Global Settings](#-global-settings)
- [Module Configuration](#-module-configuration)
- [Workflow Configuration](#-workflow-configuration)
- [Environment Variables](#-environment-variables)
- [Examples](#-examples)

---

## 🌟 Overview

> **⚠️ Note:** Configuration file support is planned for **Flux v3.1**. This document describes the future configuration format.

Currently, Flux modules are configured via:
- ✅ **Command-line arguments** - Per-execution options
- ✅ **Interactive prompts** - Menu-driven configuration
- ✅ **Environment variables** - System-wide settings

**Future (v3.1):** Configuration files will enable:
- 🔜 **Declarative configuration** - Define desired state in `flux.toml`
- 🔜 **Repeatable deployments** - Version control your infrastructure
- 🔜 **Non-interactive execution** - Automated provisioning
- 🔜 **Profile management** - Multiple configuration profiles

---

## 📝 Configuration File

### File Location

Flux will search for configuration files in this order:

1. **Command-line specified:**
   ```bash
   flux --config /path/to/flux.toml workflow essential
   ```

2. **Current directory:**
   ```
   ./flux.toml
   ```

3. **User configuration:**
   ```
   ~/.config/flux/flux.toml
   ```

4. **System configuration:**
   ```
   /etc/flux/flux.toml
   ```

### File Format

Configuration files use **TOML** (Tom's Obvious Minimal Language) format:

```toml
# flux.toml - Flux Framework Configuration

[global]
version = "3.1"
log_level = "info"
dry_run = false

[modules.ssh]
port = 2222
disable_passwords = true

[workflows.security]
enabled = true
```

---

## 🌍 Global Settings

### Basic Configuration

```toml
[global]
# Configuration version (required)
version = "3.1"

# Logging configuration
log_level = "info"  # trace, debug, info, warn, error
log_file = "/var/log/flux/flux.log"
log_format = "json"  # json, text

# Execution settings
dry_run = false
interactive = true
fail_fast = true  # Stop on first error

# Backup settings
backup_dir = "/var/backups/flux"
keep_backups = 5

# Update settings
auto_update_check = true
update_channel = "stable"  # stable, beta, nightly
```

### Advanced Settings

```toml
[global.advanced]
# Parallelism
max_parallel_modules = 1
module_timeout = 300  # seconds

# Safety settings
require_confirmation = true
backup_before_change = true

# Network settings
http_timeout = 30
retry_attempts = 3
retry_delay = 5
```

---

## 🧩 Module Configuration

Each module can have its own configuration section:

### SSH Module

```toml
[modules.ssh]
# Basic settings
port = 2222
listen_address = "0.0.0.0"

# Authentication
password_authentication = false
pubkey_authentication = true
permit_root_login = false

# Security
max_auth_tries = 3
login_grace_time = 30
client_alive_interval = 300

# Ciphers and algorithms
ciphers = [
    "chacha20-poly1305@openssh.com",
    "aes256-gcm@openssh.com",
    "aes128-gcm@openssh.com"
]

macs = [
    "hmac-sha2-512-etm@openssh.com",
    "hmac-sha2-256-etm@openssh.com"
]

kex_algorithms = [
    "curve25519-sha256",
    "diffie-hellman-group16-sha512"
]

# fail2ban integration
fail2ban = true
fail2ban_maxretry = 3
fail2ban_bantime = 3600

# Banner
banner_text = "Authorized Access Only"
```

### Firewall Module

```toml
[modules.firewall]
# Firewall type (ufw, firewalld, iptables)
firewall_type = "auto"  # auto-detect

# Default policy
default_incoming = "deny"
default_outgoing = "allow"
default_forward = "deny"

# Security preset
preset = "web-server"  # minimal, web-server, database, etc.

# Custom rules
[[modules.firewall.rules]]
port = 8080
protocol = "tcp"
source = "192.168.1.0/24"
comment = "Internal API"

[[modules.firewall.rules]]
port = 5432
protocol = "tcp"
source = "10.0.0.0/8"
comment = "PostgreSQL from internal network"

# Logging
log_level = "low"  # off, low, medium, high, full
log_dropped = true
```

### User Module

```toml
[modules.user]
# Default shell for new users
default_shell = "/bin/zsh"

# Admin users to create
[[modules.user.admins]]
username = "alice"
fullname = "Alice Administrator"
github = "alice"
groups = ["sudo", "docker"]

[[modules.user.admins]]
username = "bob"
fullname = "Bob Developer"
github = "bobdev"
groups = ["sudo", "developers"]

# Standard users
[[modules.user.users]]
username = "deploy"
fullname = "Deployment User"
system_user = false
groups = ["www-data"]
```

### Network Module

```toml
[modules.network]
# Primary interface
primary_interface = "eth0"

# Static IP configuration
[[modules.network.interfaces]]
name = "eth0"
method = "static"
address = "192.168.1.100/24"
gateway = "192.168.1.1"
dns = ["8.8.8.8", "8.8.4.4"]

# VLAN configuration
[[modules.network.vlans]]
parent = "eth0"
vlan_id = 100
address = "10.100.1.10/24"

# Routes
[[modules.network.routes]]
destination = "10.0.0.0/8"
gateway = "192.168.1.1"
metric = 100
```

### Update Module

```toml
[modules.update]
# Update strategy
auto_update = false
security_only = false
auto_reboot = false
reboot_time = "03:00"  # HH:MM format

# Package management
clean_cache = true
autoremove = true
upgrade_distro = false

# Notifications
notify_updates = true
notify_email = "admin@example.com"
```

### Sysctl Module

```toml
[modules.sysctl]
# Preset
preset = "security"  # security, performance, custom

# Custom parameters
[modules.sysctl.parameters]
"net.ipv4.ip_forward" = 0
"net.ipv4.conf.all.rp_filter" = 1
"net.ipv4.tcp_syncookies" = 1
"kernel.dmesg_restrict" = 1
"kernel.kptr_restrict" = 2
"fs.suid_dumpable" = 0
```

### ZSH Module

```toml
[modules.zsh]
# Oh-My-Zsh configuration
theme = "powerlevel10k"
plugins = [
    "git",
    "docker",
    "kubectl",
    "terraform",
    "aws",
    "sudo"
]

# Custom aliases
[modules.zsh.aliases]
ll = "ls -alh"
k = "kubectl"
tf = "terraform"
dc = "docker-compose"

# Environment variables
[modules.zsh.env]
EDITOR = "vim"
VISUAL = "vim"
TERM = "xterm-256color"
```

### MOTD Module

```toml
[modules.motd]
# Organization branding
organization = "My Company"
banner_style = "flux-large"  # flux-large, simple, minimal

# Information to display
show_system_info = true
show_cpu = true
show_memory = true
show_disk = true
show_network = true
show_updates = true
show_security_status = true

# Custom messages
welcome_message = "Welcome to Production Server"
warning_message = "Authorized access only - All activity monitored"
```

### Netdata Module

```toml
[modules.netdata]
# Installation
install = true
disable_telemetry = true

# Cloud integration
claim_to_cloud = false
claim_token = ""  # Your Netdata Cloud token

# Configuration
bind_ip = "127.0.0.1"
port = 19999
update_every = 1  # seconds

# Plugins
enabled_plugins = [
    "proc",
    "diskspace",
    "network",
    "docker",
    "apps"
]

# Alerts
enable_alerts = true
alert_email = "monitoring@example.com"
```

### Hostname Module

```toml
[modules.hostname]
hostname = "web-prod-01"
domain = "example.com"
fqdn = "web-prod-01.example.com"

# /etc/hosts entries
[[modules.hostname.hosts]]
ip = "127.0.0.1"
names = ["localhost", "localhost.localdomain"]

[[modules.hostname.hosts]]
ip = "192.168.1.100"
names = ["web-prod-01.example.com", "web-prod-01"]
```

### Certs Module

```toml
[modules.certs]
# Auto-update trust store
auto_update = true

# Custom CA certificates to install
[[modules.certs.custom_ca]]
name = "corporate-ca"
path = "/etc/pki/ca-trust/source/anchors/corporate-ca.crt"

[[modules.certs.custom_ca]]
name = "internal-ca"
path = "/etc/pki/ca-trust/source/anchors/internal-ca.crt"
```

---

## 🔗 Workflow Configuration

Configure pre-built workflows:

### Essential Workflow

```toml
[workflows.essential]
enabled = true
auto_confirm = false  # Skip confirmations

# Module-specific overrides
[workflows.essential.modules.update]
security_only = true

[workflows.essential.modules.ssh]
port = 2222
fail2ban = true
```

### Security Workflow

```toml
[workflows.security]
enabled = true

[workflows.security.modules.firewall]
preset = "minimal"

[workflows.security.modules.ssh]
port = 2222
disable_passwords = true
fail2ban = true

[workflows.security.modules.sysctl]
preset = "security"
```

### Complete Workflow

```toml
[workflows.complete]
enabled = true
skip_modules = []  # Skip specific modules

# Module execution order (optional override)
execution_order = [
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
    "netdata"
]
```

---

## 🔐 Environment Variables

Configuration can also be provided via environment variables:

### Global Variables

```bash
# Logging
export FLUX_LOG_LEVEL=debug
export FLUX_LOG_FILE=/var/log/flux/debug.log

# Execution
export FLUX_DRY_RUN=true
export FLUX_INTERACTIVE=false

# Configuration file
export FLUX_CONFIG=/path/to/flux.toml
```

### Module-Specific Variables

```bash
# SSH Module
export FLUX_SSH_PORT=2222
export FLUX_SSH_DISABLE_PASSWORDS=true

# Firewall Module
export FLUX_FIREWALL_PRESET=web-server

# User Module
export FLUX_USER_DEFAULT_SHELL=/bin/zsh

# Netdata Module
export FLUX_NETDATA_DISABLE_TELEMETRY=true
```

### Priority Order

Configuration sources in order of precedence (highest to lowest):

1. **Command-line arguments** - `--port 2222`
2. **Environment variables** - `FLUX_SSH_PORT=2222`
3. **Configuration file** - `flux.toml`
4. **Interactive prompts** - User input during execution
5. **Module defaults** - Built-in defaults

---

## 💡 Examples

### Example 1: Web Server Configuration

```toml
# flux.toml - Web Server Setup

[global]
version = "3.1"
log_level = "info"
backup_dir = "/var/backups/flux"

[modules.hostname]
hostname = "web-01"
domain = "example.com"

[modules.firewall]
preset = "web-server"

[[modules.firewall.rules]]
port = 8080
protocol = "tcp"
comment = "Application server"

[modules.ssh]
port = 2222
disable_passwords = true
fail2ban = true

[modules.user]
[[modules.user.admins]]
username = "deploy"
github = "deploy-bot"
groups = ["sudo", "www-data"]

[modules.netdata]
install = true
disable_telemetry = true
```

Usage:
```bash
sudo flux --config flux.toml workflow complete
```

### Example 2: Database Server Configuration

```toml
# flux.toml - Database Server

[global]
version = "3.1"
log_level = "warn"

[modules.hostname]
hostname = "db-01"
domain = "internal.company.com"

[modules.network]
[[modules.network.interfaces]]
name = "eth0"
method = "static"
address = "10.0.1.100/24"
gateway = "10.0.1.1"
dns = ["10.0.1.10", "10.0.1.11"]

[modules.firewall]
preset = "database"

[[modules.firewall.rules]]
port = 5432
protocol = "tcp"
source = "10.0.0.0/8"
comment = "PostgreSQL - Internal only"

[modules.ssh]
port = 2222
disable_passwords = true

[modules.sysctl]
preset = "performance"

[modules.sysctl.parameters]
"vm.swappiness" = 10
"vm.dirty_ratio" = 15
```

### Example 3: Development Workstation

```toml
# flux.toml - Developer Workstation

[global]
version = "3.1"
log_level = "debug"
interactive = true

[modules.user]
default_shell = "/bin/zsh"

[[modules.user.users]]
username = "alice"
fullname = "Alice Developer"
github = "alice"
groups = ["sudo", "docker", "developers"]

[[modules.user.users]]
username = "bob"
fullname = "Bob Engineer"
github = "bobeng"
groups = ["sudo", "docker", "developers"]

[modules.zsh]
theme = "powerlevel10k"
plugins = ["git", "docker", "kubectl", "terraform", "aws"]

[modules.zsh.aliases]
k = "kubectl"
tf = "terraform"
dc = "docker-compose"

[modules.motd]
organization = "Dev Team"
banner_style = "simple"
```

### Example 4: Multi-Environment Setup

**Production:**
```toml
# flux-prod.toml
[global]
version = "3.1"
log_level = "warn"
fail_fast = true

[modules.firewall]
preset = "minimal"

[modules.ssh]
port = 2222
disable_passwords = true
fail2ban = true
```

**Staging:**
```toml
# flux-staging.toml
[global]
version = "3.1"
log_level = "info"

[modules.firewall]
preset = "web-server"

[modules.ssh]
port = 22
disable_passwords = false
```

**Development:**
```toml
# flux-dev.toml
[global]
version = "3.1"
log_level = "debug"
interactive = true

[modules.firewall]
preset = "minimal"

[modules.ssh]
port = 22
disable_passwords = false
```

Usage:
```bash
# Deploy to production
sudo flux --config flux-prod.toml workflow security

# Deploy to staging
sudo flux --config flux-staging.toml workflow complete

# Setup development
sudo flux --config flux-dev.toml workflow development
```

---

## 🔄 Configuration Validation

**Future feature:** Validate configuration before execution:

```bash
# Validate configuration file
flux config validate flux.toml

# Expected output:
✓ Configuration file is valid
✓ All required fields present
✓ All modules available
✓ No conflicts detected
```

---

## 📋 Configuration Schema

**Future feature:** Generate configuration template:

```bash
# Generate template with all options
flux config template > flux.toml

# Generate minimal template
flux config template --minimal > flux-minimal.toml

# Generate for specific modules
flux config template --modules ssh,firewall > flux-security.toml
```

---

## 📚 Additional Resources

- 📖 [Modules Reference](MODULES.md) - Detailed module options
- 📖 [Workflows Guide](WORKFLOWS.md) - Workflow configuration
- 📖 [Examples](EXAMPLES.md) - Real-world configurations
- 📖 [TOML Specification](https://toml.io/) - TOML format reference

---

<div align="center">

**⚙️ Coming in Flux v3.1**

[GitHub](https://github.com/ethanbissbort/flux-framework-rust) •
[Documentation](../README.md) •
[Roadmap](ROADMAP.md)

</div>
