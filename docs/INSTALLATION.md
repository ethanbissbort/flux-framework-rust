# 📦 Flux Framework - Installation Guide

> **Complete installation instructions for all supported platforms**

---

## 📖 Table of Contents

- [Prerequisites](#-prerequisites)
- [Quick Start](#-quick-start)
- [Installation Methods](#-installation-methods)
  - [Build from Source](#-build-from-source)
  - [Pre-built Binaries](#-pre-built-binaries-future)
  - [Package Managers](#-package-managers-future)
- [Distribution-Specific Notes](#-distribution-specific-notes)
- [Post-Installation](#-post-installation)
- [Verification](#-verification)
- [Troubleshooting](#-troubleshooting)
- [Uninstallation](#-uninstallation)

---

## ✅ Prerequisites

### System Requirements

| Requirement | Minimum | Recommended |
|-------------|---------|-------------|
| **CPU** | 1 core | 2+ cores |
| **RAM** | 512 MB | 1 GB+ |
| **Disk Space** | 100 MB | 500 MB+ |
| **Linux Kernel** | 3.10+ | 4.15+ |

### Supported Distributions

| Distribution | Version | Status | Package Manager |
|--------------|---------|--------|-----------------|
| 🟢 **Ubuntu** | 20.04, 22.04, 24.04 | ✅ Fully Supported | apt |
| 🟢 **Debian** | 11, 12 | ✅ Fully Supported | apt |
| 🟢 **RHEL** | 8, 9 | ✅ Fully Supported | yum/dnf |
| 🟢 **CentOS Stream** | 8, 9 | ✅ Fully Supported | yum/dnf |
| 🟢 **Rocky Linux** | 8, 9 | ✅ Fully Supported | dnf |
| 🟢 **AlmaLinux** | 8, 9 | ✅ Fully Supported | dnf |
| 🟢 **Fedora** | 38, 39, 40 | ✅ Fully Supported | dnf |
| 🟡 **Alpine** | Latest | 🔜 Planned (v0.5) | apk |
| 🟡 **Arch** | Latest | 🔜 Planned (v0.5) | pacman |

### Software Dependencies

**Build Dependencies:**
```
- Rust 1.77 or later
- Cargo (comes with Rust)
- Git
- C compiler (gcc or clang)
- OpenSSL development libraries
```

**Runtime Dependencies:**
```
- systemd (optional, for service management)
- OpenSSL 1.1.1 or later
```

---

## 🚀 Quick Start

### TL;DR - One Command Installation

For Debian/Ubuntu systems with Rust already installed:

```bash
# Clone, build, and install
git clone https://github.com/ethanbissbort/flux-framework-rust.git && \
cd flux-framework-rust && \
cargo build --release && \
sudo install -m755 target/release/flux /usr/local/bin/flux && \
flux --version
```

---

## 📥 Installation Methods

---

## 🔨 Build from Source

> **Recommended method** - Full control, latest features, all platforms

### Step 1: Install Rust

If you don't have Rust installed:

```bash
# Install Rust using rustup (official installer)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Follow the prompts, then:
source $HOME/.cargo/env

# Verify installation
rustc --version
cargo --version
```

**Expected output:**
```
rustc 1.77.0 (or later)
cargo 1.77.0 (or later)
```

### Step 2: Install Build Dependencies

<details>
<summary><b>🐧 Debian/Ubuntu</b></summary>

```bash
sudo apt update
sudo apt install -y \
    git \
    build-essential \
    pkg-config \
    libssl-dev \
    curl
```

</details>

<details>
<summary><b>🎩 RHEL/CentOS/Rocky/Alma</b></summary>

```bash
sudo yum install -y \
    git \
    gcc \
    gcc-c++ \
    make \
    openssl-devel \
    pkg-config \
    curl

# Or on newer systems with dnf:
sudo dnf install -y \
    git \
    gcc \
    gcc-c++ \
    make \
    openssl-devel \
    pkg-config \
    curl
```

</details>

<details>
<summary><b>🎓 Fedora</b></summary>

```bash
sudo dnf install -y \
    git \
    gcc \
    gcc-c++ \
    make \
    openssl-devel \
    pkg-config \
    curl
```

</details>

### Step 3: Clone Repository

```bash
# Clone the repository
git clone https://github.com/ethanbissbort/flux-framework-rust.git

# Navigate to directory
cd flux-framework-rust

# Optional: Checkout specific version
git checkout v3.0.0
```

### Step 4: Build Flux

```bash
# Build in release mode (optimized)
cargo build --release

# This will take 5-10 minutes on first build
# Subsequent builds are much faster
```

**Build output location:**
```
target/release/flux
```

### Step 5: Install System-Wide

```bash
# Install to /usr/local/bin (requires sudo)
sudo install -m755 target/release/flux /usr/local/bin/flux

# Alternative: Install to user bin directory
# install -m755 target/release/flux ~/.local/bin/flux
```

### Step 6: Verify Installation

```bash
# Check version
flux --version

# Should output:
# flux-framework 3.0.0
```

### Build Customization

**Optimized Build (Smaller Binary):**
```bash
cargo build --release
strip target/release/flux
```

**Debug Build (Development):**
```bash
cargo build
# Binary at: target/debug/flux
```

**With Specific Features:**
```bash
# Build with all features
cargo build --release --all-features

# Build without default features
cargo build --release --no-default-features
```

---

## 📦 Pre-built Binaries (Future)

> **Coming in v3.1** - Download and run, no compilation needed

Pre-built binaries will be available for:

- ✅ Linux x86_64 (GNU)
- ✅ Linux x86_64 (musl) - Static binary
- ✅ Linux ARM64 (aarch64)

**Future installation:**
```bash
# Download latest release
curl -LO https://github.com/ethanbissbort/flux-framework-rust/releases/latest/download/flux-linux-x86_64

# Make executable
chmod +x flux-linux-x86_64

# Install
sudo mv flux-linux-x86_64 /usr/local/bin/flux

# Verify
flux --version
```

---

## 📦 Package Managers (Future)

> **Coming in v3.1+** - Native package manager support

### Cargo (Rust Package Manager)

**Future installation:**
```bash
# Install from crates.io
cargo install flux-framework

# Verify
flux --version
```

### APT (Debian/Ubuntu)

**Future installation:**
```bash
# Add Flux repository
curl -fsSL https://flux-framework.io/gpg | sudo apt-key add -
echo "deb https://repo.flux-framework.io/apt stable main" | sudo tee /etc/apt/sources.list.d/flux.list

# Install
sudo apt update
sudo apt install flux-framework
```

### DNF/YUM (RHEL/Fedora)

**Future installation:**
```bash
# Add Flux repository
sudo dnf config-manager --add-repo https://repo.flux-framework.io/rpm/flux.repo

# Install
sudo dnf install flux-framework
```

### Homebrew (macOS/Linux)

**Future installation:**
```bash
brew install flux-framework
```

---

## 🐧 Distribution-Specific Notes

### Ubuntu 20.04 LTS

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Install dependencies
sudo apt update
sudo apt install -y git build-essential pkg-config libssl-dev

# Build Flux
git clone https://github.com/ethanbissbort/flux-framework-rust.git
cd flux-framework-rust
cargo build --release
sudo install -m755 target/release/flux /usr/local/bin/flux

# Verify
flux --version
```

### Ubuntu 22.04/24.04 LTS

Same as Ubuntu 20.04, but you may have newer packages:

```bash
# Dependencies install faster, might have newer OpenSSL
sudo apt install -y git build-essential pkg-config libssl-dev
```

### Debian 11/12

Same process as Ubuntu:

```bash
sudo apt update
sudo apt install -y git build-essential pkg-config libssl-dev curl
```

### RHEL 8/9, Rocky Linux, AlmaLinux

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Install dependencies
sudo dnf install -y git gcc gcc-c++ make openssl-devel pkg-config

# Build Flux
git clone https://github.com/ethanbissbort/flux-framework-rust.git
cd flux-framework-rust
cargo build --release
sudo install -m755 target/release/flux /usr/local/bin/flux

# Verify
flux --version
```

### CentOS Stream 8/9

Same as RHEL:

```bash
sudo dnf install -y git gcc gcc-c++ make openssl-devel pkg-config
```

### Fedora 38/39/40

Fedora typically has the newest packages:

```bash
# Install Rust (or use system Rust)
sudo dnf install -y rust cargo

# Or use rustup for latest version
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install dependencies
sudo dnf install -y git gcc make openssl-devel pkg-config

# Build Flux
git clone https://github.com/ethanbissbort/flux-framework-rust.git
cd flux-framework-rust
cargo build --release
sudo install -m755 target/release/flux /usr/local/bin/flux
```

---

## ✅ Post-Installation

### Shell Completion

Flux supports shell completion for bash, zsh, and fish.

**Bash:**
```bash
# Generate completion script
flux --generate-completion bash | sudo tee /etc/bash_completion.d/flux

# Reload shell
source ~/.bashrc
```

**ZSH:**
```bash
# Generate completion script
flux --generate-completion zsh > ~/.zfunc/_flux

# Add to .zshrc if not already present
echo 'fpath=(~/.zfunc $fpath)' >> ~/.zshrc
echo 'autoload -Uz compinit && compinit' >> ~/.zshrc

# Reload shell
source ~/.zshrc
```

**Fish:**
```bash
# Generate completion script
flux --generate-completion fish > ~/.config/fish/completions/flux.fish

# Reload fish
source ~/.config/fish/config.fish
```

### Configuration Directory

Create default configuration directory:

```bash
# Create system config directory
sudo mkdir -p /etc/flux

# Or user config directory
mkdir -p ~/.config/flux
```

### Logging Setup

Configure logging directory:

```bash
# Create log directory
sudo mkdir -p /var/log/flux
sudo chmod 755 /var/log/flux

# For user-level logging
mkdir -p ~/.local/share/flux/logs
```

### First Run

Check system status:

```bash
# Display system information and compatibility
flux status

# List available modules
flux list

# Show help
flux --help
```

---

## 🔍 Verification

### Test Installation

Run through these verification steps:

```bash
# 1. Check version
flux --version
# Expected: flux-framework 3.0.0

# 2. Display help
flux --help
# Expected: Shows command help

# 3. List modules
flux list
# Expected: Shows all 11 modules

# 4. Check system status
flux status
# Expected: System information display

# 5. Test a module help
flux module update --help
# Expected: Update module help text
```

### Binary Information

```bash
# Check binary location
which flux
# Expected: /usr/local/bin/flux

# Check binary size
ls -lh $(which flux)
# Expected: ~3-5 MB (release build)

# Check dependencies
ldd $(which flux)
# Expected: Shows linked libraries (OpenSSL, etc.)
```

### Permissions

Ensure proper permissions:

```bash
# Binary should be executable by all
ls -l $(which flux)
# Expected: -rwxr-xr-x ... /usr/local/bin/flux
```

---

## 🔧 Troubleshooting

### Build Errors

**Issue: "error: linker `cc` not found"**

Solution:
```bash
# Debian/Ubuntu
sudo apt install build-essential

# RHEL/CentOS/Fedora
sudo dnf install gcc gcc-c++
```

**Issue: "could not find OpenSSL"**

Solution:
```bash
# Debian/Ubuntu
sudo apt install libssl-dev pkg-config

# RHEL/CentOS/Fedora
sudo dnf install openssl-devel pkg-config
```

**Issue: "Rust version too old"**

Solution:
```bash
# Update Rust
rustup update stable
rustup default stable

# Or install latest rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Runtime Errors

**Issue: "flux: command not found"**

Solution:
```bash
# Check if in PATH
echo $PATH

# If /usr/local/bin not in PATH, add it:
echo 'export PATH="/usr/local/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc

# Or reinstall to different location
sudo install -m755 target/release/flux /usr/bin/flux
```

**Issue: "Permission denied"**

Solution:
```bash
# Most commands require sudo
sudo flux module update --update

# Or fix binary permissions
sudo chmod +x /usr/local/bin/flux
```

**Issue: "error while loading shared libraries: libssl.so.X"**

Solution:
```bash
# Install/update OpenSSL
# Debian/Ubuntu
sudo apt install libssl1.1

# RHEL/CentOS/Fedora
sudo dnf install openssl-libs

# Check linked libraries
ldd /usr/local/bin/flux
```

### Compatibility Issues

**Issue: "Module not available on this system"**

Check distribution compatibility:
```bash
# View detected distribution
flux status

# Some modules may not be available on all distributions
# This is expected behavior
```

**Issue: "Unsupported distribution"**

Solution:
```bash
# Check /etc/os-release
cat /etc/os-release

# Flux supports major distros, but some detection may fail
# Report as issue: https://github.com/ethanbissbort/flux-framework-rust/issues
```

---

## 🗑️ Uninstallation

### Remove Flux Binary

```bash
# Remove installed binary
sudo rm /usr/local/bin/flux

# Or if installed elsewhere
sudo rm /usr/bin/flux
```

### Remove Configuration

```bash
# Remove system configuration
sudo rm -rf /etc/flux

# Remove user configuration
rm -rf ~/.config/flux
```

### Remove Logs

```bash
# Remove system logs
sudo rm -rf /var/log/flux

# Remove user logs
rm -rf ~/.local/share/flux
```

### Remove Shell Completions

```bash
# Bash
sudo rm /etc/bash_completion.d/flux

# ZSH
rm ~/.zfunc/_flux

# Fish
rm ~/.config/fish/completions/flux.fish
```

### Clean Build Artifacts

If you built from source:

```bash
# Navigate to source directory
cd flux-framework-rust

# Clean build artifacts
cargo clean

# Optionally remove source
cd ..
rm -rf flux-framework-rust
```

---

## 🐳 Docker Installation

For containerized environments:

```dockerfile
# Dockerfile
FROM rust:1.77 as builder

WORKDIR /build
RUN git clone https://github.com/ethanbissbort/flux-framework-rust.git .
RUN cargo build --release

FROM ubuntu:22.04
COPY --from=builder /build/target/release/flux /usr/local/bin/flux
RUN apt-get update && apt-get install -y \
    openssh-server \
    ufw \
    && rm -rf /var/lib/apt/lists/*

ENTRYPOINT ["flux"]
CMD ["--help"]
```

Build and run:
```bash
docker build -t flux-framework .
docker run --rm flux-framework --version
```

---

## 🔄 Updating Flux

### Update from Source

```bash
# Navigate to source directory
cd flux-framework-rust

# Pull latest changes
git pull origin main

# Or checkout specific version
git fetch --tags
git checkout v3.1.0

# Rebuild
cargo build --release

# Reinstall
sudo install -m755 target/release/flux /usr/local/bin/flux

# Verify new version
flux --version
```

### Update with Cargo (Future)

```bash
# Update to latest version
cargo install flux-framework --force

# Update to specific version
cargo install flux-framework --version 3.1.0 --force
```

---

## 📊 Installation Size

### Disk Space Usage

| Component | Size |
|-----------|------|
| **Source Code** | ~500 KB |
| **Build Dependencies** | ~200 MB (Rust + libs) |
| **Build Artifacts** | ~500 MB (target/ dir) |
| **Final Binary (release)** | ~3-5 MB |
| **Final Binary (stripped)** | ~2-3 MB |

### Optimization

To minimize binary size:

```bash
# Build with optimization
cargo build --release

# Strip symbols
strip target/release/flux

# Result: 2-3 MB binary
```

---

## 📚 Next Steps

After installation:

1. ✅ **Verify Installation** - Run `flux --version`
2. ✅ **Check System Status** - Run `flux status`
3. ✅ **List Modules** - Run `flux list`
4. ✅ **Read Documentation** - Browse the [Modules Reference](MODULES.md)
5. ✅ **Try a Workflow** - Run `sudo flux workflow essential`
6. ✅ **Explore Examples** - Check [Examples Guide](EXAMPLES.md)

---

## 📖 Additional Resources

- 📖 [Quick Start Guide](../README.md#-quick-start) - Get started fast
- 📖 [Modules Reference](MODULES.md) - All module documentation
- 📖 [Workflows Guide](WORKFLOWS.md) - Pre-built automation
- 📖 [Examples](EXAMPLES.md) - Real-world usage scenarios
- 📖 [Troubleshooting](../README.md#troubleshooting) - Common issues

---

<div align="center">

**⚡ Ready to Build Amazing Systems!**

[GitHub](https://github.com/ethanbissbort/flux-framework-rust) •
[Documentation](../README.md) •
[Support](https://github.com/ethanbissbort/flux-framework-rust/issues)

</div>
