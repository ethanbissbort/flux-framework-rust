// src/modules/certs.rs
// TLS/CA certificates management module

use crate::config::Config;
use crate::error::{FluxError, Result};
use crate::helpers::{
    logging::{log_debug, log_error, log_info, log_success, log_warn},
    system::{check_command, execute_command, get_os_info},
    user_input::{prompt_input, prompt_yes_no, select_from_menu},
};
use crate::modules::{Module, ModuleBase, ModuleInfo};
use async_trait::async_trait;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const DEBIAN_CERT_DIR: &str = "/usr/local/share/ca-certificates";
const RHEL_CERT_DIR: &str = "/etc/pki/ca-trust/source/anchors";

pub struct CertsModule {
    base: ModuleBase,
}

impl CertsModule {
    pub fn new() -> Self {
        let info = ModuleInfo {
            name: "certs".to_string(),
            description: "TLS/CA certificates management".to_string(),
            version: "1.0.0".to_string(),
            author: "Flux Contributors".to_string(),
            tags: vec!["security".to_string(), "ssl".to_string()],
            requires_root: true,
            supported_distros: vec!["all".to_string()],
        };
        Self {
            base: ModuleBase { info },
        }
    }

    /// Detect certificate directory based on distribution
    fn get_cert_dir(&self) -> Result<PathBuf> {
        let distro = crate::helpers::system::detect_distro()?;

        if distro.is_debian_based() {
            Ok(PathBuf::from(DEBIAN_CERT_DIR))
        } else if distro.is_redhat_based() {
            Ok(PathBuf::from(RHEL_CERT_DIR))
        } else {
            Err(FluxError::Module(
                "Unsupported distribution for certificate management".to_string()
            ))
        }
    }

    /// Validate certificate file
    async fn validate_certificate(&self, cert_path: &Path) -> Result<bool> {
        log_debug(&format!("Validating certificate: {:?}", cert_path));

        if !cert_path.exists() {
            return Err(FluxError::Module(format!(
                "Certificate file not found: {:?}",
                cert_path
            )));
        }

        // Check if it's a valid certificate using openssl
        let output = Command::new("openssl")
            .arg("x509")
            .arg("-in")
            .arg(cert_path)
            .arg("-noout")
            .arg("-text")
            .output()
            .map_err(|e| FluxError::command_failed(format!("Failed to validate certificate: {}", e)))?;

        if output.status.success() {
            log_debug("Certificate validation successful");
            Ok(true)
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            log_warn(&format!("Certificate validation failed: {}", stderr));
            Ok(false)
        }
    }

    /// Show certificate information
    async fn show_cert_info(&self, cert_path: &Path) -> Result<()> {
        log_info(&format!("Certificate information for: {:?}", cert_path));

        let output = Command::new("openssl")
            .arg("x509")
            .arg("-in")
            .arg(cert_path)
            .arg("-noout")
            .arg("-subject")
            .arg("-issuer")
            .arg("-dates")
            .arg("-fingerprint")
            .output()
            .map_err(|e| FluxError::command_failed(format!("Failed to read certificate info: {}", e)))?;

        if output.status.success() {
            println!("\n{}", String::from_utf8_lossy(&output.stdout));
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(FluxError::command_failed(format!(
                "Failed to get certificate info: {}",
                stderr
            )));
        }

        Ok(())
    }

    /// Install certificate to system trust store
    async fn install_certificate(&self, cert_path: &Path, cert_name: Option<&str>) -> Result<()> {
        log_info(&format!("Installing certificate: {:?}", cert_path));

        // Validate certificate first
        if !self.validate_certificate(cert_path).await? {
            return Err(FluxError::Module(
                "Certificate validation failed. Not installing.".to_string(),
            ));
        }

        // Show certificate info
        self.show_cert_info(cert_path).await?;

        // Confirm installation
        let confirm = prompt_yes_no("Install this certificate to system trust store?", true)?;
        if !confirm {
            log_info("Certificate installation cancelled");
            return Ok(());
        }

        let cert_dir = self.get_cert_dir()?;
        fs::create_dir_all(&cert_dir)?;

        // Determine certificate filename
        let filename = if let Some(name) = cert_name {
            if name.ends_with(".crt") {
                name.to_string()
            } else {
                format!("{}.crt", name)
            }
        } else {
            cert_path
                .file_name()
                .and_then(|n| n.to_str())
                .map(|s| {
                    if s.ends_with(".crt") {
                        s.to_string()
                    } else {
                        format!("{}.crt", s)
                    }
                })
                .unwrap_or_else(|| "custom-cert.crt".to_string())
        };

        let dest_path = cert_dir.join(&filename);

        // Copy certificate to system directory
        fs::copy(cert_path, &dest_path)?;

        // Set permissions
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&dest_path)?.permissions();
            perms.set_mode(0o644);
            fs::set_permissions(&dest_path, perms)?;
        }

        log_success(&format!(
            "Certificate copied to: {}",
            dest_path.display()
        ));

        // Update trust store
        self.update_trust_store().await?;

        log_success("Certificate installed successfully");
        Ok(())
    }

    /// Update system trust store
    async fn update_trust_store(&self) -> Result<()> {
        log_info("Updating system certificate trust store");

        let distro = crate::helpers::system::detect_distro()?;

        if distro.is_debian_based() {
            execute_command("update-ca-certificates", &[])?;
        } else if distro.is_redhat_based() {
            execute_command("update-ca-trust", &[])?;
        } else {
            return Err(FluxError::Module(
                "Unsupported distribution for trust store update".to_string(),
            ));
        }

        log_success("Trust store updated successfully");
        Ok(())
    }

    /// List installed certificates
    async fn list_certificates(&self) -> Result<()> {
        log_info("Installed custom certificates:");

        let cert_dir = self.get_cert_dir()?;

        if !cert_dir.exists() {
            log_warn("Certificate directory not found");
            return Ok(());
        }

        let entries = fs::read_dir(&cert_dir)
            .map_err(|e| FluxError::system(format!("Failed to read certificate directory: {}", e)))?;

        println!("\n{:<40} {:<15} {:<30}", "Certificate", "Size", "Modified");
        println!("{}", "-".repeat(85));

        let mut count = 0;
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("crt") {
                    if let Ok(metadata) = entry.metadata() {
                        let filename = entry.file_name();
                        let size = metadata.len();
                        let modified = metadata
                            .modified()
                            .ok()
                            .and_then(|t| {
                                use std::time::SystemTime;
                                t.duration_since(SystemTime::UNIX_EPOCH).ok()
                            })
                            .map(|d| {
                                chrono::DateTime::from_timestamp(d.as_secs() as i64, 0)
                                    .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                                    .unwrap_or_else(|| "Unknown".to_string())
                            })
                            .unwrap_or_else(|| "Unknown".to_string());

                        println!(
                            "{:<40} {:<15} {:<30}",
                            filename.to_string_lossy(),
                            format!("{} bytes", size),
                            modified
                        );
                        count += 1;
                    }
                }
            }
        }

        if count == 0 {
            println!("No custom certificates installed");
        } else {
            println!("\nTotal: {} certificate(s)", count);
        }

        Ok(())
    }

    /// Remove certificate from trust store
    async fn remove_certificate(&self, cert_name: &str) -> Result<()> {
        log_info(&format!("Removing certificate: {}", cert_name));

        let cert_dir = self.get_cert_dir()?;
        let filename = if cert_name.ends_with(".crt") {
            cert_name.to_string()
        } else {
            format!("{}.crt", cert_name)
        };

        let cert_path = cert_dir.join(&filename);

        if !cert_path.exists() {
            return Err(FluxError::Module(format!(
                "Certificate not found: {}",
                cert_name
            )));
        }

        // Show certificate info before removal
        self.show_cert_info(&cert_path).await?;

        let confirm = prompt_yes_no("Remove this certificate?", false)?;
        if !confirm {
            log_info("Certificate removal cancelled");
            return Ok(());
        }

        // Remove the certificate file
        fs::remove_file(&cert_path)?;
        log_success(&format!("Certificate file removed: {}", filename));

        // Update trust store
        self.update_trust_store().await?;

        log_success("Certificate removed successfully");
        Ok(())
    }

    /// Install certificate from URL
    async fn install_from_url(&self, url: &str, cert_name: &str) -> Result<()> {
        log_info(&format!("Downloading certificate from: {}", url));

        // Download certificate
        let client = reqwest::Client::builder()
            .danger_accept_invalid_certs(true) // Accept self-signed during download
            .build()
            .map_err(|e| FluxError::Network(format!("Failed to create HTTP client: {}", e)))?;

        let response = client
            .get(url)
            .send()
            .await
            .map_err(|e| FluxError::Network(format!("Failed to download certificate: {}", e)))?;

        if !response.status().is_success() {
            return Err(FluxError::Network(format!(
                "HTTP error: {}",
                response.status()
            )));
        }

        let cert_data = response
            .bytes()
            .await
            .map_err(|e| FluxError::Network(format!("Failed to read certificate data: {}", e)))?;

        // Save to temporary file
        let temp_dir = std::env::temp_dir();
        let temp_path = temp_dir.join(format!("flux-cert-{}.crt", cert_name));
        fs::write(&temp_path, &cert_data)?;

        log_success(&format!(
            "Certificate downloaded to: {}",
            temp_path.display()
        ));

        // Install the certificate
        let result = self.install_certificate(&temp_path, Some(cert_name)).await;

        // Clean up temporary file
        fs::remove_file(&temp_path).ok();

        result
    }

    /// Install certificate from file
    async fn install_from_file(&self, file_path: &str, cert_name: Option<&str>) -> Result<()> {
        let path = PathBuf::from(file_path);
        self.install_certificate(&path, cert_name).await
    }

    /// Show interactive menu
    async fn show_menu(&self) -> Result<()> {
        loop {
            let options = vec![
                "Install certificate from file",
                "Install certificate from URL",
                "List installed certificates",
                "Show certificate info",
                "Remove certificate",
                "Update trust store",
                "Exit",
            ];

            let choice = select_from_menu("Certificate Management", &options)?;

            match choice {
                0 => {
                    // Install from file
                    let file_path = prompt_input("Enter certificate file path")?;
                    let cert_name = prompt_input("Enter certificate name (optional)")
                    .ok();

                    if let Err(e) = self
                        .install_from_file(&file_path, cert_name.as_deref())
                        .await
                    {
                        log_error(&format!("Failed to install certificate: {}", e));
                    }
                }
                1 => {
                    // Install from URL
                    let url = prompt_input("Enter certificate URL")?;
                    let cert_name = prompt_input("Enter certificate name")?;

                    if let Err(e) = self.install_from_url(&url, &cert_name).await {
                        log_error(&format!("Failed to install certificate: {}", e));
                    }
                }
                2 => {
                    // List certificates
                    self.list_certificates().await?;
                }
                3 => {
                    // Show cert info
                    let file_path = prompt_input("Enter certificate file path")?;
                    let path = PathBuf::from(file_path);

                    if let Err(e) = self.show_cert_info(&path).await {
                        log_error(&format!("Failed to show certificate info: {}", e));
                    }
                }
                4 => {
                    // Remove certificate
                    let cert_name = prompt_input("Enter certificate name to remove")?;

                    if let Err(e) = self.remove_certificate(&cert_name).await {
                        log_error(&format!("Failed to remove certificate: {}", e));
                    }
                }
                5 => {
                    // Update trust store
                    self.update_trust_store().await?;
                }
                6 => {
                    log_info("Exiting certificate management");
                    break;
                }
                _ => {}
            }
        }

        Ok(())
    }
}

#[async_trait]
impl Module for CertsModule {
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
        check_command("openssl").is_ok()
            && (check_command("update-ca-certificates").is_ok()
                || check_command("update-ca-trust").is_ok())
    }

    fn help(&self) -> String {
        format!(
            r#"Certificate Management Module v{}

DESCRIPTION:
    {}

    This module manages SSL/TLS certificates in the system trust store,
    supporting both Debian-based and RHEL-based distributions.

USAGE:
    flux module {} [OPTIONS]

OPTIONS:
    --install <file>             Install certificate from file
    --install-url <url> <name>   Install certificate from URL
    --list                       List installed certificates
    --info <file>                Show certificate information
    --remove <name>              Remove certificate
    --update                     Update system trust store
    --menu                       Show interactive menu

CERTIFICATE LOCATIONS:
    Debian/Ubuntu:  {}
    RHEL/CentOS:    {}

EXAMPLES:
    flux module {} --menu
    flux module {} --install /path/to/cert.crt
    flux module {} --install-url https://example.com/cert.crt my-cert
    flux module {} --list
    flux module {} --remove my-cert
    flux module {} --info /path/to/cert.crt
"#,
            self.version(),
            self.description(),
            self.name(),
            DEBIAN_CERT_DIR,
            RHEL_CERT_DIR,
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
                "--install" => {
                    if i + 1 < args.len() {
                        self.install_from_file(&args[i + 1], None).await?;
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                "--install-url" => {
                    if i + 2 < args.len() {
                        self.install_from_url(&args[i + 1], &args[i + 2]).await?;
                        i += 3;
                    } else {
                        i += 1;
                    }
                }
                "--list" => {
                    self.list_certificates().await?;
                    i += 1;
                }
                "--info" => {
                    if i + 1 < args.len() {
                        let path = PathBuf::from(&args[i + 1]);
                        self.show_cert_info(&path).await?;
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                "--remove" => {
                    if i + 1 < args.len() {
                        self.remove_certificate(&args[i + 1]).await?;
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                "--update" => {
                    self.update_trust_store().await?;
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
