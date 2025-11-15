use crate::error::{FluxError, Result};
use crate::helpers::validation;
use colored::Colorize;
use dialoguer::{theme::ColorfulTheme, Confirm, Input, Password, Select};
use std::io;

/// Prompt for yes/no confirmation
pub fn prompt_yes_no(prompt: &str, default: bool) -> Result<bool> {
    Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .default(default)
        .interact()
        .map_err(|e| FluxError::External(e.into()))
}

/// Prompt for input with default value
pub fn prompt_with_default(prompt: &str, default: &str) -> Result<String> {
    Input::<String>::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .default(default.to_string())
        .interact()
        .map_err(|e| FluxError::External(e.into()))
}

/// Prompt for input without default
pub fn prompt_input(prompt: &str) -> Result<String> {
    Input::<String>::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .interact()
        .map_err(|e| FluxError::External(e.into()))
}

/// Prompt for password
pub fn prompt_password(prompt: &str) -> Result<String> {
    Password::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .interact()
        .map_err(|e| FluxError::External(e.into()))
}

/// Prompt for password with confirmation
pub fn prompt_password_confirm(prompt: &str) -> Result<String> {
    Password::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .with_confirmation("Confirm password", "Passwords do not match")
        .interact()
        .map_err(|e| FluxError::External(e.into()))
}

/// Prompt with validation
pub fn prompt_with_validation<F>(
    prompt: &str,
    default: Option<&str>,
    validator: F,
    error_msg: &str,
) -> Result<String>
where
    F: Fn(&str) -> bool,
{
    loop {
        let input = if let Some(def) = default {
            prompt_with_default(prompt, def)?
        } else {
            prompt_input(prompt)?
        };
        
        if validator(&input) {
            return Ok(input);
        }
        
        eprintln!("{} {}", "[ERROR]".red(), error_msg);
    }
}

/// Prompt for IP address
pub fn prompt_ip(prompt: &str, default: Option<&str>) -> Result<String> {
    prompt_with_validation(
        prompt,
        default,
        |ip| validation::validate_ip(ip).is_ok(),
        "Invalid IP address format (e.g., 192.168.1.100)",
    )
}

/// Prompt for hostname
pub fn prompt_hostname(prompt: &str, default: Option<&str>) -> Result<String> {
    prompt_with_validation(
        prompt,
        default,
        |hostname| validation::validate_hostname(hostname).is_ok(),
        "Invalid hostname format",
    )
}

/// Prompt for port number
pub fn prompt_port(prompt: &str, default: Option<&str>) -> Result<String> {
    prompt_with_validation(
        prompt,
        default,
        |port| validation::validate_port(port).is_ok(),
        "Invalid port number (must be 1-65535)",
    )
}

/// Prompt for network interface selection
pub fn prompt_interface(prompt: &str, show_list: bool) -> Result<String> {
    if show_list {
        // Get available interfaces
        let interfaces = list_network_interfaces()?;
        
        if interfaces.is_empty() {
            return Err(FluxError::network("No network interfaces found"));
        }
        
        // Show selection menu
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt(prompt)
            .items(&interfaces)
            .default(0)
            .interact()
            .map_err(|e| FluxError::External(e.into()))?;
        
        Ok(interfaces[selection].clone())
    } else {
        prompt_with_validation(
            prompt,
            None,
            |iface| validation::validate_interface(iface).is_ok(),
            "Interface does not exist",
        )
    }
}

/// Prompt for selection from list
pub fn prompt_select(prompt: &str, items: &[String], default: usize) -> Result<usize> {
    Select::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .items(items)
        .default(default)
        .interact()
        .map_err(|e| FluxError::External(e.into()))
}

/// Multi-select prompt
pub fn prompt_multi_select(prompt: &str, items: &[String]) -> Result<Vec<usize>> {
    use dialoguer::MultiSelect;
    
    MultiSelect::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .items(items)
        .interact()
        .map_err(|e| FluxError::External(e.into()))
}

/// Prompt to continue or abort
pub fn prompt_continue(message: &str) -> Result<()> {
    println!("{}", message);
    println!("Press Enter to continue...");
    
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    
    Ok(())
}

/// Show a warning and ask to continue
pub fn prompt_warning(warning: &str) -> Result<bool> {
    eprintln!("{} {}", "[WARNING]".yellow(), warning);
    prompt_yes_no("Continue anyway?", false)
}

/// List network interfaces (helper for prompt_interface)
fn list_network_interfaces() -> Result<Vec<String>> {
    use std::fs;
    
    let mut interfaces = Vec::new();
    
    for entry in fs::read_dir("/sys/class/net")? {
        if let Ok(entry) = entry {
            let name = entry.file_name().to_string_lossy().to_string();
            
            // Skip loopback
            if name != "lo" {
                interfaces.push(name);
            }
        }
    }
    
    interfaces.sort();
    Ok(interfaces)
}

/// Progress bar wrapper for long operations
pub struct ProgressBar {
    pb: indicatif::ProgressBar,
}

impl ProgressBar {
    /// Create a new progress bar
    pub fn new(total: u64, message: &str) -> Self {
        let pb = indicatif::ProgressBar::new(total);
        pb.set_style(
            indicatif::ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
                .unwrap()
                .progress_chars("#>-"),
        );
        pb.set_message(message.to_string());
        
        Self { pb }
    }
    
    /// Update progress
    pub fn update(&self, pos: u64) {
        self.pb.set_position(pos);
    }
    
    /// Increment progress
    pub fn inc(&self, delta: u64) {
        self.pb.inc(delta);
    }
    
    /// Finish with message
    pub fn finish(&self, message: &str) {
        self.pb.finish_with_message(message.to_string());
    }
}

/// Interactive menu builder
pub struct Menu {
    title: String,
    items: Vec<(String, String)>,
}

impl Menu {
    /// Create a new menu
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            items: Vec::new(),
        }
    }

    /// Add menu item
    pub fn add_item(mut self, key: &str, description: &str) -> Self {
        self.items.push((key.to_string(), description.to_string()));
        self
    }

    /// Show menu and get selection
    pub fn show(&self) -> Result<String> {
        println!("\n{}", self.title.cyan());
        println!("{}", "=".repeat(self.title.len()).cyan());

        let display_items: Vec<String> = self.items
            .iter()
            .enumerate()
            .map(|(i, (key, desc))| format!("{}. {} - {}", i + 1, key, desc))
            .collect();

        for item in &display_items {
            println!("{}", item);
        }

        println!("{}. Exit", display_items.len() + 1);
        println!();

        let selection = prompt_input("Select option")?;

        if let Ok(num) = selection.parse::<usize>() {
            if num > 0 && num <= self.items.len() {
                return Ok(self.items[num - 1].0.clone());
            } else if num == self.items.len() + 1 {
                return Err(FluxError::UserCancelled);
            }
        }

        // Try to match by key
        for (key, _) in &self.items {
            if key.eq_ignore_ascii_case(&selection) {
                return Ok(key.clone());
            }
        }

        Err(FluxError::validation("Invalid selection"))
    }
}

/// Select from a menu - accepts both &[String] and &[&str]
pub fn select_from_menu<T: AsRef<str>>(prompt: &str, items: &[T]) -> Result<usize> {
    let string_items: Vec<String> = items.iter().map(|s| s.as_ref().to_string()).collect();
    prompt_select(prompt, &string_items, 0)
}

/// Multi-select menu - accepts both &[String] and &[&str]
pub fn multi_select_menu<T: AsRef<str>>(prompt: &str, items: &[T]) -> Result<Vec<usize>> {
    let string_items: Vec<String> = items.iter().map(|s| s.as_ref().to_string()).collect();
    prompt_multi_select(prompt, &string_items)
}