//! Compatibility layer for the original Document API

use crate::provider::{create_provider, DocumentError, DocumentProvider};
use anyhow::Result;
use std::path::Path;

/// Legacy Document enum for backward compatibility
pub enum Document {
    Provider(Box<dyn DocumentProvider>),
}

impl Document {
    /// Open a document from a file path (legacy API)
    pub fn open(path: &Path) -> Result<Self> {
        let provider =
            create_provider(path).map_err(|e| anyhow::anyhow!("Failed to open document: {}", e))?;
        Ok(Document::Provider(provider))
    }

    /// Replace text in the document
    pub fn replace_text(&mut self, old: &str, new: &str) -> Result<usize> {
        match self {
            Document::Provider(provider) => provider
                .replace_text(old, new)
                .map_err(|e| anyhow::anyhow!("Replace text failed: {}", e)),
        }
    }

    /// Save the document
    pub fn save(&self) -> Result<()> {
        match self {
            Document::Provider(provider) => provider
                .save()
                .map_err(|e| anyhow::anyhow!("Save failed: {}", e)),
        }
    }

    /// Save the document to a different path
    pub fn save_as(&self, path: &Path) -> Result<()> {
        match self {
            Document::Provider(provider) => provider
                .save_as(path)
                .map_err(|e| anyhow::anyhow!("Save as failed: {}", e)),
        }
    }

    /// Get the document content as text
    pub fn get_text(&self) -> Result<String> {
        match self {
            Document::Provider(provider) => provider
                .get_text()
                .map_err(|e| anyhow::anyhow!("Get text failed: {}", e)),
        }
    }
}

/// Legacy DocumentOps trait for backward compatibility
pub trait DocumentOps {
    /// Replace text in the document
    fn replace_text(&mut self, old: &str, new: &str) -> Result<usize>;

    /// Save the document
    fn save(&self) -> Result<()>;

    /// Save the document to a different path
    fn save_as(&self, path: &Path) -> Result<()>;

    /// Get the document content as text
    fn get_text(&self) -> Result<String>;
}

impl DocumentOps for Document {
    fn replace_text(&mut self, old: &str, new: &str) -> Result<usize> {
        self.replace_text(old, new)
    }

    fn save(&self) -> Result<()> {
        self.save()
    }

    fn save_as(&self, path: &Path) -> Result<()> {
        self.save_as(path)
    }

    fn get_text(&self) -> Result<String> {
        self.get_text()
    }
}
