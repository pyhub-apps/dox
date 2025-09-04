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
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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

/// Output format for extracted content
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtractFormat {
    /// Plain text format
    Text,
    /// JSON format with structured data
    Json,
    /// Markdown format preserving structure
    Markdown,
    /// HTML format with rich layout
    Html,
}

impl ExtractFormat {
    /// Get file extension for this format
    pub fn extension(&self) -> &'static str {
        match self {
            ExtractFormat::Text => "txt",
            ExtractFormat::Json => "json",
            ExtractFormat::Markdown => "md",
            ExtractFormat::Html => "html",
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
            Some(DocumentType::Excel) => Ok(Box::new(extractors::ExcelExtractor::new())),
            Some(DocumentType::Text) => Ok(Box::new(extractors::TextExtractor::new())),
            None => Err(DocumentError::UnsupportedFormat {
                format: ext.to_string(),
            }),
        }
    }

    /// Get all supported file extensions
    pub fn supported_extensions() -> Vec<&'static str> {
        vec!["docx", "pptx", "pdf", "xlsx", "txt"]
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
            ExtractFormat::Html => Self::format_html(result),
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
                output.push('\n');
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
                output.push('\n');
            }

            output.push('\n');
        }

        Ok(output)
    }

    /// Format as HTML with table and layout preservation
    fn format_html(result: &ExtractResult) -> Result<String, DocumentError> {
        let mut output = String::new();

        // HTML document structure with metadata
        output.push_str("<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n");
        output.push_str("    <meta charset=\"UTF-8\">\n");
        output.push_str(
            "    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n",
        );

        // Document title from metadata
        let title = result.metadata.title.as_deref().unwrap_or(&result.filename);
        output.push_str(&format!(
            "    <title>{}</title>\n",
            Self::html_escape(title)
        ));

        // Embedded CSS for better table and layout rendering
        output.push_str("    <style>\n");
        output.push_str("        body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; line-height: 1.6; margin: 2rem; }\n");
        output.push_str("        .document-header { border-bottom: 2px solid #e2e8f0; margin-bottom: 2rem; padding-bottom: 1rem; }\n");
        output.push_str(
            "        .document-title { font-size: 2rem; margin: 0 0 0.5rem 0; color: #1a202c; }\n",
        );
        output.push_str("        .document-meta { color: #718096; font-size: 0.875rem; }\n");
        output.push_str("        .page { margin-bottom: 3rem; }\n");
        output.push_str("        .page-header { font-size: 1.25rem; font-weight: 600; margin-bottom: 1rem; color: #2d3748; border-left: 4px solid #4299e1; padding-left: 1rem; }\n");
        output.push_str("        .text-element { margin-bottom: 1rem; }\n");
        output.push_str("        .heading-1 { font-size: 1.875rem; font-weight: 700; margin: 2rem 0 1rem 0; }\n");
        output.push_str("        .heading-2 { font-size: 1.5rem; font-weight: 600; margin: 1.5rem 0 1rem 0; }\n");
        output.push_str("        .heading-3 { font-size: 1.25rem; font-weight: 600; margin: 1.25rem 0 0.75rem 0; }\n");
        output.push_str("        .paragraph { margin-bottom: 1rem; text-align: justify; }\n");
        output.push_str("        .list-item { margin-left: 1.5rem; margin-bottom: 0.5rem; }\n");
        output.push_str("        .extracted-table { margin: 2rem 0; overflow-x: auto; }\n");
        output.push_str("        .extracted-table table { width: 100%; border-collapse: collapse; font-size: 0.875rem; background: white; box-shadow: 0 1px 3px rgba(0,0,0,0.1); }\n");
        output.push_str("        .extracted-table th { background-color: #f7fafc; font-weight: 600; padding: 0.75rem; text-align: left; border: 1px solid #e2e8f0; }\n");
        output.push_str(
            "        .extracted-table td { padding: 0.75rem; border: 1px solid #e2e8f0; }\n",
        );
        output.push_str(
            "        .extracted-table tr:nth-child(even) { background-color: #f9fafb; }\n",
        );
        output.push_str("        .extracted-table tr:hover { background-color: #edf2f7; }\n");
        output.push_str(
            "        .table-marker { color: #718096; font-style: italic; margin: 1rem 0; }\n",
        );
        output.push_str("    </style>\n");
        output.push_str("</head>\n<body>\n");

        // Document header with metadata
        output.push_str("    <div class=\"document-header\">\n");
        output.push_str(&format!(
            "        <h1 class=\"document-title\">{}</h1>\n",
            Self::html_escape(title)
        ));

        if result.metadata.author.is_some()
            || result.metadata.creator.is_some()
            || result.metadata.created.is_some()
        {
            output.push_str("        <div class=\"document-meta\">\n");
            if let Some(ref author) = result.metadata.author {
                output.push_str(&format!(
                    "            <div><strong>Author:</strong> {}</div>\n",
                    Self::html_escape(author)
                ));
            }
            if let Some(ref creator) = result.metadata.creator {
                output.push_str(&format!(
                    "            <div><strong>Creator:</strong> {}</div>\n",
                    Self::html_escape(creator)
                ));
            }
            if let Some(ref created) = result.metadata.created {
                output.push_str(&format!(
                    "            <div><strong>Created:</strong> {}</div>\n",
                    Self::html_escape(created)
                ));
            }
            output.push_str(&format!(
                "            <div><strong>Format:</strong> {}</div>\n",
                Self::html_escape(&result.format)
            ));
            output.push_str(&format!(
                "            <div><strong>Pages:</strong> {}</div>\n",
                result.metadata.total_pages
            ));
            output.push_str("        </div>\n");
        }
        output.push_str("    </div>\n");

        // Process each page
        for (page_idx, page) in result.pages.iter().enumerate() {
            output.push_str("    <div class=\"page\">\n");

            if result.pages.len() > 1 {
                output.push_str(&format!(
                    "        <h2 class=\"page-header\">Page {}</h2>\n",
                    page_idx + 1
                ));
            }

            // Process structured elements if available
            if !page.elements.is_empty() {
                for element in &page.elements {
                    output.push_str("        <div class=\"text-element\">\n");

                    match element.element_type.as_str() {
                        "heading" => {
                            let level = element.level.unwrap_or(2).min(6).max(1);
                            let class_name = match level {
                                1 => "heading-1",
                                2 => "heading-2",
                                _ => "heading-3",
                            };
                            output.push_str(&format!(
                                "            <h{} class=\"{}\">{}</h{}>\n",
                                level,
                                class_name,
                                Self::html_escape(&element.content),
                                level
                            ));
                        }
                        "paragraph" => {
                            output.push_str(&format!(
                                "            <p class=\"paragraph\">{}</p>\n",
                                Self::html_escape(&element.content)
                            ));
                        }
                        "list_item" => {
                            let marker = element.marker.as_deref().unwrap_or("•");
                            output.push_str(&format!(
                                "            <div class=\"list-item\">{} {}</div>\n",
                                Self::html_escape(marker),
                                Self::html_escape(&element.content)
                            ));
                        }
                        "table_marker" => {
                            output.push_str(&format!(
                                "            <div class=\"table-marker\">{}</div>\n",
                                Self::html_escape(&element.content)
                            ));
                        }
                        _ => {
                            output.push_str(&format!(
                                "            <div class=\"paragraph\">{}</div>\n",
                                Self::html_escape(&element.content)
                            ));
                        }
                    }

                    output.push_str("        </div>\n");
                }
            } else if !page.text.trim().is_empty() {
                // Fallback to raw text if no structured elements
                let paragraphs: Vec<&str> = page
                    .text
                    .split("\n\n")
                    .filter(|p| !p.trim().is_empty())
                    .collect();
                for paragraph in paragraphs {
                    output.push_str("        <div class=\"text-element\">\n");
                    output.push_str(&format!(
                        "            <p class=\"paragraph\">{}</p>\n",
                        Self::html_escape(&paragraph.trim().replace('\n', " "))
                    ));
                    output.push_str("        </div>\n");
                }
            }

            // Add tables with proper HTML structure
            for (table_idx, table) in page.tables.iter().enumerate() {
                output.push_str(&format!(
                    "        <div class=\"extracted-table\" id=\"table-{}-{}\">\n",
                    page_idx + 1,
                    table_idx + 1
                ));
                output.push_str(&format!(
                    "            <h3>Table {} ({}×{})</h3>\n",
                    table_idx + 1,
                    table.rows,
                    table.cols
                ));

                if !table.data.is_empty() {
                    output.push_str("            <table>\n");

                    // Add table header if available
                    if let Some(header_row) = table.data.first() {
                        output.push_str("                <thead>\n                    <tr>\n");
                        for cell in header_row {
                            output.push_str(&format!(
                                "                        <th>{}</th>\n",
                                Self::html_escape(cell)
                            ));
                        }
                        output.push_str("                    </tr>\n                </thead>\n");

                        // Add table body
                        if table.data.len() > 1 {
                            output.push_str("                <tbody>\n");
                            for row in table.data.iter().skip(1) {
                                output.push_str("                    <tr>\n");
                                for cell in row {
                                    output.push_str(&format!(
                                        "                        <td>{}</td>\n",
                                        Self::html_escape(cell)
                                    ));
                                }
                                output.push_str("                    </tr>\n");
                            }
                            output.push_str("                </tbody>\n");
                        }
                    }

                    output.push_str("            </table>\n");
                } else {
                    output.push_str(
                        "            <p><em>Table detected but no data available</em></p>\n",
                    );
                }

                output.push_str("        </div>\n");
            }

            output.push_str("    </div>\n");
        }

        // Close HTML document
        output.push_str("</body>\n</html>");

        Ok(output)
    }

    /// Escape HTML special characters
    fn html_escape(text: &str) -> String {
        text.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#39;")
    }
}
