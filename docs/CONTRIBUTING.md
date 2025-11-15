# 🤝 Contributing to Flux Framework

> **Welcome! We're excited that you want to contribute to Flux Framework**

---

## 📖 Table of Contents

- [Ways to Contribute](#-ways-to-contribute)
- [Getting Started](#-getting-started)
- [Development Setup](#-development-setup)
- [Code Guidelines](#-code-guidelines)
- [Testing Requirements](#-testing-requirements)
- [Pull Request Process](#-pull-request-process)
- [Issue Guidelines](#-issue-guidelines)
- [Community](#-community)

---

## 💡 Ways to Contribute

There are many ways to contribute to Flux Framework:

### 🐛 Report Bugs
Found a bug? [Open an issue](https://github.com/ethanbissbort/flux-framework-rust/issues/new?template=bug_report.md)

### 💡 Suggest Features
Have an idea? [Start a discussion](https://github.com/ethanbissbort/flux-framework-rust/discussions/new)

### 📝 Improve Documentation
Documentation is never complete:
- Fix typos and errors
- Add examples and use cases
- Improve clarity and organization
- Translate to other languages

### 🔧 Write Code
- Fix bugs
- Implement new features
- Add new modules
- Optimize performance
- Add tests

### 🧪 Test and Review
- Test pre-release versions
- Review pull requests
- Report compatibility issues
- Validate documentation

### 🎨 Design
- Improve CLI output
- Design better error messages
- Create diagrams and visualizations

---

## 🚀 Getting Started

### Prerequisites

Before contributing, ensure you have:

- ✅ **Rust 1.77+** - [Install Rust](https://rustup.rs/)
- ✅ **Git** - Version control
- ✅ **Linux environment** - For testing (VM or container is fine)
- ✅ **GitHub account** - For pull requests

### Quick Start

1. **Fork the repository**
   ```bash
   # Click "Fork" on GitHub, then:
   git clone https://github.com/YOUR_USERNAME/flux-framework-rust.git
   cd flux-framework-rust
   ```

2. **Add upstream remote**
   ```bash
   git remote add upstream https://github.com/ethanbissbort/flux-framework-rust.git
   ```

3. **Create a branch**
   ```bash
   git checkout -b feature/amazing-feature
   ```

4. **Make your changes**
   ```bash
   # Edit files, add features, fix bugs
   ```

5. **Test your changes**
   ```bash
   cargo test
   cargo clippy
   cargo fmt
   ```

6. **Commit and push**
   ```bash
   git add .
   git commit -s -m "Add amazing feature"
   git push origin feature/amazing-feature
   ```

7. **Open a Pull Request**
   - Go to GitHub
   - Click "New Pull Request"
   - Fill in the template
   - Submit!

---

## 💻 Development Setup

### Initial Setup

```bash
# Clone your fork
git clone https://github.com/YOUR_USERNAME/flux-framework-rust.git
cd flux-framework-rust

# Add upstream
git remote add upstream https://github.com/ethanbissbort/flux-framework-rust.git

# Install development tools
rustup component add rustfmt clippy

# Build in debug mode
cargo build

# Run tests
cargo test
```

### Project Structure

```
flux-framework-rust/
├── src/
│   ├── main.rs              # CLI entry point
│   ├── lib.rs               # Library root
│   ├── cli.rs               # CLI implementation
│   ├── config.rs            # Configuration handling
│   ├── error.rs             # Error types
│   ├── modules/             # All modules
│   │   ├── mod.rs           # Module registry
│   │   ├── ssh.rs           # SSH module
│   │   ├── firewall.rs      # Firewall module
│   │   └── ...              # Other modules
│   ├── workflows/           # Workflow definitions
│   │   ├── mod.rs           # Workflow manager
│   │   ├── essential.rs     # Essential workflow
│   │   └── ...              # Other workflows
│   └── helpers/             # Helper functions
│       ├── logging.rs       # Logging utilities
│       ├── system.rs        # System detection
│       └── ...              # Other helpers
├── docs/                    # Documentation
├── tests/                   # Integration tests
├── Cargo.toml               # Dependencies
└── README.md                # Main README
```

### Development Commands

```bash
# Build in debug mode (faster compilation)
cargo build

# Build in release mode (optimized)
cargo build --release

# Run Flux in debug mode
cargo run -- --help

# Run specific module
sudo cargo run -- module ssh --help

# Run tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_ssh_module

# Check code without building
cargo check

# Format code
cargo fmt

# Lint code
cargo clippy

# Fix clippy warnings automatically
cargo clippy --fix

# Generate documentation
cargo doc --open
```

---

## 📋 Code Guidelines

### Rust Style Guide

We follow the [Rust Style Guide](https://doc.rust-lang.org/1.0.0/style/) with some additional conventions:

#### General Principles

- ✅ **Write idiomatic Rust** - Use Rust patterns and conventions
- ✅ **Keep it simple** - Prefer clarity over cleverness
- ✅ **Document public APIs** - All public items need documentation
- ✅ **Error handling** - Use proper error types, avoid `.unwrap()` in production code
- ✅ **Safety first** - Avoid `unsafe` code unless absolutely necessary

#### Code Style

**Formatting:**
```bash
# Always run before committing
cargo fmt
```

**Naming Conventions:**
```rust
// Types: PascalCase
struct SshModule { }
enum FirewallType { }

// Functions and variables: snake_case
fn execute_module() { }
let user_name = "alice";

// Constants: SCREAMING_SNAKE_CASE
const MAX_RETRIES: u32 = 3;
const DEFAULT_PORT: u16 = 22;

// Module files: snake_case
// src/modules/ssh.rs
// src/helpers/user_input.rs
```

**Documentation:**
```rust
/// Execute SSH hardening workflow
///
/// This function applies comprehensive SSH hardening including:
/// - Port configuration
/// - Cipher suite updates
/// - Authentication method restrictions
///
/// # Arguments
///
/// * `port` - SSH port number (1024-65535)
///
/// # Returns
///
/// * `Result<()>` - Success or error
///
/// # Examples
///
/// ```
/// harden_ssh(2222)?;
/// ```
pub async fn harden_ssh(port: u16) -> Result<()> {
    // Implementation
}
```

**Error Handling:**
```rust
// Good: Propagate errors
pub fn read_config() -> Result<Config> {
    let content = fs::read_to_string(path)?;
    Ok(parse_config(&content)?)
}

// Bad: Unwrap in library code
pub fn read_config() -> Config {
    let content = fs::read_to_string(path).unwrap(); // Don't do this!
    parse_config(&content).unwrap()
}

// Good: Use meaningful error messages
if port < 1024 || port > 65535 {
    return Err(FluxError::validation(
        format!("Invalid port {}. Must be between 1024-65535", port)
    ));
}
```

**Async/Await:**
```rust
// Use async/await for I/O operations
pub async fn execute_command(cmd: &str) -> Result<String> {
    let output = tokio::process::Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .output()
        .await?;

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}
```

### Module Development

When creating a new module:

1. **Create module file:** `src/modules/mymodule.rs`

2. **Implement Module trait:**
```rust
use crate::modules::{Module, ModuleBase, ModuleInfo};
use async_trait::async_trait;

pub struct MyModule {
    base: ModuleBase,
}

impl MyModule {
    pub fn new() -> Self {
        let info = ModuleInfo {
            name: "mymodule".to_string(),
            description: "Description of my module".to_string(),
            version: "1.0.0".to_string(),
            author: "Your Name".to_string(),
            tags: vec!["category".to_string()],
            requires_root: true,
            supported_distros: vec!["all".to_string()],
        };
        Self {
            base: ModuleBase { info },
        }
    }
}

#[async_trait]
impl Module for MyModule {
    fn name(&self) -> &str {
        &self.base.info.name
    }

    fn description(&self) -> &str {
        &self.base.info.description
    }

    fn version(&self) -> &str {
        &self.base.info.version
    }

    fn is_available(&self) -> bool {
        // Check if module can run on this system
        true
    }

    fn help(&self) -> String {
        format!(r#"My Module v{}

DESCRIPTION:
    {}

USAGE:
    flux module {} [OPTIONS]

OPTIONS:
    --option1    Description
    --option2    Description
"#, self.version(), self.description(), self.name())
    }

    async fn execute(&self, args: Vec<String>, config: &Config) -> Result<()> {
        // Implementation
        Ok(())
    }
}
```

3. **Register in mod.rs:**
```rust
// src/modules/mod.rs
pub mod mymodule;

// In ModuleManager::new()
let all_modules: Vec<Box<dyn Module>> = vec![
    // ... existing modules
    Box::new(mymodule::MyModule::new()),
];
```

---

## 🧪 Testing Requirements

### Unit Tests

All new code should include unit tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_port() {
        assert!(validate_port(2222).is_ok());
        assert!(validate_port(22).is_ok());
        assert!(validate_port(1023).is_err());
        assert!(validate_port(65536).is_err());
    }

    #[tokio::test]
    async fn test_module_execution() {
        let module = MyModule::new();
        let config = Config::default();
        let result = module.execute(vec![], &config).await;
        assert!(result.is_ok());
    }
}
```

### Integration Tests

For end-to-end testing:

```rust
// tests/integration_test.rs
use assert_cmd::Command;

#[test]
fn test_cli_help() {
    let mut cmd = Command::cargo_bin("flux").unwrap();
    cmd.arg("--help");
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("Flux System Administration Framework"));
}
```

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_validate_port

# Run with output
cargo test -- --nocapture

# Run integration tests only
cargo test --test '*'

# Run with coverage (requires cargo-tarpaulin)
cargo tarpaulin --out Html
```

### Test Checklist

Before submitting a PR, ensure:

- ✅ All existing tests pass
- ✅ New features have tests
- ✅ Bug fixes have regression tests
- ✅ Code coverage is maintained or improved
- ✅ Tests pass on multiple distributions (if possible)

---

## 🔄 Pull Request Process

### Before Opening a PR

1. **Update from upstream:**
   ```bash
   git fetch upstream
   git rebase upstream/main
   ```

2. **Run quality checks:**
   ```bash
   # Format code
   cargo fmt

   # Check for issues
   cargo clippy

   # Run tests
   cargo test

   # Check documentation
   cargo doc
   ```

3. **Commit with DCO sign-off:**
   ```bash
   git commit -s -m "Your descriptive commit message"
   ```

### PR Title and Description

**Good PR title:**
```
Add fail2ban integration to SSH module
```

**Bad PR title:**
```
Update ssh.rs
```

**PR Description Template:**
```markdown
## Description
Brief description of what this PR does.

## Motivation
Why is this change needed?

## Changes
- Change 1
- Change 2
- Change 3

## Testing
How was this tested?
- [ ] Unit tests added
- [ ] Integration tests added
- [ ] Manually tested on Ubuntu 22.04
- [ ] Manually tested on Rocky Linux 9

## Checklist
- [ ] Code follows style guidelines
- [ ] Documentation updated
- [ ] Tests added/updated
- [ ] All tests pass
- [ ] Commit messages are descriptive
- [ ] DCO sign-off included
```

### Review Process

1. **Automated checks** - CI runs tests, linting
2. **Maintainer review** - Code review by maintainers
3. **Revisions** - Address feedback
4. **Approval** - PR gets approved
5. **Merge** - PR is merged to main

### After Your PR is Merged

1. **Update your fork:**
   ```bash
   git checkout main
   git pull upstream main
   git push origin main
   ```

2. **Delete your branch:**
   ```bash
   git branch -d feature/amazing-feature
   git push origin --delete feature/amazing-feature
   ```

---

## 🐛 Issue Guidelines

### Reporting Bugs

Use the [bug report template](https://github.com/ethanbissbort/flux-framework-rust/issues/new?template=bug_report.md):

**Include:**
- ✅ Flux version (`flux --version`)
- ✅ Operating system and version (`cat /etc/os-release`)
- ✅ Steps to reproduce
- ✅ Expected behavior
- ✅ Actual behavior
- ✅ Relevant logs/error messages

**Example:**
```markdown
## Bug Description
SSH module fails when changing port to 2222

## Environment
- Flux version: 3.0.0
- OS: Ubuntu 22.04 LTS
- Kernel: 5.15.0-76-generic

## Steps to Reproduce
1. Run `sudo flux module ssh --port 2222`
2. Error occurs

## Expected Behavior
SSH port should be changed to 2222

## Actual Behavior
Error: "Port 2222 already in use"

## Logs
```
[ERROR] Failed to change SSH port: Port 2222 is already bound
```

## Additional Context
Port 2222 is not actually in use according to `ss -tlnp`
```

### Feature Requests

Use the [feature request template](https://github.com/ethanbissbort/flux-framework-rust/discussions/new):

**Include:**
- ✅ Problem statement
- ✅ Proposed solution
- ✅ Alternatives considered
- ✅ Use cases

---

## 👥 Community

### Communication Channels

- 💬 **Discussions** - [GitHub Discussions](https://github.com/ethanbissbort/flux-framework-rust/discussions)
- 🐛 **Issues** - [GitHub Issues](https://github.com/ethanbissbort/flux-framework-rust/issues)
- 📧 **Email** - flux-framework@example.com

### Code of Conduct

We are committed to providing a welcoming and inclusive environment. Please:

- ✅ Be respectful and professional
- ✅ Welcome newcomers
- ✅ Be patient and helpful
- ✅ Assume good faith
- ❌ No harassment or discrimination
- ❌ No trolling or inflammatory comments

### Recognition

Contributors are recognized in:
- README.md contributors section
- Release notes
- GitHub contributors page

---

## 📚 Additional Resources

### Learning Rust

- [The Rust Book](https://doc.rust-lang.org/book/)
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/)
- [Rustlings](https://github.com/rust-lang/rustlings)

### Project Documentation

- 📖 [Architecture Guide](ARCHITECTURE.md)
- 📖 [Module Development](MODULES.md)
- 📖 [Testing Guide](../tests/README.md)

### Git Best Practices

- [Conventional Commits](https://www.conventionalcommits.org/)
- [Git Branching Model](https://nvie.com/posts/a-successful-git-branching-model/)

---

## 🎯 Good First Issues

Looking for a place to start? Check out:

- [Good First Issues](https://github.com/ethanbissbort/flux-framework-rust/labels/good%20first%20issue)
- [Documentation Issues](https://github.com/ethanbissbort/flux-framework-rust/labels/documentation)
- [Help Wanted](https://github.com/ethanbissbort/flux-framework-rust/labels/help%20wanted)

---

## 📝 License

By contributing to Flux Framework, you agree that your contributions will be licensed under the project's dual license (MIT / Apache 2.0).

---

<div align="center">

**🙏 Thank You for Contributing!**

Every contribution, no matter how small, makes Flux better for everyone.

[Start Contributing](https://github.com/ethanbissbort/flux-framework-rust) •
[Report Bugs](https://github.com/ethanbissbort/flux-framework-rust/issues) •
[Suggest Features](https://github.com/ethanbissbort/flux-framework-rust/discussions)

</div>
