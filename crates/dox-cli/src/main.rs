use anyhow::Result;
use clap::Parser;
use tracing::{error, info, debug};
use dox_core::{DoxError, LogConfig, LogFormat, ErrorReporter};

mod cli;

use cli::Cli;

#[tokio::main]
async fn main() -> Result<()> {
    // Parse CLI arguments first to get verbosity level
    let cli = Cli::parse();
    
    // Initialize logging based on CLI flags
    let log_config = get_log_config(&cli);
    let is_verbose = is_verbose(&cli);
    dox_core::init_logging(log_config)?;
    
    info!("Starting dox v{}", env!("CARGO_PKG_VERSION"));
    debug!("Command: {:?}", std::env::args().collect::<Vec<_>>());
    
    // Execute command and handle errors properly
    match cli.execute().await {
        Ok(()) => {
            info!("Command completed successfully");
            Ok(())
        }
        Err(err) => {
            // Try to downcast to DoxError for better error reporting
            if let Some(dox_err) = err.downcast_ref::<DoxError>() {
                error!("Command failed: {}", dox_err);
                ErrorReporter::report(dox_err, is_verbose);
            } else {
                error!("Command failed: {}", err);
                ErrorReporter::report_generic(&*err, "Command execution failed");
            }
            std::process::exit(1);
        }
    }
}

fn get_log_config(cli: &Cli) -> LogConfig {
    // TODO: Add verbose/quiet flags to CLI struct
    // For now, check environment variables
    if std::env::var("DOX_DEBUG").is_ok() {
        LogConfig::verbose()
    } else if std::env::var("DOX_QUIET").is_ok() {
        LogConfig::quiet()
    } else {
        // Check for RUST_LOG environment variable
        let level = std::env::var("RUST_LOG")
            .unwrap_or_else(|_| "info".to_string());
        
        let format = if std::env::var("DOX_LOG_JSON").is_ok() {
            LogFormat::Json
        } else {
            LogFormat::Pretty
        };
        
        LogConfig {
            level,
            format,
            ..Default::default()
        }
    }
}

fn is_verbose(_cli: &Cli) -> bool {
    // TODO: Add verbose flag to CLI struct
    std::env::var("DOX_DEBUG").is_ok() || 
    std::env::var("RUST_LOG").map(|v| v.contains("debug") || v.contains("trace")).unwrap_or(false)
}
