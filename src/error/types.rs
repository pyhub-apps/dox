use std::fmt;
use std::path::PathBuf;
use thiserror::Error;

/// Main error type for dox application
#[derive(Error, Debug)]
pub enum DoxError {
    #[error("File not found: {path}")]
    FileNotFound { path: PathBuf },
    
    #[error("Permission denied: {path}")]
    PermissionDenied { path: PathBuf },
    
    #[error("Invalid document format: {path} (expected {expected})")]
    InvalidFormat { path: PathBuf, expected: String },
    
    #[error("Document appears to be corrupted: {path}")]
    DocumentCorrupted { path: PathBuf },
    
    #[error("Unsupported document type: {ext}")]
    UnsupportedDocumentType { ext: String },
    
    #[error("Configuration error: {message}")]
    ConfigError { message: String },
    
    #[error("Missing API key for {provider}")]
    MissingApiKey { provider: String },
    
    #[error("API error from {provider}: {message}")]
    ApiError { provider: String, message: String },
    
    #[error("Validation error for {field}: {message}")]
    ValidationError { field: String, message: String },
    
    #[error("Template error: {message}")]
    TemplateError { message: String },
    
    #[error("Parse error: {message}")]
    ParseError { message: String },
    
    #[error("IO error: {message}")]
    IoError { message: String },
    
    #[error("Network error: {message}")]
    NetworkError { message: String },
    
    #[error("Concurrent processing error: {message}")]
    ConcurrentError { message: String },
}

impl DoxError {
    /// Create a file not found error
    pub fn file_not_found(path: impl Into<PathBuf>) -> Self {
        DoxError::FileNotFound { path: path.into() }
    }
    
    /// Create a permission denied error
    pub fn permission_denied(path: impl Into<PathBuf>) -> Self {
        DoxError::PermissionDenied { path: path.into() }
    }
    
    /// Create an invalid format error
    pub fn invalid_format(path: impl Into<PathBuf>, expected: impl Into<String>) -> Self {
        DoxError::InvalidFormat {
            path: path.into(),
            expected: expected.into(),
        }
    }
    
    /// Create a document corrupted error
    pub fn document_corrupted(path: impl Into<PathBuf>) -> Self {
        DoxError::DocumentCorrupted { path: path.into() }
    }
    
    /// Create an unsupported document type error
    pub fn unsupported_type(ext: impl Into<String>) -> Self {
        DoxError::UnsupportedDocumentType { ext: ext.into() }
    }
    
    /// Create a configuration error
    pub fn config(message: impl Into<String>) -> Self {
        DoxError::ConfigError {
            message: message.into(),
        }
    }
    
    /// Create a missing API key error
    pub fn missing_api_key(provider: impl Into<String>) -> Self {
        DoxError::MissingApiKey {
            provider: provider.into(),
        }
    }
    
    /// Create an API error
    pub fn api_error(provider: impl Into<String>, message: impl Into<String>) -> Self {
        DoxError::ApiError {
            provider: provider.into(),
            message: message.into(),
        }
    }
    
    /// Create a validation error
    pub fn validation(field: impl Into<String>, message: impl Into<String>) -> Self {
        DoxError::ValidationError {
            field: field.into(),
            message: message.into(),
        }
    }
}

/// Result type alias using DoxError
pub type DoxResult<T> = Result<T, DoxError>;

/// Error codes for structured error handling
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    FileNotFound = 1001,
    PermissionDenied = 1002,
    InvalidFormat = 1003,
    DocumentCorrupted = 1004,
    UnsupportedType = 1005,
    ConfigError = 2001,
    MissingApiKey = 3001,
    ApiError = 3002,
    ValidationError = 4001,
    TemplateError = 4002,
    ParseError = 4003,
    IoError = 5001,
    NetworkError = 5002,
    ConcurrentError = 6001,
}

impl DoxError {
    /// Get the error code for this error
    pub fn code(&self) -> ErrorCode {
        match self {
            DoxError::FileNotFound { .. } => ErrorCode::FileNotFound,
            DoxError::PermissionDenied { .. } => ErrorCode::PermissionDenied,
            DoxError::InvalidFormat { .. } => ErrorCode::InvalidFormat,
            DoxError::DocumentCorrupted { .. } => ErrorCode::DocumentCorrupted,
            DoxError::UnsupportedDocumentType { .. } => ErrorCode::UnsupportedType,
            DoxError::ConfigError { .. } => ErrorCode::ConfigError,
            DoxError::MissingApiKey { .. } => ErrorCode::MissingApiKey,
            DoxError::ApiError { .. } => ErrorCode::ApiError,
            DoxError::ValidationError { .. } => ErrorCode::ValidationError,
            DoxError::TemplateError { .. } => ErrorCode::TemplateError,
            DoxError::ParseError { .. } => ErrorCode::ParseError,
            DoxError::IoError { .. } => ErrorCode::IoError,
            DoxError::NetworkError { .. } => ErrorCode::NetworkError,
            DoxError::ConcurrentError { .. } => ErrorCode::ConcurrentError,
        }
    }
    
    /// Check if the error is recoverable
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            DoxError::NetworkError { .. } | DoxError::ApiError { .. } | DoxError::ConcurrentError { .. }
        )
    }
}

impl From<std::io::Error> for DoxError {
    fn from(err: std::io::Error) -> Self {
        match err.kind() {
            std::io::ErrorKind::NotFound => {
                DoxError::IoError {
                    message: "File or directory not found".to_string(),
                }
            }
            std::io::ErrorKind::PermissionDenied => {
                DoxError::IoError {
                    message: "Permission denied".to_string(),
                }
            }
            _ => DoxError::IoError {
                message: err.to_string(),
            }
        }
    }
}

impl From<reqwest::Error> for DoxError {
    fn from(err: reqwest::Error) -> Self {
        DoxError::NetworkError {
            message: err.to_string(),
        }
    }
}

impl From<serde_yaml::Error> for DoxError {
    fn from(err: serde_yaml::Error) -> Self {
        DoxError::ParseError {
            message: format!("YAML parse error: {}", err),
        }
    }
}

impl From<serde_json::Error> for DoxError {
    fn from(err: serde_json::Error) -> Self {
        DoxError::ParseError {
            message: format!("JSON parse error: {}", err),
        }
    }
}

impl From<toml::de::Error> for DoxError {
    fn from(err: toml::de::Error) -> Self {
        DoxError::ParseError {
            message: format!("TOML parse error: {}", err),
        }
    }
}

impl From<zip::result::ZipError> for DoxError {
    fn from(err: zip::result::ZipError) -> Self {
        match err {
            zip::result::ZipError::FileNotFound => {
                DoxError::IoError {
                    message: "Archive file not found".to_string(),
                }
            }
            zip::result::ZipError::Io(io_err) => DoxError::from(io_err),
            _ => DoxError::DocumentCorrupted {
                path: PathBuf::from("unknown"),
            }
        }
    }
}

impl From<quick_xml::Error> for DoxError {
    fn from(err: quick_xml::Error) -> Self {
        DoxError::ParseError {
            message: format!("XML parse error: {}", err),
        }
    }
}

impl From<calamine::Error> for DoxError {
    fn from(err: calamine::Error) -> Self {
        match err {
            calamine::Error::Io(io_err) => DoxError::from(io_err),
            _ => DoxError::DocumentCorrupted {
                path: PathBuf::from("unknown"),
            }
        }
    }
}

impl DoxError {
    /// Add context to the error
    pub fn with_context<C>(self, context: C) -> Self
    where
        C: fmt::Display + Send + Sync + 'static,
    {
        match self {
            DoxError::IoError { message } => DoxError::IoError {
                message: format!("{}: {}", context, message),
            },
            DoxError::ParseError { message } => DoxError::ParseError {
                message: format!("{}: {}", context, message),
            },
            DoxError::NetworkError { message } => DoxError::NetworkError {
                message: format!("{}: {}", context, message),
            },
            _ => self,
        }
    }
    
    /// Get user-friendly error message with suggestions
    pub fn user_message(&self) -> String {
        match self {
            DoxError::FileNotFound { path } => {
                format!(
                    "File not found: {}\n\nSuggestion: Check if the file exists and the path is correct.",
                    path.display()
                )
            }
            DoxError::PermissionDenied { path } => {
                format!(
                    "Permission denied: {}\n\nSuggestion: Check file permissions or run with appropriate privileges.",
                    path.display()
                )
            }
            DoxError::InvalidFormat { path, expected } => {
                format!(
                    "Invalid document format: {}\nExpected: {}\n\nSuggestion: Ensure the file is a valid {} document.",
                    path.display(),
                    expected,
                    expected
                )
            }
            DoxError::DocumentCorrupted { path } => {
                format!(
                    "Document appears to be corrupted: {}\n\nSuggestion: Try opening the file in its native application to verify it's not corrupted.",
                    path.display()
                )
            }
            DoxError::UnsupportedDocumentType { ext } => {
                format!(
                    "Unsupported document type: {}\n\nSupported types: .docx, .xlsx, .pptx, .odt, .ods, .odp, .md, .txt",
                    ext
                )
            }
            DoxError::ConfigError { message } => {
                format!(
                    "Configuration error: {}\n\nSuggestion: Check your configuration file or use 'dox config' to set up.",
                    message
                )
            }
            DoxError::MissingApiKey { provider } => {
                format!(
                    "Missing API key for {}\n\nSuggestion: Set up your API key using 'dox config api-key {}'",
                    provider, provider
                )
            }
            DoxError::ApiError { provider, message } => {
                format!(
                    "API error from {}: {}\n\nSuggestion: Check your API credentials and network connection.",
                    provider, message
                )
            }
            DoxError::ValidationError { field, message } => {
                format!(
                    "Validation error for '{}': {}\n\nSuggestion: Review the input requirements for this field.",
                    field, message
                )
            }
            DoxError::TemplateError { message } => {
                format!(
                    "Template error: {}\n\nSuggestion: Check template syntax and variable names.",
                    message
                )
            }
            DoxError::ParseError { message } => {
                format!(
                    "Parse error: {}\n\nSuggestion: Verify the file format and content structure.",
                    message
                )
            }
            DoxError::IoError { message } => {
                format!(
                    "IO error: {}\n\nSuggestion: Check file permissions and available disk space.",
                    message
                )
            }
            DoxError::NetworkError { message } => {
                format!(
                    "Network error: {}\n\nSuggestion: Check your internet connection and try again.",
                    message
                )
            }
            DoxError::ConcurrentError { message } => {
                format!(
                    "Concurrent processing error: {}\n\nSuggestion: Reduce the number of concurrent operations or try again.",
                    message
                )
            }
        }
    }
}

/// Extension trait for adding context to Results
pub trait ErrorContext<T> {
    /// Add context to an error
    fn context<C>(self, context: C) -> Result<T, DoxError>
    where
        C: fmt::Display + Send + Sync + 'static;
    
    /// Add context with a closure (lazy evaluation)
    fn with_context<C, F>(self, f: F) -> Result<T, DoxError>
    where
        C: fmt::Display + Send + Sync + 'static,
        F: FnOnce() -> C;
}

impl<T, E> ErrorContext<T> for Result<T, E>
where
    E: Into<DoxError>,
{
    fn context<C>(self, context: C) -> Result<T, DoxError>
    where
        C: fmt::Display + Send + Sync + 'static,
    {
        self.map_err(|e| e.into().with_context(context))
    }
    
    fn with_context<C, F>(self, f: F) -> Result<T, DoxError>
    where
        C: fmt::Display + Send + Sync + 'static,
        F: FnOnce() -> C,
    {
        self.map_err(|e| e.into().with_context(f()))
    }
}