//! Integration tests for dox-document crate

use dox_document::{
    create_provider, DocumentProvider, DocumentType, PowerPointProvider, WordProvider,
};
use std::io::Write;
use tempfile::NamedTempFile;

fn create_test_docx() -> Vec<u8> {
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
                <w:t>Test document with placeholder: {{NAME}}</w:t>
            </w:r>
        </w:p>
        <w:p>
            <w:r>
                <w:t>Another line with {{VALUE}}</w:t>
            </w:r>
        </w:p>
    </w:body>
</w:document>"#;

        writer
            .start_file("word/document.xml", SimpleFileOptions::default())
            .unwrap();
        writer.write_all(doc_xml.as_bytes()).unwrap();

        // Add minimal _rels/.rels
        writer
            .start_file("_rels/.rels", SimpleFileOptions::default())
            .unwrap();
        writer.write_all(b"<Relationships/>").unwrap();

        writer.finish().unwrap();
    }
    zip_data
}

fn create_test_pptx() -> Vec<u8> {
    let mut zip_data = Vec::new();
    {
        use zip::{write::SimpleFileOptions, ZipWriter};
        let mut writer = ZipWriter::new(std::io::Cursor::new(&mut zip_data));

        // Add slide1.xml
        let slide_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<p:sld xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main" 
       xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main">
    <p:cSld>
        <p:spTree>
            <p:sp>
                <p:txBody>
                    <a:p>
                        <a:r>
                            <a:t>Presentation title: {{TITLE}}</a:t>
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

        // Add slide2.xml
        let slide2_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<p:sld xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main" 
       xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main">
    <p:cSld>
        <p:spTree>
            <p:sp>
                <p:txBody>
                    <a:p>
                        <a:r>
                            <a:t>Content: {{CONTENT}}</a:t>
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
fn test_document_factory() {
    // Test Word document
    let docx_data = create_test_docx();
    let temp_docx = NamedTempFile::with_suffix(".docx").unwrap();
    std::fs::write(temp_docx.path(), &docx_data).unwrap();

    let doc = create_provider(temp_docx.path()).unwrap();
    let text = doc.get_text().unwrap();
    assert!(text.contains("Test document"));
    assert!(text.contains("{{NAME}}"));

    // Test PowerPoint document
    let pptx_data = create_test_pptx();
    let temp_pptx = NamedTempFile::with_suffix(".pptx").unwrap();
    std::fs::write(temp_pptx.path(), &pptx_data).unwrap();

    let doc = create_provider(temp_pptx.path()).unwrap();
    let text = doc.get_text().unwrap();
    assert!(text.contains("Presentation title"));
    assert!(text.contains("{{TITLE}}"));
}

#[test]
fn test_word_document_operations() {
    let docx_data = create_test_docx();
    let temp_file = NamedTempFile::with_suffix(".docx").unwrap();
    std::fs::write(temp_file.path(), &docx_data).unwrap();

    let mut doc = WordProvider::open(temp_file.path()).unwrap();

    // Test initial state
    assert!(!doc.is_modified());
    assert_eq!(doc.document_type(), DocumentType::Word);

    // Test text replacement
    let count1 = doc.replace_text("{{NAME}}", "John Doe").unwrap();
    assert_eq!(count1, 1);
    assert!(doc.is_modified());

    let count2 = doc.replace_text("{{VALUE}}", "42").unwrap();
    assert_eq!(count2, 1);

    // Verify changes
    let text = doc.get_text().unwrap();
    assert!(text.contains("John Doe"));
    assert!(text.contains("42"));
    assert!(!text.contains("{{NAME}}"));
    assert!(!text.contains("{{VALUE}}"));

    // Test save functionality
    doc.save().unwrap();
}

#[test]
fn test_powerpoint_document_operations() {
    let pptx_data = create_test_pptx();
    let temp_file = NamedTempFile::with_suffix(".pptx").unwrap();
    std::fs::write(temp_file.path(), &pptx_data).unwrap();

    let mut doc = PowerPointProvider::open(temp_file.path()).unwrap();

    // Test initial state
    assert!(!doc.is_modified());
    assert_eq!(doc.document_type(), DocumentType::PowerPoint);
    assert_eq!(doc.slide_count(), 2);

    // Test slide-specific text extraction
    let slide1_text = doc.get_slide_text(0).unwrap();
    assert!(slide1_text.contains("{{TITLE}}"));

    let slide2_text = doc.get_slide_text(1).unwrap();
    assert!(slide2_text.contains("{{CONTENT}}"));

    // Test text replacement
    let count1 = doc.replace_text("{{TITLE}}", "My Presentation").unwrap();
    assert_eq!(count1, 1);
    assert!(doc.is_modified());

    let count2 = doc.replace_text("{{CONTENT}}", "Important data").unwrap();
    assert_eq!(count2, 1);

    // Verify changes
    let text = doc.get_text().unwrap();
    assert!(text.contains("My Presentation"));
    assert!(text.contains("Important data"));
    assert!(!text.contains("{{TITLE}}"));
    assert!(!text.contains("{{CONTENT}}"));

    // Test save functionality
    doc.save().unwrap();
}

#[test]
fn test_save_as_functionality() {
    let docx_data = create_test_docx();
    let temp_source = NamedTempFile::with_suffix(".docx").unwrap();
    let temp_dest = NamedTempFile::with_suffix(".docx").unwrap();

    std::fs::write(temp_source.path(), &docx_data).unwrap();

    let mut doc = WordProvider::open(temp_source.path()).unwrap();
    doc.replace_text("{{NAME}}", "Jane Doe").unwrap();

    // Save to different location
    doc.save_as(temp_dest.path()).unwrap();

    // Verify the new file contains our changes
    let new_doc = WordProvider::open(temp_dest.path()).unwrap();
    let text = new_doc.get_text().unwrap();
    assert!(text.contains("Jane Doe"));
    assert!(!text.contains("{{NAME}}"));
}

#[test]
fn test_error_handling() {
    use dox_document::DocumentError;

    // Test opening non-existent file
    let result = WordProvider::open(std::path::Path::new("nonexistent.docx"));
    assert!(matches!(
        result.unwrap_err(),
        DocumentError::DocumentNotFound { .. }
    ));

    // Test unsupported file type (using .xyz extension)
    let temp_file = NamedTempFile::with_suffix(".xyz").unwrap();
    std::fs::write(temp_file.path(), b"not a document").unwrap();

    let result = create_provider(temp_file.path());
    assert!(matches!(
        result.unwrap_err(),
        DocumentError::UnsupportedFormat { .. }
    ));
}
