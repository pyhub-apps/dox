//! PDF document processing implementation

use crate::provider::{DocumentProvider, DocumentError, DocumentType};
use std::path::{Path, PathBuf};
use std::cell::RefCell;
use tracing::{debug, warn};

/// PDF document provider for .pdf files
#[derive(Debug)]
pub struct PdfProvider {
    path: PathBuf,
    content: RefCell<Option<String>>, // Cached extracted text with interior mutability
}

impl PdfProvider {
    /// Open a PDF document from a file path
    pub fn open(path: &Path) -> Result<Self, DocumentError> {
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
        })
    }
    
    
    /// Get PDF metadata using lopdf
    pub fn get_metadata(&self) -> Result<PdfMetadata, DocumentError> {
        debug!("Extracting metadata from PDF: {}", self.path.display());
        
        let document = lopdf::Document::load(&self.path)
            .map_err(|e| DocumentError::OperationFailed {
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
                                metadata.title = Some(String::from_utf8_lossy(title_str).to_string());
                            }
                        }
                        
                        // Extract author
                        if let Ok(author_obj) = dict.get(b"Author") {
                            if let Ok(author_str) = author_obj.as_str() {
                                metadata.author = Some(String::from_utf8_lossy(author_str).to_string());
                            }
                        }
                        
                        // Extract subject
                        if let Ok(subject_obj) = dict.get(b"Subject") {
                            if let Ok(subject_str) = subject_obj.as_str() {
                                metadata.subject = Some(String::from_utf8_lossy(subject_str).to_string());
                            }
                        }
                        
                        // Extract creator
                        if let Ok(creator_obj) = dict.get(b"Creator") {
                            if let Ok(creator_str) = creator_obj.as_str() {
                                metadata.creator = Some(String::from_utf8_lossy(creator_str).to_string());
                            }
                        }
                    }
                }
            }
        }
        
        Ok(metadata)
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
        let bytes = std::fs::read(&self.path)
            .map_err(|e| DocumentError::FileReadError {
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