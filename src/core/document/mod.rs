use anyhow::Result;
use std::path::Path;

mod word;
mod powerpoint;

pub use word::WordDocument;
pub use powerpoint::PowerPointDocument;

/// Trait for document operations
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

/// Represents an open document
pub enum Document {
    Word(WordDocument),
    PowerPoint(PowerPointDocument),
}

impl Document {
    /// Open a document from a file path
    pub fn open(path: &Path) -> Result<Self> {
        let ext = path.extension()
            .and_then(|s| s.to_str())
            .ok_or_else(|| anyhow::anyhow!("No file extension"))?
            .to_lowercase();
        
        match ext.as_str() {
            "docx" => Ok(Document::Word(WordDocument::open(path)?)),
            "pptx" => Ok(Document::PowerPoint(PowerPointDocument::open(path)?)),
            _ => anyhow::bail!("Unsupported document type: {}", ext),
        }
    }
    
    /// Replace text in the document
    pub fn replace_text(&mut self, old: &str, new: &str) -> Result<usize> {
        match self {
            Document::Word(doc) => doc.replace_text(old, new),
            Document::PowerPoint(doc) => doc.replace_text(old, new),
        }
    }
    
    /// Save the document
    pub fn save(&self) -> Result<()> {
        match self {
            Document::Word(doc) => doc.save(),
            Document::PowerPoint(doc) => doc.save(),
        }
    }
    
    /// Save the document to a different path
    pub fn save_as(&self, path: &Path) -> Result<()> {
        match self {
            Document::Word(doc) => doc.save_as(path),
            Document::PowerPoint(doc) => doc.save_as(path),
        }
    }
    
    /// Get the document content as text
    pub fn get_text(&self) -> Result<String> {
        match self {
            Document::Word(doc) => doc.get_text(),
            Document::PowerPoint(doc) => doc.get_text(),
        }
    }
}