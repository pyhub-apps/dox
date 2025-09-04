//! Document-specific extractor implementations

use super::{DocumentExtractor, ExtractMetadata, ExtractResult, ExtractedElement, ExtractedPage};
use crate::provider::{DocumentError, DocumentProvider, DocumentType};
use crate::{ExcelProvider, PdfProvider, PowerPointProvider, TextProvider, WordProvider};
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

        // Extract metadata from Word document
        let word_metadata = provider.get_metadata().unwrap_or_default();
        let metadata = ExtractMetadata {
            title: word_metadata.title,
            author: word_metadata.author,
            subject: word_metadata.subject,
            creator: word_metadata.creator,
            total_pages: word_metadata.total_pages.max(1), // At least 1 page
            created: word_metadata.created,
            modified: word_metadata.modified,
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

        // Extract slides individually
        let slide_count = provider.slide_count();
        let mut pages = Vec::new();

        for slide_index in 0..slide_count {
            let slide_text = provider.get_slide_text(slide_index)?;

            if !slide_text.trim().is_empty() {
                let page = ExtractedPage {
                    number: slide_index + 1,
                    text: slide_text.clone(),
                    elements: vec![ExtractedElement {
                        element_type: "slide".to_string(),
                        content: slide_text,
                        level: None,
                        marker: None,
                    }],
                    tables: vec![], // TODO: Implement table extraction for PowerPoint
                };
                pages.push(page);
            }
        }

        // If no slides had content, create a single empty page
        if pages.is_empty() {
            pages.push(ExtractedPage {
                number: 1,
                text: String::new(),
                elements: vec![],
                tables: vec![],
            });
        }

        // Extract metadata from PowerPoint document
        let ppt_metadata = provider.get_metadata().unwrap_or_default();
        let metadata = ExtractMetadata {
            title: ppt_metadata.title,
            author: ppt_metadata.author,
            subject: ppt_metadata.subject,
            creator: ppt_metadata.creator,
            total_pages: ppt_metadata.total_slides.max(1), // At least 1 page
            created: ppt_metadata.created,
            modified: ppt_metadata.modified,
        };

        Ok(ExtractResult {
            filename: path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
            format: "PowerPoint Presentation (.pptx)".to_string(),
            pages,
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
        debug!("Extracting text from Excel document: {}", path.display());

        let provider = ExcelProvider::open(path)?;

        // Extract sheets individually
        let sheet_names = provider.get_sheet_names()?;
        let mut pages = Vec::new();

        for (sheet_index, sheet_name) in sheet_names.iter().enumerate() {
            let sheet_text = provider.get_sheet_text(sheet_name)?;

            if !sheet_text.trim().is_empty() {
                let page = ExtractedPage {
                    number: sheet_index + 1,
                    text: sheet_text.clone(),
                    elements: vec![ExtractedElement {
                        element_type: "sheet".to_string(),
                        content: sheet_text,
                        level: None,
                        marker: None,
                    }],
                    tables: vec![], // TODO: Implement table extraction for Excel
                };
                pages.push(page);
            }
        }

        // If no sheets had content, create a single empty page
        if pages.is_empty() {
            pages.push(ExtractedPage {
                number: 1,
                text: String::new(),
                elements: vec![],
                tables: vec![],
            });
        }

        // Extract metadata from Excel document
        let excel_metadata = provider.get_metadata().unwrap_or_default();
        let metadata = ExtractMetadata {
            title: excel_metadata.title,
            author: excel_metadata.author,
            subject: excel_metadata.subject,
            creator: excel_metadata.creator,
            total_pages: excel_metadata.total_sheets.max(1), // At least 1 page
            created: excel_metadata.created,
            modified: excel_metadata.modified,
        };

        Ok(ExtractResult {
            filename: path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
            format: "Excel Spreadsheet (.xlsx)".to_string(),
            pages,
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
            created: pdf_metadata.created,
            modified: pdf_metadata.modified,
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
            Some("txt") => Box::new(TextExtractor::new()) as Box<dyn DocumentExtractor>,
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

/// Text document extractor
pub struct TextExtractor;

impl TextExtractor {
    pub fn new() -> Self {
        Self
    }
}

impl DocumentExtractor for TextExtractor {
    fn extract(&self, path: &Path) -> Result<ExtractResult, DocumentError> {
        debug!("Extracting text from plain text file: {}", path.display());

        let provider = TextProvider::open(path)?;
        let text = provider.get_text()?;

        // For text files, we treat the entire file as one page
        let page = ExtractedPage {
            number: 1,
            text: text.clone(),
            elements: vec![ExtractedElement {
                element_type: "text".to_string(),
                content: text,
                level: None,
                marker: None,
            }],
            tables: vec![], // Text files don't have tables
        };

        let metadata = ExtractMetadata {
            title: path
                .file_stem()
                .and_then(|s| s.to_str())
                .map(|s| s.to_string()),
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
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string(),
            format: "text".to_string(),
            pages: vec![page],
            metadata,
            success: true,
            error: None,
        })
    }

    fn supported_types(&self) -> &[DocumentType] {
        &[DocumentType::Text]
    }
}

impl Default for TextExtractor {
    fn default() -> Self {
        Self::new()
    }
}
