use colored::Colorize;
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    EnvFilter,
};

/// Logging configuration
#[derive(Debug, Clone)]
pub struct LogConfig {
    /// Log level filter (trace, debug, info, warn, error)
    pub level: String,
    /// Output format (plain, json, pretty)
    pub format: LogFormat,
    /// Include file/line information
    pub include_location: bool,
    /// Include thread names
    pub include_thread: bool,
    /// Include timestamps
    pub include_timestamp: bool,
    /// Include span events
    pub span_events: FmtSpan,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            format: LogFormat::Pretty,
            include_location: false,
            include_thread: false,
            include_timestamp: true,
            span_events: FmtSpan::CLOSE,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LogFormat {
    Plain,
    Pretty,
    Json,
}

impl LogConfig {
    /// Create a verbose configuration for debugging
    pub fn verbose() -> Self {
        Self {
            level: "debug".to_string(),
            format: LogFormat::Pretty,
            include_location: true,
            include_thread: true,
            include_timestamp: true,
            span_events: FmtSpan::NEW | FmtSpan::CLOSE,
        }
    }

    /// Create a quiet configuration (errors only)
    pub fn quiet() -> Self {
        Self {
            level: "error".to_string(),
            format: LogFormat::Plain,
            include_location: false,
            include_thread: false,
            include_timestamp: false,
            span_events: FmtSpan::NONE,
        }
    }
}

/// Initialize the logging system with the given configuration
pub fn init_logging(config: LogConfig) -> anyhow::Result<()> {
    let env_filter = build_env_filter(&config.level)?;

    match config.format {
        LogFormat::Plain => init_plain_logging(config, env_filter),
        LogFormat::Pretty => init_pretty_logging(config, env_filter),
        LogFormat::Json => init_json_logging(config, env_filter),
    }

    Ok(())
}

/// Build environment filter from string
fn build_env_filter(level: &str) -> anyhow::Result<EnvFilter> {
    // Try to parse from environment first, then fall back to provided level
    EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new(level))
        .map_err(|e| anyhow::anyhow!("Invalid log level: {}", e))
}

/// Initialize plain text logging
fn init_plain_logging(config: LogConfig, filter: EnvFilter) {
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .with_thread_ids(config.include_thread)
        .with_thread_names(config.include_thread)
        .with_file(config.include_location)
        .with_line_number(config.include_location)
        .with_level(true)
        .with_ansi(false)
        .with_span_events(config.span_events)
        .init();
}

/// Initialize pretty colored logging
fn init_pretty_logging(config: LogConfig, filter: EnvFilter) {
    let format = fmt::format()
        .with_level(true)
        .with_target(false)
        .with_thread_ids(config.include_thread)
        .with_thread_names(config.include_thread)
        .with_file(config.include_location)
        .with_line_number(config.include_location);

    tracing_subscriber::fmt()
        .event_format(format)
        .with_env_filter(filter)
        .with_span_events(config.span_events)
        .with_ansi(true)
        .init();
}

/// Initialize JSON logging
fn init_json_logging(config: LogConfig, filter: EnvFilter) {
    // JSON formatting requires the json feature flag
    // For now, use plain format with structured fields
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(true)
        .with_thread_ids(config.include_thread)
        .with_thread_names(config.include_thread)
        .with_file(config.include_location)
        .with_line_number(config.include_location)
        .with_level(true)
        .with_span_events(config.span_events)
        .init();
}

/// Custom error reporter that formats errors with context
pub struct ErrorReporter;

impl ErrorReporter {
    /// Report an error to the user with formatted output
    pub fn report(error: &crate::error::DoxError, verbose: bool) {
        eprintln!();
        eprintln!("{}", "Error:".red().bold());

        // Show error code if available
        let code = error.code();
        eprintln!(
            "  {} {}",
            "Code:".yellow(),
            format!("{:?}", code).bright_black()
        );

        // Show main error message
        if verbose {
            // Show full error chain in verbose mode
            eprintln!("  {} {}", "Message:".yellow(), error);

            // Show file/line if available through backtrace
            if std::env::var("RUST_BACKTRACE").is_ok() {
                eprintln!("\n{}", "Stack trace:".yellow());
                eprintln!("{:?}", error);
            }
        } else {
            // Show user-friendly message
            let user_msg = error.user_message();
            for line in user_msg.lines() {
                if line.starts_with("Suggestion:") {
                    eprintln!("\n  {} {}", "💡".cyan(), line.cyan());
                } else if !line.is_empty() {
                    eprintln!("  {}", line);
                }
            }
        }

        // Show recovery hint if error is recoverable
        if error.is_recoverable() {
            eprintln!(
                "\n  {} This error may be temporary. Please try again.",
                "↻".green()
            );
        }

        eprintln!();

        // Show help text
        if !verbose {
            eprintln!(
                "For more details, run with {} or set {}",
                "--verbose".bright_blue(),
                "RUST_LOG=debug".bright_blue()
            );
        }
    }

    /// Report a generic error (non-DoxError)
    pub fn report_generic<E: std::error::Error + ?Sized>(error: &E, context: &str) {
        eprintln!();
        eprintln!("{} {}", "Error:".red().bold(), context);
        eprintln!("  {}", error);

        // Print error chain if available
        let mut source = error.source();
        while let Some(err) = source {
            eprintln!("  {} {}", "Caused by:".yellow(), err);
            source = err.source();
        }

        eprintln!();
    }
}

/// Logging macros with structured fields
#[macro_export]
macro_rules! log_operation {
    ($op:expr, $($field:tt)*) => {
        tracing::info_span!("operation", op = $op, $($field)*).in_scope(|| {
            tracing::info!("Starting");
        })
    };
}

#[macro_export]
macro_rules! log_error_with_context {
    ($err:expr, $($field:tt)*) => {
        tracing::error!(
            error = %$err,
            error_code = ?$err.code(),
            recoverable = $err.is_recoverable(),
            $($field)*,
            "Operation failed"
        );
    };
}

/// Progress logging for long-running operations
pub struct ProgressLogger {
    operation: String,
    start_time: std::time::Instant,
}

impl ProgressLogger {
    pub fn new(operation: impl Into<String>) -> Self {
        let operation = operation.into();
        tracing::info!("Starting: {}", operation);
        Self {
            operation,
            start_time: std::time::Instant::now(),
        }
    }

    pub fn update(&self, message: impl std::fmt::Display) {
        tracing::info!("{}: {}", self.operation, message);
    }

    pub fn complete(self) {
        let duration = self.start_time.elapsed();
        tracing::info!(
            "Completed: {} (took {:.2}s)",
            self.operation,
            duration.as_secs_f64()
        );
    }

    pub fn failed(self, error: &crate::error::DoxError) {
        let duration = self.start_time.elapsed();
        tracing::error!(
            "Failed: {} (after {:.2}s) - {}",
            self.operation,
            duration.as_secs_f64(),
            error
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_config_default() {
        let config = LogConfig::default();
        assert_eq!(config.level, "info");
        assert_eq!(config.format, LogFormat::Pretty);
    }

    #[test]
    fn test_log_config_verbose() {
        let config = LogConfig::verbose();
        assert_eq!(config.level, "debug");
        assert!(config.include_location);
        assert!(config.include_thread);
    }

    #[test]
    fn test_log_config_quiet() {
        let config = LogConfig::quiet();
        assert_eq!(config.level, "error");
        assert!(!config.include_location);
        assert!(!config.include_thread);
    }
}
