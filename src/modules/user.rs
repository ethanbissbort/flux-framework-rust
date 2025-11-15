// src/modules/user.rs
// User and group management module

use crate::config::Config;
use crate::error::{FluxError, Result};
use crate::helpers::{
    file_ops::{backup_file, safe_write_file},
    logging::{log_debug, log_error, log_info, log_success, log_warn},
    system::{check_command, execute_command, get_os_info},
    user_input::{multi_select_menu, prompt_input, prompt_password, prompt_with_default, prompt_yes_no, select_from_menu},
    validation::validate_username,
};
use crate::modules::{Module, ModuleBase, ModuleInfo};
use async_trait::async_trait;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use users::{get_user_by_name, get_group_by_name};
use users::os::unix::UserExt;

pub struct UserModule {
    base: ModuleBase,
}

impl UserModule {
    pub fn new() -> Self {
        let info = ModuleInfo {
            name: "user".to_string(),
            description: "Local user and group management".to_string(),
            version: "1.0.0".to_string(),
            author: "Flux Contributors".to_string(),
            tags: vec!["core".to_string(), "security".to_string()],
            requires_root: true,
            supported_distros: vec!["all".to_string()],
        };
        Self {
            base: ModuleBase { info },
        }
    }

    /// Create a new user on the system
    async fn create_user(
        &self,
        username: &str,
        fullname: Option<&str>,
        shell: Option<&str>,
        home_dir: Option<&str>,
        system_user: bool,
        groups: Option<Vec<&str>>,
    ) -> Result<()> {
        log_info(&format!("Creating user: {}", username));

        // Validate username
        if let Err(e) = validate_username(username) {
            return Err(e);
        }

        // Check if user already exists
        if get_user_by_name(username).is_some() {
            return Err(FluxError::Module(format!(
                "User '{}' already exists",
                username
            )));
        }

        // Build useradd command
        let mut cmd = Command::new("useradd");

        // Add flags
        if system_user {
            cmd.arg("--system");
        } else {
            cmd.arg("--create-home");
        }

        // Set shell
        if let Some(sh) = shell {
            cmd.arg("--shell").arg(sh);
        }

        // Set home directory
        if let Some(home) = home_dir {
            cmd.arg("--home-dir").arg(home);
        }

        // Set full name (comment field)
        if let Some(name) = fullname {
            cmd.arg("--comment").arg(name);
        }

        // Add username
        cmd.arg(username);

        // Execute command
        let output = cmd.output().map_err(|e| {
            FluxError::command_failed(format!("Failed to execute useradd: {}", e))
        })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(FluxError::command_failed(format!(
                "Failed to create user: {}",
                stderr
            )));
        }

        log_success(&format!("User '{}' created successfully", username));

        // Add to groups if specified
        if let Some(group_list) = groups {
            self.add_user_to_groups(username, &group_list).await?;
        }

        Ok(())
    }

    /// Create an admin user with sudo privileges
    async fn create_admin_user(&self, username: &str, github_user: Option<&str>) -> Result<()> {
        log_info(&format!("Creating admin user: {}", username));

        // Create the user
        self.create_user(
            username,
            Some(&format!("Administrator - {}", username)),
            Some("/bin/bash"),
            None,
            false,
            None,
        )
        .await?;

        // Determine sudo group based on distro
        let distro = crate::helpers::system::detect_distro()?;
        let sudo_group = if distro.is_debian_based() {
            "sudo"
        } else {
            "wheel"
        };

        // Add to admin groups
        let admin_groups = vec![sudo_group, "adm", "systemd-journal"];
        self.add_user_to_groups(username, &admin_groups).await?;

        // Setup SSH directory
        self.setup_ssh_directory(username).await?;

        // Add GitHub SSH keys if specified
        if let Some(gh_user) = github_user {
            log_info(&format!("Fetching SSH keys from GitHub for {}", gh_user));
            if let Err(e) = self.add_github_keys(username, gh_user).await {
                log_warn(&format!("Failed to fetch GitHub keys: {}", e));
            }
        }

        // Set password
        log_info(&format!("Set password for user '{}'", username));
        if let Err(e) = self.set_user_password(username).await {
            log_warn(&format!("Failed to set password: {}", e));
        }

        log_success(&format!("Admin user '{}' created successfully", username));
        Ok(())
    }

    /// Add user to specified groups
    async fn add_user_to_groups(&self, username: &str, groups: &[&str]) -> Result<()> {
        for group in groups {
            // Check if group exists, create if it's a custom group
            if get_group_by_name(group).is_none() {
                // Create group if it doesn't exist (for custom groups)
                if !["sudo", "wheel", "adm", "docker", "systemd-journal"].contains(group) {
                    log_info(&format!("Creating group: {}", group));
                    let output = Command::new("groupadd")
                        .arg(group)
                        .output()
                        .map_err(|e| FluxError::command_failed(format!("Failed to create group: {}", e)))?;

                    if !output.status.success() {
                        log_warn(&format!("Group '{}' creation failed", group));
                        continue;
                    }
                }
            }

            // Add user to group
            let output = Command::new("usermod")
                .arg("-a")
                .arg("-G")
                .arg(group)
                .arg(username)
                .output()
                .map_err(|e| FluxError::command_failed(format!("Failed to add user to group: {}", e)))?;

            if output.status.success() {
                log_success(&format!("Added '{}' to group '{}'", username, group));
            } else {
                log_warn(&format!("Failed to add '{}' to group '{}'", username, group));
            }
        }

        Ok(())
    }

    /// Remove user from specified groups
    async fn remove_user_from_groups(&self, username: &str, groups: &[&str]) -> Result<()> {
        for group in groups {
            let output = Command::new("gpasswd")
                .arg("-d")
                .arg(username)
                .arg(group)
                .output()
                .map_err(|e| {
                    FluxError::command_failed(format!("Failed to remove user from group: {}", e))
                })?;

            if output.status.success() {
                log_success(&format!("Removed '{}' from group '{}'", username, group));
            } else {
                log_warn(&format!(
                    "Failed to remove '{}' from group '{}'",
                    username, group
                ));
            }
        }

        Ok(())
    }

    /// Setup SSH directory for user
    async fn setup_ssh_directory(&self, username: &str) -> Result<()> {
        let user = get_user_by_name(username)
            .ok_or_else(|| FluxError::Module(format!("User '{}' not found", username)))?;

        let home_dir = PathBuf::from(user.home_dir().to_str().unwrap());
        let ssh_dir = home_dir.join(".ssh");
        let auth_keys = ssh_dir.join("authorized_keys");

        // Create .ssh directory
        if !ssh_dir.exists() {
            fs::create_dir_all(&ssh_dir).map_err(|e| {
                FluxError::system(format!("Failed to create .ssh directory: {}", e))
            })?;

            // Set permissions to 700
            let mut perms = fs::metadata(&ssh_dir).unwrap().permissions();
            perms.set_mode(0o700);
            fs::set_permissions(&ssh_dir, perms).map_err(|e| {
                FluxError::system(format!("Failed to set .ssh permissions: {}", e))
            })?;

            log_success(&format!("Created .ssh directory for '{}'", username));
        }

        // Create authorized_keys if it doesn't exist
        if !auth_keys.exists() {
            fs::write(&auth_keys, "").map_err(|e| {
                FluxError::system(format!("Failed to create authorized_keys: {}", e))
            })?;

            // Set permissions to 600
            let mut perms = fs::metadata(&auth_keys).unwrap().permissions();
            perms.set_mode(0o600);
            fs::set_permissions(&auth_keys, perms).map_err(|e| {
                FluxError::system(format!("Failed to set authorized_keys permissions: {}", e))
            })?;
        }

        // Set ownership
        let uid = user.uid();
        let gid = user.primary_group_id();

        Command::new("chown")
            .arg("-R")
            .arg(format!("{}:{}", uid, gid))
            .arg(&ssh_dir)
            .output()
            .map_err(|e| FluxError::command_failed(format!("Failed to set ownership: {}", e)))?;

        log_success(&format!("SSH directory configured for '{}'", username));
        Ok(())
    }

    /// Add SSH key from GitHub
    async fn add_github_keys(&self, username: &str, github_user: &str) -> Result<()> {
        let url = format!("https://github.com/{}.keys", github_user);

        log_info(&format!("Fetching SSH keys from {}", url));

        let client = reqwest::Client::new();
        let response = client
            .get(&url)
            .send()
            .await
            .map_err(|e| FluxError::Network(format!("Failed to fetch GitHub keys: {}", e)))?;

        if !response.status().is_success() {
            return Err(FluxError::Network(format!(
                "GitHub returned status: {}",
                response.status()
            )));
        }

        let keys = response
            .text()
            .await
            .map_err(|e| FluxError::Network(format!("Failed to read response: {}", e)))?;

        if keys.trim().is_empty() {
            return Err(FluxError::Module(format!(
                "No SSH keys found for GitHub user '{}'",
                github_user
            )));
        }

        // Add each key
        for key in keys.lines() {
            if !key.trim().is_empty() {
                self.add_ssh_key(username, key).await?;
            }
        }

        log_success(&format!(
            "Added GitHub SSH keys for '{}'",
            github_user
        ));
        Ok(())
    }

    /// Add SSH public key to user's authorized_keys
    async fn add_ssh_key(&self, username: &str, key: &str) -> Result<()> {
        let user = get_user_by_name(username)
            .ok_or_else(|| FluxError::Module(format!("User '{}' not found", username)))?;

        let home_dir = PathBuf::from(user.home_dir().to_str().unwrap());
        let auth_keys = home_dir.join(".ssh/authorized_keys");

        // Read existing keys
        let existing_keys = if auth_keys.exists() {
            fs::read_to_string(&auth_keys).unwrap_or_default()
        } else {
            String::new()
        };

        // Check for duplicates
        if existing_keys.contains(key.trim()) {
            log_warn("SSH key already exists in authorized_keys");
            return Ok(());
        }

        // Append key
        let mut updated_keys = existing_keys;
        if !updated_keys.ends_with('\n') && !updated_keys.is_empty() {
            updated_keys.push('\n');
        }
        updated_keys.push_str(key.trim());
        updated_keys.push('\n');

        // Write back
        fs::write(&auth_keys, updated_keys).map_err(|e| {
            FluxError::system(format!("Failed to write authorized_keys: {}", e))
        })?;

        log_success("SSH key added to authorized_keys");
        Ok(())
    }

    /// Set user password
    async fn set_user_password(&self, username: &str) -> Result<()> {
        let password = prompt_password(&format!("Enter password for '{}'", username))?;

        let mut child = Command::new("chpasswd")
            .stdin(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| FluxError::command_failed(format!("Failed to start chpasswd: {}", e)))?;

        if let Some(mut stdin) = child.stdin.take() {
            use std::io::Write;
            writeln!(stdin, "{}:{}", username, password).map_err(|e| {
                FluxError::command_failed(format!("Failed to write to chpasswd: {}", e))
            })?;
        }

        let output = child
            .wait_with_output()
            .map_err(|e| FluxError::command_failed(format!("Failed to wait for chpasswd: {}", e)))?;

        if !output.status.success() {
            return Err(FluxError::command_failed(
                "Failed to set password".to_string(),
            ));
        }

        log_success(&format!("Password set for '{}'", username));
        Ok(())
    }

    /// Delete a user
    async fn delete_user(&self, username: &str, remove_home: bool, backup: bool) -> Result<()> {
        log_info(&format!("Deleting user: {}", username));

        // Check if user exists
        let user = get_user_by_name(username)
            .ok_or_else(|| FluxError::Module(format!("User '{}' not found", username)))?;

        // Backup home directory if requested
        if backup && remove_home {
            let home_dir = user.home_dir();
            let backup_dir = PathBuf::from("/var/backups/users");
            fs::create_dir_all(&backup_dir).ok();

            let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
            let backup_path = backup_dir.join(format!("{}-{}.tar.gz", username, timestamp));

            log_info(&format!("Backing up home directory to {:?}", backup_path));

            let output = Command::new("tar")
                .arg("-czf")
                .arg(&backup_path)
                .arg("-C")
                .arg(home_dir.parent().unwrap())
                .arg(home_dir.file_name().unwrap())
                .output()
                .map_err(|e| FluxError::command_failed(format!("Failed to backup home directory: {}", e)))?;

            if output.status.success() {
                log_success(&format!("Home directory backed up to {:?}", backup_path));
            } else {
                log_warn("Failed to backup home directory");
            }
        }

        // Delete user
        let mut cmd = Command::new("userdel");
        if remove_home {
            cmd.arg("-r");
        }
        cmd.arg(username);

        let output = cmd
            .output()
            .map_err(|e| FluxError::command_failed(format!("Failed to execute userdel: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(FluxError::command_failed(format!(
                "Failed to delete user: {}",
                stderr
            )));
        }

        log_success(&format!("User '{}' deleted successfully", username));
        Ok(())
    }

    /// List users on the system
    async fn list_users(&self, min_uid: u32) -> Result<()> {
        log_info("Listing system users:");

        let passwd = fs::read_to_string("/etc/passwd")
            .map_err(|e| FluxError::system(format!("Failed to read /etc/passwd: {}", e)))?;

        println!("\n{:<20} {:<10} {:<10} {:<30}", "Username", "UID", "GID", "Full Name");
        println!("{}", "-".repeat(70));

        for line in passwd.lines() {
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() >= 5 {
                let username = parts[0];
                let uid: u32 = parts[2].parse().unwrap_or(0);
                let gid = parts[3];
                let fullname = parts[4].split(',').next().unwrap_or("");

                if uid >= min_uid {
                    println!("{:<20} {:<10} {:<10} {:<30}", username, uid, gid, fullname);
                }
            }
        }

        Ok(())
    }

    /// Show interactive menu
    async fn show_menu(&self) -> Result<()> {
        loop {
            let options = vec![
                "Create user",
                "Create admin user",
                "Delete user",
                "Add user to groups",
                "Setup SSH keys",
                "List users",
                "Exit",
            ];

            let choice = select_from_menu("User Management", &options)?;

            match choice {
                0 => {
                    // Create user
                    let username = prompt_input("Enter username")?;
                    let fullname = prompt_input("Enter full name (optional)").ok();
                    let shell = prompt_with_default("Enter shell (default: /bin/bash)", "/bin/bash")?;

                    self.create_user(
                        &username,
                        fullname.as_deref(),
                        Some(&shell),
                        None,
                        false,
                        None,
                    )
                    .await?;
                }
                1 => {
                    // Create admin user
                    let username = prompt_input("Enter username")?;
                    let use_github = prompt_yes_no("Import SSH keys from GitHub?", false)?;
                    let github_user = if use_github {
                        Some(prompt_input("Enter GitHub username")?)
                    } else {
                        None
                    };

                    self.create_admin_user(&username, github_user.as_deref()).await?;
                }
                2 => {
                    // Delete user
                    let username = prompt_input("Enter username to delete")?;
                    let remove_home = prompt_yes_no("Remove home directory?", false)?;
                    let backup = if remove_home {
                        prompt_yes_no("Backup home directory first?", true)?
                    } else {
                        false
                    };

                    self.delete_user(&username, remove_home, backup).await?;
                }
                3 => {
                    // Add to groups
                    let username = prompt_input("Enter username")?;
                    let groups_str = prompt_input("Enter groups (comma-separated)")?;
                    let groups: Vec<&str> = groups_str.split(',').map(|s| s.trim()).collect();

                    self.add_user_to_groups(&username, &groups).await?;
                }
                4 => {
                    // Setup SSH keys
                    let username = prompt_input("Enter username")?;
                    self.setup_ssh_directory(&username).await?;

                    if prompt_yes_no("Add SSH key from GitHub?", false)? {
                        let github_user = prompt_input("Enter GitHub username")?;
                        self.add_github_keys(&username, &github_user).await?;
                    }
                }
                5 => {
                    // List users
                    self.list_users(1000).await?;
                }
                6 => {
                    log_info("Exiting user management");
                    break;
                }
                _ => {}
            }
        }

        Ok(())
    }
}

#[async_trait]
impl Module for UserModule {
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
        check_command("useradd").is_ok() && check_command("usermod").is_ok()
    }

    fn help(&self) -> String {
        format!(
            r#"User Management Module v{}

DESCRIPTION:
    {}

USAGE:
    flux module {} [OPTIONS]

OPTIONS:
    --create <username>          Create a new user
    --admin <username>           Create an admin user with sudo access
    --delete <username>          Delete a user
    --list                       List all users (UID >= 1000)
    --menu                       Show interactive menu

EXAMPLES:
    flux module {} --menu
    flux module {} --create john --groups sudo,docker
    flux module {} --admin alice --github alice123
    flux module {} --list
"#,
            self.version(),
            self.description(),
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

        // Parse arguments
        let mut i = 0;
        while i < args.len() {
            match args[i].as_str() {
                "--create" => {
                    if i + 1 < args.len() {
                        let username = &args[i + 1];
                        self.create_user(username, None, Some("/bin/bash"), None, false, None)
                            .await?;
                        i += 2;
                    }
                }
                "--admin" => {
                    if i + 1 < args.len() {
                        let username = &args[i + 1];
                        self.create_admin_user(username, None).await?;
                        i += 2;
                    }
                }
                "--delete" => {
                    if i + 1 < args.len() {
                        let username = &args[i + 1];
                        self.delete_user(username, false, false).await?;
                        i += 2;
                    }
                }
                "--list" => {
                    self.list_users(1000).await?;
                    i += 1;
                }
                _ => {
                    i += 1;
                }
            }
        }

        Ok(())
    }
}
