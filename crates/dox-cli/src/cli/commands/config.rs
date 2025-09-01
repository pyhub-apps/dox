use anyhow::Result;
use clap::Args;

/// Manage dox configuration
#[derive(Args, Debug)]
pub struct ConfigArgs {
    /// Initialize configuration file
    #[arg(long, conflicts_with_all = ["list", "get", "set", "unset"])]
    pub init: bool,
    
    /// List all configuration values
    #[arg(long, conflicts_with_all = ["init", "get", "set", "unset"])]
    pub list: bool,
    
    /// Get a specific configuration value
    #[arg(long, value_name = "KEY", conflicts_with_all = ["init", "list", "set", "unset"])]
    pub get: Option<String>,
    
    /// Set a configuration value
    #[arg(long, value_name = "KEY=VALUE", conflicts_with_all = ["init", "list", "get", "unset"])]
    pub set: Option<String>,
    
    /// Remove a configuration value
    #[arg(long, value_name = "KEY", conflicts_with_all = ["init", "list", "get", "set"])]
    pub unset: Option<String>,
}

pub async fn execute(args: ConfigArgs) -> Result<()> {
    use dox_core::utils::{config::Config, ui};
    
    if args.init {
        ui::print_info("Initializing configuration...");
        Config::init()?;
        ui::print_success("Configuration initialized successfully");
    } else if args.list {
        let config = Config::load()?;
        ui::print_header("Current Configuration");
        println!("{}", config.display());
    } else if let Some(key) = args.get {
        let config = Config::load()?;
        match config.get(&key) {
            Some(value) => println!("{}", value),
            None => ui::print_error(&format!("Configuration key '{}' not found", key)),
        }
    } else if let Some(key_value) = args.set {
        let parts: Vec<&str> = key_value.splitn(2, '=').collect();
        if parts.len() != 2 {
            ui::print_error("Invalid format. Use: --set KEY=VALUE");
            return Ok(());
        }
        
        let mut config = Config::load()?;
        config.set(parts[0], parts[1])?;
        config.save()?;
        ui::print_success(&format!("Set {} = {}", parts[0], parts[1]));
    } else if let Some(key) = args.unset {
        let mut config = Config::load()?;
        config.unset(&key)?;
        config.save()?;
        ui::print_success(&format!("Removed configuration key '{}'", key));
    } else {
        ui::print_info("Use --help to see available configuration commands");
    }
    
    Ok(())
}