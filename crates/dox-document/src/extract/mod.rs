//! Text extraction functionality for various document formats

pub mod extractors;

use crate::provider::{DocumentError, DocumentType};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Result of document text extraction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractResult {
    /// Source filename
    pub filename: String,
    /// Document format detected
    pub format: String,
    /// Extracted pages
    pub pages: Vec<ExtractedPage>,
    /// Document metadata
    pub metadata: ExtractMetadata,
    /// Success status
    pub success: bool,
    /// Error message if extraction failed
    pub error: Option<String>,
}

/// A single extracted page
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedPage {
    /// Page number (1-based)
    pub number: usize,
    /// Raw text content
    pub text: String,
    /// Structured elements (headings, paragraphs, etc.)
    pub elements: Vec<ExtractedElement>,
    /// Tables found on this page
    pub tables: Vec<ExtractedTable>,
}

/// A structured text element
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedElement {
    /// Element type (heading, paragraph, list_item, etc.)
    pub element_type: String,
    /// Text content
    pub content: String,
    /// Heading level (for headings)
    pub level: Option<u8>,
    /// List marker (for list items)
    pub marker: Option<String>,
}

/// An extracted table
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedTable {
    /// Table index on page
    pub index: usize,
    /// Table data as rows and columns
    pub data: Vec<Vec<String>>,
    /// Number of rows
    pub rows: usize,
    /// Number of columns
    pub cols: usize,
}

/// Document metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractMetadata {
    /// Document title
    pub title: Option<String>,
    /// Document author
    pub author: Option<String>,
    /// Document subject
    pub subject: Option<String>,
    /// Document creator/producer
    pub creator: Option<String>,
    /// Total number of pages
    pub total_pages: usize,
    /// Creation date
    pub created: Option<String>,
    /// Last modified date
    pub modified: Option<String>,
}

impl Default for ExtractMetadata {
    fn default() -> Self {
        Self {
            title: None,
            author: None,
            subject: None,
            creator: None,
            total_pages: 0,
            created: None,
            modified: None,
        }
    }
}

/// Output format for extracted content
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtractFormat {
    /// Plain text format
    Text,
    /// JSON format with structured data
    Json,
    /// Markdown format preserving structure
    Markdown,
}

impl ExtractFormat {
    /// Get file extension for this format
    pub fn extension(&self) -> &'static str {
        match self {
            ExtractFormat::Text => "txt",
            ExtractFormat::Json => "json",
            ExtractFormat::Markdown => "md",
        }
    }
}

/// Trait for document extractors
pub trait DocumentExtractor: Send + Sync {
    /// Extract content from a document
    fn extract(&self, path: &Path) -> Result<ExtractResult, DocumentError>;

    /// Get supported document types
    fn supported_types(&self) -> &[DocumentType];

    /// Check if this extractor supports the given file
    fn supports_file(&self, path: &Path) -> bool {
        if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
            if let Some(doc_type) = DocumentType::from_extension(ext) {
                return self.supported_types().contains(&doc_type);
            }
        }
        false
    }
}

/// Factory for creating appropriate extractors
pub struct ExtractorFactory;

impl ExtractorFactory {
    /// Create an extractor for the given file
    pub fn create_extractor(path: &Path) -> Result<Box<dyn DocumentExtractor>, DocumentError> {
        let ext = path.extension().and_then(|s| s.to_str()).ok_or_else(|| {
            DocumentError::UnsupportedFormat {
                format: "none".to_string(),
            }
        })?;

        match DocumentType::from_extension(ext) {
            Some(DocumentType::Word) => Ok(Box::new(extractors::WordExtractor::new())),
            Some(DocumentType::PowerPoint) => Ok(Box::new(extractors::PowerPointExtractor::new())),
            Some(DocumentType::Pdf) => Ok(Box::new(extractors::PdfExtractor::new())),
            None => Err(DocumentError::UnsupportedFormat {
                format: ext.to_string(),
            }),
        }
    }

    /// Get all supported file extensions
    pub fn supported_extensions() -> Vec<&'static str> {
        vec!["docx", "pptx", "pdf"]
    }
}

/// Output formatter for different formats
pub struct OutputFormatter;

impl OutputFormatter {
    /// Format extraction result to specified output format
    pub fn format(result: &ExtractResult, format: ExtractFormat) -> Result<String, DocumentError> {
        match format {
            ExtractFormat::Text => Self::format_text(result),
            ExtractFormat::Json => Self::format_json(result),
            ExtractFormat::Markdown => Self::format_markdown(result),
        }
    }

    /// Format as plain text
    fn format_text(result: &ExtractResult) -> Result<String, DocumentError> {
        let mut output = String::new();

        for page in &result.pages {
            if !page.text.trim().is_empty() {
                output.push_str(&page.text);
                if !output.ends_with('\n') {
                    output.push('\n');
                }
            }
        }

        Ok(output)
    }

    /// Format as JSON
    fn format_json(result: &ExtractResult) -> Result<String, DocumentError> {
        serde_json::to_string_pretty(result).map_err(|e| DocumentError::OperationFailed {
            reason: format!("JSON serialization failed: {}", e),
        })
    }

    /// Format as Markdown
    fn format_markdown(result: &ExtractResult) -> Result<String, DocumentError> {
        let mut output = String::new();

        // Add document header if metadata available
        if let Some(ref title) = result.metadata.title {
            output.push_str(&format!("# {}\n\n", title));
        }

        if result.metadata.author.is_some() || result.metadata.creator.is_some() {
            output.push_str("---\n");
            if let Some(ref author) = result.metadata.author {
                output.push_str(&format!("Author: {}\n", author));
            }
            if let Some(ref creator) = result.metadata.creator {
                output.push_str(&format!("Creator: {}\n", creator));
            }
            output.push_str("---\n\n");
        }

        // Process each page
        for (page_idx, page) in result.pages.iter().enumerate() {
            if result.pages.len() > 1 {
                output.push_str(&format!("## Page {}\n\n", page_idx + 1));
            }

            // Process structured elements if available
            if !page.elements.is_empty() {
                for element in &page.elements {
                    match element.element_type.as_str() {
                        "heading" => {
                            let level = element.level.unwrap_or(1).min(6);
                            let hashes = "#".repeat(level as usize);
                            output.push_str(&format!("{} {}\n\n", hashes, element.content));
                        }
                        "paragraph" => {
                            output.push_str(&format!("{}\n\n", element.content));
                        }
                        "list_item" => {
                            let marker = element.marker.as_deref().unwrap_or("*");
                            output.push_str(&format!("{} {}\n", marker, element.content));
                        }
                        _ => {
                            output.push_str(&format!("{}\n\n", element.content));
                        }
                    }
                }
            } else {
                // Fallback to raw text if no structured elements
                output.push_str(&page.text);
                if !output.ends_with('\n') {
                    output.push('\n');
                }
            }

            // Add tables
            for table in &page.tables {
                output.push_str("\n");
                if !table.data.is_empty() {
                    // Add table header
                    if let Some(header_row) = table.data.first() {
                        output.push_str(&format!("| {} |\n", header_row.join(" | ")));
                        output.push_str(&format!("|{}|\n", "---|".repeat(header_row.len())));

                        // Add table rows
                        for row in table.data.iter().skip(1) {
                            output.push_str(&format!("| {} |\n", row.join(" | ")));
                        }
                    }
                }
                output.push_str("\n");
            }

            output.push_str("\n");
        }

        Ok(output)
    }
}
