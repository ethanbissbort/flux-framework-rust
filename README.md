<div align="center">

# ⚡ Flux Framework

### Modern Linux System Administration & Hardening Toolkit

**Enterprise-grade server provisioning written in Rust** 🦀

[![Build Status](https://img.shields.io/badge/build-passing-brightgreen?style=for-the-badge)](https://github.com/ethanbissbort/flux-framework-rust)
[![License](https://img.shields.io/badge/license-MIT%20%2F%20Apache--2.0-blue?style=for-the-badge)](LICENSE-MIT)
[![Rust Version](https://img.shields.io/badge/rust-1.77%2B-orange?style=for-the-badge&logo=rust)](https://www.rust-lang.org)
[![Platform](https://img.shields.io/badge/platform-Linux-lightgrey?style=for-the-badge&logo=linux)](https://www.kernel.org)

[Features](#-features) • [Quick Start](#-quick-start) • [Modules](#-modules) • [Documentation](#-documentation) • [Contributing](#-contributing)

---

</div>

## 🌟 What is Flux?

Flux is a **powerful, type-safe system administration framework** that automates Linux server configuration, security hardening, and ongoing maintenance. Think of it as Infrastructure-as-Code meets Security-by-Default, all in a single blazingly fast binary.

### 💎 Why Flux?

```
🚀 Fast         → Native Rust performance, not shell scripts
🔒 Secure       → Security-first design with sensible defaults
🎯 Focused      → One tool for system provisioning & hardening
📦 Portable     → Single binary, no dependencies
🔧 Flexible     → 11 modules, 5 workflows, fully composable
✅ Reliable     → Idempotent operations, automatic backups
```

---

## ✨ Features

<table>
<tr>
<td width="50%">

### 🎛️ **System Management**
- ✅ Package updates & security patches
- ✅ Network configuration (static IP, VLANs)
- ✅ User & group management
- ✅ Hostname & FQDN setup
- ✅ Certificate management

</td>
<td width="50%">

### 🔐 **Security Hardening**
- ✅ SSH hardening & fail2ban
- ✅ Firewall (UFW/firewalld)
- ✅ Kernel parameter tuning
- ✅ Key-based authentication
- ✅ Security compliance presets

</td>
</tr>
<tr>
<td width="50%">

### 🎨 **Developer Experience**
- ✅ ZSH + Oh-My-Zsh setup
- ✅ Custom MOTD banners
- ✅ Interactive & scriptable modes
- ✅ Comprehensive logging
- ✅ Detailed help system

</td>
<td width="50%">

### 📊 **Monitoring**
- ✅ Netdata integration
- ✅ System health checks
- ✅ Resource usage tracking
- ✅ Service status monitoring
- ✅ Custom dashboards

</td>
</tr>
</table>

---

## 🏗️ Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                         Flux CLI                             │
│                    (Clap + Interactive)                      │
└──────────────────────┬──────────────────────────────────────┘
                       │
        ┌──────────────┴──────────────┐
        │                             │
┌───────▼────────┐          ┌─────────▼────────┐
│   Workflows    │          │     Modules      │
│                │          │                  │
│ • Essential    │◄─────────┤ • Network        │
│ • Security     │          │ • Hostname       │
│ • Complete     │          │ • Update         │
│ • Development  │          │ • User           │
│ • Monitoring   │          │ • SSH            │
│                │          │ • Firewall       │
│                │          │ • Sysctl         │
│                │          │ • Certs          │
│                │          │ • ZSH            │
│                │          │ • MOTD           │
│                │          │ • Netdata        │
└────────────────┘          └──────────────────┘
         │                           │
         └───────────┬───────────────┘
                     │
         ┌───────────▼────────────┐
         │   Helper Functions     │
         │                        │
         │ • Logging              │
         │ • Validation           │
         │ • System Detection     │
         │ • File Operations      │
         │ • User Input           │
         └────────────────────────┘
```

---

## 🚀 Quick Start

### 📦 Installation

<details>
<summary><b>Option 1: Build from Source (Recommended)</b></summary>

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone and build
git clone https://github.com/ethanbissbort/flux-framework-rust.git
cd flux-framework-rust
cargo build --release

# Install system-wide
sudo install -m755 target/release/flux /usr/local/bin/flux

# Verify installation
flux --version
```

</details>

<details>
<summary><b>Option 2: From Crates.io (Future)</b></summary>

```bash
cargo install flux-framework
```

</details>

### ⚙️ First Run

```bash
# 1. Check system compatibility
flux status

# 2. List available modules
flux list modules

# 3. Run essential setup (updates, certs, hardening)
sudo flux apply essential

# 4. Harden security (firewall, SSH, kernel)
sudo flux apply security
```

### 🎯 Common Tasks

```bash
# Configure SSH hardening
sudo flux module ssh --harden

# Setup a new admin user
sudo flux module user --admin alice --github alice

# Configure firewall with web server preset
sudo flux module firewall --preset web-server

# Install ZSH with Oh-My-Zsh
sudo flux module zsh --theme powerlevel10k

# Setup monitoring
sudo flux module netdata --install
```

---

## 🧩 Modules

Flux provides **11 specialized modules** for comprehensive system management:

| Module | Status | Description | Key Features |
|--------|--------|-------------|--------------|
| 🔄 **update** | ✅ | System updates & patches | Security updates, package management, reboot detection |
| 🌐 **network** | ✅ | Network configuration | Static IP, VLANs, diagnostics, interface management |
| 🏷️ **hostname** | ✅ | Hostname & FQDN setup | System naming, /etc/hosts management |
| 👤 **user** | ✅ | User & group management | Admin users, SSH keys, GitHub integration |
| 🔐 **ssh** | ✅ | SSH hardening | Port changes, key-only auth, fail2ban |
| 🛡️ **firewall** | ✅ | Firewall management | UFW/firewalld, presets, rule management |
| ⚙️ **sysctl** | ✅ | Kernel hardening | IPv4/IPv6 security, ASLR, performance tuning |
| 📜 **certs** | ✅ | Certificate management | System trust store, CA certificates |
| 💻 **zsh** | ✅ | ZSH shell setup | Oh-My-Zsh, themes, plugins, aliases |
| 📋 **motd** | ✅ | Dynamic MOTD | System info, resource usage, security status |
| 📊 **netdata** | ✅ | Monitoring agent | Real-time metrics, health checks, dashboards |

> 💡 **Tip:** Run `flux module <name> --help` for detailed usage information

📖 **[Full Module Documentation →](docs/MODULES.md)**

---

## 🔗 Workflows

**Workflows** combine multiple modules into cohesive provisioning pipelines:

### 🌟 Essential
> **Perfect for**: Fresh server setup, base configuration

```bash
sudo flux apply essential
```

**Includes:** `update` → `certs` → `sysctl` → `ssh`

**What it does:**
- ✅ Updates all packages to latest versions
- ✅ Installs required certificates
- ✅ Applies kernel hardening parameters
- ✅ Configures SSH security

---

### 🔒 Security
> **Perfect for**: Hardening existing servers, security compliance

```bash
sudo flux apply security
```

**Includes:** `firewall` → `ssh` → `sysctl`

**What it does:**
- ✅ Configures firewall with deny-all default
- ✅ SSH hardening (port change, key-only auth)
- ✅ Kernel security parameters

---

### 🎯 Complete
> **Perfect for**: Full server provisioning from scratch

```bash
sudo flux apply complete
```

**Includes:** All 11 modules in optimized sequence

---

### 💻 Development
> **Perfect for**: Developer workstations, coding environments

```bash
sudo flux apply development
```

**Includes:** `user` → `zsh` → `certs`

---

### 📊 Monitoring
> **Perfect for**: Setting up observability stack

```bash
sudo flux apply monitoring
```

**Includes:** `netdata` → `certs` → `motd`

---

📖 **[Full Workflow Guide →](docs/WORKFLOWS.md)**

---

## 🎨 Usage Examples

<details>
<summary><b>🔧 Setup a Web Server</b></summary>

```bash
# 1. Essential baseline
sudo flux apply essential

# 2. Create admin user
sudo flux module user --admin deploy --github deploybot

# 3. Configure firewall for web traffic
sudo flux module firewall --preset web-server

# 4. Harden SSH
sudo flux module ssh --port 2222 --disable-passwords

# 5. Setup monitoring
sudo flux module netdata --install

# 6. Custom MOTD
sudo flux module motd --org "MyCompany" --banner flux-large
```

</details>

<details>
<summary><b>🗄️ Database Server Hardening</b></summary>

```bash
# Run security workflow
sudo flux apply security

# Configure firewall for database
sudo flux module firewall --preset database-server

# Apply strict kernel parameters
sudo flux module sysctl --apply

# Setup monitoring
sudo flux module netdata --install
```

</details>

<details>
<summary><b>💻 Developer Workstation Setup</b></summary>

```bash
# Run development workflow
sudo flux apply development

# Install ZSH with custom theme
sudo flux module zsh --theme agnoster --plugins "git docker kubectl"

# Setup custom MOTD
sudo flux module motd --org "Dev Team" --banner simple
```

</details>

📖 **[More Examples →](docs/EXAMPLES.md)**

---

## 📚 Documentation

### 📖 User Guides
- **[Installation Guide](docs/INSTALLATION.md)** - Detailed installation instructions
- **[Module Reference](docs/MODULES.md)** - Complete module documentation
- **[Workflow Guide](docs/WORKFLOWS.md)** - Workflow usage and customization
- **[Configuration Reference](docs/CONFIGURATION.md)** - Config file documentation
- **[Examples](docs/EXAMPLES.md)** - Real-world usage scenarios

### 🛠️ Development
- **[Contributing Guide](docs/CONTRIBUTING.md)** - How to contribute
- **[Architecture](docs/ARCHITECTURE.md)** - System design and internals
- **[Roadmap](docs/ROADMAP.md)** - Future plans and features

### 📝 Reference
- **[claude.md](claude.md)** - Complete framework reference for AI assistants

---

## 🐧 Supported Distributions

| Distribution | Support Status | Notes |
|--------------|----------------|-------|
| 🟢 **Ubuntu** | ✅ Full | 20.04+, 22.04+, 24.04+ |
| 🟢 **Debian** | ✅ Full | 11, 12 |
| 🟢 **RHEL** | ✅ Full | 8, 9 |
| 🟢 **CentOS** | ✅ Full | Stream 8, 9 |
| 🟢 **Rocky Linux** | ✅ Full | 8, 9 |
| 🟢 **AlmaLinux** | ✅ Full | 8, 9 |
| 🟢 **Fedora** | ✅ Full | 38, 39, 40 |
| 🟡 **Alpine** | 🔜 Planned | v0.5 |
| 🟡 **Arch** | 🔜 Planned | v0.5 |

---

## 🤝 Contributing

We ❤️ contributions! Whether it's:

- 🐛 **Bug reports** - Found an issue? [Open an issue](https://github.com/ethanbissbort/flux-framework-rust/issues)
- 💡 **Feature requests** - Have an idea? [Start a discussion](https://github.com/ethanbissbort/flux-framework-rust/discussions)
- 📝 **Documentation** - Improve our docs with a PR
- 🔧 **Code contributions** - See our [Contributing Guide](docs/CONTRIBUTING.md)

### Quick Contribution Guide

```bash
# 1. Fork and clone
git clone https://github.com/YOUR_USERNAME/flux-framework-rust.git

# 2. Create a branch
git checkout -b feature/amazing-feature

# 3. Make your changes and test
cargo test
cargo clippy
cargo fmt

# 4. Commit with DCO sign-off
git commit -s -m "Add amazing feature"

# 5. Push and create PR
git push origin feature/amazing-feature
```

📖 **[Full Contributing Guide →](docs/CONTRIBUTING.md)**

---

## 🗺️ Roadmap

### ✅ Version 3.0 (Current)
- [x] Complete Rust migration from shell scripts
- [x] All 11 modules implemented
- [x] 5 workflows operational
- [x] Comprehensive error handling
- [x] Interactive & scriptable modes

### 🎯 Version 3.1 (Next)
- [ ] Configuration file support (`flux.toml`)
- [ ] Dry-run mode for all operations
- [ ] Enhanced logging with JSON output
- [ ] Module dependency resolution
- [ ] Automatic rollback on failure

### 🚀 Version 3.2
- [ ] Plugin system for custom modules
- [ ] Remote execution support
- [ ] Multi-server orchestration
- [ ] Web UI dashboard
- [ ] API server mode

### 🌟 Version 4.0
- [ ] Alpine & Arch Linux support
- [ ] Container-based testing
- [ ] Integration with Ansible/Terraform
- [ ] Cloud provider integrations
- [ ] Compliance reporting (CIS, NIST)

📖 **[Detailed Roadmap →](docs/ROADMAP.md)**

---

## 📊 Project Stats

```
📦 Modules:     11 ✅ | 0 🔜
🔗 Workflows:   5 ✅  | 0 🔜
🧪 Tests:       Coverage in progress
📄 Lines:       ~12,000 lines of Rust
⚡ Binary Size: <5 MB (release)
```

---

## 🙏 Acknowledgements

Flux stands on the shoulders of giants:

- 🦀 **[Rust](https://www.rust-lang.org/)** - The language that makes this possible
- ⚡ **[Tokio](https://tokio.rs/)** - Async runtime
- 🎯 **[Clap](https://github.com/clap-rs/clap)** - CLI framework
- 📦 **[Serde](https://serde.rs/)** - Serialization framework
- 🔐 **[CIS Benchmarks](https://www.cisecurity.org/cis-benchmarks)** - Security guidelines
- 🛡️ **[Mozilla SSH Guidelines](https://infosec.mozilla.org/guidelines/openssh)** - SSH hardening

Special thanks to all [contributors](https://github.com/ethanbissbort/flux-framework-rust/graphs/contributors)!

---

## 📜 License

Flux Framework is dual-licensed under your choice of:

- **Apache License 2.0** ([LICENSE-APACHE](LICENSE-APACHE))
- **MIT License** ([LICENSE-MIT](LICENSE-MIT))

This means you can use Flux in your projects under either license.

---

## 📬 Contact & Support

- 🐛 **Issues**: [GitHub Issues](https://github.com/ethanbissbort/flux-framework-rust/issues)
- 💬 **Discussions**: [GitHub Discussions](https://github.com/ethanbissbort/flux-framework-rust/discussions)
- 📧 **Email**: flux-framework@example.com
- 🌐 **Website**: https://flux-framework.io

---

<div align="center">

**⚡ Built with ❤️ and Rust 🦀**

*Making Linux system administration fast, safe, and enjoyable*

[⬆ Back to Top](#-flux-framework)

</div>
