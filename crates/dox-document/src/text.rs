//! Text file provider for plain text documents

use crate::provider::{DocumentError, DocumentProvider, DocumentType};
use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};

/// Provider for plain text files
#[derive(Debug)]
pub struct TextProvider {
    path: PathBuf,
    content: String,
    modified: bool,
}

impl TextProvider {
    /// Open a text file
    pub fn open(path: &Path) -> Result<Self, DocumentError> {
        let content = fs::read_to_string(path).map_err(|e| DocumentError::FileReadError {
            path: path.display().to_string(),
            source: e.into(),
        })?;

        Ok(TextProvider {
            path: path.to_path_buf(),
            content,
            modified: false,
        })
    }
}

impl DocumentProvider for TextProvider {
    fn replace_text(&mut self, old: &str, new: &str) -> Result<usize, DocumentError> {
        let original_content = self.content.clone();
        self.content = self.content.replace(old, new);
        
        let replacements = original_content.matches(old).count();
        if replacements > 0 {
            self.modified = true;
        }
        
        Ok(replacements)
    }

    fn save(&self) -> Result<(), DocumentError> {
        fs::write(&self.path, &self.content).map_err(|e| DocumentError::Io(e))?;
        Ok(())
    }

    fn save_as(&self, path: &Path) -> Result<(), DocumentError> {
        fs::write(path, &self.content).map_err(|e| DocumentError::Io(e))?;
        Ok(())
    }

    fn get_text(&self) -> Result<String, DocumentError> {
        Ok(self.content.clone())
    }

    fn is_modified(&self) -> bool {
        self.modified
    }

    fn get_path(&self) -> &Path {
        &self.path
    }

    fn document_type(&self) -> DocumentType {
        DocumentType::Text
    }
}