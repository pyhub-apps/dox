//! # dox-document
//!
//! Document processing crate for Word (.docx) and PowerPoint (.pptx) files.
//! This crate provides implementations of the DocumentProvider trait for
//! Microsoft Office document formats.

pub mod provider;
pub mod word;
pub mod powerpoint;
pub mod utils;
pub mod compat;
pub mod replace;
pub mod template;
pub mod markdown;

// Re-export main types
pub use provider::{DocumentProvider, DocumentError, DocumentType, create_provider};
pub use word::WordProvider;
pub use powerpoint::PowerPointProvider;
pub use utils::{extract_zip, create_zip, is_office_document};

// Re-export compatibility layer
pub use compat::{Document, DocumentOps};

// Re-export document processing modules
pub use replace::*;
pub use template::*;
pub use markdown::*;

/// Result type for document operations
pub type DocumentResult<T> = Result<T, DocumentError>;