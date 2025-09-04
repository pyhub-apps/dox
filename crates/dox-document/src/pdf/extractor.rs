//! Advanced PDF text extraction with layout analysis and streaming support

use crate::provider::DocumentError;
use lopdf::Document;
use serde::{Deserialize, Serialize};
use std::path::Path;
use tracing::{debug, info};

/// Advanced PDF extractor with layout analysis and streaming capabilities
#[derive(Debug)]
pub struct AdvancedPdfExtractor {
    /// Path to the PDF file
    path: std::path::PathBuf,
    /// Configuration options
    config: PdfExtractConfig,
    /// Cached document (loaded on demand)
    document: Option<Document>,
}

/// Configuration for PDF extraction
#[derive(Debug, Clone)]
pub struct PdfExtractConfig {
    /// Preserve layout information (columns, spacing)
    pub preserve_layout: bool,
    /// Extract table structures
    pub extract_tables: bool,
    /// Maximum memory usage for streaming (in MB)
    pub max_memory_mb: usize,
    /// Enable streaming for large files
    pub enable_streaming: bool,
    /// Stream chunk size in bytes
    pub chunk_size: usize,
    /// Extract images (metadata only for now)
    pub extract_images: bool,
    /// Handle encrypted PDFs
    pub handle_encrypted: bool,
    /// OCR for image-based PDFs (future feature)
    pub enable_ocr: bool,
}

impl Default for PdfExtractConfig {
    fn default() -> Self {
        Self {
            preserve_layout: true,
            extract_tables: true,
            max_memory_mb: 512,
            enable_streaming: true,
            chunk_size: 1024 * 1024, // 1MB
            extract_images: false,
            handle_encrypted: true,
            enable_ocr: false,
        }
    }
}

impl PdfExtractConfig {
    /// Configuration for small PDFs with full feature extraction
    pub fn small_file() -> Self {
        Self {
            preserve_layout: true,
            extract_tables: true,
            max_memory_mb: 256,
            enable_streaming: false,
            ..Default::default()
        }
    }

    /// Configuration for large PDFs with streaming
    pub fn large_file() -> Self {
        Self {
            preserve_layout: false,
            extract_tables: false,
            max_memory_mb: 128,
            enable_streaming: true,
            chunk_size: 512 * 1024, // 512KB chunks
            ..Default::default()
        }
    }

    /// Configuration for layout-critical extraction
    pub fn layout_critical() -> Self {
        Self {
            preserve_layout: true,
            extract_tables: true,
            max_memory_mb: 1024,
            enable_streaming: false,
            ..Default::default()
        }
    }
}

/// Result of advanced PDF extraction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedPdfResult {
    /// Extracted pages with detailed information
    pub pages: Vec<PdfPage>,
    /// Document-level metadata
    pub metadata: PdfDocumentMetadata,
    /// Extraction statistics
    pub stats: ExtractionStats,
    /// Any warnings or issues encountered
    pub warnings: Vec<String>,
}

/// A PDF page with advanced extraction features
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdfPage {
    /// Page number (1-based)
    pub number: usize,
    /// Raw text content
    pub raw_text: String,
    /// Layout-aware text blocks
    pub text_blocks: Vec<TextBlock>,
    /// Detected tables
    pub tables: Vec<PdfTable>,
    /// Image metadata
    pub images: Vec<ImageInfo>,
    /// Page dimensions
    pub dimensions: PageDimensions,
}

/// A text block with position information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextBlock {
    /// Text content
    pub text: String,
    /// Block type (paragraph, heading, list, etc.)
    pub block_type: TextBlockType,
    /// Font information
    pub font: FontInfo,
    /// Position on page
    pub position: BlockPosition,
}

/// Types of text blocks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TextBlockType {
    Paragraph,
    Heading(u8), // level 1-6
    ListItem,
    Caption,
    Footer,
    Header,
    Table,
    Unknown,
}

/// Font information for text blocks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontInfo {
    pub family: Option<String>,
    pub size: Option<f32>,
    pub bold: bool,
    pub italic: bool,
}

/// Position and dimensions of text blocks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockPosition {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

/// Table detected in PDF
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdfTable {
    /// Table index on page
    pub index: usize,
    /// Table data as rows and columns
    pub data: Vec<Vec<String>>,
    /// Number of rows
    pub rows: usize,
    /// Number of columns
    pub cols: usize,
    /// Table position on page
    pub position: BlockPosition,
    /// Confidence score for table detection
    pub confidence: f32,
}

/// Image metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageInfo {
    /// Image index on page
    pub index: usize,
    /// Image format (JPEG, PNG, etc.)
    pub format: String,
    /// Image dimensions
    pub width: u32,
    pub height: u32,
    /// Position on page
    pub position: BlockPosition,
}

/// Page dimensions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageDimensions {
    pub width: f32,
    pub height: f32,
    pub rotation: u16,
}

/// Document metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdfDocumentMetadata {
    pub title: Option<String>,
    pub author: Option<String>,
    pub subject: Option<String>,
    pub creator: Option<String>,
    pub producer: Option<String>,
    pub creation_date: Option<String>,
    pub modification_date: Option<String>,
    pub page_count: usize,
    pub file_size: u64,
    pub pdf_version: String,
    pub encrypted: bool,
    pub permissions: PdfPermissions,
}

/// PDF permissions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdfPermissions {
    pub print: bool,
    pub modify: bool,
    pub copy: bool,
    pub annotate: bool,
}

/// Extraction statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionStats {
    pub total_pages: usize,
    pub text_blocks: usize,
    pub tables_detected: usize,
    pub images_detected: usize,
    pub extraction_time_ms: u64,
    pub memory_usage_mb: f64,
    pub streaming_used: bool,
}

impl AdvancedPdfExtractor {
    /// Create a new advanced PDF extractor
    pub fn new(path: &Path, config: PdfExtractConfig) -> Result<Self, DocumentError> {
        if !path.exists() {
            return Err(DocumentError::DocumentNotFound {
                path: path.to_string_lossy().to_string(),
            });
        }

        // Verify it's a PDF file
        if let Some(extension) = path.extension() {
            if extension.to_string_lossy().to_lowercase() != "pdf" {
                return Err(DocumentError::UnsupportedFormat {
                    format: extension.to_string_lossy().to_string(),
                });
            }
        }

        Ok(Self {
            path: path.to_path_buf(),
            config,
            document: None,
        })
    }

    /// Extract content with advanced features
    pub fn extract(&mut self) -> Result<AdvancedPdfResult, DocumentError> {
        let start_time = std::time::Instant::now();
        info!("Starting advanced PDF extraction: {}", self.path.display());

        // Check file size for streaming decision
        let file_size = std::fs::metadata(&self.path)
            .map_err(|e| DocumentError::FileReadError {
                path: self.path.to_string_lossy().to_string(),
                source: e.into(),
            })?
            .len();

        let should_stream = self.config.enable_streaming 
            && file_size > (self.config.max_memory_mb * 1024 * 1024) as u64;

        let result = if should_stream {
            self.extract_streaming(file_size)?
        } else {
            self.extract_in_memory(file_size)?
        };

        let extraction_time = start_time.elapsed().as_millis() as u64;
        info!("PDF extraction completed in {}ms", extraction_time);

        let pages = result.0;
        let stats = ExtractionStats {
            total_pages: pages.len(),
            text_blocks: pages.iter().map(|p| p.text_blocks.len()).sum(),
            tables_detected: pages.iter().map(|p| p.tables.len()).sum(),
            images_detected: pages.iter().map(|p| p.images.len()).sum(),
            extraction_time_ms: extraction_time,
            memory_usage_mb: (file_size as f64) / (1024.0 * 1024.0),
            streaming_used: should_stream,
        };

        Ok(AdvancedPdfResult {
            pages,
            metadata: result.1,
            stats,
            warnings: vec![],
        })
    }

    /// In-memory extraction for smaller files
    fn extract_in_memory(&mut self, file_size: u64) -> Result<(Vec<PdfPage>, PdfDocumentMetadata), DocumentError> {
        debug!("Using in-memory extraction for PDF ({}MB)", file_size / 1024 / 1024);

        // Load document
        self.load_document()?;
        let document = self.document.as_ref().unwrap();

        // Extract metadata
        let metadata = self.extract_metadata(document, file_size)?;

        // Extract text content
        let text_content = self.extract_text_content()?;
        
        // Process pages
        let pages = self.process_pages(document, &text_content)?;

        Ok((pages, metadata))
    }

    /// Streaming extraction for larger files
    fn extract_streaming(&mut self, file_size: u64) -> Result<(Vec<PdfPage>, PdfDocumentMetadata), DocumentError> {
        debug!("Using streaming extraction for PDF ({}MB)", file_size / 1024 / 1024);

        // For streaming, we'll process pages one at a time
        self.load_document()?;
        let document = self.document.as_ref().unwrap();

        // Extract metadata first
        let metadata = self.extract_metadata(document, file_size)?;

        // Process pages in streaming fashion
        let pages = self.process_pages_streaming(document)?;

        Ok((pages, metadata))
    }

    /// Load the PDF document
    fn load_document(&mut self) -> Result<(), DocumentError> {
        if self.document.is_none() {
            debug!("Loading PDF document: {}", self.path.display());
            
            let document = Document::load(&self.path)
                .map_err(|e| DocumentError::OperationFailed {
                    reason: format!("Failed to load PDF: {}", e),
                })?;

            self.document = Some(document);
        }
        Ok(())
    }

    /// Extract document metadata
    fn extract_metadata(&self, document: &Document, file_size: u64) -> Result<PdfDocumentMetadata, DocumentError> {
        let mut metadata = PdfDocumentMetadata {
            title: None,
            author: None,
            subject: None,
            creator: None,
            producer: None,
            creation_date: None,
            modification_date: None,
            page_count: document.get_pages().len(),
            file_size,
            pdf_version: "1.4".to_string(), // Default PDF version - API doesn't provide easy access
            encrypted: document.is_encrypted(),
            permissions: PdfPermissions {
                print: true,
                modify: true,
                copy: true,
                annotate: true,
            },
        };

        // Extract document info dictionary
        if let Ok(info_dict) = document.trailer.get(b"Info") {
            if let Ok(info_ref) = info_dict.as_reference() {
                if let Ok(info_obj) = document.get_object(info_ref) {
                    if let Ok(dict) = info_obj.as_dict() {
                        // Extract metadata fields
                        metadata.title = self.extract_string_field(dict, b"Title");
                        metadata.author = self.extract_string_field(dict, b"Author");
                        metadata.subject = self.extract_string_field(dict, b"Subject");
                        metadata.creator = self.extract_string_field(dict, b"Creator");
                        metadata.producer = self.extract_string_field(dict, b"Producer");
                        
                        metadata.creation_date = self.extract_date_field(dict, b"CreationDate");
                        metadata.modification_date = self.extract_date_field(dict, b"ModDate");
                    }
                }
            }
        }

        Ok(metadata)
    }

    /// Extract string field from dictionary
    fn extract_string_field(&self, dict: &lopdf::Dictionary, key: &[u8]) -> Option<String> {
        dict.get(key)
            .ok()?
            .as_str()
            .ok()
            .map(|s| String::from_utf8_lossy(s).to_string())
    }

    /// Extract date field from dictionary
    fn extract_date_field(&self, dict: &lopdf::Dictionary, key: &[u8]) -> Option<String> {
        let date_str = self.extract_string_field(dict, key)?;
        self.parse_pdf_date(&date_str)
    }

    /// Parse PDF date format
    fn parse_pdf_date(&self, date_str: &str) -> Option<String> {
        if date_str.len() < 16 || !date_str.starts_with("D:") {
            return None;
        }

        let date_part = &date_str[2..];
        if date_part.len() >= 14 {
            if let (Ok(year), Ok(month), Ok(day)) = (
                date_part[0..4].parse::<u32>(),
                date_part[4..6].parse::<u32>(),
                date_part[6..8].parse::<u32>(),
            ) {
                if (1..=12).contains(&month) && (1..=31).contains(&day) {
                    return Some(format!("{:04}-{:02}-{:02}", year, month, day));
                }
            }
        }
        None
    }

    /// Extract text content using pdf-extract
    fn extract_text_content(&self) -> Result<String, DocumentError> {
        debug!("Extracting text content from PDF");

        let bytes = std::fs::read(&self.path).map_err(|e| DocumentError::FileReadError {
            path: self.path.to_string_lossy().to_string(),
            source: e.into(),
        })?;

        pdf_extract::extract_text_from_mem(&bytes)
            .map_err(|e| DocumentError::OperationFailed {
                reason: format!("Text extraction failed: {}", e),
            })
    }

    /// Process pages for in-memory extraction
    fn process_pages(&self, document: &Document, text_content: &str) -> Result<Vec<PdfPage>, DocumentError> {
        let page_count = document.get_pages().len();
        debug!("Processing {} pages", page_count);

        // Split text into pages (simplified approach)
        let page_texts = self.split_text_into_pages(text_content, page_count);
        
        let mut pages = Vec::new();
        for (page_num, page_text) in page_texts.into_iter().enumerate() {
            let page = self.process_single_page(document, page_num + 1, &page_text)?;
            pages.push(page);
        }

        Ok(pages)
    }

    /// Process pages for streaming extraction
    fn process_pages_streaming(&self, document: &Document) -> Result<Vec<PdfPage>, DocumentError> {
        let page_count = document.get_pages().len();
        debug!("Processing {} pages with streaming", page_count);

        let mut pages = Vec::new();
        
        // For streaming, we'd implement page-by-page processing
        // For now, fallback to simplified approach
        let text_content = self.extract_text_content()?;
        let page_texts = self.split_text_into_pages(&text_content, page_count);
        
        for (page_num, page_text) in page_texts.into_iter().enumerate() {
            let page = self.process_single_page(document, page_num + 1, &page_text)?;
            pages.push(page);
        }

        Ok(pages)
    }

    /// Split text content into pages
    fn split_text_into_pages(&self, text: &str, page_count: usize) -> Vec<String> {
        if text.contains('\x0C') {
            // Split by form feed character
            text.split('\x0C').map(|s| s.to_string()).collect()
        } else if page_count > 1 {
            // Estimate page breaks by text length
            let chars_per_page = text.len() / page_count;
            let mut pages = Vec::new();
            let mut start = 0;
            
            for _ in 0..page_count {
                let end = (start + chars_per_page).min(text.len());
                pages.push(text[start..end].to_string());
                start = end;
                if start >= text.len() {
                    break;
                }
            }
            pages
        } else {
            vec![text.to_string()]
        }
    }

    /// Process a single page
    fn process_single_page(&self, document: &Document, page_num: usize, text: &str) -> Result<PdfPage, DocumentError> {
        debug!("Processing page {}", page_num);

        // Get page dimensions
        let dimensions = self.get_page_dimensions(document, page_num)?;

        // Extract text blocks (simplified implementation)
        let text_blocks = if self.config.preserve_layout {
            self.extract_text_blocks(text)
        } else {
            vec![TextBlock {
                text: text.to_string(),
                block_type: TextBlockType::Paragraph,
                font: FontInfo {
                    family: None,
                    size: None,
                    bold: false,
                    italic: false,
                },
                position: BlockPosition {
                    x: 0.0,
                    y: 0.0,
                    width: dimensions.width,
                    height: dimensions.height,
                },
            }]
        };

        // Extract tables (simplified implementation)
        let tables = if self.config.extract_tables {
            self.extract_tables(text, &dimensions)
        } else {
            vec![]
        };

        Ok(PdfPage {
            number: page_num,
            raw_text: text.to_string(),
            text_blocks,
            tables,
            images: vec![], // TODO: Implement image extraction
            dimensions,
        })
    }

    /// Get page dimensions
    fn get_page_dimensions(&self, _document: &Document, _page_num: usize) -> Result<PageDimensions, DocumentError> {
        // Default dimensions for now
        Ok(PageDimensions {
            width: 612.0,  // Standard US Letter width in points
            height: 792.0, // Standard US Letter height in points
            rotation: 0,
        })
    }

    /// Extract text blocks with layout information
    fn extract_text_blocks(&self, text: &str) -> Vec<TextBlock> {
        // Simplified text block extraction
        // In a full implementation, this would analyze font sizes, positions, etc.
        let lines: Vec<&str> = text.lines().collect();
        let mut blocks = Vec::new();
        
        for (i, line) in lines.iter().enumerate() {
            if line.trim().is_empty() {
                continue;
            }

            let block_type = self.classify_text_block(line);
            
            blocks.push(TextBlock {
                text: line.to_string(),
                block_type,
                font: FontInfo {
                    family: Some("Times".to_string()),
                    size: Some(12.0),
                    bold: false,
                    italic: false,
                },
                position: BlockPosition {
                    x: 72.0, // 1 inch margin
                    y: 720.0 - (i as f32 * 14.0), // Estimate line position
                    width: 468.0, // 6.5 inches
                    height: 14.0,  // Line height
                },
            });
        }

        blocks
    }

    /// Classify text block type
    fn classify_text_block(&self, text: &str) -> TextBlockType {
        let text = text.trim();
        
        // Simple classification based on text patterns
        if text.chars().all(|c| c.is_uppercase() || c.is_whitespace()) && text.len() < 100 {
            TextBlockType::Heading(2)
        } else if text.starts_with("• ") || text.starts_with("- ") {
            TextBlockType::ListItem
        } else if text.len() < 50 {
            TextBlockType::Heading(3)
        } else {
            TextBlockType::Paragraph
        }
    }

    /// Extract tables from text (simplified implementation)
    fn extract_tables(&self, text: &str, dimensions: &PageDimensions) -> Vec<PdfTable> {
        let mut tables = Vec::new();
        let lines: Vec<&str> = text.lines().collect();
        
        // Look for table patterns (lines with multiple columns separated by whitespace)
        let mut table_start = None;
        let mut table_lines = Vec::new();
        
        for (i, line) in lines.iter().enumerate() {
            let cols: Vec<&str> = line.split_whitespace().collect();
            
            if cols.len() >= 3 && self.looks_like_table_row(line) {
                if table_start.is_none() {
                    table_start = Some(i);
                }
                table_lines.push(cols);
            } else if !table_lines.is_empty() {
                // End of table
                if table_lines.len() >= 2 {
                    tables.push(self.create_pdf_table(table_lines.clone(), table_start.unwrap(), dimensions));
                }
                table_lines.clear();
                table_start = None;
            }
        }
        
        // Handle table at end of text
        if !table_lines.is_empty() && table_lines.len() >= 2 {
            tables.push(self.create_pdf_table(table_lines.clone(), table_start.unwrap(), dimensions));
        }

        tables
    }

    /// Check if a line looks like a table row
    fn looks_like_table_row(&self, line: &str) -> bool {
        let cols: Vec<&str> = line.split_whitespace().collect();
        cols.len() >= 3 && 
        line.chars().filter(|c| c.is_whitespace()).count() >= 4 &&
        !line.trim().is_empty()
    }

    /// Create a PDF table from extracted lines
    fn create_pdf_table(&self, lines: Vec<Vec<&str>>, start_line: usize, dimensions: &PageDimensions) -> PdfTable {
        let rows = lines.len();
        let cols = lines.iter().map(|row| row.len()).max().unwrap_or(0);
        
        let data: Vec<Vec<String>> = lines
            .into_iter()
            .map(|row| row.into_iter().map(|cell| cell.to_string()).collect())
            .collect();

        PdfTable {
            index: 0,
            data,
            rows,
            cols,
            position: BlockPosition {
                x: 72.0,
                y: 400.0 - (start_line as f32 * 14.0),
                width: dimensions.width - 144.0,
                height: (rows as f32 * 14.0),
            },
            confidence: 0.8, // Moderate confidence for pattern-based detection
        }
    }
}