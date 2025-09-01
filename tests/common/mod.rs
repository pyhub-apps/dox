use assert_fs::prelude::*;
use assert_fs::TempDir;
use std::path::{Path, PathBuf};
use std::fs;
use tempfile::Builder;

/// Test fixture for managing temporary directories and files
pub struct TestFixture {
    pub temp_dir: TempDir,
    pub base_path: PathBuf,
}

impl TestFixture {
    /// Create a new test fixture with a temporary directory
    pub fn new() -> Self {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path().to_path_buf();
        
        Self {
            temp_dir,
            base_path,
        }
    }
    
    /// Create a test file with content
    pub fn create_file(&self, name: &str, content: &str) -> PathBuf {
        let file_path = self.base_path.join(name);
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(&file_path, content).unwrap();
        file_path
    }
    
    /// Create a YAML rule file for testing
    pub fn create_rule_file(&self, rules: &str) -> PathBuf {
        self.create_file("rules.yaml", rules)
    }
    
    /// Create a test Word document structure
    pub fn create_word_doc(&self, name: &str) -> PathBuf {
        let doc_path = self.base_path.join(name);
        
        // Create a minimal Word document structure
        let zip_path = doc_path.with_extension("docx");
        let file = fs::File::create(&zip_path).unwrap();
        let mut zip = zip::ZipWriter::new(file);
        
        // Add minimal required files
        let options = zip::write::SimpleFileOptions::default();
        
        // _rels/.rels
        zip.add_directory("_rels", options).unwrap();
        zip.start_file("_rels/.rels", options).unwrap();
        use std::io::Write;
        zip.write_all(br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
    <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>
</Relationships>"#).unwrap();
        
        // word/document.xml
        zip.add_directory("word", options).unwrap();
        zip.start_file("word/document.xml", options).unwrap();
        zip.write_all(br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
    <w:body>
        <w:p>
            <w:r>
                <w:t>Test Document</w:t>
            </w:r>
        </w:p>
    </w:body>
</w:document>"#).unwrap();
        
        // [Content_Types].xml
        zip.start_file("[Content_Types].xml", options).unwrap();
        zip.write_all(br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
    <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
    <Default Extension="xml" ContentType="application/xml"/>
    <Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>
</Types>"#).unwrap();
        
        zip.finish().unwrap();
        zip_path
    }
    
    /// Create a test PowerPoint document structure
    pub fn create_ppt_doc(&self, name: &str) -> PathBuf {
        let doc_path = self.base_path.join(name);
        
        // Create a minimal PowerPoint document structure
        let zip_path = doc_path.with_extension("pptx");
        let file = fs::File::create(&zip_path).unwrap();
        let mut zip = zip::ZipWriter::new(file);
        
        let options = zip::write::SimpleFileOptions::default();
        
        // _rels/.rels
        zip.add_directory("_rels", options).unwrap();
        zip.start_file("_rels/.rels", options).unwrap();
        use std::io::Write;
        zip.write_all(br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
    <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="ppt/presentation.xml"/>
</Relationships>"#).unwrap();
        
        // ppt/presentation.xml
        zip.add_directory("ppt", options).unwrap();
        zip.start_file("ppt/presentation.xml", options).unwrap();
        zip.write_all(br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:presentation xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main">
    <p:sldIdLst/>
    <p:sldSz cx="9144000" cy="6858000"/>
</p:presentation>"#).unwrap();
        
        // [Content_Types].xml
        zip.start_file("[Content_Types].xml", options).unwrap();
        zip.write_all(br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
    <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
    <Default Extension="xml" ContentType="application/xml"/>
    <Override PartName="/ppt/presentation.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.presentation.main+xml"/>
</Types>"#).unwrap();
        
        zip.finish().unwrap();
        zip_path
    }
    
    /// Create a test markdown file
    pub fn create_markdown(&self, name: &str, content: &str) -> PathBuf {
        let file_path = self.base_path.join(name);
        fs::write(&file_path, content).unwrap();
        file_path
    }
    
    /// Create a test Excel file
    pub fn create_excel(&self, name: &str) -> PathBuf {
        let file_path = self.base_path.join(name);
        let workbook = rust_xlsxwriter::Workbook::new();
        workbook.save(&file_path).unwrap();
        file_path
    }
    
    /// Get the contents of a file
    pub fn read_file(&self, path: &Path) -> String {
        fs::read_to_string(path).unwrap()
    }
    
    /// Check if a file exists
    pub fn file_exists(&self, name: &str) -> bool {
        self.base_path.join(name).exists()
    }
}

impl Default for TestFixture {
    fn default() -> Self {
        Self::new()
    }
}

/// Sample YAML rules for testing
pub mod sample_rules {
    pub const BASIC_REPLACE: &str = r#"
rules:
  - pattern: "{{name}}"
    replacement: "John Doe"
  - pattern: "{{date}}"
    replacement: "2024-01-01"
  - pattern: "{{company}}"
    replacement: "Acme Corp"
"#;
    
    pub const REGEX_REPLACE: &str = r#"
rules:
  - pattern: '\d{4}-\d{2}-\d{2}'
    replacement: "2024-12-31"
    regex: true
  - pattern: '[A-Z]{2,}'
    replacement: "REPLACED"
    regex: true
"#;
    
    pub const CONDITIONAL_REPLACE: &str = r#"
rules:
  - pattern: "{{env:USER}}"
    replacement: "test_user"
    condition: "env"
  - pattern: "{{file:data.txt}}"
    replacement: "file_content"
    condition: "file_exists"
"#;
}

/// Sample document contents for testing
pub mod sample_docs {
    pub const MARKDOWN_TEMPLATE: &str = r#"
# {{title}}

Author: {{name}}
Date: {{date}}

## Introduction

Welcome to {{company}}!

## Content

This document contains {{count}} items.
"#;
    
    pub const TEXT_TEMPLATE: &str = r#"
Dear {{name}},

This is a confirmation for your appointment on {{date}} at {{time}}.

Location: {{address}}

Best regards,
{{company}}
"#;
}

/// Helper functions for assertions
pub mod assertions {
    use super::*;
    
    /// Assert that a file contains specific text
    pub fn assert_file_contains(path: &Path, expected: &str) {
        let content = fs::read_to_string(path).unwrap();
        assert!(
            content.contains(expected),
            "File {:?} does not contain '{}'",
            path,
            expected
        );
    }
    
    /// Assert that a file does not contain specific text
    pub fn assert_file_not_contains(path: &Path, unexpected: &str) {
        let content = fs::read_to_string(path).unwrap();
        assert!(
            !content.contains(unexpected),
            "File {:?} unexpectedly contains '{}'",
            path,
            unexpected
        );
    }
    
    /// Assert that two files have the same content
    pub fn assert_files_equal(path1: &Path, path2: &Path) {
        let content1 = fs::read_to_string(path1).unwrap();
        let content2 = fs::read_to_string(path2).unwrap();
        assert_eq!(
            content1, content2,
            "Files {:?} and {:?} have different contents",
            path1, path2
        );
    }
}