# 🧩 Flux Framework - Modules Reference

> **Comprehensive documentation for all Flux modules**

---

## 📖 Table of Contents

- [Overview](#-overview)
- [Module Categories](#-module-categories)
- [All Modules](#-all-modules)
  - [1. Update Module](#1--update-module)
  - [2. Network Module](#2--network-module)
  - [3. Hostname Module](#3--hostname-module)
  - [4. User Module](#4--user-module)
  - [5. SSH Module](#5--ssh-module)
  - [6. Firewall Module](#6--firewall-module)
  - [7. Sysctl Module](#7--sysctl-module)
  - [8. Certs Module](#8--certs-module)
  - [9. ZSH Module](#9--zsh-module)
  - [10. MOTD Module](#10--motd-module)
  - [11. Netdata Module](#11--netdata-module)
- [Module Usage Patterns](#-module-usage-patterns)

---

## 🌟 Overview

Flux Framework provides **11 specialized modules** for comprehensive Linux system management. Each module is:

- ✅ **Self-contained** - Works independently or as part of workflows
- ✅ **Interactive** - Provides menu-driven interfaces for ease of use
- ✅ **Scriptable** - Supports command-line arguments for automation
- ✅ **Idempotent** - Safe to run multiple times without side effects
- ✅ **Logged** - All operations are logged for audit trails

### Common Module Features

All modules support these standard options:

| Option | Description |
|--------|-------------|
| `--help`, `-h` | Display module help and usage information |
| `--menu` | Launch interactive menu interface |
| `--verbose`, `-v` | Enable verbose output |
| `--dry-run` | Preview changes without applying them (where applicable) |

---

## 🏷️ Module Categories

### 🔄 **System Maintenance**
- **update** - System package updates and security patches
- **certs** - Certificate management and trust store

### 🌐 **Network & Identity**
- **network** - Network interface and routing configuration
- **hostname** - System hostname and FQDN setup

### 👥 **User Management**
- **user** - User and group administration

### 🔐 **Security & Hardening**
- **ssh** - SSH server hardening and configuration
- **firewall** - Host firewall management (UFW/firewalld)
- **sysctl** - Kernel parameter tuning and security

### 💻 **Developer Tools**
- **zsh** - ZSH shell with Oh-My-Zsh configuration

### 📊 **Monitoring & Display**
- **motd** - Dynamic Message of the Day
- **netdata** - Real-time monitoring agent

---

## 📦 All Modules

---

## 1. 🔄 Update Module

> **System package updates and security patches**

### Overview

The Update module keeps your system current with the latest packages and security patches. It supports all major Linux distributions and package managers.

### Key Features

- ✅ Automatic package manager detection (apt, yum, dnf, zypper)
- ✅ Security-only updates option
- ✅ Automatic reboot detection for kernel updates
- ✅ Update cache refresh
- ✅ Distribution upgrade support
- ✅ Package hold/pin management

### Command-Line Options

```bash
flux module update [OPTIONS]

Options:
  --update              Update all packages
  --security-only       Install security updates only
  --upgrade-distro      Perform distribution upgrade
  --reboot-check        Check if reboot is needed
  --menu                Interactive menu
```

### Interactive Menu Features

1. 🔄 Update all packages
2. 🔐 Security updates only
3. 📦 Upgrade distribution
4. 🔍 Check for available updates
5. 🔁 Check if reboot is needed
6. 🚪 Exit

### Usage Examples

```bash
# Update all packages
sudo flux module update --update

# Security updates only
sudo flux module update --security-only

# Interactive menu
sudo flux module update --menu

# Check if reboot is needed after updates
sudo flux module update --reboot-check
```

### Configuration

The module automatically detects your package manager and adjusts behavior accordingly:

- **Debian/Ubuntu**: Uses `apt-get`
- **RHEL/CentOS/Rocky/Alma**: Uses `yum` or `dnf`
- **Fedora**: Uses `dnf`
- **SUSE/openSUSE**: Uses `zypper`

### Security Considerations

- 🔒 Always backs up package lists before major updates
- 🔒 Validates package signatures
- 🔒 Provides option for security-only updates in production
- 🔒 Warns before distribution upgrades

---

## 2. 🌐 Network Module

> **Network interface and routing configuration**

### Overview

Comprehensive network configuration including static IPs, VLANs, bonding, and diagnostics.

### Key Features

- ✅ Static IP configuration
- ✅ VLAN support (802.1Q)
- ✅ Network interface bonding
- ✅ DNS configuration
- ✅ Routing table management
- ✅ Network diagnostics and testing
- ✅ Interface statistics

### Command-Line Options

```bash
flux module network [OPTIONS]

Options:
  --configure           Configure network interface
  --static-ip <ip>      Set static IP address
  --interface <name>    Specify network interface
  --vlan <id>           Create VLAN interface
  --dns <servers>       Set DNS servers (comma-separated)
  --status              Show network status
  --diagnostics         Run network diagnostics
  --menu                Interactive menu
```

### Interactive Menu Features

1. 🔍 Show network status
2. 🔧 Configure static IP
3. 🏷️ Create VLAN interface
4. 🔗 Configure bonding/teaming
5. 🌐 Configure DNS servers
6. 📡 Configure routes
7. 🧪 Run network diagnostics
8. 🚪 Exit

### Usage Examples

```bash
# Show network status
sudo flux module network --status

# Configure static IP
sudo flux module network --static-ip 192.168.1.100 --interface eth0

# Create VLAN
sudo flux module network --vlan 100 --interface eth0

# Set DNS servers
sudo flux module network --dns "8.8.8.8,8.8.4.4"

# Interactive configuration
sudo flux module network --menu
```

### Configuration Examples

**Static IP Configuration:**
```
Interface: eth0
IP Address: 192.168.1.100/24
Gateway: 192.168.1.1
DNS: 8.8.8.8, 8.8.4.4
```

**VLAN Configuration:**
```
Parent Interface: eth0
VLAN ID: 100
IP Address: 10.100.1.10/24
```

### Security Considerations

- 🔒 Validates IP addresses and network masks
- 🔒 Backs up existing network configuration
- 🔒 Tests connectivity before finalizing changes
- 🔒 Prevents accidental network lockout (SSH check)

---

## 3. 🏷️ Hostname Module

> **System hostname and FQDN configuration**

### Overview

Manages system hostname, FQDN, and /etc/hosts file for proper system identification.

### Key Features

- ✅ Hostname validation (RFC compliant)
- ✅ FQDN support
- ✅ /etc/hosts management
- ✅ Cloud-init compatibility
- ✅ Persistent across reboots
- ✅ DNS verification

### Command-Line Options

```bash
flux module hostname [OPTIONS]

Options:
  --set <name>          Set system hostname
  --fqdn <domain>       Set fully qualified domain name
  --show                Display current hostname
  --verify              Verify hostname configuration
  --menu                Interactive menu
```

### Interactive Menu Features

1. 🖥️ Show current hostname
2. ✏️ Set new hostname
3. 🌐 Set FQDN
4. 📝 Update /etc/hosts
5. ✅ Verify configuration
6. 🚪 Exit

### Usage Examples

```bash
# Show current hostname
flux module hostname --show

# Set hostname
sudo flux module hostname --set webserver01

# Set FQDN
sudo flux module hostname --fqdn webserver01.example.com

# Verify configuration
sudo flux module hostname --verify

# Interactive setup
sudo flux module hostname --menu
```

### Configuration Examples

**Simple Hostname:**
```bash
sudo flux module hostname --set prod-db-01
# Result: prod-db-01
```

**Full FQDN:**
```bash
sudo flux module hostname --fqdn prod-db-01.internal.company.com
# Hostname: prod-db-01
# Domain: internal.company.com
```

### Security Considerations

- 🔒 Validates hostname format (no special characters)
- 🔒 Updates all hostname-related files atomically
- 🔒 Backs up configuration before changes
- 🔒 Verifies DNS resolution after changes

---

## 4. 👤 User Module

> **User and group management**

### Overview

Comprehensive user account management including admin user creation, SSH key setup, and GitHub key integration.

### Key Features

- ✅ User creation and deletion
- ✅ Admin user setup with sudo privileges
- ✅ SSH directory and authorized_keys management
- ✅ GitHub SSH key import
- ✅ Group membership management
- ✅ Shell configuration
- ✅ Password management

### Command-Line Options

```bash
flux module user [OPTIONS]

Options:
  --create <username>   Create new user
  --admin <username>    Create admin user with sudo
  --github <username>   Import SSH keys from GitHub
  --groups <list>       Add user to groups (comma-separated)
  --shell <path>        Set user shell
  --delete <username>   Delete user
  --list                List all users
  --menu                Interactive menu
```

### Interactive Menu Features

1. 👤 Create standard user
2. 👨‍💼 Create admin user
3. 🔑 Setup SSH keys
4. 📥 Import GitHub SSH keys
5. 👥 Manage groups
6. 🔐 Change user password
7. 📋 List users
8. ❌ Delete user
9. 🚪 Exit

### Usage Examples

```bash
# Create standard user
sudo flux module user --create john --shell /bin/bash

# Create admin user with GitHub keys
sudo flux module user --admin alice --github alice

# Add user to groups
sudo flux module user --create bob --groups "docker,developers"

# List all users
sudo flux module user --list

# Interactive management
sudo flux module user --menu
```

### Configuration Examples

**Admin User with GitHub Keys:**
```bash
sudo flux module user --admin deploy --github deploybot
# Creates user 'deploy'
# Adds to sudo/wheel group
# Imports SSH keys from github.com/deploybot
```

**Developer User:**
```bash
sudo flux module user --create dev1 \
  --shell /bin/zsh \
  --groups "docker,developers,sudo"
```

### Security Considerations

- 🔒 Enforces strong password policies
- 🔒 Sets proper file permissions on SSH directories (700, 600)
- 🔒 Validates usernames against system requirements
- 🔒 Securely downloads SSH keys over HTTPS
- 🔒 Prevents accidental deletion of system users
- 🔒 Logs all user management operations

---

## 5. 🔐 SSH Module

> **SSH server hardening and configuration**

### Overview

Hardens SSH server configuration following industry best practices and security benchmarks (CIS, Mozilla SSH Guidelines).

### Key Features

- ✅ Full SSH hardening (port, ciphers, authentication)
- ✅ Password authentication disabling
- ✅ fail2ban integration
- ✅ Host key regeneration
- ✅ Configuration validation
- ✅ SSH banner creation
- ✅ Modern cipher suites

### Command-Line Options

```bash
flux module ssh [OPTIONS]

Options:
  --harden              Apply full SSH hardening
  --port <port>         Change SSH port
  --disable-passwords   Disable password authentication
  --fail2ban            Setup fail2ban protection
  --generate-keys       Generate new host keys
  --validate            Validate SSH configuration
  --status              Show SSH status
  --menu                Interactive menu
```

### Interactive Menu Features

1. 🔐 Run hardening wizard
2. 🔢 Change SSH port
3. 🚫 Disable password authentication
4. 🛡️ Setup fail2ban
5. 🔑 Generate new host keys
6. ✅ Validate configuration
7. 📊 Show SSH status
8. 🚪 Exit

### Usage Examples

```bash
# Full hardening wizard
sudo flux module ssh --harden

# Change SSH port
sudo flux module ssh --port 2222

# Disable passwords and setup fail2ban
sudo flux module ssh --disable-passwords --fail2ban

# Validate configuration
sudo flux module ssh --validate

# Interactive menu
sudo flux module ssh --menu
```

### Hardening Configuration

**Applied Settings:**

```
Port: 2222 (or custom)
Protocol: 2
PermitRootLogin: no
PasswordAuthentication: no
PubkeyAuthentication: yes
MaxAuthTries: 3
LoginGraceTime: 30
ClientAliveInterval: 300

Ciphers:
  - chacha20-poly1305@openssh.com
  - aes256-gcm@openssh.com
  - aes128-gcm@openssh.com

MACs:
  - hmac-sha2-512-etm@openssh.com
  - hmac-sha2-256-etm@openssh.com

KexAlgorithms:
  - curve25519-sha256
  - diffie-hellman-group16-sha512
```

### fail2ban Configuration

**Automatic Setup:**
- Monitors SSH login attempts
- Blocks after 3 failed attempts
- 1-hour ban duration
- DDOS protection enabled

### Security Considerations

- 🔒 Backs up sshd_config before changes
- 🔒 Validates configuration before restart
- 🔒 Warns about port changes (firewall updates needed)
- 🔒 Ensures SSH access before disabling passwords
- 🔒 Implements rate limiting
- 🔒 Uses modern, secure cryptographic algorithms

---

## 6. 🛡️ Firewall Module

> **Host firewall management (UFW/firewalld)**

### Overview

Manages host-based firewalls with support for UFW (Debian/Ubuntu) and firewalld (RHEL/CentOS/Fedora).

### Key Features

- ✅ Automatic firewall detection and installation
- ✅ Security presets (web, database, mail server, etc.)
- ✅ Custom rule management
- ✅ Service-based rules
- ✅ Port forwarding
- ✅ Zone management (firewalld)
- ✅ Configuration backup/restore

### Command-Line Options

```bash
flux module firewall [OPTIONS]

Options:
  --status              Show firewall status
  --enable              Enable firewall
  --preset <name>       Apply security preset
  --allow <port/proto>  Allow port through firewall
  --list                List firewall rules
  --backup              Backup configuration
  --wizard              Run interactive setup wizard
  --menu                Interactive menu
```

### Interactive Menu Features

1. 🔧 Run setup wizard
2. 📊 Show firewall status
3. 🎯 Apply security preset
4. ➕ Add custom rule
5. 📋 List rules
6. 💾 Backup configuration
7. ✅ Enable firewall
8. 🚪 Exit

### Security Presets

| Preset | Ports Opened | Use Case |
|--------|-------------|----------|
| **minimal** | 22/tcp (SSH) | Maximum security, SSH only |
| **web-server** | 80/tcp, 443/tcp | HTTP/HTTPS web servers |
| **database** | 3306/tcp, 5432/tcp | MySQL, PostgreSQL servers |
| **mail-server** | 25, 465, 587, 143, 993, 110, 995 | Email servers |
| **docker-host** | 2376, 2377, 7946, 4789 | Docker Swarm |
| **kubernetes** | 6443, 2379-2380, 10250-10252 | Kubernetes cluster |

### Usage Examples

```bash
# Apply web server preset
sudo flux module firewall --preset web-server

# Add custom rule
sudo flux module firewall --allow 8080/tcp

# Show status
sudo flux module firewall --status

# Setup wizard
sudo flux module firewall --wizard

# Backup configuration
sudo flux module firewall --backup
```

### Configuration Examples

**Web Server Setup:**
```bash
sudo flux module firewall --preset web-server
# Opens: 22/tcp (SSH), 80/tcp (HTTP), 443/tcp (HTTPS)
```

**Custom Application:**
```bash
sudo flux module firewall --allow 3000/tcp
sudo flux module firewall --allow 5432/tcp
```

### Security Considerations

- 🔒 Always ensures SSH access before enabling
- 🔒 Default deny policy for incoming traffic
- 🔒 Automatic configuration backup
- 🔒 Validates rules before applying
- 🔒 Logs all firewall changes
- 🔒 Prevents accidental lockout

---

## 7. ⚙️ Sysctl Module

> **Kernel parameter tuning and security hardening**

### Overview

Manages kernel runtime parameters for security hardening, performance optimization, and system tuning.

### Key Features

- ✅ Security hardening presets
- ✅ Network stack optimization
- ✅ IPv4/IPv6 configuration
- ✅ Memory and filesystem tuning
- ✅ Custom parameter management
- ✅ Persistent configuration
- ✅ Validation and rollback

### Command-Line Options

```bash
flux module sysctl [OPTIONS]

Options:
  --apply               Apply hardening parameters
  --security            Apply security hardening
  --performance         Apply performance tuning
  --show                Show current parameters
  --set <key=value>     Set custom parameter
  --reset               Reset to defaults
  --menu                Interactive menu
```

### Interactive Menu Features

1. 🔐 Apply security hardening
2. ⚡ Apply performance tuning
3. 📊 Show current parameters
4. ✏️ Set custom parameter
5. 🔄 Reset to defaults
6. ✅ Validate configuration
7. 🚪 Exit

### Hardening Parameters

**Security Settings Applied:**

```
# Network Security
net.ipv4.conf.all.rp_filter = 1
net.ipv4.conf.default.rp_filter = 1
net.ipv4.conf.all.accept_source_route = 0
net.ipv4.conf.default.accept_source_route = 0
net.ipv4.icmp_echo_ignore_broadcasts = 1
net.ipv4.icmp_ignore_bogus_error_responses = 1
net.ipv4.tcp_syncookies = 1

# IPv6 Security
net.ipv6.conf.all.accept_ra = 0
net.ipv6.conf.default.accept_ra = 0
net.ipv6.conf.all.accept_redirects = 0

# Kernel Security
kernel.dmesg_restrict = 1
kernel.kptr_restrict = 2
kernel.yama.ptrace_scope = 1
fs.suid_dumpable = 0
```

### Usage Examples

```bash
# Apply security hardening
sudo flux module sysctl --security

# Apply performance tuning
sudo flux module sysctl --performance

# Set custom parameter
sudo flux module sysctl --set "net.ipv4.ip_forward=1"

# Show current parameters
sudo flux module sysctl --show

# Interactive menu
sudo flux module sysctl --menu
```

### Configuration Examples

**Enable IPv4 Forwarding (for routers):**
```bash
sudo flux module sysctl --set "net.ipv4.ip_forward=1"
```

**Optimize for High-Load Web Server:**
```bash
sudo flux module sysctl --performance
# Adjusts connection limits, buffer sizes, etc.
```

### Security Considerations

- 🔒 Backs up current configuration before changes
- 🔒 Validates parameters before applying
- 🔒 Tests changes before making persistent
- 🔒 Provides rollback mechanism
- 🔒 Logs all parameter changes
- 🔒 Follows CIS benchmark recommendations

---

## 8. 📜 Certs Module

> **Certificate management and trust store**

### Overview

Manages SSL/TLS certificates and system trust stores for secure communications.

### Key Features

- ✅ Certificate installation
- ✅ Trust store updates
- ✅ CA certificate management
- ✅ Certificate validation
- ✅ Automatic trust store refresh
- ✅ PEM/DER format support

### Command-Line Options

```bash
flux module certs [OPTIONS]

Options:
  --install <path>      Install certificate to trust store
  --update              Update system trust store
  --list                List installed certificates
  --verify <path>       Verify certificate
  --remove <name>       Remove certificate
  --menu                Interactive menu
```

### Interactive Menu Features

1. 📥 Install certificate
2. 🔄 Update trust store
3. 📋 List certificates
4. ✅ Verify certificate
5. ❌ Remove certificate
6. 🚪 Exit

### Usage Examples

```bash
# Update trust store
sudo flux module certs --update

# Install CA certificate
sudo flux module certs --install /path/to/ca.crt

# List certificates
sudo flux module certs --list

# Verify certificate
sudo flux module certs --verify /path/to/cert.pem

# Interactive menu
sudo flux module certs --menu
```

### Configuration Examples

**Install Corporate CA:**
```bash
sudo flux module certs --install /tmp/corporate-ca.crt
sudo flux module certs --update
```

### Security Considerations

- 🔒 Validates certificate format before installation
- 🔒 Checks certificate expiration
- 🔒 Updates trust store atomically
- 🔒 Logs all certificate operations
- 🔒 Backs up trust store before changes

---

## 9. 💻 ZSH Module

> **ZSH shell with Oh-My-Zsh configuration**

### Overview

Installs and configures ZSH shell with Oh-My-Zsh framework, themes, and plugins for an enhanced terminal experience.

### Key Features

- ✅ ZSH installation
- ✅ Oh-My-Zsh framework setup
- ✅ Theme selection (powerlevel10k, agnoster, etc.)
- ✅ Plugin management (git, docker, kubectl, etc.)
- ✅ Custom aliases
- ✅ Per-user or system-wide configuration

### Command-Line Options

```bash
flux module zsh [OPTIONS]

Options:
  --install             Install ZSH and Oh-My-Zsh
  --user <username>     Configure for specific user
  --theme <name>        Set Oh-My-Zsh theme
  --plugins <list>      Install plugins (comma-separated)
  --set-default         Set ZSH as default shell
  --menu                Interactive menu
```

### Interactive Menu Features

1. 📦 Install ZSH
2. 🎨 Configure Oh-My-Zsh
3. 🖌️ Select theme
4. 🔌 Manage plugins
5. ⚡ Set as default shell
6. 👤 Configure for user
7. 🚪 Exit

### Available Themes

| Theme | Description | Features |
|-------|-------------|----------|
| **powerlevel10k** | Modern, fast, customizable | Git, icons, segments |
| **agnoster** | Popular, clean design | Git, virtualenv |
| **robbyrussell** | Default, simple | Git branch |
| **avit** | Minimal, informative | Git, time |

### Popular Plugins

- **git** - Git aliases and functions
- **docker** - Docker completions
- **kubectl** - Kubernetes completions
- **terraform** - Terraform completions
- **aws** - AWS CLI completions
- **sudo** - ESC-ESC to prefix sudo
- **history** - Enhanced history search

### Usage Examples

```bash
# Install ZSH with powerlevel10k theme
sudo flux module zsh --install --theme powerlevel10k

# Configure with plugins for user
sudo flux module zsh --user alice --plugins "git,docker,kubectl"

# Set as default shell
sudo flux module zsh --set-default

# Interactive setup
sudo flux module zsh --menu
```

### Configuration Examples

**Developer Workstation:**
```bash
sudo flux module zsh --install \
  --theme powerlevel10k \
  --plugins "git,docker,kubectl,terraform,aws"
```

**Minimal Server Setup:**
```bash
sudo flux module zsh --install \
  --theme robbyrussell \
  --plugins "git,sudo"
```

### Security Considerations

- 🔒 Downloads Oh-My-Zsh from official repository
- 🔒 Validates downloaded files
- 🔒 Sets proper file permissions
- 🔒 User confirmation before changing default shell

---

## 10. 📋 MOTD Module

> **Dynamic Message of the Day**

### Overview

Creates dynamic, informative MOTD (Message of the Day) banners with system information, resource usage, and security status.

### Key Features

- ✅ System information display
- ✅ Resource usage (CPU, memory, disk)
- ✅ Service status monitoring
- ✅ Security warnings
- ✅ Custom banners
- ✅ Update notifications
- ✅ Dynamic content

### Command-Line Options

```bash
flux module motd [OPTIONS]

Options:
  --install             Install MOTD scripts
  --org <name>          Set organization name
  --banner <style>      Set banner style (flux-large, simple, minimal)
  --disable             Disable MOTD
  --enable              Enable MOTD
  --preview             Preview MOTD
  --menu                Interactive menu
```

### Interactive Menu Features

1. 📝 Install/Update MOTD
2. 🏢 Set organization name
3. 🎨 Choose banner style
4. 👁️ Preview MOTD
5. ✅ Enable MOTD
6. ❌ Disable MOTD
7. 🚪 Exit

### Banner Styles

**flux-large** - ASCII art Flux logo with full details
**simple** - Clean, minimal information
**minimal** - Essential info only

### Information Displayed

- 🖥️ System hostname and uptime
- 📊 CPU usage and load averages
- 💾 Memory usage (used/total)
- 💿 Disk usage by partition
- 🌐 IP addresses
- 🔐 Security status
- 📦 Update availability
- 👥 Active user sessions

### Usage Examples

```bash
# Install with custom organization
sudo flux module motd --install --org "MyCompany"

# Set banner style
sudo flux module motd --banner flux-large

# Preview before applying
sudo flux module motd --preview

# Interactive setup
sudo flux module motd --menu
```

### Configuration Examples

**Corporate Server:**
```bash
sudo flux module motd --install \
  --org "Acme Corporation" \
  --banner flux-large
```

**Minimal Server:**
```bash
sudo flux module motd --install --banner minimal
```

### Security Considerations

- 🔒 Doesn't expose sensitive information
- 🔒 Warns about security updates
- 🔒 Alerts on failed login attempts
- 🔒 Shows firewall status

---

## 11. 📊 Netdata Module

> **Real-time monitoring agent**

### Overview

Installs and configures Netdata for real-time system monitoring with beautiful dashboards and alerting.

### Key Features

- ✅ One-line installation
- ✅ Real-time metrics (1s granularity)
- ✅ 2000+ metrics collected
- ✅ Beautiful web dashboard
- ✅ Alert notifications
- ✅ Cloud integration support
- ✅ Automatic plugin detection

### Command-Line Options

```bash
flux module netdata [OPTIONS]

Options:
  --install             Install Netdata
  --disable-telemetry   Disable anonymous telemetry
  --claim-token <token> Claim to Netdata Cloud
  --configure           Configure Netdata
  --start               Start Netdata service
  --stop                Stop Netdata service
  --status              Show Netdata status
  --menu                Interactive menu
```

### Interactive Menu Features

1. 📥 Install Netdata
2. ⚙️ Configure Netdata
3. ☁️ Claim to Netdata Cloud
4. ▶️ Start service
5. ⏹️ Stop service
6. 📊 Show status
7. 🌐 Open dashboard
8. 🚪 Exit

### Usage Examples

```bash
# Install Netdata
sudo flux module netdata --install

# Install without telemetry
sudo flux module netdata --install --disable-telemetry

# Claim to Netdata Cloud
sudo flux module netdata --claim-token "your-token-here"

# Check status
sudo flux module netdata --status

# Interactive menu
sudo flux module netdata --menu
```

### Dashboard Access

After installation, access Netdata dashboard at:
```
http://your-server-ip:19999
```

### Metrics Collected

- 📊 CPU usage per core
- 💾 Memory usage breakdown
- 💿 Disk I/O and usage
- 🌐 Network traffic
- 🔌 Services and processes
- 🐳 Docker containers (if installed)
- 🔥 Application metrics
- 🌡️ Hardware sensors

### Configuration Examples

**Basic Installation:**
```bash
sudo flux module netdata --install --disable-telemetry
```

**With Cloud Integration:**
```bash
sudo flux module netdata --install --claim-token "abc123..."
```

### Security Considerations

- 🔒 Binds to localhost by default
- 🔒 Option to disable telemetry
- 🔒 Access control via firewall
- 🔒 HTTPS available for dashboard
- 🔒 API authentication supported

---

## 🎯 Module Usage Patterns

### Running Multiple Modules

Modules can be chained together in scripts:

```bash
# Setup web server from scratch
sudo flux module update --update
sudo flux module firewall --preset web-server
sudo flux module ssh --harden
sudo flux module user --admin deploy --github deploybot
sudo flux module motd --install --banner flux-large
sudo flux module netdata --install
```

### Interactive vs. Scripted Mode

**Interactive Mode** (with `--menu`):
- Best for one-time setups
- Guided workflows
- Safe for beginners

**Scripted Mode** (with CLI options):
- Best for automation
- Infrastructure as Code
- Repeatable deployments

### Module Dependencies

Some modules work better together:

- **ssh** + **firewall** - Ensure firewall allows SSH port
- **user** + **ssh** - Setup users with SSH key auth
- **update** + **certs** - Update packages and trust store
- **netdata** + **motd** - Monitoring with status display

### Best Practices

1. ✅ **Always run `update` module first** on new systems
2. ✅ **Use `--menu` for exploration**, CLI options for automation
3. ✅ **Test in development** before production deployment
4. ✅ **Backup configurations** before major changes
5. ✅ **Check module help** with `--help` flag
6. ✅ **Review logs** after module execution
7. ✅ **Use workflows** for common scenarios

---

## 📚 Additional Resources

- 📖 [Workflows Guide](WORKFLOWS.md) - Pre-built module combinations
- 📖 [Examples](EXAMPLES.md) - Real-world usage scenarios
- 📖 [Configuration](CONFIGURATION.md) - Configuration file reference
- 📖 [Architecture](ARCHITECTURE.md) - Technical design details

---

<div align="center">

**⚡ Need Help?**

[GitHub Issues](https://github.com/ethanbissbort/flux-framework-rust/issues) •
[Discussions](https://github.com/ethanbissbort/flux-framework-rust/discussions) •
[Documentation](README.md)

</div>
