//! PDF document processing implementation with advanced features

use crate::provider::{DocumentError, DocumentProvider, DocumentType};
use super::{AdvancedPdfExtractor, PdfExtractConfig, EncryptedPdfHandler, EncryptionInfo, PdfOcrProcessor, OcrConfig, OcrAnalysis};
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};

/// PDF document provider for .pdf files with advanced features
#[derive(Debug)]
pub struct PdfProvider {
    path: PathBuf,
    content: RefCell<Option<String>>, // Cached extracted text with interior mutability
    extract_config: PdfExtractConfig, // Configuration for advanced extraction
    encryption_info: RefCell<Option<EncryptionInfo>>, // Cached encryption info
    ocr_analysis: RefCell<Option<OcrAnalysis>>, // Cached OCR analysis
}

impl PdfProvider {
    /// Open a PDF document from a file path
    pub fn open(path: &Path) -> Result<Self, DocumentError> {
        Self::open_with_config(path, PdfExtractConfig::default())
    }

    /// Open a PDF document with custom extraction configuration
    pub fn open_with_config(path: &Path, config: PdfExtractConfig) -> Result<Self, DocumentError> {
        debug!("Opening PDF document: {}", path.display());

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
        } else {
            return Err(DocumentError::UnsupportedFormat {
                format: "unknown".to_string(),
            });
        }

        Ok(PdfProvider {
            path: path.to_path_buf(),
            content: RefCell::new(None),
            extract_config: config,
            encryption_info: RefCell::new(None),
            ocr_analysis: RefCell::new(None),
        })
    }

    /// Create provider optimized for small PDFs
    pub fn open_small_file(path: &Path) -> Result<Self, DocumentError> {
        Self::open_with_config(path, PdfExtractConfig::small_file())
    }

    /// Create provider optimized for large PDFs
    pub fn open_large_file(path: &Path) -> Result<Self, DocumentError> {
        Self::open_with_config(path, PdfExtractConfig::large_file())
    }

    /// Create provider optimized for layout-critical extraction
    pub fn open_layout_critical(path: &Path) -> Result<Self, DocumentError> {
        Self::open_with_config(path, PdfExtractConfig::layout_critical())
    }

    /// Get PDF metadata using lopdf
    pub fn get_metadata(&self) -> Result<PdfMetadata, DocumentError> {
        debug!("Extracting metadata from PDF: {}", self.path.display());

        let document =
            lopdf::Document::load(&self.path).map_err(|e| DocumentError::OperationFailed {
                reason: format!("Failed to load PDF for metadata: {}", e),
            })?;

        let mut metadata = PdfMetadata::default();

        // Get page count
        metadata.page_count = document.get_pages().len();

        // Try to get document info
        if let Ok(info_dict) = document.trailer.get(b"Info") {
            if let Ok(info_ref) = info_dict.as_reference() {
                if let Ok(info_obj) = document.get_object(info_ref) {
                    if let Ok(dict) = info_obj.as_dict() {
                        // Extract title
                        if let Ok(title_obj) = dict.get(b"Title") {
                            if let Ok(title_str) = title_obj.as_str() {
                                metadata.title =
                                    Some(String::from_utf8_lossy(title_str).to_string());
                            }
                        }

                        // Extract author
                        if let Ok(author_obj) = dict.get(b"Author") {
                            if let Ok(author_str) = author_obj.as_str() {
                                metadata.author =
                                    Some(String::from_utf8_lossy(author_str).to_string());
                            }
                        }

                        // Extract subject
                        if let Ok(subject_obj) = dict.get(b"Subject") {
                            if let Ok(subject_str) = subject_obj.as_str() {
                                metadata.subject =
                                    Some(String::from_utf8_lossy(subject_str).to_string());
                            }
                        }

                        // Extract creator
                        if let Ok(creator_obj) = dict.get(b"Creator") {
                            if let Ok(creator_str) = creator_obj.as_str() {
                                metadata.creator =
                                    Some(String::from_utf8_lossy(creator_str).to_string());
                            }
                        }

                        // Extract creation date
                        if let Ok(created_obj) = dict.get(b"CreationDate") {
                            if let Ok(created_str) = created_obj.as_str() {
                                let date_string = String::from_utf8_lossy(created_str).to_string();
                                metadata.created = Self::parse_pdf_date(&date_string);
                            }
                        }

                        // Extract modification date
                        if let Ok(modified_obj) = dict.get(b"ModDate") {
                            if let Ok(modified_str) = modified_obj.as_str() {
                                let date_string = String::from_utf8_lossy(modified_str).to_string();
                                metadata.modified = Self::parse_pdf_date(&date_string);
                            }
                        }
                    }
                }
            }
        }

        Ok(metadata)
    }

    /// Parse PDF date format (D:YYYYMMDDHHmmSSOHH'mm')
    fn parse_pdf_date(date_str: &str) -> Option<String> {
        // PDF date format: D:YYYYMMDDHHmmSSOHH'mm'
        // We'll extract and format the basic parts
        if date_str.len() < 16 || !date_str.starts_with("D:") {
            return None;
        }

        let date_part = &date_str[2..]; // Remove "D:" prefix

        if date_part.len() >= 14 {
            // Extract YYYYMMDDHHMMSS
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

    /// Check if PDF is encrypted and get encryption information
    pub fn check_encryption(&self) -> Result<EncryptionInfo, DocumentError> {
        // Check cache first
        if let Some(cached_info) = self.encryption_info.borrow().as_ref() {
            return Ok(cached_info.clone());
        }

        debug!("Checking PDF encryption: {}", self.path.display());
        let mut handler = EncryptedPdfHandler::new(&self.path)?;
        let encryption_info = handler.check_encryption()?;

        // Cache the result
        *self.encryption_info.borrow_mut() = Some(encryption_info.clone());

        Ok(encryption_info)
    }

    /// Attempt to authenticate with password for encrypted PDFs
    pub fn authenticate(&self, password: &str) -> Result<bool, DocumentError> {
        info!("Attempting PDF authentication");
        let mut handler = EncryptedPdfHandler::new(&self.path)?;
        
        match handler.authenticate(password) {
            Ok(result) => match result {
                crate::pdf::PasswordResult::Success => {
                    info!("PDF authentication successful");
                    Ok(true)
                }
                crate::pdf::PasswordResult::NotNeeded => {
                    debug!("PDF authentication not needed");
                    Ok(true)
                }
                crate::pdf::PasswordResult::Incorrect => {
                    warn!("PDF authentication failed - incorrect password");
                    Ok(false)
                }
                crate::pdf::PasswordResult::Error(err) => {
                    Err(DocumentError::OperationFailed {
                        reason: format!("PDF authentication error: {}", err),
                    })
                }
            },
            Err(e) => Err(DocumentError::OperationFailed {
                reason: format!("PDF authentication failed: {}", e),
            }),
        }
    }

    /// Try common passwords for encrypted PDFs
    pub fn try_common_passwords(&self) -> Result<Option<String>, DocumentError> {
        info!("Trying common passwords for PDF");
        let mut handler = EncryptedPdfHandler::new(&self.path)?;
        
        handler.try_common_passwords().map_err(|e| DocumentError::OperationFailed {
            reason: format!("Failed to try common passwords: {}", e),
        })
    }

    /// Analyze PDF for OCR requirements
    pub fn analyze_for_ocr(&self) -> Result<OcrAnalysis, DocumentError> {
        // Check cache first
        if let Some(cached_analysis) = self.ocr_analysis.borrow().as_ref() {
            return Ok(cached_analysis.clone());
        }

        debug!("Analyzing PDF for OCR requirements: {}", self.path.display());
        let processor = PdfOcrProcessor::new(OcrConfig::default());
        let analysis = processor.analyze_pdf_for_ocr(&self.path)?;

        // Cache the result
        *self.ocr_analysis.borrow_mut() = Some(analysis.clone());

        Ok(analysis)
    }

    /// Extract text with advanced features (layout preservation, tables, etc.)
    pub fn get_advanced_text(&self) -> Result<String, DocumentError> {
        debug!("Extracting advanced text from PDF: {}", self.path.display());

        // Check encryption status first
        let encryption_info = self.check_encryption()?;
        if encryption_info.is_encrypted {
            info!("PDF is encrypted, attempting common passwords");
            if let Some(_password) = self.try_common_passwords()? {
                info!("Found working password for encrypted PDF");
            } else {
                warn!("Could not authenticate encrypted PDF");
                return Err(DocumentError::OperationFailed {
                    reason: "PDF is encrypted and requires authentication".to_string(),
                });
            }
        }

        // Use advanced extractor
        let mut extractor = AdvancedPdfExtractor::new(&self.path, self.extract_config.clone())?;
        let result = extractor.extract()?;

        // Combine all page text
        let mut combined_text = String::new();
        for page in result.pages {
            if self.extract_config.preserve_layout {
                // Use structured text blocks
                for block in page.text_blocks {
                    combined_text.push_str(&block.text);
                    combined_text.push('\n');
                }
            } else {
                // Use raw text
                combined_text.push_str(&page.raw_text);
                combined_text.push('\n');
            }

            // Add tables if extracted
            for table in page.tables {
                combined_text.push_str("\n[TABLE]\n");
                for row in table.data {
                    combined_text.push_str(&format!("{}\n", row.join("\t")));
                }
                combined_text.push_str("[/TABLE]\n\n");
            }
        }

        Ok(combined_text)
    }

    /// Get extraction statistics
    pub fn get_extraction_stats(&self) -> Result<crate::pdf::ExtractionStats, DocumentError> {
        debug!("Getting extraction statistics for PDF: {}", self.path.display());

        let mut extractor = AdvancedPdfExtractor::new(&self.path, self.extract_config.clone())?;
        let result = extractor.extract()?;

        Ok(result.stats)
    }

    /// Extract tables from PDF
    pub fn extract_tables(&self) -> Result<Vec<crate::pdf::PdfTable>, DocumentError> {
        debug!("Extracting tables from PDF: {}", self.path.display());

        let mut extractor = AdvancedPdfExtractor::new(&self.path, self.extract_config.clone())?;
        let result = extractor.extract()?;

        let mut all_tables = Vec::new();
        for page in result.pages {
            all_tables.extend(page.tables);
        }

        Ok(all_tables)
    }

    /// Process PDF with OCR (for image-based PDFs)
    pub fn process_with_ocr(&self, config: Option<OcrConfig>) -> Result<String, DocumentError> {
        info!("Processing PDF with OCR: {}", self.path.display());

        let ocr_config = config.unwrap_or_default();
        let mut processor = PdfOcrProcessor::new(ocr_config);
        
        processor.initialize_engine().map_err(|e| DocumentError::OperationFailed {
            reason: format!("OCR initialization failed: {}", e),
        })?;

        // For now, return a placeholder - in a full implementation, this would
        // process the PDF pages with OCR
        Ok("OCR processing would extract text from image-based PDF pages here".to_string())
    }
}

/// PDF metadata structure
#[derive(Debug, Default, Clone)]
pub struct PdfMetadata {
    pub title: Option<String>,
    pub author: Option<String>,
    pub subject: Option<String>,
    pub creator: Option<String>,
    pub page_count: usize,
    pub created: Option<String>,
    pub modified: Option<String>,
}

impl DocumentProvider for PdfProvider {
    fn replace_text(&mut self, _old: &str, _new: &str) -> Result<usize, DocumentError> {
        // PDF text replacement is not supported in this implementation
        // as it would require complex PDF manipulation
        Err(DocumentError::OperationFailed {
            reason: "Text replacement is not supported for PDF documents".to_string(),
        })
    }

    fn save(&self) -> Result<(), DocumentError> {
        // PDF saving is not supported in this read-only implementation
        Err(DocumentError::OperationFailed {
            reason: "Saving PDF documents is not supported".to_string(),
        })
    }

    fn save_as(&self, _path: &Path) -> Result<(), DocumentError> {
        // PDF saving is not supported in this read-only implementation
        Err(DocumentError::OperationFailed {
            reason: "Saving PDF documents is not supported".to_string(),
        })
    }

    fn get_text(&self) -> Result<String, DocumentError> {
        // Check if content is already cached
        if let Some(cached_text) = self.content.borrow().as_ref() {
            return Ok(cached_text.clone());
        }

        // Extract text and cache it
        debug!("Extracting text from PDF: {}", self.path.display());

        // Read PDF file
        let bytes = std::fs::read(&self.path).map_err(|e| DocumentError::FileReadError {
            path: self.path.to_string_lossy().to_string(),
            source: e.into(),
        })?;

        // Extract text using pdf-extract
        let text = match pdf_extract::extract_text_from_mem(&bytes) {
            Ok(text) => text,
            Err(e) => {
                warn!("Failed to extract text from PDF: {}", e);
                return Err(DocumentError::OperationFailed {
                    reason: format!("PDF text extraction failed: {}", e),
                });
            }
        };

        // Cache the extracted text
        *self.content.borrow_mut() = Some(text.clone());

        Ok(text)
    }

    fn is_modified(&self) -> bool {
        // PDF provider is read-only
        false
    }

    fn get_path(&self) -> &Path {
        &self.path
    }

    fn document_type(&self) -> DocumentType {
        DocumentType::Pdf
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_pdf_provider_creation() {
        // Create a temporary PDF file (just for path testing)
        let mut temp_file = NamedTempFile::with_suffix(".pdf").unwrap();
        writeln!(temp_file, "dummy pdf content").unwrap();
        let temp_path = temp_file.path();

        let provider = PdfProvider::open(temp_path);
        assert!(provider.is_ok());

        let provider = provider.unwrap();
        assert_eq!(provider.get_path(), temp_path);
        assert_eq!(provider.document_type(), DocumentType::Pdf);
        assert!(!provider.is_modified());
    }

    #[test]
    fn test_pdf_provider_nonexistent_file() {
        let result = PdfProvider::open(Path::new("/nonexistent/file.pdf"));
        assert!(result.is_err());

        if let Err(DocumentError::DocumentNotFound { path }) = result {
            assert!(path.contains("nonexistent"));
        } else {
            panic!("Expected DocumentNotFound error");
        }
    }

    #[test]
    fn test_pdf_provider_wrong_extension() {
        let mut temp_file = NamedTempFile::with_suffix(".txt").unwrap();
        writeln!(temp_file, "not a pdf").unwrap();

        let result = PdfProvider::open(temp_file.path());
        assert!(result.is_err());

        if let Err(DocumentError::UnsupportedFormat { format }) = result {
            assert_eq!(format, "txt");
        } else {
            panic!("Expected UnsupportedFormat error");
        }
    }
}
