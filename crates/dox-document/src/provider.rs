//! Document provider trait and error types

use anyhow::Result;
use std::path::Path;
use thiserror::Error;

/// Errors that can occur during document operations
#[derive(Error, Debug)]
pub enum DocumentError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("ZIP archive error: {0}")]
    Zip(#[from] zip::result::ZipError),

    #[error("XML parsing error: {0}")]
    Xml(#[from] quick_xml::Error),

    #[error("Unsupported document format: {format}")]
    UnsupportedFormat { format: String },

    #[error("File read error for {path}: {source}")]
    FileReadError {
        path: String,
        #[source]
        source: anyhow::Error,
    },

    #[error("Document not found: {path}")]
    DocumentNotFound { path: String },

    #[error("Document is read-only")]
    ReadOnly,

    #[error("Invalid document structure: {reason}")]
    InvalidStructure { reason: String },

    #[error("Operation failed: {reason}")]
    OperationFailed { reason: String },
}

/// Trait for document operations
pub trait DocumentProvider: std::fmt::Debug {
    /// Replace text in the document
    ///
    /// # Arguments
    /// * `old` - The text to replace
    /// * `new` - The replacement text
    ///
    /// # Returns
    /// The number of replacements made
    fn replace_text(&mut self, old: &str, new: &str) -> Result<usize, DocumentError>;

    /// Save the document to its original location
    fn save(&self) -> Result<(), DocumentError>;

    /// Save the document to a different path
    ///
    /// # Arguments
    /// * `path` - The path where to save the document
    fn save_as(&self, path: &Path) -> Result<(), DocumentError>;

    /// Get the document content as plain text
    fn get_text(&self) -> Result<String, DocumentError>;

    /// Check if the document has been modified
    fn is_modified(&self) -> bool;

    /// Get the document file path
    fn get_path(&self) -> &Path;

    /// Get document type information
    fn document_type(&self) -> DocumentType;
}

/// Document type enumeration
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DocumentType {
    Word,
    PowerPoint,
    Pdf,
    Excel,
    Text,
}

impl DocumentType {
    /// Get file extensions associated with this document type
    pub fn extensions(&self) -> &[&str] {
        match self {
            DocumentType::Word => &["docx"],
            DocumentType::PowerPoint => &["pptx"],
            DocumentType::Pdf => &["pdf"],
            DocumentType::Excel => &["xlsx"],
            DocumentType::Text => &["txt"],
        }
    }

    /// Check if a file extension matches this document type
    pub fn matches_extension(&self, ext: &str) -> bool {
        self.extensions().contains(&ext.to_lowercase().as_str())
    }

    /// Get document type from file extension
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "docx" => Some(DocumentType::Word),
            "pptx" => Some(DocumentType::PowerPoint),
            "pdf" => Some(DocumentType::Pdf),
            "xlsx" => Some(DocumentType::Excel),
            "txt" => Some(DocumentType::Text),
            _ => None,
        }
    }
}

/// Factory function to create appropriate document provider
pub fn create_provider(path: &Path) -> Result<Box<dyn DocumentProvider>, DocumentError> {
    let ext = path.extension().and_then(|s| s.to_str()).ok_or_else(|| {
        DocumentError::UnsupportedFormat {
            format: "none".to_string(),
        }
    })?;

    match DocumentType::from_extension(ext) {
        Some(DocumentType::Word) => Ok(Box::new(crate::WordProvider::open(path)?)),
        Some(DocumentType::PowerPoint) => Ok(Box::new(crate::PowerPointProvider::open(path)?)),
        Some(DocumentType::Pdf) => Ok(Box::new(crate::pdf::PdfProvider::open(path)?)),
        Some(DocumentType::Excel) => Ok(Box::new(crate::ExcelProvider::open(path)?)),
        Some(DocumentType::Text) => Ok(Box::new(crate::text::TextProvider::open(path)?)),
        None => Err(DocumentError::UnsupportedFormat {
            format: ext.to_string(),
        }),
    }
}
