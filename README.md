# Flux Framework (Rust)

> **Modern, declarative server provisioning & hardening toolkit written in Rust**

![CI](https://github.com/yourorg/flux-framework-rust/workflows/CI/badge.svg)
![License](https://img.shields.io/github/license/yourorg/flux-framework-rust)
![Crates.io](https://img.shields.io/crates/v/flux-framework)

---

## ✨ Key Features

| Domain              | Highlights                                                                    |
| ------------------- | ----------------------------------------------------------------------------- |
| **Cross‑distro**    | Debian / Ubuntu, Alma / Rocky / RHEL, Fedora (others coming)                  |
| **Declarative**     | Single `flux.toml` drives packages, users, firewall, SSH, sysctl, MOTD & more |
| **Safe by default** | Dry‑run & interactive modes, idempotent operations, automatic rollback hints  |
| **Secure**          | Opinionated hardening (nftables/ufw, kernel sysctl, SSH key‑only auth)        |
| **Extensible**      | Plug‑in architecture (`Module`, `Workflow` traits) + async runtime            |
| **Tiny footprint**  | Pure‑Rust binary (★ no Python ★) < 5 MB static release                        |

---

## 🗺️ Project Layout

```
flux-framework-rust/
├── Cargo.toml           # crate manifest
├── src/
│   ├── main.rs          # CLI entrypoint
│   ├── cli.rs           # Clap‑based argument parser
│   ├── config.rs        # Configuration loader/validator
│   ├── error.rs         # Unified error type
│   ├── helpers/         # Cross‑cutting utilities
│   ├── modules/         # Provisioning primitives (pkg update, user, ssh…)
│   └── workflows/       # High‑level orchestration (essential, security…)
└── config/
    └── default.toml     # Sample configuration (copy & edit)
```

---

## 🚀 Getting Started

### 1 — Prerequisites

* **Rust ≥ 1.77** (for contributors) – install with `rustup`.
* Target server needs:

  * x86‑64 / aarch64 Linux
  * `sudo` or root privileges
  * Package manager (`apt` / `dnf` / `yum` / `apk` soon)

### 2 — Install Binary

```bash
# From crates.io (recommended)
cargo install flux-framework

# Or clone + build
git clone https://github.com/yourorg/flux-framework-rust.git
cd flux-framework-rust
cargo build --release
sudo install -m755 target/release/flux /usr/local/bin/flux
```

### 3 — Configure

```bash
sudo mkdir -p /etc/flux
sudo cp config/default.toml /etc/flux/flux.toml
sudoedit /etc/flux/flux.toml   # tweak settings
```

> **Tip:** To run non‑interactively, set `[global].mode = "auto"`.

### 4 — Run

```bash
# Dry‑run, show planned actions
sudo flux plan

# Apply essential baseline (updates, hostname, network)
sudo flux apply essential

# Harden security stack
sudo flux apply security
```

All operations stream coloured logs and create a JSON execution report in `/var/log/flux/`. Use `flux --help` for the full CLI.

---

## ⚙️ Configuration Reference

### Minimal Example

```toml
[global]
mode = "auto"

[hostname]
set_hostname = "web‑01"

[update]
auto_security_updates = true
```

### Full Schema

The sample [`config/default.toml`](config/default.toml) is exhaustive—every key is documented in comments (see annotated version in docs/). Parameters map 1‑to‑1 to module fields.

| Section    | Purpose                                  |
| ---------- | ---------------------------------------- |
| `global`   | run‑mode, logging level                  |
| `update`   | OS package updates + unattended‑upgrades |
| `network`  | DNS, static IP, VLANs                    |
| `hostname` | hostname / FQDN                          |
| `user`     | create user, sudo rights, SSH keys       |
| `ssh`      | port, root login, password auth          |
| `firewall` | ufw / firewalld / nftables rules         |
| `sysctl`   | kernel hardening knobs                   |
| `certs`    | LetsEncrypt via acme.sh                  |
| `zsh`      | Oh‑My‑Zsh & theme                        |
| `motd`     | dynamic MOTD banner                      |
| `netdata`  | install Netdata monitoring agent         |

---

## 🧩 Modules

> Use `flux list modules` to inspect availability on the current host.

| Module     | Status | Summary                          |
| ---------- | ------ | -------------------------------- |
| `update`   | ✅      | OS updates, reboot whisperer     |
| `network`  | ✅      | DNS / static IP / VLANs          |
| `hostname` | ✅      | Hostname & /etc/hosts            |
| `user`     | ⏳      | Create user + authorized\_keys   |
| `ssh`      | ⏳      | Key‑only auth, hardening options |
| `firewall` | ⏳      | ufw / firewalld presets          |
| `sysctl`   | ⏳      | Kernel CIS‑style tweaks          |
| `certs`    | ⏳      | LetsEncrypt automation           |
| `zsh`      | ⏳      | Oh‑My‑Zsh + Powerlevel10k        |
| `motd`     | ⏳      | Pretty login banner              |
| `netdata`  | ⏳      | Monitoring agent                 |

Legend: ✅ implemented • ⏳ in progress

---

## 🔗 Workflows

| Workflow      | Modules Executed              | Use‑case         |
| ------------- | ----------------------------- | ---------------- |
| `essential`   | update → hostname → network   | clean base image |
| `security`    | firewall → ssh → sysctl       | hardening pass   |
| `complete`    | essential + security + extras | full stack       |
| `development` | user → zsh → certs            | dev workstation  |
| `monitoring`  | netdata + certs               | metrics node     |

Workflows guarantee ordering and stop on first fatal error by default.

---

## 🛠️ Development

```bash
# Run lints & unit tests
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test

# Integration (inside container)
just test-docker   # see justfile
```

### Contribution Guide

1. Fork & branch from `main`.
2. Write code + tests (unit or mocked).
3. Run CI locally (`just ci`).
4. Open PR—link to an open issue or create one.

All code is licensed MIT‑OR‑Apache‑2.0. All contributions require DCO sign‑off (`git commit -s`).

---

## 🛤️ Roadmap

* **0.3** – Core hardening modules complete (ssh, firewall, sysctl)
* **0.4** – Monitoring & TLS automation
* **0.5** – Alpine & Arch support
* **1.0** – Stable API, plugin SDK & binary releases

Track progress in [`docs/ROADMAP.md`](docs/ROADMAP.md).

---

## 🤝 Acknowledgements

Flux stands on the shoulders of:

* [tokio](https://tokio.rs/) – async runtime
* [clap](https://github.com/clap-rs/clap) – CLI
* [serde](https://serde.rs/) – TOML deserialization
* CIS Benchmarks & Mozilla SSH Guidelines for hardening profiles

---

## 📜 License

Dual‑licensed under **Apache‑2.0** or **MIT** – choose either for your project.
See [`LICENSE-APACHE`](LICENSE-APACHE) and [`LICENSE-MIT`](LICENSE-MIT).

---

*© 2025 Flux Contributors*
