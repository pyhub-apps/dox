//! Document-specific extractor implementations

use super::{DocumentExtractor, ExtractMetadata, ExtractResult, ExtractedElement, ExtractedPage};
use crate::provider::{DocumentError, DocumentProvider, DocumentType};
use crate::{ExcelProvider, PdfProvider, PowerPointProvider, WordProvider};
use std::path::Path;
use tracing::debug;

/// Word document extractor
pub struct WordExtractor;

impl WordExtractor {
    pub fn new() -> Self {
        Self
    }
}

impl DocumentExtractor for WordExtractor {
    fn extract(&self, path: &Path) -> Result<ExtractResult, DocumentError> {
        debug!("Extracting text from Word document: {}", path.display());

        let provider = WordProvider::open(path)?;
        let text = provider.get_text()?;

        // For Word documents, we treat the entire document as one page
        let page = ExtractedPage {
            number: 1,
            text: text.clone(),
            elements: vec![ExtractedElement {
                element_type: "paragraph".to_string(),
                content: text,
                level: None,
                marker: None,
            }],
            tables: vec![], // TODO: Implement table extraction for Word
        };

        let metadata = ExtractMetadata {
            title: None, // TODO: Extract from Word document properties
            author: None,
            subject: None,
            creator: None,
            total_pages: 1,
            created: None,
            modified: None,
        };

        Ok(ExtractResult {
            filename: path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
            format: "Word Document (.docx)".to_string(),
            pages: vec![page],
            metadata,
            success: true,
            error: None,
        })
    }

    fn supported_types(&self) -> &[DocumentType] {
        &[DocumentType::Word]
    }
}

/// PowerPoint document extractor
pub struct PowerPointExtractor;

impl PowerPointExtractor {
    pub fn new() -> Self {
        Self
    }
}

impl DocumentExtractor for PowerPointExtractor {
    fn extract(&self, path: &Path) -> Result<ExtractResult, DocumentError> {
        debug!(
            "Extracting text from PowerPoint document: {}",
            path.display()
        );

        let provider = PowerPointProvider::open(path)?;
        let text = provider.get_text()?;

        // For PowerPoint documents, we treat the entire document as one page for now
        // TODO: Implement proper slide-by-slide extraction
        let page = ExtractedPage {
            number: 1,
            text: text.clone(),
            elements: vec![ExtractedElement {
                element_type: "paragraph".to_string(),
                content: text,
                level: None,
                marker: None,
            }],
            tables: vec![], // TODO: Implement table extraction for PowerPoint
        };

        let metadata = ExtractMetadata {
            title: None, // TODO: Extract from PowerPoint document properties
            author: None,
            subject: None,
            creator: None,
            total_pages: 1, // TODO: Count actual slides
            created: None,
            modified: None,
        };

        Ok(ExtractResult {
            filename: path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
            format: "PowerPoint Presentation (.pptx)".to_string(),
            pages: vec![page],
            metadata,
            success: true,
            error: None,
        })
    }

    fn supported_types(&self) -> &[DocumentType] {
        &[DocumentType::PowerPoint]
    }
}

/// Excel document extractor
pub struct ExcelExtractor;

impl ExcelExtractor {
    pub fn new() -> Self {
        Self
    }
}

impl DocumentExtractor for ExcelExtractor {
    fn extract(&self, path: &Path) -> Result<ExtractResult, DocumentError> {
        debug!(
            "Extracting text from Excel document: {}",
            path.display()
        );

        let provider = ExcelProvider::open(path)?;
        let text = provider.get_text()?;

        // For Excel documents, we treat the entire document as one page
        // TODO: Implement proper sheet-by-sheet extraction
        let page = ExtractedPage {
            number: 1,
            text: text.clone(),
            elements: vec![ExtractedElement {
                element_type: "data".to_string(),
                content: text,
                level: None,
                marker: None,
            }],
            tables: vec![], // TODO: Implement table extraction for Excel
        };

        let metadata = ExtractMetadata {
            title: None, // TODO: Extract from Excel document properties
            author: None,
            subject: None,
            creator: None,
            total_pages: 1, // TODO: Count actual sheets
            created: None,
            modified: None,
        };

        Ok(ExtractResult {
            filename: path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
            format: "Excel Spreadsheet (.xlsx)".to_string(),
            pages: vec![page],
            metadata,
            success: true,
            error: None,
        })
    }

    fn supported_types(&self) -> &[DocumentType] {
        &[DocumentType::Excel]
    }
}

/// PDF document extractor
pub struct PdfExtractor;

impl PdfExtractor {
    pub fn new() -> Self {
        Self
    }

    /// Extract pages from PDF text
    fn split_into_pages(&self, full_text: &str) -> Vec<ExtractedPage> {
        // Simple page splitting - look for form feed characters or page break patterns
        let pages: Vec<&str> = if full_text.contains('\x0C') {
            // Split by form feed character
            full_text.split('\x0C').collect()
        } else {
            // If no clear page breaks, treat as single page
            vec![full_text]
        };

        pages
            .into_iter()
            .enumerate()
            .filter(|(_, text)| !text.trim().is_empty())
            .map(|(index, text)| {
                let clean_text = text.trim().to_string();

                // Basic element extraction - split into paragraphs
                let elements: Vec<ExtractedElement> = clean_text
                    .split("\n\n")
                    .filter(|para| !para.trim().is_empty())
                    .map(|para| {
                        let para_text = para.trim().replace('\n', " ");

                        // Simple heuristic for detecting headings
                        let is_heading = para_text.len() < 100
                            && para_text.chars().any(|c| c.is_uppercase())
                            && !para_text.ends_with('.');

                        ExtractedElement {
                            element_type: if is_heading { "heading" } else { "paragraph" }
                                .to_string(),
                            content: para_text,
                            level: if is_heading { Some(2) } else { None },
                            marker: None,
                        }
                    })
                    .collect();

                ExtractedPage {
                    number: index + 1,
                    text: clean_text,
                    elements,
                    tables: vec![], // TODO: Implement table extraction for PDF
                }
            })
            .collect()
    }
}

impl DocumentExtractor for PdfExtractor {
    fn extract(&self, path: &Path) -> Result<ExtractResult, DocumentError> {
        debug!("Extracting text from PDF document: {}", path.display());

        let provider = PdfProvider::open(path)?;

        // Extract text content
        let full_text = provider.get_text()?;
        let pages = self.split_into_pages(&full_text);

        // Extract metadata
        let pdf_metadata = provider.get_metadata().unwrap_or_default();
        let metadata = ExtractMetadata {
            title: pdf_metadata.title,
            author: pdf_metadata.author,
            subject: pdf_metadata.subject,
            creator: pdf_metadata.creator,
            total_pages: pdf_metadata.page_count,
            created: None,  // TODO: Extract creation date
            modified: None, // TODO: Extract modification date
        };

        Ok(ExtractResult {
            filename: path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
            format: "PDF Document (.pdf)".to_string(),
            pages,
            metadata,
            success: true,
            error: None,
        })
    }

    fn supported_types(&self) -> &[DocumentType] {
        &[DocumentType::Pdf]
    }
}

/// Multi-format extractor that can handle any supported document type
pub struct UniversalExtractor;

impl UniversalExtractor {
    pub fn new() -> Self {
        Self
    }

    /// Extract from any supported document format
    pub fn extract_from_path(path: &Path) -> Result<ExtractResult, DocumentError> {
        let extractor = match path.extension().and_then(|s| s.to_str()) {
            Some("docx") => Box::new(WordExtractor::new()) as Box<dyn DocumentExtractor>,
            Some("pptx") => Box::new(PowerPointExtractor::new()) as Box<dyn DocumentExtractor>,
            Some("pdf") => Box::new(PdfExtractor::new()) as Box<dyn DocumentExtractor>,
            Some("xlsx") => Box::new(ExcelExtractor::new()) as Box<dyn DocumentExtractor>,
            Some(ext) => {
                return Err(DocumentError::UnsupportedFormat {
                    format: ext.to_string(),
                })
            }
            None => {
                return Err(DocumentError::UnsupportedFormat {
                    format: "none".to_string(),
                })
            }
        };

        extractor.extract(path)
    }
}

impl Default for WordExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for PowerPointExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for ExcelExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for PdfExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for UniversalExtractor {
    fn default() -> Self {
        Self::new()
    }
}
