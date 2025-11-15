// src/modules/zsh.rs
// ZSH and Oh-My-Zsh installation and configuration module

use crate::config::Config;
use crate::error::{FluxError, Result};
use crate::helpers::{
    file_ops::safe_write_file,
    logging::{log_debug, log_error, log_info, log_success, log_warn},
    system::{check_command, execute_command, get_os_info},
    user_input::{prompt_input, prompt_yes_no, select_from_menu},
};
use crate::modules::{Module, ModuleBase, ModuleInfo};
use async_trait::async_trait;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use users::{get_user_by_name};
use users::os::unix::UserExt;

const OH_MY_ZSH_INSTALL_URL: &str = "https://raw.githubusercontent.com/ohmyzsh/ohmyzsh/master/tools/install.sh";

pub struct ZshModule {
    base: ModuleBase,
}

impl ZshModule {
    pub fn new() -> Self {
        let info = ModuleInfo {
            name: "zsh".to_string(),
            description: "ZSH and Oh-My-Zsh installation and configuration".to_string(),
            version: "1.0.0".to_string(),
            author: "Flux Contributors".to_string(),
            tags: vec!["shell".to_string(), "ux".to_string()],
            requires_root: true,
            supported_distros: vec!["all".to_string()],
        };
        Self {
            base: ModuleBase { info },
        }
    }

    /// Install ZSH package
    async fn install_zsh(&self) -> Result<()> {
        if check_command("zsh").is_ok() {
            log_info("ZSH is already installed");
            return Ok(());
        }

        log_info("Installing ZSH");
        let distro = crate::helpers::system::detect_distro()?;

        if distro.is_debian_based() {
            execute_command("apt-get", &["update"])?;
            execute_command("apt-get", &["install", "-y", "zsh", "curl", "git"])?;
        } else if distro.is_redhat_based() {
            execute_command("yum", &["install", "-y", "zsh", "curl", "git"])?;
        } else {
            return Err(FluxError::Module(
                "Unsupported distribution for ZSH installation".to_string()
            ));
        }

        log_success("ZSH installed successfully");
        Ok(())
    }

    /// Install Oh-My-Zsh for a user
    async fn install_oh_my_zsh(&self, username: &str) -> Result<()> {
        log_info(&format!("Installing Oh-My-Zsh for user: {}", username));

        let user = get_user_by_name(username)
            .ok_or_else(|| FluxError::Module(format!("User '{}' not found", username)))?;

        let home_dir = PathBuf::from(user.home_dir().to_str().unwrap());
        let oh_my_zsh_dir = home_dir.join(".oh-my-zsh");

        // Check if Oh-My-Zsh is already installed
        if oh_my_zsh_dir.exists() {
            log_info("Oh-My-Zsh is already installed");
            return Ok(());
        }

        // Download and install Oh-My-Zsh
        log_info("Downloading Oh-My-Zsh installer");

        let output = Command::new("su")
            .arg("-")
            .arg(username)
            .arg("-c")
            .arg(format!(
                "sh -c \"$(curl -fsSL {})\" \"\" --unattended",
                OH_MY_ZSH_INSTALL_URL
            ))
            .env("RUNZSH", "no")
            .env("CHSH", "no")
            .output()
            .map_err(|e| FluxError::command_failed(format!("Failed to install Oh-My-Zsh: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            log_warn(&format!("Oh-My-Zsh installation warning: {}", stderr));
        }

        log_success("Oh-My-Zsh installed successfully");
        Ok(())
    }

    /// Install ZSH plugins
    async fn install_plugins(&self, username: &str, plugins: Vec<&str>) -> Result<()> {
        log_info("Installing ZSH plugins");

        let user = get_user_by_name(username)
            .ok_or_else(|| FluxError::Module(format!("User '{}' not found", username)))?;

        let home_dir = PathBuf::from(user.home_dir().to_str().unwrap());
        let custom_plugins_dir = home_dir.join(".oh-my-zsh/custom/plugins");

        for plugin in plugins {
            match plugin {
                "zsh-autosuggestions" => {
                    let plugin_dir = custom_plugins_dir.join("zsh-autosuggestions");
                    if !plugin_dir.exists() {
                        log_info("Installing zsh-autosuggestions");
                        execute_command(
                            "git",
                            &[
                                "clone",
                                "https://github.com/zsh-users/zsh-autosuggestions",
                                plugin_dir.to_str().unwrap(),
                            ],
                        )?;
                    }
                }
                "zsh-syntax-highlighting" => {
                    let plugin_dir = custom_plugins_dir.join("zsh-syntax-highlighting");
                    if !plugin_dir.exists() {
                        log_info("Installing zsh-syntax-highlighting");
                        execute_command(
                            "git",
                            &[
                                "clone",
                                "https://github.com/zsh-users/zsh-syntax-highlighting.git",
                                plugin_dir.to_str().unwrap(),
                            ],
                        )?;
                    }
                }
                "zsh-completions" => {
                    let plugin_dir = custom_plugins_dir.join("zsh-completions");
                    if !plugin_dir.exists() {
                        log_info("Installing zsh-completions");
                        execute_command(
                            "git",
                            &[
                                "clone",
                                "https://github.com/zsh-users/zsh-completions",
                                plugin_dir.to_str().unwrap(),
                            ],
                        )?;
                    }
                }
                _ => {
                    log_debug(&format!("Plugin '{}' is built-in, skipping", plugin));
                }
            }
        }

        // Fix ownership
        let uid = user.uid();
        let gid = user.primary_group_id();
        Command::new("chown")
            .arg("-R")
            .arg(format!("{}:{}", uid, gid))
            .arg(&custom_plugins_dir)
            .output()
            .ok();

        log_success("ZSH plugins installed");
        Ok(())
    }

    /// Install Powerlevel10k theme
    async fn install_powerlevel10k(&self, username: &str) -> Result<()> {
        log_info("Installing Powerlevel10k theme");

        let user = get_user_by_name(username)
            .ok_or_else(|| FluxError::Module(format!("User '{}' not found", username)))?;

        let home_dir = PathBuf::from(user.home_dir().to_str().unwrap());
        let theme_dir = home_dir.join(".oh-my-zsh/custom/themes/powerlevel10k");

        if !theme_dir.exists() {
            execute_command(
                "git",
                &[
                    "clone",
                    "--depth=1",
                    "https://github.com/romkatv/powerlevel10k.git",
                    theme_dir.to_str().unwrap(),
                ],
            )?;

            // Fix ownership
            let uid = user.uid();
            let gid = user.primary_group_id();
            Command::new("chown")
                .arg("-R")
                .arg(format!("{}:{}", uid, gid))
                .arg(&theme_dir)
                .output()
                .ok();

            log_success("Powerlevel10k installed");
        } else {
            log_info("Powerlevel10k already installed");
        }

        Ok(())
    }

    /// Generate .zshrc configuration
    fn generate_zshrc(&self, theme: &str, plugins: Vec<&str>) -> String {
        let plugin_list = plugins.join(" ");

        format!(
            r#"# Flux Framework - ZSH Configuration
# Path to Oh-My-Zsh installation
export ZSH="$HOME/.oh-my-zsh"

# Set theme
ZSH_THEME="{theme}"

# Plugins
plugins=({plugins})

# Load Oh-My-Zsh
source $ZSH/oh-my-zsh.sh

# User configuration

# History settings
HISTSIZE=10000
SAVEHIST=10000
setopt HIST_IGNORE_DUPS
setopt HIST_FIND_NO_DUPS
setopt SHARE_HISTORY

# Aliases
alias ll='ls -lah'
alias la='ls -A'
alias l='ls -CF'
alias ..='cd ..'
alias ...='cd ../..'
alias grep='grep --color=auto'
alias update='sudo apt update && sudo apt upgrade -y'
alias ports='netstat -tulanp'
alias meminfo='free -m -l -t'
alias psg='ps aux | grep -v grep | grep -i -e VSZ -e'

# Git aliases
alias gs='git status'
alias ga='git add'
alias gc='git commit'
alias gp='git push'
alias gl='git log --oneline --graph --decorate'

# Functions
mkcd() {{
    mkdir -p "$1" && cd "$1"
}}

extract() {{
    if [ -f $1 ]; then
        case $1 in
            *.tar.bz2)   tar xjf $1     ;;
            *.tar.gz)    tar xzf $1     ;;
            *.bz2)       bunzip2 $1     ;;
            *.rar)       unrar e $1     ;;
            *.gz)        gunzip $1      ;;
            *.tar)       tar xf $1      ;;
            *.tbz2)      tar xjf $1     ;;
            *.tgz)       tar xzf $1     ;;
            *.zip)       unzip $1       ;;
            *.Z)         uncompress $1  ;;
            *.7z)        7z x $1        ;;
            *)           echo "'$1' cannot be extracted via extract()" ;;
        esac
    else
        echo "'$1' is not a valid file"
    fi
}}

# Flux Framework shortcuts
alias flux-update='sudo flux module update --menu'
alias flux-config='sudo flux config'
alias flux-modules='sudo flux module list'

# Enable command correction
setopt CORRECT

# Enable advanced globbing
setopt EXTENDED_GLOB

# Case-insensitive completion
zstyle ':completion:*' matcher-list 'm:{{a-zA-Z}}={{A-Za-z}}'

# Colored completion
zstyle ':completion:*' list-colors "${{(s.:.)LS_COLORS}}"

# Load custom configuration
if [ -f ~/.zshrc.local ]; then
    source ~/.zshrc.local
fi
"#,
            theme = theme,
            plugins = plugin_list
        )
    }

    /// Configure ZSH for a user
    async fn configure_zsh(&self, username: &str, theme: &str) -> Result<()> {
        log_info(&format!("Configuring ZSH for user: {}", username));

        let user = get_user_by_name(username)
            .ok_or_else(|| FluxError::Module(format!("User '{}' not found", username)))?;

        let home_dir = PathBuf::from(user.home_dir().to_str().unwrap());
        let zshrc_path = home_dir.join(".zshrc");

        // Backup existing .zshrc
        if zshrc_path.exists() {
            let backup_path = home_dir.join(".zshrc.backup");
            fs::copy(&zshrc_path, &backup_path)?;
            log_info("Backed up existing .zshrc");
        }

        // Determine plugins
        let plugins = vec![
            "git",
            "docker",
            "kubectl",
            "sudo",
            "zsh-autosuggestions",
            "zsh-syntax-highlighting",
        ];

        // Generate and write .zshrc
        let zshrc_content = self.generate_zshrc(theme, plugins);
        safe_write_file(zshrc_path.to_str().unwrap(), &zshrc_content, true)?;

        // Fix ownership
        let uid = user.uid();
        let gid = user.primary_group_id();
        Command::new("chown")
            .arg(format!("{}:{}", uid, gid))
            .arg(&zshrc_path)
            .output()
            .ok();

        log_success("ZSH configured successfully");
        Ok(())
    }

    /// Set ZSH as default shell
    async fn set_default_shell(&self, username: &str) -> Result<()> {
        log_info(&format!("Setting ZSH as default shell for: {}", username));

        // Get ZSH path
        let output = Command::new("which")
            .arg("zsh")
            .output()
            .map_err(|e| FluxError::command_failed(format!("Failed to find zsh: {}", e)))?;

        let zsh_path = String::from_utf8_lossy(&output.stdout).trim().to_string();

        if zsh_path.is_empty() {
            return Err(FluxError::Module("ZSH not found".to_string()));
        }

        // Ensure ZSH is in /etc/shells
        let shells_content = fs::read_to_string("/etc/shells").unwrap_or_default();
        if !shells_content.contains(&zsh_path) {
            log_info("Adding ZSH to /etc/shells");
            let mut shells = shells_content;
            shells.push_str(&format!("\n{}\n", zsh_path));
            safe_write_file("/etc/shells", &shells, true)?;
        }

        // Change user shell
        execute_command("chsh", &["-s", &zsh_path, username])?;

        log_success(&format!("ZSH set as default shell for {}", username));
        log_info("Logout and login again to use ZSH");

        Ok(())
    }

    /// Full ZSH setup for a user
    async fn full_setup(&self, username: &str, theme: &str, set_default: bool) -> Result<()> {
        log_info(&format!("Starting full ZSH setup for: {}", username));

        // Install ZSH
        self.install_zsh().await?;

        // Install Oh-My-Zsh
        self.install_oh_my_zsh(username).await?;

        // Install plugins
        let plugins = vec!["zsh-autosuggestions", "zsh-syntax-highlighting"];
        self.install_plugins(username, plugins).await?;

        // Install theme if Powerlevel10k
        if theme == "powerlevel10k/powerlevel10k" {
            self.install_powerlevel10k(username).await?;
        }

        // Configure ZSH
        self.configure_zsh(username, theme).await?;

        // Set as default shell
        if set_default {
            self.set_default_shell(username).await?;
        }

        log_success("ZSH setup completed successfully!");
        Ok(())
    }

    /// Show interactive menu
    async fn show_menu(&self) -> Result<()> {
        loop {
            let options = vec![
                "Full ZSH setup (recommended)",
                "Install ZSH only",
                "Install Oh-My-Zsh",
                "Install plugins",
                "Install Powerlevel10k theme",
                "Set ZSH as default shell",
                "Exit",
            ];

            let choice = select_from_menu("ZSH Management", &options)?;

            match choice {
                0 => {
                    // Full setup
                    let username = prompt_input("Enter username")?;
                    let themes = vec!["robbyrussell", "agnoster", "powerlevel10k/powerlevel10k"];
                    let theme_choice = select_from_menu("Select theme", &themes)?;
                    let set_default = prompt_yes_no("Set ZSH as default shell?", true)?;

                    if let Err(e) = self.full_setup(&username, themes[theme_choice], set_default).await {
                        log_error(&format!("Setup failed: {}", e));
                    }
                }
                1 => {
                    // Install ZSH only
                    self.install_zsh().await?;
                }
                2 => {
                    // Install Oh-My-Zsh
                    let username = prompt_input("Enter username")?;
                    self.install_oh_my_zsh(&username).await?;
                }
                3 => {
                    // Install plugins
                    let username = prompt_input("Enter username")?;
                    let plugins = vec!["zsh-autosuggestions", "zsh-syntax-highlighting"];
                    self.install_plugins(&username, plugins).await?;
                }
                4 => {
                    // Install Powerlevel10k
                    let username = prompt_input("Enter username")?;
                    self.install_powerlevel10k(&username).await?;
                }
                5 => {
                    // Set default shell
                    let username = prompt_input("Enter username")?;
                    self.set_default_shell(&username).await?;
                }
                6 => {
                    log_info("Exiting ZSH management");
                    break;
                }
                _ => {}
            }
        }

        Ok(())
    }
}

#[async_trait]
impl Module for ZshModule {
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
        check_command("curl").is_ok() && check_command("git").is_ok()
    }

    fn help(&self) -> String {
        format!(
            r#"ZSH Setup Module v{}

DESCRIPTION:
    {}

    This module installs and configures ZSH shell with Oh-My-Zsh framework,
    plugins, themes, and optimizations for an enhanced command-line experience.

USAGE:
    flux module {} [OPTIONS]

OPTIONS:
    --setup <user>               Full ZSH setup for user
    --theme <theme>              Specify theme (default: robbyrussell)
    --install                    Install ZSH package only
    --install-omz <user>         Install Oh-My-Zsh for user
    --set-default <user>         Set ZSH as default shell
    --menu                       Show interactive menu

THEMES:
    robbyrussell                 Default Oh-My-Zsh theme
    agnoster                     Powerline-inspired theme
    powerlevel10k                Advanced customizable theme

PLUGINS:
    git                          Git aliases and functions
    docker                       Docker completions
    kubectl                      Kubernetes completions
    zsh-autosuggestions         Fish-like autosuggestions
    zsh-syntax-highlighting     Command syntax highlighting

EXAMPLES:
    flux module {} --menu
    flux module {} --setup john
    flux module {} --setup alice --theme powerlevel10k
    flux module {} --install-omz bob
    flux module {} --set-default charlie
"#,
            self.version(),
            self.description(),
            self.name(),
            self.name(),
            self.name(),
            self.name(),
            self.name(),
            self.name()
        )
    }

    async fn execute(&self, args: Vec<String>, _config: &Config) -> Result<()> {
        if args.is_empty() || args.contains(&"--menu".to_string()) {
            return self.show_menu().await;
        }

        let mut i = 0;
        while i < args.len() {
            match args[i].as_str() {
                "--setup" => {
                    if i + 1 < args.len() {
                        let username = &args[i + 1];
                        let theme = if args.contains(&"--theme".to_string()) {
                            let theme_idx = args.iter().position(|s| s == "--theme").unwrap();
                            if theme_idx + 1 < args.len() {
                                &args[theme_idx + 1]
                            } else {
                                "robbyrussell"
                            }
                        } else {
                            "robbyrussell"
                        };

                        self.full_setup(username, theme, true).await?;
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                "--install" => {
                    self.install_zsh().await?;
                    i += 1;
                }
                "--install-omz" => {
                    if i + 1 < args.len() {
                        self.install_oh_my_zsh(&args[i + 1]).await?;
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                "--set-default" => {
                    if i + 1 < args.len() {
                        self.set_default_shell(&args[i + 1]).await?;
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                "--theme" => {
                    // Handled in --setup
                    i += 2;
                }
                _ => {
                    i += 1;
                }
            }
        }

        Ok(())
    }
}
