//! # dox-document
//!
//! Document processing crate for Word (.docx), PowerPoint (.pptx), and PDF files.
//! This crate provides implementations of the DocumentProvider trait for
//! Microsoft Office document formats and PDF documents.

pub mod provider;
pub mod word;
pub mod powerpoint;
pub mod pdf;
pub mod utils;
pub mod compat;
pub mod replace;
pub mod template;
pub mod markdown;
pub mod extract;

// Re-export main types
pub use provider::{DocumentProvider, DocumentError, DocumentType, create_provider};
pub use word::WordProvider;
pub use powerpoint::PowerPointProvider;
pub use pdf::{PdfProvider, PdfMetadata};
pub use utils::{extract_zip, create_zip, is_office_document};

// Re-export compatibility layer
pub use compat::{Document, DocumentOps};

// Re-export document processing modules
pub use replace::*;
pub use template::*;
pub use markdown::*;
pub use extract::{ExtractResult, ExtractFormat, ExtractMetadata, ExtractorFactory, OutputFormatter};

/// Result type for document operations
pub type DocumentResult<T> = Result<T, DocumentError>;