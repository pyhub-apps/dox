//! PowerPoint document (.pptx) processing implementation

use crate::provider::{DocumentError, DocumentProvider, DocumentType};
use crate::utils::{
    copy_zip_with_replacements, extract_text_from_xml, read_zip_file, replace_text_in_xml,
};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tracing::{debug, info};
use xml::reader::{EventReader, XmlEvent};
use zip::ZipArchive;

/// PowerPoint document metadata
#[derive(Debug, Default, Clone)]
pub struct PowerPointMetadata {
    pub title: Option<String>,
    pub author: Option<String>,
    pub subject: Option<String>,
    pub creator: Option<String>,
    pub total_slides: usize,
    pub created: Option<String>,
    pub modified: Option<String>,
}

/// PowerPoint document provider for .pptx files
#[derive(Debug)]
pub struct PowerPointProvider {
    path: PathBuf,
    archive_data: Vec<u8>,
    slide_contents: Vec<(String, Vec<u8>)>,
    modified: bool,
}

impl PowerPointProvider {
    /// Open a PowerPoint document from a file path
    pub fn open(path: &Path) -> Result<Self, DocumentError> {
        debug!("Opening PowerPoint document: {}", path.display());

        if !path.exists() {
            return Err(DocumentError::DocumentNotFound {
                path: path.to_string_lossy().to_string(),
            });
        }

        // Read entire file into memory
        let archive_data = std::fs::read(path)?;

        // Extract slide contents
        let slide_contents = Self::extract_slide_contents(&archive_data)?;

        info!(
            "Loaded PowerPoint document with {} slides",
            slide_contents.len()
        );

        Ok(PowerPointProvider {
            path: path.to_path_buf(),
            archive_data,
            slide_contents,
            modified: false,
        })
    }

    /// Create a new PowerPoint document (placeholder for future implementation)
    pub fn create(_path: &Path) -> Result<Self, DocumentError> {
        Err(DocumentError::OperationFailed {
            reason: "Creating new PowerPoint documents is not yet implemented".to_string(),
        })
    }

    /// Extract slide contents from the archive
    fn extract_slide_contents(
        archive_data: &[u8],
    ) -> Result<Vec<(String, Vec<u8>)>, DocumentError> {
        let reader = std::io::Cursor::new(archive_data);
        let mut archive = ZipArchive::new(reader)?;

        let mut slide_contents = Vec::new();
        let mut slide_names = Vec::new();

        // First, collect slide file names
        for i in 0..archive.len() {
            let file = archive.by_index(i)?;
            let name = file.name().to_string();

            // Check if this is a slide file
            if name.starts_with("ppt/slides/slide") && name.ends_with(".xml") {
                slide_names.push(name);
            }
        }

        // Sort slide names to ensure consistent ordering
        slide_names.sort();

        // Then read the content of each slide
        for name in slide_names {
            let content = read_zip_file(&mut archive, &name)?;
            slide_contents.push((name, content));
        }

        Ok(slide_contents)
    }

    /// Get the PowerPoint text tags used for text extraction and replacement
    fn text_tags() -> &'static [&'static str] {
        &["a:t"]
    }

    /// Get the number of slides in the presentation
    pub fn slide_count(&self) -> usize {
        self.slide_contents.len()
    }

    /// Get text from a specific slide
    pub fn get_slide_text(&self, slide_index: usize) -> Result<String, DocumentError> {
        if slide_index >= self.slide_contents.len() {
            return Err(DocumentError::InvalidStructure {
                reason: format!(
                    "Slide index {} out of range (0-{})",
                    slide_index,
                    self.slide_contents.len() - 1
                ),
            });
        }

        let (_, content) = &self.slide_contents[slide_index];
        extract_text_from_xml(content, Self::text_tags())
    }

    /// Replace text in a specific slide
    pub fn replace_text_in_slide(
        &mut self,
        slide_index: usize,
        old: &str,
        new: &str,
    ) -> Result<usize, DocumentError> {
        if slide_index >= self.slide_contents.len() {
            return Err(DocumentError::InvalidStructure {
                reason: format!(
                    "Slide index {} out of range (0-{})",
                    slide_index,
                    self.slide_contents.len() - 1
                ),
            });
        }

        let (_, content) = &mut self.slide_contents[slide_index];
        let (new_content, count) = replace_text_in_xml(content, Self::text_tags(), old, new)?;

        if count > 0 {
            *content = new_content;
            self.modified = true;
            debug!("Replaced {} occurrences in slide {}", count, slide_index);
        }

        Ok(count)
    }

    /// Extract metadata from core.xml properties
    pub fn get_metadata(&self) -> Result<PowerPointMetadata, DocumentError> {
        use crate::utils::extract_zip;
        let mut archive = extract_zip(&self.archive_data)?;

        let mut metadata = PowerPointMetadata::default();
        metadata.total_slides = self.slide_contents.len();

        // Try to read core.xml for basic metadata
        if let Ok(core_xml) = read_zip_file(&mut archive, "docProps/core.xml") {
            metadata = self.parse_core_properties(&core_xml)?;
            metadata.total_slides = self.slide_contents.len(); // Override with actual slide count
        }

        // Try to read app.xml for additional metadata
        if let Ok(app_xml) = read_zip_file(&mut archive, "docProps/app.xml") {
            self.parse_app_properties(&app_xml, &mut metadata)?;
        }

        Ok(metadata)
    }

    /// Parse core properties XML
    fn parse_core_properties(&self, xml_data: &[u8]) -> Result<PowerPointMetadata, DocumentError> {
        let mut metadata = PowerPointMetadata::default();
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
        metadata: &mut PowerPointMetadata,
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
                        "Slides" => {
                            if let Ok(slides) = text_content.parse::<usize>() {
                                metadata.total_slides = slides;
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

impl DocumentProvider for PowerPointProvider {
    fn replace_text(&mut self, old: &str, new: &str) -> Result<usize, DocumentError> {
        debug!(
            "Replacing text '{}' with '{}' in PowerPoint document",
            old, new
        );

        let mut total_count = 0;

        for (slide_name, content) in &mut self.slide_contents {
            let (new_content, count) = replace_text_in_xml(content, Self::text_tags(), old, new)?;

            if count > 0 {
                *content = new_content;
                total_count += count;
                debug!("Replaced {} occurrences in {}", count, slide_name);
            }
        }

        if total_count > 0 {
            self.modified = true;
            debug!(
                "Replaced {} total occurrences in PowerPoint document",
                total_count
            );
        }

        Ok(total_count)
    }

    fn save(&self) -> Result<(), DocumentError> {
        if !self.modified {
            debug!("No changes to save in PowerPoint document");
            return Ok(());
        }

        debug!("Saving PowerPoint document to: {}", self.path.display());
        self.save_as(&self.path)
    }

    fn save_as(&self, path: &Path) -> Result<(), DocumentError> {
        debug!("Saving PowerPoint document as: {}", path.display());

        let file = std::fs::File::create(path)?;

        // Prepare replacements map
        let mut replacements = HashMap::new();
        for (slide_name, content) in &self.slide_contents {
            replacements.insert(slide_name.clone(), content.clone());
        }

        // Copy archive with replacements
        copy_zip_with_replacements(&self.archive_data, file, &replacements)?;

        debug!("PowerPoint document saved successfully");
        Ok(())
    }

    fn get_text(&self) -> Result<String, DocumentError> {
        debug!("Extracting text from PowerPoint document");

        let mut all_text = String::new();

        for (slide_index, (_, content)) in self.slide_contents.iter().enumerate() {
            let slide_text = extract_text_from_xml(content, Self::text_tags())?;

            if !slide_text.is_empty() {
                if slide_index > 0 {
                    all_text.push_str("\n\n--- Slide ");
                    all_text.push_str(&(slide_index + 1).to_string());
                    all_text.push_str(" ---\n");
                }
                all_text.push_str(&slide_text);
            }
        }

        Ok(all_text.trim().to_string())
    }

    fn is_modified(&self) -> bool {
        self.modified
    }

    fn get_path(&self) -> &Path {
        &self.path
    }

    fn document_type(&self) -> DocumentType {
        DocumentType::PowerPoint
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_minimal_pptx() -> Vec<u8> {
        // This is a minimal PowerPoint document structure for testing
        // In a real implementation, you'd want to use a proper minimal template
        let mut zip_data = Vec::new();
        {
            use zip::{write::SimpleFileOptions, ZipWriter};
            let mut writer = ZipWriter::new(std::io::Cursor::new(&mut zip_data));

            // Add minimal slide1.xml
            let slide_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<p:sld xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main" 
       xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main">
    <p:cSld>
        <p:spTree>
            <p:sp>
                <p:txBody>
                    <a:p>
                        <a:r>
                            <a:t>Hello PowerPoint</a:t>
                        </a:r>
                    </a:p>
                </p:txBody>
            </p:sp>
        </p:spTree>
    </p:cSld>
</p:sld>"#;

            writer
                .start_file("ppt/slides/slide1.xml", SimpleFileOptions::default())
                .unwrap();
            writer.write_all(slide_xml.as_bytes()).unwrap();

            // Add a second slide
            let slide2_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<p:sld xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main" 
       xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main">
    <p:cSld>
        <p:spTree>
            <p:sp>
                <p:txBody>
                    <a:p>
                        <a:r>
                            <a:t>Second slide</a:t>
                        </a:r>
                    </a:p>
                </p:txBody>
            </p:sp>
        </p:spTree>
    </p:cSld>
</p:sld>"#;

            writer
                .start_file("ppt/slides/slide2.xml", SimpleFileOptions::default())
                .unwrap();
            writer.write_all(slide2_xml.as_bytes()).unwrap();

            writer.finish().unwrap();
        }
        zip_data
    }

    #[test]
    fn test_powerpoint_document_text_extraction() {
        let zip_data = create_minimal_pptx();
        let temp_file = NamedTempFile::new().unwrap();
        std::fs::write(temp_file.path(), &zip_data).unwrap();

        let doc = PowerPointProvider::open(temp_file.path()).unwrap();
        let text = doc.get_text().unwrap();
        assert!(text.contains("Hello PowerPoint"));
        assert!(text.contains("Second slide"));
        assert_eq!(doc.slide_count(), 2);
    }

    #[test]
    fn test_powerpoint_document_text_replacement() {
        let zip_data = create_minimal_pptx();
        let temp_file = NamedTempFile::new().unwrap();
        std::fs::write(temp_file.path(), &zip_data).unwrap();

        let mut doc = PowerPointProvider::open(temp_file.path()).unwrap();
        let count = doc.replace_text("Hello", "Hi").unwrap();
        assert_eq!(count, 1);
        assert!(doc.is_modified());

        let text = doc.get_text().unwrap();
        assert!(text.contains("Hi PowerPoint"));
    }

    #[test]
    fn test_powerpoint_slide_specific_operations() {
        let zip_data = create_minimal_pptx();
        let temp_file = NamedTempFile::new().unwrap();
        std::fs::write(temp_file.path(), &zip_data).unwrap();

        let mut doc = PowerPointProvider::open(temp_file.path()).unwrap();

        // Test slide-specific text extraction
        let slide1_text = doc.get_slide_text(0).unwrap();
        assert_eq!(slide1_text, "Hello PowerPoint");

        let slide2_text = doc.get_slide_text(1).unwrap();
        assert_eq!(slide2_text, "Second slide");

        // Test slide-specific text replacement
        let count = doc.replace_text_in_slide(1, "Second", "Third").unwrap();
        assert_eq!(count, 1);

        let updated_text = doc.get_slide_text(1).unwrap();
        assert_eq!(updated_text, "Third slide");
    }
}
