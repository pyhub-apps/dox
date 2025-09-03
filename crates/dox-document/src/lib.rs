//! # dox-document
//!
//! Document processing crate for Word (.docx), PowerPoint (.pptx), Excel (.xlsx), and PDF files.
//! This crate provides implementations of the DocumentProvider trait for
//! Microsoft Office document formats and PDF documents.

pub mod compat;
pub mod excel;
pub mod extract;
pub mod markdown;
pub mod pdf;
pub mod powerpoint;
pub mod provider;
pub mod replace;
pub mod template;
pub mod utils;
pub mod word;

// Re-export main types
pub use excel::ExcelProvider;
pub use pdf::{PdfMetadata, PdfProvider};
pub use powerpoint::PowerPointProvider;
pub use provider::{create_provider, DocumentError, DocumentProvider, DocumentType};
pub use utils::{create_zip, extract_zip, is_office_document};
pub use word::WordProvider;

// Re-export compatibility layer
pub use compat::{Document, DocumentOps};

// Re-export document processing modules
pub use extract::{
    ExtractFormat, ExtractMetadata, ExtractResult, ExtractorFactory, OutputFormatter,
};
pub use markdown::*;
pub use replace::*;
pub use template::*;

/// Result type for document operations
pub type DocumentResult<T> = Result<T, DocumentError>;
