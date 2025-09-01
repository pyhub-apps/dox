# Error Handling and Logging Framework

This document describes the comprehensive error handling and logging framework implemented for the dox Rust application.

## Table of Contents

- [Error Types](#error-types)
- [Error Recovery](#error-recovery)
- [Logging System](#logging-system)
- [Usage Examples](#usage-examples)
- [Best Practices](#best-practices)

## Error Types

### DoxError Enum

The main error type for the application is `DoxError`, which covers all possible error scenarios:

```rust
use dox::error::{DoxError, DoxResult};

// Creating errors
let err = DoxError::file_not_found("/path/to/file");
let err = DoxError::config("Invalid configuration");
let err = DoxError::api_error("OpenAI", "Rate limit exceeded");
```

### Error Categories

1. **File System Errors**
   - `FileNotFound`: File doesn't exist
   - `PermissionDenied`: Insufficient permissions
   - `IoError`: General I/O errors

2. **Document Errors**
   - `InvalidFormat`: Wrong document format
   - `DocumentCorrupted`: Corrupted document file
   - `UnsupportedDocumentType`: Unsupported file extension

3. **Configuration Errors**
   - `ConfigError`: Configuration problems
   - `MissingApiKey`: Missing API credentials

4. **Network Errors**
   - `ApiError`: External API failures
   - `NetworkError`: Network connectivity issues

5. **Processing Errors**
   - `ValidationError`: Input validation failures
   - `TemplateError`: Template processing errors
   - `ParseError`: Parsing failures
   - `ConcurrentError`: Concurrent processing issues

### Error Codes

Each error has an associated error code for programmatic handling:

```rust
let err = DoxError::file_not_found("/path");
let code = err.code(); // ErrorCode::FileNotFound (1001)
```

### User-Friendly Messages

Errors provide helpful suggestions for users:

```rust
let err = DoxError::missing_api_key("OpenAI");
println!("{}", err.user_message());
// Output:
// Missing API key for OpenAI
// 
// Suggestion: Set up your API key using 'dox config api-key OpenAI'
```

## Error Recovery

### Retry Policies

The framework includes sophisticated retry mechanisms for recoverable errors:

```rust
use dox::error::{RetryPolicy, retry_async};

// Network operations with exponential backoff
let policy = RetryPolicy::for_network();
let result = retry_async(&policy, "API call", || async {
    // Your async operation here
    make_api_call().await
}).await;

// File I/O with quick retries
let policy = RetryPolicy::for_file_io();
let result = retry_sync(&policy, "File read", || {
    std::fs::read_to_string("file.txt")
        .map_err(|e| e.into())
});
```

### Circuit Breaker

Prevent cascading failures with circuit breaker pattern:

```rust
use dox::error::{CircuitBreaker, with_circuit_breaker};

let mut breaker = CircuitBreaker::new(
    3,  // failure threshold
    2,  // success threshold to close
    Duration::from_secs(30)  // timeout
);

let result = with_circuit_breaker(&mut breaker, "External API", || async {
    // Your operation here
    external_api_call().await
}).await;
```

### Error Context

Add context to errors for better debugging:

```rust
use dox::error::ErrorContext;

let result = std::fs::read_to_string("config.toml")
    .context("Failed to read configuration file")?;

// Or with lazy evaluation
let result = parse_document(&path)
    .with_context(|| format!("Failed to parse {}", path.display()))?;
```

## Logging System

### Configuration

The logging system supports multiple formats and verbosity levels:

```rust
use dox::logging::{LogConfig, LogFormat};

// Verbose logging for debugging
let config = LogConfig::verbose();

// Quiet mode (errors only)
let config = LogConfig::quiet();

// Custom configuration
let config = LogConfig {
    level: "debug".to_string(),
    format: LogFormat::Pretty,
    include_location: true,
    include_thread: false,
    include_timestamp: true,
    span_events: FmtSpan::CLOSE,
};

logging::init_logging(config)?;
```

### Environment Variables

Control logging behavior through environment variables:

- `RUST_LOG`: Set log level (trace, debug, info, warn, error)
- `DOX_DEBUG`: Enable verbose debugging
- `DOX_QUIET`: Enable quiet mode (errors only)
- `DOX_LOG_JSON`: Use JSON output format

### Structured Logging

Use spans and structured fields for better log analysis:

```rust
use tracing::{info, debug, error, info_span};

let span = info_span!("document_processing", 
    path = %document_path.display(),
    doc_type = "docx"
);

let _guard = span.enter();
info!("Processing document");
debug!("Document size: {} bytes", size);
```

### Progress Logging

Track long-running operations:

```rust
use dox::logging::ProgressLogger;

let progress = ProgressLogger::new("Converting documents");
progress.update("Processing file 1 of 10");
// ... do work ...
progress.complete(); // Logs duration automatically
```

### Error Reporting

Report errors with appropriate formatting:

```rust
use dox::logging::ErrorReporter;

match operation() {
    Err(err) => {
        ErrorReporter::report(&err, verbose_mode);
        // Shows error code, message, and suggestions
    }
    Ok(_) => {}
}
```

## Usage Examples

### Complete Error Handling Example

```rust
use dox::error::{DoxError, DoxResult, ErrorContext, RetryPolicy, retry_async};
use tracing::{info, error};

async fn process_document(path: &Path) -> DoxResult<()> {
    // Add context to file operations
    let content = tokio::fs::read_to_string(path)
        .await
        .context("Failed to read document")?;
    
    // Retry network operations
    let policy = RetryPolicy::for_network();
    let analysis = retry_async(&policy, "AI analysis", || async {
        analyze_with_ai(&content).await
    }).await?;
    
    // Log success
    info!("Document processed successfully");
    Ok(())
}

#[tokio::main]
async fn main() {
    // Initialize logging
    let config = LogConfig::default();
    logging::init_logging(config).unwrap();
    
    // Process with error handling
    if let Err(err) = process_document(Path::new("doc.txt")).await {
        error!("Processing failed: {}", err);
        ErrorReporter::report(&err, false);
        std::process::exit(1);
    }
}
```

### Custom Error Recovery

```rust
use dox::error::{DoxError, DoxResult};

fn process_with_fallback(primary_path: &Path, fallback_path: &Path) -> DoxResult<String> {
    // Try primary source
    match std::fs::read_to_string(primary_path) {
        Ok(content) => Ok(content),
        Err(_) => {
            // Fall back to secondary source
            std::fs::read_to_string(fallback_path)
                .map_err(|_| DoxError::file_not_found(fallback_path))
        }
    }
}
```

## Best Practices

### 1. Always Add Context

```rust
// Bad
let file = std::fs::read_to_string(path)?;

// Good
let file = std::fs::read_to_string(path)
    .context(format!("Failed to read {}", path.display()))?;
```

### 2. Use Appropriate Retry Policies

```rust
// Network operations: longer delays, more attempts
let policy = RetryPolicy::for_network();

// File I/O: quick retries, fewer attempts
let policy = RetryPolicy::for_file_io();

// Critical operations: aggressive retries
let policy = RetryPolicy::aggressive();
```

### 3. Check Error Recoverability

```rust
match operation() {
    Err(err) if err.is_recoverable() => {
        // Retry or attempt recovery
    }
    Err(err) => {
        // Fail immediately
        return Err(err);
    }
    Ok(result) => result,
}
```

### 4. Use Structured Logging

```rust
// Bad
debug!("Processing file");

// Good
debug!(
    file = %path.display(),
    size = file_size,
    "Processing document"
);
```

### 5. Provide User-Friendly Errors

```rust
// Always show suggestions in user-facing errors
let err = DoxError::validation("email", "Invalid format")
    .with_context("Registration failed");
eprintln!("{}", err.user_message());
```

### 6. Use Error Codes for Programmatic Handling

```rust
match err.code() {
    ErrorCode::FileNotFound => {
        // Prompt user to select a different file
    }
    ErrorCode::NetworkError => {
        // Suggest checking internet connection
    }
    _ => {
        // Generic error handling
    }
}
```

## Testing

The error handling system includes comprehensive tests:

```bash
# Run error handling tests
cargo test error --lib

# Run with verbose output
RUST_LOG=debug cargo test error --lib -- --nocapture
```

## Performance Considerations

1. **Error Creation**: Error types are lightweight and cheap to create
2. **Context Addition**: Uses zero-cost abstractions when possible
3. **Retry Delays**: Configurable with jitter to prevent thundering herd
4. **Circuit Breaker**: Minimal overhead for tracking state
5. **Logging**: Async-friendly, doesn't block on I/O

## Future Improvements

- [ ] Add telemetry integration for error tracking
- [ ] Implement error serialization for API responses
- [ ] Add internationalization for error messages
- [ ] Create error recovery middleware for async operations
- [ ] Add metrics collection for error rates