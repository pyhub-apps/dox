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

    /// Extract pages from PDF text (legacy method, kept for compatibility)
    #[allow(dead_code)]
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
        debug!(
            "Extracting text from PDF document with advanced features: {}",
            path.display()
        );

        // Use advanced PDF provider
        let provider = PdfProvider::open_layout_critical(path)?;

        // Check if PDF is encrypted
        let encryption_info = provider.check_encryption()?;
        if encryption_info.is_encrypted {
            debug!("PDF is encrypted, attempting authentication");
            if let Some(password) = provider.try_common_passwords()? {
                debug!("Found working password: {}", password);
            }
        }

        // Try advanced text extraction first
        let full_text = match provider.get_advanced_text() {
            Ok(text) => text,
            Err(_) => {
                debug!("Advanced extraction failed, falling back to basic extraction");
                provider.get_text()?
            }
        };

        // Extract advanced pages with tables
        let pages = self.extract_advanced_pages(&provider, &full_text)?;

        // Extract comprehensive metadata
        let pdf_metadata = provider.get_metadata().unwrap_or_default();
        let stats = provider.get_extraction_stats().ok();

        let metadata = ExtractMetadata {
            title: pdf_metadata.title,
            author: pdf_metadata.author,
            subject: pdf_metadata.subject,
            creator: pdf_metadata.creator,
            total_pages: if let Some(ref stats) = stats {
                stats.total_pages
            } else {
                pdf_metadata.page_count
            },
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

impl PdfExtractor {
    /// Extract pages with advanced features (tables, layout information)
    fn extract_advanced_pages(
        &self,
        provider: &PdfProvider,
        full_text: &str,
    ) -> Result<Vec<ExtractedPage>, DocumentError> {
        debug!("Extracting pages with advanced features");

        // Try to extract tables first
        let pdf_tables = provider.extract_tables().unwrap_or_default();

        // Split text into pages (simplified)
        let page_texts = if full_text.contains('\x0C') {
            full_text.split('\x0C').map(|s| s.to_string()).collect()
        } else {
            vec![full_text.to_string()]
        };

        let mut pages = Vec::new();
        for (page_num, page_text) in page_texts.into_iter().enumerate() {
            // Extract text elements for this page
            let elements = self.extract_text_elements(&page_text);

            // Convert PDF tables to extracted tables (simplified distribution)
            let page_tables: Vec<super::ExtractedTable> = if page_num == 0 {
                // Put all tables on first page for simplicity
                pdf_tables
                    .iter()
                    .enumerate()
                    .map(|(idx, pdf_table)| super::ExtractedTable {
                        index: idx,
                        data: pdf_table.data.clone(),
                        rows: pdf_table.rows,
                        cols: pdf_table.cols,
                    })
                    .collect()
            } else {
                vec![]
            };

            pages.push(ExtractedPage {
                number: page_num + 1,
                text: page_text.clone(),
                elements,
                tables: page_tables,
            });
        }

        Ok(pages)
    }

    /// Extract structured text elements from page text
    fn extract_text_elements(&self, page_text: &str) -> Vec<ExtractedElement> {
        let mut elements = Vec::new();
        let lines: Vec<&str> = page_text.lines().collect();

        for line in lines {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            // Detect element type based on content
            let (element_type, level, marker) = self.classify_text_line(trimmed);

            elements.push(ExtractedElement {
                element_type,
                content: trimmed.to_string(),
                level,
                marker,
            });
        }

        elements
    }

    /// Classify a text line into element type
    fn classify_text_line(&self, line: &str) -> (String, Option<u8>, Option<String>) {
        // Check for table markers
        if line == "[TABLE]" || line == "[/TABLE]" {
            return ("table_marker".to_string(), None, None);
        }

        // Check for headings (all caps, short lines)
        if line.len() < 100
            && line
                .chars()
                .filter(|c| c.is_alphabetic())
                .all(|c| c.is_uppercase())
        {
            return ("heading".to_string(), Some(2), None);
        }

        // Check for list items
        if line.starts_with("• ") || line.starts_with("- ") || line.starts_with("* ") {
            let marker = line.chars().next().map(|c| c.to_string());
            return ("list_item".to_string(), None, marker);
        }

        // Default to paragraph
        ("paragraph".to_string(), None, None)
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
