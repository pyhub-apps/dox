//! Excel document provider using calamine for text extraction

use crate::provider::{DocumentError, DocumentProvider, DocumentType};
use anyhow::Result;
use calamine::{open_workbook, Reader, Xlsx};
use std::path::Path;
use tracing::debug;

/// Excel document provider for XLSX files
#[derive(Debug)]
pub struct ExcelProvider {
    path: std::path::PathBuf,
    modified: bool,
}

impl ExcelProvider {
    /// Open an Excel document from the given path
    pub fn open(path: &Path) -> Result<Self, DocumentError> {
        debug!("Opening Excel document: {}", path.display());

        if !path.exists() {
            return Err(DocumentError::DocumentNotFound {
                path: path.display().to_string(),
            });
        }

        // Test if we can open the file
        let _workbook: Xlsx<_> = open_workbook(path)
            .map_err(|e| DocumentError::FileReadError {
                path: path.display().to_string(),
                source: anyhow::anyhow!("Failed to open Excel file: {}", e),
            })?;

        Ok(ExcelProvider {
            path: path.to_path_buf(),
            modified: false,
        })
    }

    /// Extract text from all sheets in the Excel workbook
    fn extract_text_from_workbook(&self) -> Result<String, DocumentError> {
        debug!("Extracting text from Excel workbook: {}", self.path.display());

        let mut workbook: Xlsx<_> = open_workbook(&self.path)
            .map_err(|e| DocumentError::FileReadError {
                path: self.path.display().to_string(),
                source: anyhow::anyhow!("Failed to open Excel file: {}", e),
            })?;

        let mut full_text = String::new();

        // Get all sheet names
        let sheet_names = workbook.sheet_names();
        
        for sheet_name in sheet_names {
            debug!("Processing sheet: {}", sheet_name);
            
            // Add sheet name as a header
            if !full_text.is_empty() {
                full_text.push_str("\n\n");
            }
            full_text.push_str(&format!("=== {} ===\n", sheet_name));

            // Read the sheet data
            match workbook.worksheet_range(&sheet_name) {
                Ok(range) => {
                    for row in range.rows() {
                        let mut row_text = Vec::new();
                        
                        for cell in row {
                            let cell_value = match cell {
                                calamine::Data::Int(i) => i.to_string(),
                                calamine::Data::Float(f) => f.to_string(),
                                calamine::Data::String(s) => s.clone(),
                                calamine::Data::Bool(b) => b.to_string(),
                                calamine::Data::DateTime(dt) => dt.to_string(),
                                calamine::Data::DateTimeIso(s) => s.clone(),
                                calamine::Data::DurationIso(s) => s.clone(),
                                calamine::Data::Error(e) => format!("#ERR: {:?}", e),
                                calamine::Data::Empty => String::new(),
                            };
                            
                            if !cell_value.is_empty() {
                                row_text.push(cell_value);
                            }
                        }
                        
                        if !row_text.is_empty() {
                            full_text.push_str(&row_text.join("\t"));
                            full_text.push('\n');
                        }
                    }
                }
                Err(e) => {
                    debug!("Failed to read sheet '{}': {}", sheet_name, e);
                    full_text.push_str(&format!("Error reading sheet: {}\n", e));
                }
            }
        }

        Ok(full_text)
    }
}

impl DocumentProvider for ExcelProvider {
    fn replace_text(&mut self, old: &str, new: &str) -> Result<usize, DocumentError> {
        // Excel text replacement is not implemented for now
        // This would require reading, modifying, and writing back the Excel file
        debug!("Excel text replacement not implemented: '{}' -> '{}'", old, new);
        Ok(0)
    }

    fn save(&self) -> Result<(), DocumentError> {
        if !self.modified {
            return Ok(());
        }
        
        // Excel saving is not implemented for now
        debug!("Excel save not implemented");
        Err(DocumentError::OperationFailed {
            reason: "Excel save operation not implemented".to_string(),
        })
    }

    fn save_as(&self, _path: &Path) -> Result<(), DocumentError> {
        // Excel save_as is not implemented for now
        debug!("Excel save_as not implemented");
        Err(DocumentError::OperationFailed {
            reason: "Excel save_as operation not implemented".to_string(),
        })
    }

    fn get_text(&self) -> Result<String, DocumentError> {
        self.extract_text_from_workbook()
    }

    fn is_modified(&self) -> bool {
        self.modified
    }

    fn get_path(&self) -> &Path {
        &self.path
    }

    fn document_type(&self) -> DocumentType {
        DocumentType::Excel
    }
}