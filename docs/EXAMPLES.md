# 💼 Flux Framework - Real-World Examples

> **Practical usage scenarios and deployment guides**

---

## 📖 Table of Contents

- [Overview](#-overview)
- [Quick Examples](#-quick-examples)
- [Server Scenarios](#-server-scenarios)
  - [Web Server Setup](#1--web-server-setup)
  - [Database Server Hardening](#2--database-server-hardening)
  - [Developer Workstation](#3--developer-workstation)
  - [Container Host](#4--container-host-docker--kubernetes)
  - [Monitoring Server](#5--monitoring-server)
  - [Multi-Tier Application](#6--multi-tier-application)
- [Advanced Scenarios](#-advanced-scenarios)
- [Automation Scripts](#-automation-scripts)

---

## 🌟 Overview

This guide provides **complete, copy-paste-ready examples** for common server provisioning scenarios. Each example includes:

- 📋 **Use case description**
- 🎯 **Step-by-step commands**
- ⚙️ **Configuration details**
- ✅ **Verification steps**
- 🔒 **Security considerations**

---

## ⚡ Quick Examples

### One-Liner Setups

**Minimal secure server:**
```bash
sudo flux workflow essential
```

**Web server from scratch:**
```bash
sudo flux workflow essential && \
sudo flux module firewall --preset web-server && \
sudo flux module user --admin deploy --github deploybot
```

**Hardened production server:**
```bash
sudo flux workflow security && \
sudo flux module netdata --install && \
sudo flux module motd --install --org "Production"
```

---

## 🖥️ Server Scenarios

---

## 1. 🌐 Web Server Setup

> **Scenario:** Deploy a production web server (NGINX/Apache) with HTTPS, monitoring, and security hardening.

### Server Details
- **OS:** Ubuntu 22.04 LTS
- **Role:** Web server
- **Services:** NGINX, SSL/TLS, Netdata
- **Security:** Hardened SSH, Firewall, fail2ban

### Step-by-Step Setup

#### Step 1: Initial System Setup

```bash
# Run essential workflow
sudo flux workflow essential
```

**What this does:**
- ✅ Updates all packages
- ✅ Installs CA certificates
- ✅ Hardens kernel parameters
- ✅ Secures SSH configuration

#### Step 2: Configure Hostname

```bash
sudo flux module hostname --fqdn web01.example.com
```

**Expected output:**
```
✓ Hostname set to: web01
✓ FQDN set to: web01.example.com
✓ /etc/hosts updated
✓ Configuration verified
```

#### Step 3: Setup Firewall for Web Traffic

```bash
sudo flux module firewall --preset web-server
```

**Ports opened:**
- `22/tcp` - SSH
- `80/tcp` - HTTP
- `443/tcp` - HTTPS

#### Step 4: Harden SSH

```bash
sudo flux module ssh --port 2222 --disable-passwords --fail2ban
```

**Configuration:**
- SSH port: 2222
- Password auth: Disabled
- fail2ban: Enabled

**⚠️ Important:** Update firewall for new SSH port:
```bash
sudo flux module firewall --allow 2222/tcp
```

#### Step 5: Create Deployment User

```bash
sudo flux module user --admin deploy --github your-github-username
```

**What this does:**
- ✅ Creates 'deploy' user
- ✅ Adds to sudo group
- ✅ Imports SSH keys from GitHub
- ✅ Sets up SSH directory

#### Step 6: Install Monitoring

```bash
sudo flux module netdata --install --disable-telemetry
```

**Access dashboard:**
```
http://web01.example.com:19999
```

#### Step 7: Setup Custom MOTD

```bash
sudo flux module motd --install --org "MyCompany Web" --banner flux-large
```

### Install Web Server

```bash
# Install NGINX
sudo apt update
sudo apt install -y nginx

# Install Certbot for SSL
sudo apt install -y certbot python3-certbot-nginx

# Get SSL certificate
sudo certbot --nginx -d web01.example.com
```

### Verification

```bash
# Check services
sudo systemctl status nginx
sudo systemctl status netdata
sudo systemctl status fail2ban

# Check firewall
sudo ufw status verbose

# Test web access
curl http://localhost
curl https://web01.example.com

# Test SSH on new port
ssh -p 2222 deploy@web01.example.com
```

### Security Checklist

- ✅ SSH hardened (port 2222, key-only auth)
- ✅ Firewall active (UFW)
- ✅ fail2ban protecting SSH
- ✅ Kernel hardening enabled
- ✅ SSL/TLS certificates configured
- ✅ Monitoring active (Netdata)
- ✅ Admin user with SSH keys
- ✅ No root login

### Complete Script

```bash
#!/bin/bash
# web-server-setup.sh

set -e

echo "🚀 Setting up web server..."

# Essential setup
sudo flux workflow essential

# Hostname
sudo flux module hostname --fqdn web01.example.com

# Firewall
sudo flux module firewall --preset web-server
sudo flux module firewall --allow 2222/tcp

# SSH hardening
sudo flux module ssh --port 2222 --disable-passwords --fail2ban

# User management
sudo flux module user --admin deploy --github your-github-username

# Monitoring
sudo flux module netdata --install --disable-telemetry

# MOTD
sudo flux module motd --install --org "MyCompany Web" --banner flux-large

# Install NGINX
sudo apt update
sudo apt install -y nginx certbot python3-certbot-nginx

echo "✅ Web server setup complete!"
echo "📊 Dashboard: http://$(hostname -f):19999"
echo "🔐 SSH: ssh -p 2222 deploy@$(hostname -f)"
```

---

## 2. 🗄️ Database Server Hardening

> **Scenario:** Secure PostgreSQL database server with network isolation and monitoring.

### Server Details
- **OS:** Rocky Linux 9
- **Role:** PostgreSQL database server
- **Network:** Static IP, isolated VLAN
- **Security:** Strict firewall, kernel hardening

### Step-by-Step Setup

#### Step 1: Essential Setup

```bash
sudo flux workflow essential
```

#### Step 2: Configure Static IP

```bash
sudo flux module network --menu
```

**Interactive configuration:**
```
Select option: Configure static IP
Interface: eth0
IP Address: 10.0.1.100
Netmask: 255.255.255.0 (or /24)
Gateway: 10.0.1.1
DNS Servers: 10.0.1.10, 10.0.1.11
```

#### Step 3: Set Hostname

```bash
sudo flux module hostname --fqdn db01.internal.company.com
```

#### Step 4: Configure Firewall for Database Access

```bash
# Apply database preset
sudo flux module firewall --preset database

# Restrict PostgreSQL to internal network only
sudo flux module firewall --menu
```

**Custom rules:**
```
Add custom rule
Port: 5432
Protocol: tcp
Source: 10.0.0.0/8
Comment: PostgreSQL - Internal network only
```

#### Step 5: Extreme SSH Hardening

```bash
sudo flux module ssh --harden
```

**Wizard selections:**
- Change SSH port: Yes → 2222
- Disable password auth: Yes
- Setup fail2ban: Yes

#### Step 6: Apply Performance Tuning

```bash
sudo flux module sysctl --menu
```

**Select:**
- Apply performance tuning

**Custom parameters for PostgreSQL:**
```bash
echo "vm.swappiness=10" | sudo tee -a /etc/sysctl.d/99-flux-postgres.conf
echo "vm.dirty_ratio=15" | sudo tee -a /etc/sysctl.d/99-flux-postgres.conf
echo "vm.dirty_background_ratio=5" | sudo tee -a /etc/sysctl.d/99-flux-postgres.conf
sudo sysctl -p /etc/sysctl.d/99-flux-postgres.conf
```

#### Step 7: Create Database Admin User

```bash
sudo flux module user --admin dbadmin --github dbadmin-keys
```

#### Step 8: Install Monitoring

```bash
sudo flux module netdata --install --disable-telemetry
```

### Install PostgreSQL

```bash
# Install PostgreSQL
sudo dnf install -y postgresql-server postgresql-contrib

# Initialize database
sudo postgresql-setup --initdb

# Configure PostgreSQL to listen on internal IP
sudo vi /var/lib/pgsql/data/postgresql.conf
# Set: listen_addresses = '10.0.1.100'

# Configure client authentication
sudo vi /var/lib/pgsql/data/pg_hba.conf
# Add: host all all 10.0.0.0/8 scram-sha-256

# Start and enable PostgreSQL
sudo systemctl enable --now postgresql

# Create admin user
sudo -u postgres createuser --superuser dbadmin
```

### Verification

```bash
# Check PostgreSQL is running
sudo systemctl status postgresql

# Check listening ports
sudo ss -tlnp | grep 5432

# Check firewall rules
sudo firewall-cmd --list-all

# Test database connection (from allowed network)
psql -h 10.0.1.100 -U dbadmin -d postgres

# Check monitoring
curl http://localhost:19999
```

### Security Checklist

- ✅ Static IP configured
- ✅ Firewall: PostgreSQL access restricted to internal network
- ✅ SSH: Port 2222, key-only, fail2ban
- ✅ PostgreSQL: Listening on private IP only
- ✅ PostgreSQL: Strong authentication (scram-sha-256)
- ✅ Kernel: Performance tuning applied
- ✅ Monitoring: Active
- ✅ No public database access

---

## 3. 💻 Developer Workstation

> **Scenario:** Setup a developer-friendly workstation with ZSH, tools, and convenience features.

### Server Details
- **OS:** Ubuntu 24.04 LTS
- **Role:** Developer workstation
- **Users:** Multiple developers
- **Tools:** ZSH, Docker, development tools

### Step-by-Step Setup

#### Step 1: Run Development Workflow

```bash
sudo flux workflow development
```

#### Step 2: Create Developer Users

```bash
# Create multiple developers
sudo flux module user --create alice --shell /bin/zsh --groups "docker,developers,sudo"
sudo flux module user --create bob --shell /bin/zsh --groups "docker,developers,sudo"
sudo flux module user --create charlie --shell /bin/zsh --groups "developers"
```

**With GitHub SSH keys:**
```bash
sudo flux module user --admin alice --github alice
sudo flux module user --admin bob --github bobdev
```

#### Step 3: Configure ZSH for All Users

```bash
# For Alice
sudo flux module zsh --user alice --theme powerlevel10k \
  --plugins "git,docker,kubectl,terraform,aws,sudo"

# For Bob
sudo flux module zsh --user bob --theme agnoster \
  --plugins "git,docker,kubectl,sudo"

# For Charlie
sudo flux module zsh --user charlie --theme robbyrussell \
  --plugins "git,sudo"
```

#### Step 4: Install Development Tools

```bash
# Install Docker
sudo apt update
sudo apt install -y docker.io docker-compose

# Enable Docker service
sudo systemctl enable --now docker

# Install kubectl
curl -LO "https://dl.k8s.io/release/$(curl -L -s https://dl.k8s.io/release/stable.txt)/bin/linux/amd64/kubectl"
sudo install -o root -g root -m 0755 kubectl /usr/local/bin/kubectl

# Install development packages
sudo apt install -y \
  git \
  vim \
  tmux \
  htop \
  jq \
  python3-pip \
  nodejs \
  npm
```

#### Step 5: Setup Custom MOTD

```bash
sudo flux module motd --install --org "Dev Team" --banner simple
```

#### Step 6: Minimal Security (Optional)

For development environments, you might want lighter security:

```bash
# Minimal firewall (just SSH)
sudo flux module firewall --preset minimal

# SSH on standard port (no fail2ban)
sudo flux module ssh --menu
# Select: Change SSH port → No
# Select: Disable passwords → No
```

### Additional Developer Setup

**Install VS Code:**
```bash
wget -qO- https://packages.microsoft.com/keys/microsoft.asc | gpg --dearmor > packages.microsoft.gpg
sudo install -o root -g root -m 644 packages.microsoft.gpg /etc/apt/trusted.gpg.d/
sudo sh -c 'echo "deb [arch=amd64] https://packages.microsoft.com/repos/vscode stable main" > /etc/apt/sources.list.d/vscode.list'
sudo apt update
sudo apt install -y code
```

**Setup Git config for all users:**
```bash
# Create a setup script for new users
cat << 'EOF' | sudo tee /usr/local/bin/dev-setup
#!/bin/bash
echo "Setting up development environment..."

git config --global color.ui auto
git config --global core.editor vim
git config --global init.defaultBranch main

echo "Git configured!"
echo "Don't forget to set your name and email:"
echo "  git config --global user.name 'Your Name'"
echo "  git config --global user.email 'you@example.com'"
EOF

sudo chmod +x /usr/local/bin/dev-setup
```

### Verification

```bash
# Check users
cat /etc/passwd | grep -E "alice|bob|charlie"

# Check ZSH installation
which zsh
echo $SHELL

# Check Docker
docker --version
docker ps

# Check kubectl
kubectl version --client

# Check developer tools
git --version
node --version
npm --version
python3 --version
```

---

## 4. 🐳 Container Host (Docker & Kubernetes)

> **Scenario:** Setup a container orchestration host for Docker Swarm or Kubernetes.

### Server Details
- **OS:** Ubuntu 22.04 LTS
- **Role:** Container host
- **Services:** Docker, containerd, Kubernetes
- **Network:** Multiple interfaces for cluster communication

### Step-by-Step Setup

#### Step 1: Essential Baseline

```bash
sudo flux workflow essential
```

#### Step 2: Configure Hostname

```bash
sudo flux module hostname --fqdn kube-node-01.cluster.local
```

#### Step 3: Configure Network

For Kubernetes, you may need specific networking:

```bash
sudo flux module network --menu
```

**Disable swap (required for Kubernetes):**
```bash
sudo swapoff -a
sudo sed -i '/ swap / s/^/#/' /etc/fstab
```

#### Step 4: Firewall for Container Host

```bash
sudo flux module firewall --preset docker-host
```

**Or for Kubernetes:**
```bash
sudo flux module firewall --preset kubernetes
```

#### Step 5: Kernel Parameters for Containers

```bash
# Enable IP forwarding
sudo flux module sysctl --set "net.ipv4.ip_forward=1"
sudo flux module sysctl --set "net.bridge.bridge-nf-call-iptables=1"
sudo flux module sysctl --set "net.bridge.bridge-nf-call-ip6tables=1"
```

#### Step 6: Create Container Admin User

```bash
sudo flux module user --admin k8sadmin --github k8sadmin --groups "docker,sudo"
```

### Install Docker

```bash
# Install Docker
sudo apt update
sudo apt install -y docker.io docker-compose

# Enable Docker service
sudo systemctl enable --now docker

# Add users to docker group
sudo usermod -aG docker k8sadmin

# Test Docker
docker run hello-world
```

### Install Kubernetes (kubeadm)

```bash
# Install prerequisites
sudo apt update
sudo apt install -y apt-transport-https ca-certificates curl

# Add Kubernetes repository
curl -fsSL https://packages.cloud.google.com/apt/doc/apt-key.gpg | sudo apt-key add -
echo "deb https://apt.kubernetes.io/ kubernetes-xenial main" | sudo tee /etc/apt/sources.list.d/kubernetes.list

# Install Kubernetes components
sudo apt update
sudo apt install -y kubelet kubeadm kubectl
sudo apt-mark hold kubelet kubeadm kubectl

# Initialize cluster (master node only)
sudo kubeadm init --pod-network-cidr=10.244.0.0/16

# Setup kubectl for admin user
mkdir -p $HOME/.kube
sudo cp -i /etc/kubernetes/admin.conf $HOME/.kube/config
sudo chown $(id -u):$(id -g) $HOME/.kube/config

# Install pod network (Flannel)
kubectl apply -f https://raw.githubusercontent.com/flannel-io/flannel/master/Documentation/kube-flannel.yml
```

### Monitoring

```bash
sudo flux module netdata --install --disable-telemetry
```

Netdata will automatically detect Docker containers and Kubernetes metrics.

### Verification

```bash
# Check Docker
docker --version
docker ps
docker info

# Check Kubernetes
kubectl version
kubectl get nodes
kubectl get pods --all-namespaces

# Check networking
ip route
iptables -L -n | head -20

# Check monitoring
curl http://localhost:19999
```

---

## 5. 📊 Monitoring Server

> **Scenario:** Centralized monitoring server collecting metrics from multiple hosts.

### Server Details
- **OS:** Debian 12
- **Role:** Monitoring and observability
- **Services:** Netdata, Grafana, Prometheus (optional)
- **Access:** Web dashboards

### Step-by-Step Setup

#### Step 1: Run Monitoring Workflow

```bash
sudo flux workflow monitoring
```

#### Step 2: Configure Hostname

```bash
sudo flux module hostname --fqdn monitor.internal.company.com
```

#### Step 3: Setup Firewall

```bash
# Apply web server preset (for dashboards)
sudo flux module firewall --preset web-server

# Add Netdata port
sudo flux module firewall --allow 19999/tcp

# Add Grafana port (if installing)
sudo flux module firewall --allow 3000/tcp

# Add Prometheus port (if installing)
sudo flux module firewall --allow 9090/tcp
```

#### Step 4: Install Additional Monitoring Tools

**Install Grafana:**
```bash
# Add Grafana repository
sudo apt-get install -y software-properties-common
sudo add-apt-repository "deb https://packages.grafana.com/oss/deb stable main"
wget -q -O - https://packages.grafana.com/gpg.key | sudo apt-key add -

# Install Grafana
sudo apt-get update
sudo apt-get install -y grafana

# Start Grafana
sudo systemctl enable --now grafana-server

# Access at: http://monitor.internal.company.com:3000
# Default login: admin/admin
```

**Install Prometheus (Optional):**
```bash
# Download Prometheus
wget https://github.com/prometheus/prometheus/releases/download/v2.45.0/prometheus-2.45.0.linux-amd64.tar.gz
tar xvfz prometheus-*.tar.gz
cd prometheus-*

# Create Prometheus user
sudo useradd --no-create-home --shell /bin/false prometheus

# Create directories
sudo mkdir /etc/prometheus /var/lib/prometheus

# Copy binaries
sudo cp prometheus promtool /usr/local/bin/
sudo cp -r consoles console_libraries /etc/prometheus/

# Set ownership
sudo chown -R prometheus:prometheus /etc/prometheus /var/lib/prometheus

# Create systemd service
sudo tee /etc/systemd/system/prometheus.service > /dev/null <<EOF
[Unit]
Description=Prometheus
Wants=network-online.target
After=network-online.target

[Service]
User=prometheus
Group=prometheus
Type=simple
ExecStart=/usr/local/bin/prometheus \
  --config.file /etc/prometheus/prometheus.yml \
  --storage.tsdb.path /var/lib/prometheus/

[Install]
WantedBy=multi-user.target
EOF

# Start Prometheus
sudo systemctl enable --now prometheus
```

### Configure Netdata Streaming

To collect metrics from multiple hosts:

```bash
# On monitoring server (/etc/netdata/stream.conf)
sudo tee /etc/netdata/stream.conf > /dev/null <<EOF
[API_KEY_HERE]
    enabled = yes
    default history = 3600
    default memory mode = dbengine
EOF

sudo systemctl restart netdata
```

### Verification

```bash
# Check services
sudo systemctl status netdata
sudo systemctl status grafana-server
sudo systemctl status prometheus

# Check ports
sudo ss -tlnp | grep -E "19999|3000|9090"

# Access dashboards
curl http://localhost:19999  # Netdata
curl http://localhost:3000   # Grafana
curl http://localhost:9090   # Prometheus
```

### Dashboard URLs

- **Netdata:** `http://monitor.internal.company.com:19999`
- **Grafana:** `http://monitor.internal.company.com:3000`
- **Prometheus:** `http://monitor.internal.company.com:9090`

---

## 6. 🏢 Multi-Tier Application

> **Scenario:** Deploy a complete multi-tier application (web, app, database) across multiple servers.

### Architecture

```
┌─────────────────┐
│  Load Balancer  │ → Port 80/443
└────────┬────────┘
         │
    ┌────┴────┐
    │         │
┌───▼──┐   ┌──▼───┐
│ Web1 │   │ Web2 │ → Port 8080
└───┬──┘   └──┬───┘
    │         │
    └────┬────┘
         │
    ┌────▼─────┐
    │  App     │ → Port 3000
    └────┬─────┘
         │
    ┌────▼─────┐
    │  DB      │ → Port 5432
    └──────────┘
```

### Server 1 & 2: Web Tier (NGINX)

```bash
#!/bin/bash
# setup-web.sh - Run on web1 and web2

# Essential setup
sudo flux workflow essential

# Hostname (adjust for each server)
sudo flux module hostname --fqdn web1.app.company.com

# Firewall for web tier
sudo flux module firewall --preset web-server
sudo flux module firewall --allow 8080/tcp

# SSH hardening
sudo flux module ssh --port 2222 --disable-passwords --fail2ban

# User
sudo flux module user --admin deploy --github deploybot

# Monitoring
sudo flux module netdata --install

# Install NGINX
sudo apt update
sudo apt install -y nginx

# Configure NGINX as reverse proxy
sudo tee /etc/nginx/sites-available/app > /dev/null <<'EOF'
upstream app_backend {
    server app.app.company.com:3000;
}

server {
    listen 8080;
    server_name _;

    location / {
        proxy_pass http://app_backend;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }
}
EOF

sudo ln -s /etc/nginx/sites-available/app /etc/nginx/sites-enabled/
sudo nginx -t
sudo systemctl reload nginx
```

### Server 3: Application Tier (Node.js)

```bash
#!/bin/bash
# setup-app.sh - Run on app server

# Essential setup
sudo flux workflow essential

# Hostname
sudo flux module hostname --fqdn app.app.company.com

# Firewall (allow only from web tier)
sudo flux module firewall --menu
# Custom rules:
#   Port 3000/tcp, Source: 10.0.1.0/24 (web tier subnet)

# SSH hardening
sudo flux module ssh --port 2222 --disable-passwords

# User
sudo flux module user --admin appuser --github appuser

# Monitoring
sudo flux module netdata --install

# Install Node.js
curl -fsSL https://deb.nodesource.com/setup_18.x | sudo -E bash -
sudo apt install -y nodejs

# Application setup (example)
sudo mkdir -p /opt/app
sudo chown appuser:appuser /opt/app
```

### Server 4: Database Tier (PostgreSQL)

Use the [Database Server Hardening](#2--database-server-hardening) example above.

**Additional:**
```bash
# Allow application server to connect
sudo flux module firewall --menu
# Custom rule:
#   Port 5432/tcp, Source: 10.0.2.0/24 (app tier subnet)
```

### Load Balancer Configuration

```nginx
# /etc/nginx/nginx.conf (on separate load balancer)
upstream web_backend {
    server web1.app.company.com:8080;
    server web2.app.company.com:8080;
}

server {
    listen 80;
    server_name app.company.com;

    location / {
        proxy_pass http://web_backend;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
    }
}
```

---

## 🚀 Advanced Scenarios

### Scenario: Air-Gapped Environment

For environments without internet access:

```bash
# 1. Build on internet-connected machine
git clone https://github.com/ethanbissbort/flux-framework-rust.git
cd flux-framework-rust
cargo build --release

# 2. Package dependencies
cargo vendor

# 3. Transfer to air-gapped system
# Copy: target/release/flux and vendor directory

# 4. Install on air-gapped system
sudo install -m755 flux /usr/local/bin/flux
```

### Scenario: Automated Provisioning with Ansible

```yaml
# playbook.yml
---
- name: Provision servers with Flux
  hosts: all
  become: yes
  tasks:
    - name: Install Flux
      copy:
        src: flux
        dest: /usr/local/bin/flux
        mode: '0755'

    - name: Run essential workflow
      command: flux workflow essential
      args:
        creates: /var/log/flux/essential.done

    - name: Configure firewall
      command: flux module firewall --preset {{ firewall_preset }}

    - name: Harden SSH
      command: flux module ssh --port 2222 --disable-passwords
```

---

## 📋 Automation Scripts

### Complete Production Server Script

```bash
#!/bin/bash
# production-server.sh
# Complete production server setup

set -e

HOSTNAME="${1:-prod-server-01}"
DOMAIN="${2:-example.com}"
GITHUB_USER="${3:-deploybot}"

echo "🚀 Setting up production server: $HOSTNAME.$DOMAIN"

# Essential baseline
echo "📦 Running essential workflow..."
sudo flux workflow essential

# Hostname
echo "🏷️ Setting hostname..."
sudo flux module hostname --fqdn "$HOSTNAME.$DOMAIN"

# Network (assuming DHCP, skip if static IP needed)
echo "🌐 Network configuration (DHCP)..."

# Firewall
echo "🛡️ Configuring firewall..."
sudo flux module firewall --preset web-server
sudo flux module firewall --allow 2222/tcp

# SSH hardening
echo "🔐 Hardening SSH..."
sudo flux module ssh --port 2222 --disable-passwords --fail2ban

# User management
echo "👤 Creating deploy user..."
sudo flux module user --admin deploy --github "$GITHUB_USER"

# ZSH for admin
echo "💻 Installing ZSH..."
sudo flux module zsh --user deploy --theme powerlevel10k --plugins "git,docker,kubectl"

# Monitoring
echo "📊 Installing Netdata..."
sudo flux module netdata --install --disable-telemetry

# MOTD
echo "📋 Setting up MOTD..."
sudo flux module motd --install --org "Production" --banner flux-large

echo "✅ Production server setup complete!"
echo ""
echo "📊 Netdata Dashboard: http://$HOSTNAME.$DOMAIN:19999"
echo "🔐 SSH Access: ssh -p 2222 deploy@$HOSTNAME.$DOMAIN"
echo ""
echo "🔄 Next steps:"
echo "  1. Reboot if kernel updates were installed"
echo "  2. Install application-specific software"
echo "  3. Configure backups"
echo "  4. Update DNS records"
```

**Usage:**
```bash
chmod +x production-server.sh
sudo ./production-server.sh web01 example.com deploybot
```

---

## 📚 Additional Resources

- 📖 [Modules Reference](MODULES.md) - Detailed module options
- 📖 [Workflows Guide](WORKFLOWS.md) - Pre-built workflows
- 📖 [Configuration](CONFIGURATION.md) - Config file format
- 📖 [Installation](INSTALLATION.md) - Setup instructions

---

<div align="center">

**💼 Real-World, Production-Ready Examples**

[GitHub](https://github.com/ethanbissbort/flux-framework-rust) •
[Documentation](../README.md) •
[Get Help](https://github.com/ethanbissbort/flux-framework-rust/issues)

</div>
