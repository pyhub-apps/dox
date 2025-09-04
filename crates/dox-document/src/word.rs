//! Word document (.docx) processing implementation

use crate::provider::{DocumentError, DocumentProvider, DocumentType};
use crate::utils::{
    copy_zip_with_replacements, extract_text_from_xml, extract_zip, read_zip_file,
    replace_text_in_xml,
};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tracing::debug;
use xml::reader::{EventReader, XmlEvent};

/// Word document metadata
#[derive(Debug, Default, Clone)]
pub struct WordMetadata {
    pub title: Option<String>,
    pub author: Option<String>,
    pub subject: Option<String>,
    pub creator: Option<String>,
    pub total_pages: usize,
    pub created: Option<String>,
    pub modified: Option<String>,
}

/// Word document provider for .docx files
#[derive(Debug)]
pub struct WordProvider {
    path: PathBuf,
    archive_data: Vec<u8>,
    content: Vec<u8>,
    modified: bool,
}

impl WordProvider {
    /// Open a Word document from a file path
    pub fn open(path: &Path) -> Result<Self, DocumentError> {
        debug!("Opening Word document: {}", path.display());

        if !path.exists() {
            return Err(DocumentError::DocumentNotFound {
                path: path.to_string_lossy().to_string(),
            });
        }

        // Read entire file into memory
        let archive_data = std::fs::read(path)?;

        // Extract document.xml content
        let mut archive = extract_zip(&archive_data)?;
        let content = read_zip_file(&mut archive, "word/document.xml").map_err(|_| {
            DocumentError::InvalidStructure {
                reason: "Missing word/document.xml".to_string(),
            }
        })?;

        Ok(WordProvider {
            path: path.to_path_buf(),
            archive_data,
            content,
            modified: false,
        })
    }

    /// Create a new Word document (placeholder for future implementation)
    pub fn create(_path: &Path) -> Result<Self, DocumentError> {
        Err(DocumentError::OperationFailed {
            reason: "Creating new Word documents is not yet implemented".to_string(),
        })
    }

    /// Get the Word text tags used for text extraction and replacement
    fn text_tags() -> &'static [&'static str] {
        &["w:t"]
    }

    /// Extract metadata from core.xml properties
    pub fn get_metadata(&self) -> Result<WordMetadata, DocumentError> {
        let mut archive = extract_zip(&self.archive_data)?;

        let mut metadata = WordMetadata::default();

        // Try to read core.xml for basic metadata
        if let Ok(core_xml) = read_zip_file(&mut archive, "docProps/core.xml") {
            metadata = self.parse_core_properties(&core_xml)?;
        }

        // Try to read app.xml for additional metadata
        if let Ok(app_xml) = read_zip_file(&mut archive, "docProps/app.xml") {
            self.parse_app_properties(&app_xml, &mut metadata)?;
        }

        Ok(metadata)
    }

    /// Parse core properties XML
    fn parse_core_properties(&self, xml_data: &[u8]) -> Result<WordMetadata, DocumentError> {
        let mut metadata = WordMetadata::default();
        let reader = EventReader::new(std::io::Cursor::new(xml_data));

        let mut current_element = String::new();
        let mut text_content = String::new();

        for event in reader {
            match event {
                Ok(XmlEvent::StartElement { name, .. }) => {
                    current_element = name.local_name.clone();
                    text_content.clear();
                }
                Ok(XmlEvent::Characters(text)) => {
                    text_content.push_str(&text);
                }
                Ok(XmlEvent::EndElement { .. }) => {
                    match current_element.as_str() {
                        "title" => metadata.title = Some(text_content.clone()),
                        "creator" => metadata.author = Some(text_content.clone()),
                        "subject" => metadata.subject = Some(text_content.clone()),
                        "created" => metadata.created = Some(text_content.clone()),
                        "modified" => metadata.modified = Some(text_content.clone()),
                        _ => {}
                    }
                    current_element.clear();
                }
                Ok(XmlEvent::EndDocument) => break,
                Err(_) => break,
                _ => {}
            }
        }

        Ok(metadata)
    }

    /// Parse app properties XML for additional metadata
    fn parse_app_properties(
        &self,
        xml_data: &[u8],
        metadata: &mut WordMetadata,
    ) -> Result<(), DocumentError> {
        let reader = EventReader::new(std::io::Cursor::new(xml_data));

        let mut current_element = String::new();
        let mut text_content = String::new();

        for event in reader {
            match event {
                Ok(XmlEvent::StartElement { name, .. }) => {
                    current_element = name.local_name.clone();
                    text_content.clear();
                }
                Ok(XmlEvent::Characters(text)) => {
                    text_content.push_str(&text);
                }
                Ok(XmlEvent::EndElement { .. }) => {
                    match current_element.as_str() {
                        "Application" => metadata.creator = Some(text_content.clone()),
                        "Pages" => {
                            if let Ok(pages) = text_content.parse::<usize>() {
                                metadata.total_pages = pages;
                            }
                        }
                        _ => {}
                    }
                    current_element.clear();
                }
                Ok(XmlEvent::EndDocument) => break,
                Err(_) => break,
                _ => {}
            }
        }

        Ok(())
    }
}

impl DocumentProvider for WordProvider {
    fn replace_text(&mut self, old: &str, new: &str) -> Result<usize, DocumentError> {
        debug!("Replacing text '{}' with '{}' in Word document", old, new);

        let (new_content, count) = replace_text_in_xml(&self.content, Self::text_tags(), old, new)?;

        if count > 0 {
            self.content = new_content;
            self.modified = true;
            debug!("Replaced {} occurrences in Word document", count);
        }

        Ok(count)
    }

    fn save(&self) -> Result<(), DocumentError> {
        if !self.modified {
            debug!("No changes to save in Word document");
            return Ok(());
        }

        debug!("Saving Word document to: {}", self.path.display());
        self.save_as(&self.path)
    }

    fn save_as(&self, path: &Path) -> Result<(), DocumentError> {
        debug!("Saving Word document as: {}", path.display());

        let file = std::fs::File::create(path)?;

        // Prepare replacements map
        let mut replacements = HashMap::new();
        replacements.insert("word/document.xml".to_string(), self.content.clone());

        // Copy archive with replacements
        copy_zip_with_replacements(&self.archive_data, file, &replacements)?;

        debug!("Word document saved successfully");
        Ok(())
    }

    fn get_text(&self) -> Result<String, DocumentError> {
        debug!("Extracting text from Word document");
        let text = extract_text_from_xml(&self.content, Self::text_tags())?;
        Ok(text)
    }

    fn is_modified(&self) -> bool {
        self.modified
    }

    fn get_path(&self) -> &Path {
        &self.path
    }

    fn document_type(&self) -> DocumentType {
        DocumentType::Word
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_minimal_docx() -> Vec<u8> {
        // This is a minimal Word document structure for testing
        // In a real implementation, you'd want to use a proper minimal template
        let mut zip_data = Vec::new();
        {
            use zip::{write::SimpleFileOptions, ZipWriter};
            let mut writer = ZipWriter::new(std::io::Cursor::new(&mut zip_data));

            // Add minimal document.xml
            let doc_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
    <w:body>
        <w:p>
            <w:r>
                <w:t>Hello World</w:t>
            </w:r>
        </w:p>
    </w:body>
</w:document>"#;

            writer
                .start_file("word/document.xml", SimpleFileOptions::default())
                .unwrap();
            writer.write_all(doc_xml.as_bytes()).unwrap();
            writer.finish().unwrap();
        }
        zip_data
    }

    #[test]
    fn test_word_document_text_extraction() {
        let zip_data = create_minimal_docx();
        let temp_file = NamedTempFile::new().unwrap();
        std::fs::write(temp_file.path(), &zip_data).unwrap();

        let doc = WordProvider::open(temp_file.path()).unwrap();
        let text = doc.get_text().unwrap();
        assert_eq!(text, "Hello World");
    }

    #[test]
    fn test_word_document_text_replacement() {
        let zip_data = create_minimal_docx();
        let temp_file = NamedTempFile::new().unwrap();
        std::fs::write(temp_file.path(), &zip_data).unwrap();

        let mut doc = WordProvider::open(temp_file.path()).unwrap();
        let count = doc.replace_text("Hello", "Hi").unwrap();
        assert_eq!(count, 1);
        assert!(doc.is_modified());

        let text = doc.get_text().unwrap();
        assert_eq!(text, "Hi World");
    }
}
