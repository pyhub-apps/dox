use anyhow::Result;
use clap::Parser;
use dox_core::{DoxError, ErrorReporter, LogConfig, LogFormat};
use tracing::{debug, error, info};

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

    info!("dox v{} 시작", env!("CARGO_PKG_VERSION"));
    debug!("명령어: {:?}", std::env::args().collect::<Vec<_>>());

    // Execute command and handle errors properly
    match cli.execute().await {
        Ok(()) => {
            info!("명령어가 성공적으로 완료되었습니다");
            Ok(())
        }
        Err(err) => {
            // Try to downcast to DoxError for better error reporting
            if let Some(dox_err) = err.downcast_ref::<DoxError>() {
                error!("명령어 실행 실패: {}", dox_err);
                ErrorReporter::report(dox_err, is_verbose);
            } else {
                error!("명령어 실행 실패: {}", err);
                ErrorReporter::report_generic(&*err, "명령어 실행 실패");
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
        let level = std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());

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
    std::env::var("DOX_DEBUG").is_ok()
        || std::env::var("RUST_LOG")
            .map(|v| v.contains("debug") || v.contains("trace"))
            .unwrap_or(false)
}
