use crate::config::Config;
use crate::error::{FluxError, Result};
use crate::helpers::{
    logging::{log_info, log_success},
    system::execute_command,
    user_input::{prompt_input, prompt_with_default},
};
use crate::modules::{Module, ModuleBase, ModuleContext, ModuleInfo};
use async_trait::async_trait;
use clap::{Arg, ArgAction, ArgMatches, Command};
use colored::Colorize;

/// Timezone configuration module
pub struct TimezoneModule {
    base: ModuleBase,
}

impl TimezoneModule {
    pub fn new() -> Self {
        let info = ModuleInfo {
            name: "timezone".to_string(),
            description: "Timezone configuration".to_string(),
            version: "1.0.0".to_string(),
            author: "Flux Contributors".to_string(),
            tags: vec!["system".to_string(), "timezone".to_string()],
            requires_root: true,
            supported_distros: vec!["all".to_string()],
        };

        Self {
            base: ModuleBase { info },
        }
    }

    fn create_cli(&self) -> Command {
        self.base
            .create_args_parser()
            .arg(
                Arg::new("list")
                    .short('l')
                    .long("list")
                    .help("List available timezones")
                    .action(ArgAction::SetTrue),
            )
            .arg(
                Arg::new("show")
                    .short('s')
                    .long("show")
                    .help("Show current timezone")
                    .action(ArgAction::SetTrue),
            )
            .arg(
                Arg::new("set")
                    .long("set")
                    .help("Set timezone (e.g., America/Toronto)")
                    .num_args(1)
                    .value_name("TIMEZONE"),
            )
            .arg(
                Arg::new("interactive")
                    .short('i')
                    .long("interactive")
                    .help("Interactive timezone configuration")
                    .action(ArgAction::SetTrue),
            )
    }

    async fn execute_timezone(&self, matches: &ArgMatches, _ctx: &ModuleContext<'_>) -> Result<()> {
        if matches.get_flag("list") {
            return self.list_timezones().await;
        }

        if matches.get_flag("show") {
            return self.show_current_timezone().await;
        }

        if let Some(timezone) = matches.get_one::<String>("set") {
            return self.set_timezone(timezone).await;
        }

        if matches.get_flag("interactive") {
            return self.configure_interactive().await;
        }

        // Default: show current timezone
        self.show_current_timezone().await
    }

    async fn show_current_timezone(&self) -> Result<()> {
        println!("{}", "=== Current Timezone ===".cyan());
        println!();

        // Get current timezone using timedatectl
        let output = execute_command("timedatectl", &["show", "--property=Timezone", "--value"])?;
        let timezone = output.trim();

        println!("Current timezone: {}", timezone.green());

        // Also show current date/time
        let datetime_output = execute_command("date", &["+%Y-%m-%d %H:%M:%S %Z"])?;
        println!("Current date/time: {}", datetime_output.trim());

        Ok(())
    }

    async fn list_timezones(&self) -> Result<()> {
        println!("{}", "=== Available Timezones ===".cyan());
        println!();
        println!("Listing common timezones. Use 'timedatectl list-timezones' for a complete list.");
        println!();

        // Show common timezones by region
        let common_timezones = vec![
            ("Americas", vec![
                "America/Toronto",
                "America/New_York",
                "America/Chicago",
                "America/Denver",
                "America/Los_Angeles",
                "America/Anchorage",
                "America/Phoenix",
                "America/Vancouver",
                "America/Montreal",
            ]),
            ("Europe", vec![
                "Europe/London",
                "Europe/Paris",
                "Europe/Berlin",
                "Europe/Rome",
                "Europe/Madrid",
                "Europe/Amsterdam",
                "Europe/Brussels",
                "Europe/Zurich",
            ]),
            ("Asia", vec![
                "Asia/Tokyo",
                "Asia/Shanghai",
                "Asia/Hong_Kong",
                "Asia/Singapore",
                "Asia/Dubai",
                "Asia/Kolkata",
                "Asia/Seoul",
            ]),
            ("Pacific", vec![
                "Pacific/Auckland",
                "Pacific/Sydney",
                "Pacific/Fiji",
                "Pacific/Honolulu",
            ]),
        ];

        for (region, zones) in common_timezones {
            println!("{}", format!("{}:", region).white().bold());
            for zone in zones {
                println!("  {}", zone);
            }
            println!();
        }

        println!("Use the full timezone name (e.g., America/Toronto) when setting.");

        Ok(())
    }

    async fn set_timezone(&self, timezone: &str) -> Result<()> {
        log_info(format!("Setting timezone to {}", timezone));

        // Validate timezone exists
        let result = execute_command("timedatectl", &["list-timezones"]);
        if let Ok(output) = result {
            if !output.lines().any(|line| line.trim() == timezone) {
                return Err(FluxError::validation(format!(
                    "Timezone '{}' not found. Use --list to see available timezones.",
                    timezone
                )));
            }
        }

        // Set timezone using timedatectl
        execute_command("timedatectl", &["set-timezone", timezone])?;

        log_success(format!("Timezone set to {}", timezone));

        // Show current time in new timezone
        let datetime_output = execute_command("date", &["+%Y-%m-%d %H:%M:%S %Z"])?;
        println!("Current date/time: {}", datetime_output.trim());

        Ok(())
    }

    async fn configure_interactive(&self) -> Result<()> {
        log_info("Starting interactive timezone configuration");

        println!("{}", "=== Timezone Configuration ===".cyan());
        println!();

        // Show current timezone
        let current_tz = execute_command("timedatectl", &["show", "--property=Timezone", "--value"])?;
        let current_tz = current_tz.trim();
        println!("Current timezone: {}", current_tz.green());
        println!();

        // Prompt for new timezone with default
        let timezone = prompt_with_default(
            "Enter timezone (e.g., America/Toronto)",
            "America/Toronto",
        )?;

        // Set the timezone
        self.set_timezone(&timezone).await?;

        Ok(())
    }
}

#[async_trait]
impl Module for TimezoneModule {
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
        // Check if timedatectl is available
        execute_command("which", &["timedatectl"]).is_ok()
    }

    fn help(&self) -> String {
        self.create_cli().render_help().to_string()
    }

    async fn execute(&self, args: Vec<String>, config: &Config) -> Result<()> {
        let ctx = ModuleContext::new(config, args.clone());

        let args_strs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        let mut all_args = vec!["timezone"];
        all_args.extend(args_strs);

        let matches = self.create_cli()
            .try_get_matches_from(all_args)
            .map_err(|e| FluxError::validation(format!("Invalid arguments: {}", e)))?;

        self.execute_timezone(&matches, &ctx).await
    }
}
