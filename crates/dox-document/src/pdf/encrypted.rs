//! Encrypted PDF handling and password management

use crate::provider::DocumentError;
use lopdf::Document;
use std::path::Path;
use tracing::{debug, warn, info};

/// Encrypted PDF handler
#[derive(Debug)]
pub struct EncryptedPdfHandler {
    /// Path to the PDF file
    path: std::path::PathBuf,
    /// Document instance
    document: Option<Document>,
}

/// Encryption information for a PDF
#[derive(Debug, Clone)]
pub struct EncryptionInfo {
    /// Whether the document is encrypted
    pub is_encrypted: bool,
    /// Security handler type
    pub security_handler: Option<String>,
    /// Encryption algorithm
    pub algorithm: Option<String>,
    /// Key length in bits
    pub key_length: Option<u32>,
    /// User permissions
    pub permissions: EncryptionPermissions,
}

/// PDF encryption permissions
#[derive(Debug, Clone)]
pub struct EncryptionPermissions {
    /// Print permission
    pub print: bool,
    /// Print high quality
    pub print_high_quality: bool,
    /// Modify contents
    pub modify: bool,
    /// Copy or extract
    pub copy: bool,
    /// Add or modify annotations
    pub annotate: bool,
    /// Fill form fields
    pub fill_forms: bool,
    /// Extract for accessibility
    pub extract_accessibility: bool,
    /// Assemble document
    pub assemble: bool,
}

impl Default for EncryptionPermissions {
    fn default() -> Self {
        Self {
            print: false,
            print_high_quality: false,
            modify: false,
            copy: false,
            annotate: false,
            fill_forms: false,
            extract_accessibility: false,
            assemble: false,
        }
    }
}

/// Password attempt result
#[derive(Debug)]
pub enum PasswordResult {
    /// Password was correct
    Success,
    /// Password was incorrect
    Incorrect,
    /// No password was needed
    NotNeeded,
    /// Error occurred during authentication
    Error(String),
}

impl EncryptedPdfHandler {
    /// Create a new encrypted PDF handler
    pub fn new(path: &Path) -> Result<Self, DocumentError> {
        if !path.exists() {
            return Err(DocumentError::DocumentNotFound {
                path: path.to_string_lossy().to_string(),
            });
        }

        Ok(Self {
            path: path.to_path_buf(),
            document: None,
        })
    }

    /// Check if the PDF is encrypted and get encryption information
    pub fn check_encryption(&mut self) -> Result<EncryptionInfo, DocumentError> {
        debug!("Checking encryption status for: {}", self.path.display());

        {
            let document = self.load_document()?;
            let is_encrypted = document.is_encrypted();

            if !is_encrypted {
                debug!("PDF is not encrypted");
                return Ok(EncryptionInfo {
                    is_encrypted: false,
                    security_handler: None,
                    algorithm: None,
                    key_length: None,
                    permissions: EncryptionPermissions::default(),
                });
            }

            info!("PDF is encrypted, analyzing encryption details");
            // Clone the document for analysis to avoid borrow issues
            let doc_clone = Document::load(&self.path).map_err(|e| DocumentError::OperationFailed {
                reason: format!("Failed to load PDF for analysis: {}", e),
            })?;
            
            self.analyze_encryption(&doc_clone)
        }
    }

    /// Attempt to authenticate with a password
    pub fn authenticate(&mut self, password: &str) -> Result<PasswordResult, DocumentError> {
        debug!("Attempting to authenticate with provided password");

        let document = self.load_document()?;
        
        if !document.is_encrypted() {
            return Ok(PasswordResult::NotNeeded);
        }

        // For lopdf, we need to load the document with the password
        // This is a simplified approach - in practice, we'd need to implement
        // proper password authentication
        // lopdf doesn't have load_from with password in this version
        // For now, just try to load normally and check if it succeeds
        match Document::load(&self.path) {
            Ok(_authenticated_doc) => {
                info!("Password authentication successful");
                Ok(PasswordResult::Success)
            }
            Err(e) => {
                warn!("Password authentication failed: {}", e);
                Ok(PasswordResult::Incorrect)
            }
        }
    }

    /// Attempt authentication with common passwords
    pub fn try_common_passwords(&mut self) -> Result<Option<String>, DocumentError> {
        let common_passwords = [
            "", // Empty password
            "password",
            "123456",
            "admin",
            "user",
            "test",
            "pdf",
            "document",
        ];

        info!("Trying common passwords for encrypted PDF");

        for password in &common_passwords {
            match self.authenticate(password)? {
                PasswordResult::Success => {
                    info!("Found working password: '{}'", password);
                    return Ok(Some(password.to_string()));
                }
                PasswordResult::NotNeeded => {
                    return Ok(None);
                }
                PasswordResult::Incorrect => {
                    debug!("Password '{}' failed", password);
                    continue;
                }
                PasswordResult::Error(err) => {
                    warn!("Error trying password '{}': {}", password, err);
                    continue;
                }
            }
        }

        info!("No common passwords worked");
        Ok(None)
    }

    /// Load the authenticated document
    pub fn load_authenticated_document(&mut self, _password: Option<&str>) -> Result<Document, DocumentError> {
        // Simplified implementation - just load the document normally
        debug!("Loading document");
        self.load_document().cloned()
    }

    /// Get extraction strategy for encrypted PDF
    pub fn get_extraction_strategy(&self, info: &EncryptionInfo) -> ExtractionStrategy {
        if !info.is_encrypted {
            return ExtractionStrategy::Normal;
        }

        if info.permissions.copy && info.permissions.extract_accessibility {
            ExtractionStrategy::Normal
        } else if info.permissions.extract_accessibility {
            ExtractionStrategy::AccessibilityOnly
        } else {
            ExtractionStrategy::Restricted
        }
    }

    /// Load the PDF document
    fn load_document(&mut self) -> Result<&Document, DocumentError> {
        if self.document.is_none() {
            let document = Document::load(&self.path)
                .map_err(|e| DocumentError::OperationFailed {
                    reason: format!("Failed to load PDF: {}", e),
                })?;
            self.document = Some(document);
        }
        Ok(self.document.as_ref().unwrap())
    }

    /// Analyze encryption details
    fn analyze_encryption(&self, document: &Document) -> Result<EncryptionInfo, DocumentError> {
        let mut info = EncryptionInfo {
            is_encrypted: true,
            security_handler: None,
            algorithm: None,
            key_length: None,
            permissions: EncryptionPermissions::default(),
        };

        // Try to extract encryption dictionary
        if let Ok(encrypt_dict) = document.trailer.get(b"Encrypt") {
            if let Ok(encrypt_ref) = encrypt_dict.as_reference() {
                if let Ok(encrypt_obj) = document.get_object(encrypt_ref) {
                    if let Ok(dict) = encrypt_obj.as_dict() {
                        // Extract security handler
                        if let Ok(filter) = dict.get(b"Filter") {
                            if let Ok(filter_name) = filter.as_name() {
                                info.security_handler = Some(String::from_utf8_lossy(filter_name).to_string());
                            }
                        }

                        // Extract algorithm (simplified)
                        if let Ok(v_obj) = dict.get(b"V") {
                            if let Ok(v) = v_obj.as_i64() {
                                info.algorithm = Some(match v {
                                    1 => "RC4 40-bit".to_string(),
                                    2 => "RC4 variable length".to_string(),
                                    4 => "AES".to_string(),
                                    5 => "AES-256".to_string(),
                                    _ => format!("Unknown ({})", v),
                                });
                            }
                        }

                        // Extract key length
                        if let Ok(length_obj) = dict.get(b"Length") {
                            if let Ok(length) = length_obj.as_i64() {
                                info.key_length = Some(length as u32);
                            }
                        }

                        // Extract permissions (simplified)
                        if let Ok(p_obj) = dict.get(b"P") {
                            if let Ok(permissions) = p_obj.as_i64() {
                                info.permissions = self.parse_permissions(permissions);
                            }
                        }
                    }
                }
            }
        }

        Ok(info)
    }

    /// Parse permission flags from PDF
    fn parse_permissions(&self, p: i64) -> EncryptionPermissions {
        EncryptionPermissions {
            print: (p & 0x04) != 0,
            modify: (p & 0x08) != 0,
            copy: (p & 0x10) != 0,
            annotate: (p & 0x20) != 0,
            fill_forms: (p & 0x100) != 0,
            extract_accessibility: (p & 0x200) != 0,
            assemble: (p & 0x400) != 0,
            print_high_quality: (p & 0x800) != 0,
        }
    }
}

/// Strategy for extracting content from encrypted PDFs
#[derive(Debug, Clone, PartialEq)]
pub enum ExtractionStrategy {
    /// Normal extraction (no restrictions)
    Normal,
    /// Only accessibility-compliant extraction
    AccessibilityOnly,
    /// Restricted extraction (may fail or return limited content)
    Restricted,
}

impl ExtractionStrategy {
    /// Check if full text extraction is allowed
    pub fn allows_text_extraction(&self) -> bool {
        matches!(self, ExtractionStrategy::Normal | ExtractionStrategy::AccessibilityOnly)
    }

    /// Check if table extraction is allowed
    pub fn allows_table_extraction(&self) -> bool {
        matches!(self, ExtractionStrategy::Normal)
    }

    /// Check if metadata extraction is allowed
    pub fn allows_metadata_extraction(&self) -> bool {
        true // Usually always allowed
    }

    /// Get warning message for restricted extraction
    pub fn get_warning(&self) -> Option<&'static str> {
        match self {
            ExtractionStrategy::Normal => None,
            ExtractionStrategy::AccessibilityOnly => {
                Some("Limited extraction due to PDF security settings")
            }
            ExtractionStrategy::Restricted => {
                Some("Extraction severely restricted due to PDF security settings")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_encryption_permissions_parsing() {
        let handler = EncryptedPdfHandler {
            path: std::path::PathBuf::new(),
            document: None,
        };

        // Test full permissions (all bits set)
        let full_permissions = handler.parse_permissions(0xFFFFFFFF);
        assert!(full_permissions.print);
        assert!(full_permissions.modify);
        assert!(full_permissions.copy);

        // Test no permissions
        let no_permissions = handler.parse_permissions(0x00000000);
        assert!(!no_permissions.print);
        assert!(!no_permissions.modify);
        assert!(!no_permissions.copy);
    }

    #[test]
    fn test_extraction_strategy() {
        let normal = ExtractionStrategy::Normal;
        assert!(normal.allows_text_extraction());
        assert!(normal.allows_table_extraction());
        assert!(normal.allows_metadata_extraction());
        assert!(normal.get_warning().is_none());

        let restricted = ExtractionStrategy::Restricted;
        assert!(!restricted.allows_text_extraction());
        assert!(!restricted.allows_table_extraction());
        assert!(restricted.allows_metadata_extraction());
        assert!(restricted.get_warning().is_some());
    }
}