# 🔗 Flux Framework - Workflows Guide

> **Pre-built automation pipelines for common server provisioning scenarios**

---

## 📖 Table of Contents

- [What are Workflows?](#-what-are-workflows)
- [Available Workflows](#-available-workflows)
  - [Essential Workflow](#-essential-workflow)
  - [Security Workflow](#-security-workflow)
  - [Complete Workflow](#-complete-workflow)
  - [Development Workflow](#-development-workflow)
  - [Monitoring Workflow](#-monitoring-workflow)
- [Workflow Execution](#-workflow-execution)
- [Customization](#-customization)
- [Advanced Usage](#-advanced-usage)

---

## 🌟 What are Workflows?

**Workflows** are pre-configured sequences of modules that automate common server provisioning and configuration tasks. Instead of running modules individually, workflows execute multiple modules in a specific order to achieve a particular goal.

### Key Benefits

- ✅ **Time-Saving** - One command instead of many
- ✅ **Best Practices** - Modules run in optimal order
- ✅ **Consistency** - Same configuration every time
- ✅ **Safety** - Validated module sequences
- ✅ **Interactive** - Confirm each step or run automatically

### How Workflows Work

```
┌─────────────────────────────────────────────┐
│  flux workflow <name>                       │
└──────────────┬──────────────────────────────┘
               │
               ▼
┌─────────────────────────────────────────────┐
│  Display Workflow Information               │
│  - Name and description                     │
│  - List of modules to execute               │
└──────────────┬──────────────────────────────┘
               │
               ▼
┌─────────────────────────────────────────────┐
│  Confirm Execution? (Interactive)           │
└──────────────┬──────────────────────────────┘
               │
               ▼
┌─────────────────────────────────────────────┐
│  Execute Each Module in Sequence            │
│  - Confirm before each module               │
│  - Show progress                            │
│  - Handle errors gracefully                 │
└──────────────┬──────────────────────────────┘
               │
               ▼
┌─────────────────────────────────────────────┐
│  Display Summary                            │
│  - Completed modules                        │
│  - Failed modules                           │
│  - Skipped modules                          │
└─────────────────────────────────────────────┘
```

---

## 📦 Available Workflows

Flux provides **5 curated workflows** for common scenarios:

| Workflow | Modules | Use Case | Time |
|----------|---------|----------|------|
| 🌟 [Essential](#-essential-workflow) | 4 modules | Fresh server setup | ~5 min |
| 🔒 [Security](#-security-workflow) | 3 modules | Hardening existing servers | ~3 min |
| 🎯 [Complete](#-complete-workflow) | 11 modules | Full provisioning | ~15 min |
| 💻 [Development](#-development-workflow) | 3 modules | Developer workstations | ~5 min |
| 📊 [Monitoring](#-monitoring-workflow) | 3 modules | Observability setup | ~5 min |

---

## 🌟 Essential Workflow

> **Perfect for: Fresh server setup, baseline configuration**

### Overview

The Essential workflow provides a solid foundation for any Linux server. It updates packages, installs certificates, hardens the kernel, and secures SSH - the bare minimum for a production-ready server.

### Modules Executed

```
1. 🔄 update   → System updates and security patches
2. 📜 certs    → Certificate trust store updates
3. ⚙️ sysctl   → Kernel security hardening
4. 🔐 ssh      → SSH server hardening
```

### Execution Order

```
┌──────────┐     ┌──────────┐     ┌──────────┐     ┌──────────┐
│  update  │────▶│  certs   │────▶│ sysctl   │────▶│   ssh    │
└──────────┘     └──────────┘     └──────────┘     └──────────┘
```

### What Gets Configured

#### 1. Update Module
- ✅ Updates all system packages to latest versions
- ✅ Installs security patches
- ✅ Refreshes package cache
- ✅ Checks if reboot is needed

#### 2. Certs Module
- ✅ Updates CA certificates
- ✅ Refreshes system trust store
- ✅ Installs required root certificates

#### 3. Sysctl Module
- ✅ Network security (disable IP forwarding, source routing)
- ✅ Kernel hardening (ASLR, ptrace restrictions)
- ✅ IPv4/IPv6 security parameters
- ✅ DDoS protection settings

#### 4. SSH Module
- ✅ Changes SSH port (optional, default: 22)
- ✅ Disables root login
- ✅ Configures modern ciphers and MACs
- ✅ Sets up fail2ban (optional)
- ✅ Creates security banner

### Usage

```bash
# Run essential workflow
sudo flux workflow essential

# Alternative command
sudo flux apply essential
```

### Example Session

```
=== Workflow: essential ===
Basic system setup including updates, certificates, system hardening, and SSH configuration

This workflow will execute the following modules:
  1. update
  2. certs
  3. sysctl
  4. ssh

Continue with workflow execution? [Y/n]: y

[1/4] Module: update
Execute update module? [Y/n]: y
✓ Package cache updated
✓ 45 packages upgraded
✓ System is up to date

[2/4] Module: certs
Execute certs module? [Y/n]: y
✓ CA certificates updated
✓ Trust store refreshed

[3/4] Module: sysctl
Execute sysctl module? [Y/n]: y
✓ Security parameters applied
✓ Kernel hardening enabled

[4/4] Module: ssh
Execute ssh module? [Y/n]: y
✓ SSH configuration hardened
✓ Modern ciphers configured
✓ Security banner created

=== Workflow Summary ===
✓ Completed: 4
```

### When to Use

- 🎯 **Fresh server deployments** - First thing to run on new servers
- 🎯 **Baseline security** - Minimum security requirements
- 🎯 **Quick setup** - Fast, essential configuration
- 🎯 **Before other workflows** - Foundation for additional setup

### Time Required

⏱️ **Approximately 5 minutes** (varies by network speed and package updates)

---

## 🔒 Security Workflow

> **Perfect for: Hardening existing servers, security compliance**

### Overview

The Security workflow focuses exclusively on hardening your server's security posture. It configures the firewall, hardens SSH, and applies kernel security parameters.

### Modules Executed

```
1. 🛡️ firewall → Host firewall configuration
2. 🔐 ssh      → SSH server hardening
3. ⚙️ sysctl   → Kernel security parameters
```

### Execution Order

```
┌──────────┐     ┌──────────┐     ┌──────────┐
│firewall  │────▶│   ssh    │────▶│ sysctl   │
└──────────┘     └──────────┘     └──────────┘
```

### What Gets Configured

#### 1. Firewall Module
- ✅ Installs UFW (Debian) or firewalld (RHEL)
- ✅ Configures default deny policy
- ✅ Allows SSH (prevents lockout)
- ✅ Optionally applies security presets
- ✅ Enables firewall service

#### 2. SSH Module
- ✅ Port change (default: 2222)
- ✅ Disables password authentication
- ✅ Key-only authentication enforced
- ✅ Sets up fail2ban for brute-force protection
- ✅ Configures rate limiting

#### 3. Sysctl Module
- ✅ IP spoofing protection
- ✅ SYN flood protection
- ✅ ICMP redirect protection
- ✅ Kernel pointer restrictions
- ✅ Process tracing restrictions

### Usage

```bash
# Run security workflow
sudo flux workflow security

# Alternative command
sudo flux apply security
```

### Example Session

```
=== Workflow: security ===
Security hardening: firewall setup, SSH hardening, and kernel parameters

This workflow will execute the following modules:
  1. firewall
  2. ssh
  3. sysctl

Continue with workflow execution? [Y/n]: y

[1/3] Module: firewall
Execute firewall module? [Y/n]: y
Apply a security preset? [Y/n]: y
Select security preset:
  1. minimal (SSH only)
  2. web-server (HTTP/HTTPS)
> 1
✓ UFW installed and configured
✓ SSH access allowed
✓ Firewall enabled

[2/3] Module: ssh
Execute ssh module? [Y/n]: y
Change SSH port from default (22)? [Y/n]: y
Enter new SSH port: 2222
Disable password authentication (key-only)? [Y/n]: y
⚠  Make sure you have SSH key access configured!
Are you sure you want to continue? [y/N]: y
Setup fail2ban for SSH protection? [Y/n]: y
✓ SSH configuration hardened
✓ Port changed to 2222
✓ Password authentication disabled
✓ fail2ban configured

[3/3] Module: sysctl
Execute sysctl module? [Y/n]: y
✓ Security parameters applied
✓ IP spoofing protection enabled
✓ SYN cookies enabled

=== Workflow Summary ===
✓ Completed: 3

⚠  IMPORTANT: Update your firewall to allow port 2222
⚠  Next SSH connection: ssh -p 2222 user@host
```

### Security Compliance

This workflow helps meet requirements for:

- 📋 **CIS Benchmarks** - Center for Internet Security
- 📋 **NIST Guidelines** - Network security standards
- 📋 **PCI DSS** - Payment card industry requirements
- 📋 **SOC 2** - Security compliance audits

### When to Use

- 🎯 **Production servers** - Before going live
- 🎯 **Compliance requirements** - Meeting security standards
- 🎯 **After compromise** - Hardening after incidents
- 🎯 **Regular audits** - Periodic security reviews

### Time Required

⏱️ **Approximately 3 minutes**

---

## 🎯 Complete Workflow

> **Perfect for: Full server provisioning from scratch**

### Overview

The Complete workflow is the most comprehensive, executing all 11 modules in an optimized sequence. This is a "kitchen sink" approach that fully provisions a server from bare metal to production-ready.

### Modules Executed

```
1.  🔄 update    → System updates
2.  🏷️ hostname  → System identification
3.  🌐 network   → Network configuration
4.  🛡️ firewall  → Firewall setup
5.  🔐 ssh       → SSH hardening
6.  ⚙️ sysctl    → Kernel tuning
7.  📜 certs     → Certificate management
8.  👤 user      → User accounts
9.  💻 zsh       → Shell configuration
10. 📋 motd      → Login banner
11. 📊 netdata   → Monitoring
```

### Execution Order

```
┌─────────┐   ┌──────────┐   ┌─────────┐   ┌──────────┐   ┌─────┐
│ update  │──▶│ hostname │──▶│ network │──▶│ firewall │──▶│ ssh │
└─────────┘   └──────────┘   └─────────┘   └──────────┘   └─────┘
                                                               │
┌─────────┐   ┌─────┐   ┌──────┐   ┌──────┐   ┌──────┐      │
│ netdata │◀──│ motd │◀──│ zsh  │◀──│ user │◀──│ certs│◀─────┘
└─────────┘   └─────┘   └──────┘   └──────┘   └──────┘
```

### What Gets Configured

This workflow configures **everything**:

- ✅ **System Updates** - Latest packages and security patches
- ✅ **Identity** - Hostname and FQDN
- ✅ **Networking** - IP addressing, DNS, routes
- ✅ **Security** - Firewall, SSH hardening, kernel parameters
- ✅ **Certificates** - CA trust store
- ✅ **Users** - Admin accounts with SSH keys
- ✅ **Shell** - ZSH with Oh-My-Zsh
- ✅ **MOTD** - Dynamic system information banner
- ✅ **Monitoring** - Netdata for observability

### Usage

```bash
# Run complete workflow
sudo flux workflow complete

# Alternative command
sudo flux apply complete
```

### Example Use Cases

**Scenario 1: New Production Web Server**
```bash
sudo flux workflow complete
# During execution:
# - Set hostname: web-prod-01.company.com
# - Configure network: Static IP
# - Apply firewall preset: web-server
# - Create admin user with GitHub keys
# - Install monitoring
```

**Scenario 2: Development Server**
```bash
sudo flux workflow complete
# During execution:
# - Set hostname: dev-server-01
# - Keep DHCP networking
# - Minimal firewall (SSH only)
# - Create multiple developer accounts
# - Install ZSH with powerlevel10k theme
```

### Interactive Prompts

The workflow will interactively ask for:

- 🖥️ **Hostname** - System name and domain
- 🌐 **Network** - DHCP or static IP configuration
- 🛡️ **Firewall** - Security preset selection
- 🔐 **SSH** - Port and authentication settings
- 👤 **Users** - Admin account creation
- 💻 **ZSH** - Theme and plugin preferences
- 📊 **Monitoring** - Netdata installation

### When to Use

- 🎯 **New servers** - Fresh installations
- 🎯 **Standardization** - Consistent server builds
- 🎯 **Infrastructure as Code** - Repeatable deployments
- 🎯 **Testing** - Spin up identical test environments

### Time Required

⏱️ **Approximately 15 minutes** (varies by selections and network speed)

---

## 💻 Development Workflow

> **Perfect for: Developer workstations, coding environments**

### Overview

The Development workflow sets up a developer-friendly environment with user management, ZSH shell, and certificates.

### Modules Executed

```
1. 👤 user → User account creation
2. 💻 zsh  → ZSH shell with Oh-My-Zsh
3. 📜 certs → Certificate trust store
```

### Execution Order

```
┌──────────┐     ┌──────────┐     ┌──────────┐
│   user   │────▶│   zsh    │────▶│  certs   │
└──────────┘     └──────────┘     └──────────┘
```

### What Gets Configured

#### 1. User Module
- ✅ Creates developer user accounts
- ✅ Sets up SSH directories
- ✅ Imports GitHub SSH keys
- ✅ Adds to appropriate groups (docker, etc.)

#### 2. ZSH Module
- ✅ Installs ZSH shell
- ✅ Configures Oh-My-Zsh framework
- ✅ Installs developer-focused plugins:
  - `git` - Git aliases and completions
  - `docker` - Docker completions
  - `kubectl` - Kubernetes completions
  - `terraform` - Terraform completions
  - `aws` - AWS CLI completions
- ✅ Sets up powerlevel10k theme (optional)

#### 3. Certs Module
- ✅ Updates CA certificates
- ✅ Installs corporate certificates (if needed)
- ✅ Ensures HTTPS connectivity

### Usage

```bash
# Run development workflow
sudo flux workflow development

# Alternative command
sudo flux apply development
```

### Example Session

```
=== Workflow: development ===
Development environment setup

This workflow will execute the following modules:
  1. user
  2. zsh
  3. certs

[1/3] Module: user
Execute user module? [Y/n]: y
Create standard user
Enter username: alice
Enter full name [Alice]: Alice Developer
Import SSH keys from GitHub? [Y/n]: y
Enter GitHub username: alice
Add to additional groups? [Y/n]: y
Enter groups (comma-separated): docker,developers
✓ User 'alice' created
✓ SSH keys imported from GitHub
✓ Added to groups: docker, developers

[2/3] Module: zsh
Execute zsh module? [Y/n]: y
Configure for which user?: alice
Select theme:
  1. powerlevel10k
  2. agnoster
  3. robbyrussell
> 1
Select plugins (space to select, enter to confirm):
  [x] git
  [x] docker
  [x] kubectl
  [ ] terraform
  [x] sudo
✓ ZSH installed
✓ Oh-My-Zsh configured for alice
✓ Theme: powerlevel10k
✓ Plugins installed

[3/3] Module: certs
Execute certs module? [Y/n]: y
✓ CA certificates updated

=== Workflow Summary ===
✓ Completed: 3
```

### When to Use

- 🎯 **Developer workstations** - Local or remote development machines
- 🎯 **Onboarding** - Setting up new developers
- 🎯 **Coding servers** - Shared development environments
- 🎯 **Jump boxes** - SSH bastion hosts

### Time Required

⏱️ **Approximately 5 minutes**

---

## 📊 Monitoring Workflow

> **Perfect for: Setting up observability stack**

### Overview

The Monitoring workflow installs Netdata for real-time system monitoring, configures certificate trust, and sets up an informative MOTD.

### Modules Executed

```
1. 📊 netdata → Real-time monitoring agent
2. 📜 certs   → Certificate trust store
3. 📋 motd    → Dynamic login banner
```

### Execution Order

```
┌──────────┐     ┌──────────┐     ┌──────────┐
│ netdata  │────▶│  certs   │────▶│   motd   │
└──────────┘     └──────────┘     └──────────┘
```

### What Gets Configured

#### 1. Netdata Module
- ✅ Installs Netdata monitoring agent
- ✅ Configures data collection
- ✅ Sets up web dashboard (port 19999)
- ✅ Optionally claims to Netdata Cloud
- ✅ Configures alert notifications

#### 2. Certs Module
- ✅ Updates CA certificates
- ✅ Ensures HTTPS access to dashboards

#### 3. MOTD Module
- ✅ Creates dynamic system information banner
- ✅ Shows CPU, memory, disk usage
- ✅ Displays service status
- ✅ Shows update notifications
- ✅ Displays monitoring link

### Usage

```bash
# Run monitoring workflow
sudo flux workflow monitoring

# Alternative command
sudo flux apply monitoring
```

### Example Session

```
=== Workflow: monitoring ===
Monitoring tools installation

This workflow will execute the following modules:
  1. netdata
  2. certs
  3. motd

[1/3] Module: netdata
Execute netdata module? [Y/n]: y
Disable anonymous telemetry? [Y/n]: y
Claim to Netdata Cloud? [y/N]: n
✓ Netdata installed
✓ Service started
✓ Dashboard available at: http://localhost:19999

[2/3] Module: certs
Execute certs module? [Y/n]: y
✓ CA certificates updated

[3/3] Module: motd
Execute motd module? [Y/n]: y
Enter organization name [Flux Server]: MyCompany
Select banner style:
  1. flux-large
  2. simple
  3. minimal
> 2
✓ MOTD installed
✓ Organization: MyCompany
✓ Banner style: simple

=== Workflow Summary ===
✓ Completed: 3

ℹ️  Access Netdata dashboard:
   http://YOUR_SERVER_IP:19999
```

### Dashboard Features

After installation, Netdata provides:

- 📊 **Real-time metrics** - 1-second granularity
- 📈 **2000+ metrics** - CPU, memory, disk, network, processes
- 🔔 **Alerts** - Configurable thresholds and notifications
- 🐳 **Container monitoring** - Docker, Kubernetes integration
- 📱 **Mobile-friendly** - Responsive web interface

### When to Use

- 🎯 **Production servers** - Observability requirements
- 🎯 **Performance tuning** - Identifying bottlenecks
- 🎯 **Troubleshooting** - Real-time diagnostics
- 🎯 **Capacity planning** - Resource usage trends

### Time Required

⏱️ **Approximately 5 minutes**

---

## ⚙️ Workflow Execution

### Basic Usage

```bash
# Execute workflow by name
sudo flux workflow <workflow-name>

# Alternative syntax
sudo flux apply <workflow-name>
```

### Available Workflows

```bash
sudo flux workflow essential     # Essential setup
sudo flux workflow security      # Security hardening
sudo flux workflow complete      # Full provisioning
sudo flux workflow development   # Developer environment
sudo flux workflow monitoring    # Monitoring setup
```

### Interactive Mode

By default, workflows run in **interactive mode**:

- ✅ Shows workflow description
- ✅ Lists all modules to be executed
- ✅ Confirms before starting
- ✅ Prompts before each module
- ✅ Allows skipping individual modules
- ✅ Displays progress and results
- ✅ Shows summary at completion

### Execution Flow

```
1. Display Workflow Info
   ├─ Name
   ├─ Description
   └─ Module list

2. Confirm Execution
   └─ Continue? [Y/n]

3. For each module:
   ├─ Show module name
   ├─ Execute module? [Y/n]
   │  ├─ Yes → Run module
   │  └─ No → Skip to next
   └─ Show result

4. Display Summary
   ├─ Completed count
   ├─ Failed count
   └─ Skipped count

5. Post-Workflow Actions
   ├─ Check if reboot needed
   └─ Show next steps
```

### Error Handling

If a module fails during workflow execution:

```
✗ Module 'ssh' failed: Port 2222 already in use

Continue with remaining modules? [Y/n]:
  Y → Skip failed module, continue with others
  N → Abort workflow
```

---

## 🎨 Customization

### Creating Custom Workflows

Currently, workflows are built-in, but you can create custom sequences using shell scripts:

```bash
#!/bin/bash
# custom-web-setup.sh

set -e

echo "🚀 Custom Web Server Setup"

# Essential baseline
sudo flux module update --update
sudo flux module certs --update
sudo flux module sysctl --security

# Web-specific configuration
sudo flux module firewall --preset web-server
sudo flux module ssh --port 2222 --disable-passwords
sudo flux module user --admin deploy --github deploybot

# Monitoring
sudo flux module netdata --install --disable-telemetry
sudo flux module motd --install --org "Web Hosting" --banner simple

echo "✅ Web server setup complete!"
```

### Module Selection in Workflows

You can skip modules interactively:

```
[3/5] Module: firewall
Execute firewall module? [Y/n]: n
○ Skipped module: firewall

[4/5] Module: ssh
Execute ssh module? [Y/n]: y
```

### Configuration Presets

Pass configurations to modules within workflows:

```bash
# In the future, with flux.toml support:
[workflow.security]
firewall_preset = "web-server"
ssh_port = 2222
ssh_disable_passwords = true
fail2ban = true
```

---

## 🚀 Advanced Usage

### Workflow Chaining

Run multiple workflows in sequence:

```bash
# Setup base system, then add monitoring
sudo flux workflow essential
sudo flux workflow monitoring
```

### Partial Workflow Execution

Run specific modules from a workflow:

```bash
# Run only first 2 modules of essential workflow
# (Requires scripting approach)

sudo flux module update --update
sudo flux module certs --update
```

### Automated Execution

For non-interactive automation (future feature):

```bash
# Future: Non-interactive mode
sudo flux workflow essential --non-interactive --config flux.toml
```

### Testing Workflows

Use dry-run mode to preview changes (future feature):

```bash
# Future: Dry-run mode
sudo flux workflow security --dry-run
```

### Workflow Logging

All workflow executions are logged:

```bash
# View workflow logs
sudo journalctl -u flux -n 100

# Or check application logs
sudo tail -f /var/log/flux/workflow.log
```

---

## 📋 Workflow Comparison

### Quick Reference

| Workflow | Modules | Time | Use Case | Security Level |
|----------|---------|------|----------|----------------|
| **Essential** | 4 | 5 min | Fresh servers | ⭐⭐⭐ Medium |
| **Security** | 3 | 3 min | Hardening | ⭐⭐⭐⭐⭐ High |
| **Complete** | 11 | 15 min | Full setup | ⭐⭐⭐⭐ High |
| **Development** | 3 | 5 min | Dev machines | ⭐⭐ Low |
| **Monitoring** | 3 | 5 min | Observability | ⭐⭐⭐ Medium |

### Choosing the Right Workflow

```
┌─────────────────────────────────────┐
│ What's your primary goal?          │
└──────────────┬──────────────────────┘
               │
      ┌────────┴────────┐
      │                 │
  Fresh Server?    Existing Server?
      │                 │
      ▼                 ▼
  ┌─────────┐     ┌──────────┐
  │Essential│     │  What do │
  │    or   │     │you need? │
  │Complete │     └─────┬────┘
  └─────────┘           │
                  ┌─────┴─────┐
                  │           │
           Security?    Monitoring?
                  │           │
                  ▼           ▼
            ┌─────────┐ ┌──────────┐
            │Security │ │Monitoring│
            └─────────┘ └──────────┘
```

---

## 📚 Next Steps

After running a workflow:

1. ✅ **Verify Configuration** - Check that all modules completed successfully
2. ✅ **Test Connectivity** - Especially after security workflow (SSH, firewall)
3. ✅ **Update Documentation** - Record server configuration
4. ✅ **Setup Backups** - Configure backup solutions
5. ✅ **Monitor System** - Check Netdata dashboard if installed
6. ✅ **Apply Updates** - Keep system current

---

## 📖 Additional Resources

- 📖 [Modules Reference](MODULES.md) - Detailed module documentation
- 📖 [Examples](EXAMPLES.md) - Real-world usage scenarios
- 📖 [Installation Guide](INSTALLATION.md) - Getting started with Flux
- 📖 [Architecture](ARCHITECTURE.md) - How workflows are implemented

---

<div align="center">

**⚡ Build Production-Ready Servers in Minutes**

[GitHub](https://github.com/ethanbissbort/flux-framework-rust) •
[Documentation](README.md) •
[Issues](https://github.com/ethanbissbort/flux-framework-rust/issues)

</div>
