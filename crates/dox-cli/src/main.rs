use anyhow::Result;
use clap::Parser;
use colored;
use dox_core::{config::Config, DoxError, ErrorReporter, LogConfig, LogFormat};
use tracing::{debug, error, info};

mod cli;

use cli::Cli;

#[tokio::main]
async fn main() -> Result<()> {
    // Parse CLI arguments first to get verbosity level
    let cli = Cli::parse();

    // Load configuration with priority: --config flag > default config file
    let config = load_config_with_priority(&cli)?;

    // Initialize logging based on CLI flags and config
    let log_config = get_log_config(&cli, &config);
    let is_verbose = is_verbose(&cli, &config);
    dox_core::init_logging(log_config)?;

    // Apply global settings from config
    apply_global_settings(&cli, &config);

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

/// Load configuration with priority: --config flag > default config file > defaults
fn load_config_with_priority(cli: &Cli) -> Result<Config> {
    if let Some(config_path) = &cli.config {
        debug!("사용자 지정 설정 파일 로딩: {:?}", config_path);
        Config::load_from(config_path)
    } else {
        debug!("기본 설정 파일 로딩");
        Config::load()
    }
}

/// Apply global settings from config file
fn apply_global_settings(cli: &Cli, config: &Config) {
    // Apply no_color setting with CLI flag priority
    let no_color = cli.no_color || config.global.no_color || std::env::var("NO_COLOR").is_ok();
    if no_color {
        colored::control::set_override(false);
    }
}

fn get_log_config(cli: &Cli, config: &Config) -> LogConfig {
    // CLI 플래그 우선순위: CLI args > config file > env vars > defaults
    if cli.verbose {
        LogConfig::verbose()
    } else if cli.quiet {
        LogConfig::quiet()
    } else if config.global.verbose {
        LogConfig::verbose()
    } else if config.global.quiet {
        LogConfig::quiet()
    } else if std::env::var("DOX_DEBUG").is_ok() {
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

fn is_verbose(cli: &Cli, config: &Config) -> bool {
    cli.verbose
        || config.global.verbose
        || std::env::var("DOX_DEBUG").is_ok()
        || std::env::var("RUST_LOG")
            .map(|v| v.contains("debug") || v.contains("trace"))
            .unwrap_or(false)
}
