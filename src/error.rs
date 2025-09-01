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