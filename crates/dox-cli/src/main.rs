use anyhow::Result;
use clap::Parser;
use tracing::{error, info, debug};

mod cli;

use cli::Cli;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize basic tracing
    tracing_subscriber::fmt::init();
    
    // Parse CLI arguments
    let cli = Cli::parse();
    
    info!("Starting dox v{}", env!("CARGO_PKG_VERSION"));
    debug!("Command: {:?}", std::env::args().collect::<Vec<_>>());
    
    // Execute command
    match cli.execute().await {
        Ok(()) => {
            info!("Command completed successfully");
            Ok(())
        }
        Err(err) => {
            error!("Command failed: {}", err);
            std::process::exit(1);
        }
    }
}
