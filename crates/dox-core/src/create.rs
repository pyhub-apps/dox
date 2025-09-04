//! Document creation from Markdown
//!
//! This module provides functionality to create Word and PowerPoint documents
//! from Markdown content, with support for templates and advanced formatting.

use anyhow::Result;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;
use std::path::Path;
use zip::{write::SimpleFileOptions, ZipWriter};

/// Document creation request
#[derive(Debug, Clone)]
pub struct CreateRequest {
    /// Markdown content to convert
    pub content: String,
    /// Output format
    pub format: OutputFormat,
    /// Template file path (optional)
    pub template_path: Option<String>,
    /// Output file path
    pub output_path: String,
    /// Creation options
    pub options: CreateOptions,
}

/// Output format for document creation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    /// Word document (.docx)
    Word,
    /// PowerPoint presentation (.pptx)
    PowerPoint,
}

impl OutputFormat {
    /// Get the file extension for this format
    pub fn extension(&self) -> &'static str {
        match self {
            OutputFormat::Word => "docx",
            OutputFormat::PowerPoint => "pptx",
        }
    }

    /// Detect format from file extension
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "docx" => Some(OutputFormat::Word),
            "pptx" => Some(OutputFormat::PowerPoint),
            _ => None,
        }
    }

    /// Get format name as string
    pub fn as_str(&self) -> &'static str {
        match self {
            OutputFormat::Word => "word",
            OutputFormat::PowerPoint => "powerpoint",
        }
    }
}

/// Document creation options
#[derive(Debug, Clone)]
pub struct CreateOptions {
    /// Preserve original formatting
    pub preserve_formatting: bool,
    /// Include table of contents
    pub include_toc: bool,
    /// Page numbering
    pub page_numbers: bool,
    /// Header and footer text
    pub header: Option<String>,
    pub footer: Option<String>,
    /// Document title
    pub title: Option<String>,
    /// Document author
    pub author: Option<String>,
    /// Language for document
    pub language: String,
}

impl Default for CreateOptions {
    fn default() -> Self {
        CreateOptions {
            preserve_formatting: true,
            include_toc: false,
            page_numbers: true,
            header: None,
            footer: None,
            title: None,
            author: None,
            language: "ko".to_string(),
        }
    }
}

/// Parsed Markdown document structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkdownDocument {
    /// Document title (extracted from first H1 or metadata)
    pub title: Option<String>,
    /// Document metadata
    pub metadata: MarkdownMetadata,
    /// Document sections
    pub sections: Vec<MarkdownSection>,
}

/// Markdown document metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct MarkdownMetadata {
    /// Document title
    pub title: Option<String>,
    /// Document author
    pub author: Option<String>,
    /// Creation date
    pub date: Option<String>,
    /// Tags or keywords
    pub tags: Vec<String>,
    /// Custom metadata fields
    pub custom: std::collections::HashMap<String, String>,
}


/// Markdown document section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkdownSection {
    /// Section title
    pub title: Option<String>,
    /// Section level (1-6 for headings)
    pub level: u8,
    /// Section content elements
    pub content: Vec<MarkdownElement>,
}

/// Markdown content element
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MarkdownElement {
    /// Heading with level and text
    Heading { level: u8, text: String },
    /// Paragraph with text content
    Paragraph { text: String },
    /// Unordered list
    UnorderedList { items: Vec<String> },
    /// Ordered list
    OrderedList { items: Vec<String> },
    /// Code block with language and content
    CodeBlock {
        language: Option<String>,
        code: String,
    },
    /// Inline code
    InlineCode { code: String },
    /// Table with headers and rows
    Table {
        headers: Vec<String>,
        rows: Vec<Vec<String>>,
    },
    /// Image with alt text and URL
    Image { alt: String, url: String },
    /// Link with text and URL
    Link { text: String, url: String },
    /// Blockquote with content
    Blockquote { content: String },
    /// Horizontal rule
    HorizontalRule,
    /// Line break
    LineBreak,
    /// Bold text
    Bold { text: String },
    /// Italic text
    Italic { text: String },
    /// Strikethrough text
    Strikethrough { text: String },
}

/// Markdown parser for document creation
pub struct MarkdownParser {
    options: CreateOptions,
}

impl MarkdownParser {
    /// Create a new Markdown parser with options
    pub fn new(options: CreateOptions) -> Self {
        MarkdownParser { options }
    }

    /// Parse Markdown content into structured document
    pub fn parse(&self, content: &str) -> Result<MarkdownDocument> {
        use pulldown_cmark::{Event, Parser, Tag, TagEnd};

        let mut document = MarkdownDocument {
            title: None,
            metadata: MarkdownMetadata::default(),
            sections: Vec::new(),
        };

        // Parse YAML frontmatter if present
        let (content_without_frontmatter, metadata) = self.extract_frontmatter(content);
        document.metadata = metadata;

        if document.metadata.title.is_some() {
            document.title = document.metadata.title.clone();
        }

        let parser = Parser::new(&content_without_frontmatter);
        let mut current_section = MarkdownSection {
            title: None,
            level: 1,
            content: Vec::new(),
        };

        let mut in_heading = false;
        let mut heading_level = 1u8;
        let mut heading_text = String::new();
        let mut in_paragraph = false;
        let mut paragraph_text = String::new();
        let mut in_code_block = false;
        let mut code_language = None;
        let mut code_content = String::new();
        let mut list_items = Vec::new();
        let mut in_list = false;
        let mut list_ordered = false;

        for event in parser {
            match event {
                Event::Start(Tag::Heading { level, .. }) => {
                    self.finish_current_element(
                        &mut current_section,
                        &mut in_paragraph,
                        &mut paragraph_text,
                        &mut in_code_block,
                        &mut code_content,
                        &mut code_language,
                        &mut in_list,
                        &mut list_items,
                        list_ordered,
                    );

                    in_heading = true;
                    heading_level = level as u8;
                    heading_text.clear();
                }
                Event::End(TagEnd::Heading(_)) => {
                    in_heading = false;

                    // If this is the first H1 and we don't have a title, use it
                    if heading_level == 1 && document.title.is_none() {
                        document.title = Some(heading_text.clone());
                    }

                    // Save previous section and start new one for headings
                    if heading_level <= 2 && !current_section.content.is_empty() {
                        document.sections.push(current_section);
                        current_section = MarkdownSection {
                            title: Some(heading_text.clone()),
                            level: heading_level,
                            content: Vec::new(),
                        };
                    }

                    current_section.content.push(MarkdownElement::Heading {
                        level: heading_level,
                        text: heading_text.clone(),
                    });
                }
                Event::Start(Tag::Paragraph) => {
                    self.finish_current_element(
                        &mut current_section,
                        &mut in_paragraph,
                        &mut paragraph_text,
                        &mut in_code_block,
                        &mut code_content,
                        &mut code_language,
                        &mut in_list,
                        &mut list_items,
                        list_ordered,
                    );
                    in_paragraph = true;
                    paragraph_text.clear();
                }
                Event::End(TagEnd::Paragraph) => {
                    if in_paragraph {
                        current_section.content.push(MarkdownElement::Paragraph {
                            text: paragraph_text.clone(),
                        });
                        in_paragraph = false;
                    }
                }
                Event::Start(Tag::CodeBlock(kind)) => {
                    self.finish_current_element(
                        &mut current_section,
                        &mut in_paragraph,
                        &mut paragraph_text,
                        &mut in_code_block,
                        &mut code_content,
                        &mut code_language,
                        &mut in_list,
                        &mut list_items,
                        list_ordered,
                    );

                    in_code_block = true;
                    code_content.clear();
                    code_language = match kind {
                        pulldown_cmark::CodeBlockKind::Fenced(lang) => {
                            if lang.is_empty() {
                                None
                            } else {
                                Some(lang.to_string())
                            }
                        }
                        pulldown_cmark::CodeBlockKind::Indented => None,
                    };
                }
                Event::End(TagEnd::CodeBlock) => {
                    if in_code_block {
                        current_section.content.push(MarkdownElement::CodeBlock {
                            language: code_language.clone(),
                            code: code_content.clone(),
                        });
                        in_code_block = false;
                    }
                }
                Event::Start(Tag::List(start)) => {
                    self.finish_current_element(
                        &mut current_section,
                        &mut in_paragraph,
                        &mut paragraph_text,
                        &mut in_code_block,
                        &mut code_content,
                        &mut code_language,
                        &mut in_list,
                        &mut list_items,
                        list_ordered,
                    );

                    in_list = true;
                    list_ordered = start.is_some();
                    list_items.clear();
                }
                Event::End(TagEnd::List(_)) => {
                    if in_list {
                        if list_ordered {
                            current_section.content.push(MarkdownElement::OrderedList {
                                items: list_items.clone(),
                            });
                        } else {
                            current_section
                                .content
                                .push(MarkdownElement::UnorderedList {
                                    items: list_items.clone(),
                                });
                        }
                        in_list = false;
                    }
                }
                Event::Start(Tag::Item) => {
                    // Start collecting list item text
                }
                Event::End(TagEnd::Item) => {
                    // This gets handled in the text event
                }
                Event::Text(text) => {
                    if in_heading {
                        heading_text.push_str(&text);
                    } else if in_paragraph {
                        paragraph_text.push_str(&text);
                    } else if in_code_block {
                        code_content.push_str(&text);
                    } else if in_list {
                        list_items.push(text.to_string());
                    }
                }
                Event::Code(code) => {
                    let code_element = MarkdownElement::InlineCode {
                        code: code.to_string(),
                    };
                    if in_paragraph {
                        paragraph_text.push_str(&format!("`{}`", code));
                    } else {
                        current_section.content.push(code_element);
                    }
                }
                Event::Rule => {
                    self.finish_current_element(
                        &mut current_section,
                        &mut in_paragraph,
                        &mut paragraph_text,
                        &mut in_code_block,
                        &mut code_content,
                        &mut code_language,
                        &mut in_list,
                        &mut list_items,
                        list_ordered,
                    );
                    current_section
                        .content
                        .push(MarkdownElement::HorizontalRule);
                }
                _ => {
                    // Handle other events as needed
                }
            }
        }

        // Finish any remaining elements
        self.finish_current_element(
            &mut current_section,
            &mut in_paragraph,
            &mut paragraph_text,
            &mut in_code_block,
            &mut code_content,
            &mut code_language,
            &mut in_list,
            &mut list_items,
            list_ordered,
        );

        // Add the last section
        if !current_section.content.is_empty() {
            document.sections.push(current_section);
        }

        // If no sections were created, create a default one
        if document.sections.is_empty() && !content_without_frontmatter.trim().is_empty() {
            document.sections.push(MarkdownSection {
                title: document.title.clone(),
                level: 1,
                content: vec![MarkdownElement::Paragraph {
                    text: content_without_frontmatter.trim().to_string(),
                }],
            });
        }

        Ok(document)
    }

    /// Helper function to finish current parsing element
    fn finish_current_element(
        &self,
        current_section: &mut MarkdownSection,
        in_paragraph: &mut bool,
        paragraph_text: &mut String,
        in_code_block: &mut bool,
        code_content: &mut String,
        code_language: &mut Option<String>,
        in_list: &mut bool,
        list_items: &mut Vec<String>,
        list_ordered: bool,
    ) {
        if *in_paragraph && !paragraph_text.trim().is_empty() {
            current_section.content.push(MarkdownElement::Paragraph {
                text: paragraph_text.clone(),
            });
            *in_paragraph = false;
        }

        if *in_code_block && !code_content.trim().is_empty() {
            current_section.content.push(MarkdownElement::CodeBlock {
                language: code_language.clone(),
                code: code_content.clone(),
            });
            *in_code_block = false;
        }

        if *in_list && !list_items.is_empty() {
            if list_ordered {
                current_section.content.push(MarkdownElement::OrderedList {
                    items: list_items.clone(),
                });
            } else {
                current_section
                    .content
                    .push(MarkdownElement::UnorderedList {
                        items: list_items.clone(),
                    });
            }
            *in_list = false;
            list_items.clear();
        }
    }

    /// Extract YAML frontmatter from Markdown content
    fn extract_frontmatter(&self, content: &str) -> (String, MarkdownMetadata) {
        let mut metadata = MarkdownMetadata::default();

        if content.starts_with("---\n") {
            if let Some(end_pos) = content[4..].find("\n---\n") {
                let frontmatter = &content[4..end_pos + 4];
                let remaining_content = &content[end_pos + 8..];

                // Parse YAML frontmatter
                if let Ok(yaml_value) = serde_yaml::from_str::<serde_yaml::Value>(frontmatter) {
                    if let Some(mapping) = yaml_value.as_mapping() {
                        for (key, value) in mapping {
                            if let (Some(key_str), Some(value_str)) = (key.as_str(), value.as_str())
                            {
                                match key_str {
                                    "title" => metadata.title = Some(value_str.to_string()),
                                    "author" => metadata.author = Some(value_str.to_string()),
                                    "date" => metadata.date = Some(value_str.to_string()),
                                    _ => {
                                        metadata
                                            .custom
                                            .insert(key_str.to_string(), value_str.to_string());
                                    }
                                }
                            }
                        }
                    }
                }

                return (remaining_content.to_string(), metadata);
            }
        }

        (content.to_string(), metadata)
    }

    /// Parse Markdown file
    pub fn parse_file(&self, path: &Path) -> Result<MarkdownDocument> {
        let content = std::fs::read_to_string(path)?;
        self.parse(&content)
    }
}

/// Document creator trait for different formats
pub trait DocumentCreator {
    /// Create document from parsed Markdown
    fn create_document(&self, markdown: &MarkdownDocument, request: &CreateRequest) -> Result<()>;

    /// Get supported format
    fn supported_format(&self) -> OutputFormat;
}

/// Factory for creating document creators
pub struct DocumentCreatorFactory;

impl DocumentCreatorFactory {
    /// Create appropriate document creator for format
    pub fn create_creator(format: OutputFormat) -> Result<Box<dyn DocumentCreator>> {
        match format {
            OutputFormat::Word => Ok(Box::new(WordDocumentCreator::new())),
            OutputFormat::PowerPoint => Ok(Box::new(PowerPointDocumentCreator::new())),
        }
    }
}

/// Word document creator
pub struct WordDocumentCreator;

impl Default for WordDocumentCreator {
    fn default() -> Self {
        Self::new()
    }
}

impl WordDocumentCreator {
    pub fn new() -> Self {
        WordDocumentCreator
    }
}

impl DocumentCreator for WordDocumentCreator {
    fn create_document(&self, markdown: &MarkdownDocument, request: &CreateRequest) -> Result<()> {
        // Create Word document structure
        let word_generator = WordDocumentGenerator::new(request.clone());
        word_generator.generate(markdown)?;

        Ok(())
    }

    fn supported_format(&self) -> OutputFormat {
        OutputFormat::Word
    }
}

/// Word document generator using XML-based approach
struct WordDocumentGenerator {
    request: CreateRequest,
}

impl WordDocumentGenerator {
    fn new(request: CreateRequest) -> Self {
        WordDocumentGenerator { request }
    }

    fn generate(&self, markdown: &MarkdownDocument) -> Result<()> {
        use std::fs::File;

        // Create output file
        let output_file = File::create(&self.request.output_path)?;
        let mut zip_writer = ZipWriter::new(output_file);

        // Generate Word document structure
        self.write_content_types(&mut zip_writer)?;
        self.write_app_properties(&mut zip_writer, markdown)?;
        self.write_core_properties(&mut zip_writer, markdown)?;
        self.write_document_relationships(&mut zip_writer)?;
        self.write_main_document(&mut zip_writer, markdown)?;
        self.write_styles(&mut zip_writer)?;

        zip_writer.finish()?;
        Ok(())
    }

    fn write_content_types(&self, zip_writer: &mut ZipWriter<File>) -> Result<()> {
        let content = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
    <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
    <Default Extension="xml" ContentType="application/xml"/>
    <Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>
    <Override PartName="/word/styles.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml"/>
    <Override PartName="/docProps/app.xml" ContentType="application/vnd.openxmlformats-officedocument.extended-properties+xml"/>
    <Override PartName="/docProps/core.xml" ContentType="application/vnd.openxmlformats-package.core-properties+xml"/>
</Types>"#;

        zip_writer.start_file("[Content_Types].xml", SimpleFileOptions::default())?;
        zip_writer.write_all(content.as_bytes())?;
        Ok(())
    }

    fn write_app_properties(
        &self,
        zip_writer: &mut ZipWriter<File>,
        markdown: &MarkdownDocument,
    ) -> Result<()> {
        let content = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Properties xmlns="http://schemas.openxmlformats.org/officeDocument/2006/extended-properties" xmlns:vt="http://schemas.openxmlformats.org/officeDocument/2006/docPropsVTypes">
    <Application>dox CLI</Application>
    <ScaleCrop>false</ScaleCrop>
    <DocSecurity>0</DocSecurity>
    <Company>PyHub Korea</Company>
    <LinksUpToDate>false</LinksUpToDate>
    <SharedDoc>false</SharedDoc>
    <HyperlinksChanged>false</HyperlinksChanged>
    <AppVersion>1.0</AppVersion>
</Properties>"#.to_string();

        zip_writer.start_file("docProps/app.xml", SimpleFileOptions::default())?;
        zip_writer.write_all(content.as_bytes())?;
        Ok(())
    }

    fn write_core_properties(
        &self,
        zip_writer: &mut ZipWriter<File>,
        markdown: &MarkdownDocument,
    ) -> Result<()> {
        let title = markdown.title.as_deref().unwrap_or("Untitled Document");
        let author = markdown.metadata.author.as_deref().unwrap_or("dox CLI");
        let created = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ");

        let content = format!(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<cp:coreProperties xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties" xmlns:dc="http://purl.org/dc/elements/1.1/" xmlns:dcterms="http://purl.org/dc/terms/" xmlns:dcmitype="http://purl.org/dc/dcmitype/" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
    <dc:title>{}</dc:title>
    <dc:creator>{}</dc:creator>
    <cp:lastModifiedBy>{}</cp:lastModifiedBy>
    <cp:revision>1</cp:revision>
    <dcterms:created xsi:type="dcterms:W3CDTF">{}</dcterms:created>
    <dcterms:modified xsi:type="dcterms:W3CDTF">{}</dcterms:modified>
</cp:coreProperties>"#,
            title, author, author, created, created
        );

        zip_writer.start_file("docProps/core.xml", SimpleFileOptions::default())?;
        zip_writer.write_all(content.as_bytes())?;
        Ok(())
    }

    fn write_document_relationships(&self, zip_writer: &mut ZipWriter<File>) -> Result<()> {
        // Main relationships
        let content = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
    <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>
    <Relationship Id="rId2" Type="http://schemas.openxmlformats.org/package/2006/relationships/metadata/core-properties" Target="docProps/core.xml"/>
    <Relationship Id="rId3" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/extended-properties" Target="docProps/app.xml"/>
</Relationships>"#;

        zip_writer.start_file("_rels/.rels", SimpleFileOptions::default())?;
        zip_writer.write_all(content.as_bytes())?;

        // Document relationships
        let doc_rels = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
    <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/>
</Relationships>"#;

        zip_writer.start_file("word/_rels/document.xml.rels", SimpleFileOptions::default())?;
        zip_writer.write_all(doc_rels.as_bytes())?;
        Ok(())
    }

    fn write_main_document(
        &self,
        zip_writer: &mut ZipWriter<File>,
        markdown: &MarkdownDocument,
    ) -> Result<()> {
        let mut document_xml = String::new();
        document_xml.push_str(r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" xmlns:w14="http://schemas.microsoft.com/office/word/2010/wordml">
    <w:body>"#);

        // Add document title if present
        if let Some(title) = &markdown.title {
            document_xml.push_str(&self.create_title_paragraph(title));
        }

        // Convert sections to Word paragraphs
        for section in &markdown.sections {
            for element in &section.content {
                document_xml.push_str(&self.convert_element_to_word(element));
            }
        }

        // Close document
        document_xml.push_str(r#"
        <w:sectPr>
            <w:pgSz w:w="11906" w:h="16838"/>
            <w:pgMar w:top="1440" w:right="1440" w:bottom="1440" w:left="1440" w:header="708" w:footer="708" w:gutter="0"/>
            <w:cols w:space="708"/>
            <w:docGrid w:linePitch="360"/>
        </w:sectPr>
    </w:body>
</w:document>"#);

        zip_writer.start_file("word/document.xml", SimpleFileOptions::default())?;
        zip_writer.write_all(document_xml.as_bytes())?;
        Ok(())
    }

    fn create_title_paragraph(&self, title: &str) -> String {
        format!(
            r#"
        <w:p>
            <w:pPr>
                <w:pStyle w:val="Title"/>
                <w:jc w:val="center"/>
            </w:pPr>
            <w:r>
                <w:rPr>
                    <w:b/>
                    <w:sz w:val="32"/>
                </w:rPr>
                <w:t>{}</w:t>
            </w:r>
        </w:p>"#,
            self.escape_xml(title)
        )
    }

    fn convert_element_to_word(&self, element: &MarkdownElement) -> String {
        match element {
            MarkdownElement::Heading { level, text } => {
                let style = format!("Heading{}", level);
                format!(
                    r#"
        <w:p>
            <w:pPr>
                <w:pStyle w:val="{}"/>
            </w:pPr>
            <w:r>
                <w:rPr>
                    <w:b/>
                </w:rPr>
                <w:t>{}</w:t>
            </w:r>
        </w:p>"#,
                    style,
                    self.escape_xml(text)
                )
            }
            MarkdownElement::Paragraph { text } => {
                format!(
                    r#"
        <w:p>
            <w:r>
                <w:t>{}</w:t>
            </w:r>
        </w:p>"#,
                    self.escape_xml(text)
                )
            }
            MarkdownElement::CodeBlock { language: _, code } => {
                format!(
                    r#"
        <w:p>
            <w:pPr>
                <w:pStyle w:val="Code"/>
            </w:pPr>
            <w:r>
                <w:rPr>
                    <w:rFonts w:ascii="Courier New" w:hAnsi="Courier New"/>
                    <w:sz w:val="20"/>
                </w:rPr>
                <w:t xml:space="preserve">{}</w:t>
            </w:r>
        </w:p>"#,
                    self.escape_xml(code)
                )
            }
            MarkdownElement::UnorderedList { items } => {
                let mut list_xml = String::new();
                for item in items {
                    list_xml.push_str(&format!(
                        r#"
        <w:p>
            <w:pPr>
                <w:pStyle w:val="ListParagraph"/>
                <w:numPr>
                    <w:ilvl w:val="0"/>
                    <w:numId w:val="1"/>
                </w:numPr>
            </w:pPr>
            <w:r>
                <w:t>{}</w:t>
            </w:r>
        </w:p>"#,
                        self.escape_xml(item)
                    ));
                }
                list_xml
            }
            MarkdownElement::OrderedList { items } => {
                let mut list_xml = String::new();
                for item in items {
                    list_xml.push_str(&format!(
                        r#"
        <w:p>
            <w:pPr>
                <w:pStyle w:val="ListParagraph"/>
                <w:numPr>
                    <w:ilvl w:val="0"/>
                    <w:numId w:val="2"/>
                </w:numPr>
            </w:pPr>
            <w:r>
                <w:t>{}</w:t>
            </w:r>
        </w:p>"#,
                        self.escape_xml(item)
                    ));
                }
                list_xml
            }
            MarkdownElement::InlineCode { code } => {
                format!(
                    r#"
        <w:p>
            <w:r>
                <w:rPr>
                    <w:rFonts w:ascii="Courier New" w:hAnsi="Courier New"/>
                </w:rPr>
                <w:t>{}</w:t>
            </w:r>
        </w:p>"#,
                    self.escape_xml(code)
                )
            }
            MarkdownElement::HorizontalRule => r#"
        <w:p>
            <w:pPr>
                <w:pBdr>
                    <w:bottom w:val="single" w:sz="6" w:space="1" w:color="auto"/>
                </w:pBdr>
            </w:pPr>
        </w:p>"#
                .to_string(),
            _ => {
                // Handle other elements or skip
                String::new()
            }
        }
    }

    fn write_styles(&self, zip_writer: &mut ZipWriter<File>) -> Result<()> {
        let styles_xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:styles xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
    <w:docDefaults>
        <w:rPrDefault>
            <w:rPr>
                <w:rFonts w:ascii="Calibri" w:eastAsia="맑은 고딕" w:hAnsi="Calibri"/>
                <w:sz w:val="22"/>
                <w:szCs w:val="22"/>
                <w:lang w:val="ko-KR" w:eastAsia="ko-KR"/>
            </w:rPr>
        </w:rPrDefault>
        <w:pPrDefault>
            <w:pPr>
                <w:spacing w:after="200" w:line="276" w:lineRule="auto"/>
            </w:pPr>
        </w:pPrDefault>
    </w:docDefaults>
    
    <w:style w:type="paragraph" w:default="1" w:styleId="Normal">
        <w:name w:val="Normal"/>
        <w:qFormat/>
    </w:style>
    
    <w:style w:type="paragraph" w:styleId="Title">
        <w:name w:val="Title"/>
        <w:basedOn w:val="Normal"/>
        <w:qFormat/>
        <w:pPr>
            <w:spacing w:before="480" w:after="0"/>
            <w:jc w:val="center"/>
        </w:pPr>
        <w:rPr>
            <w:rFonts w:asciiTheme="majorHAnsi" w:eastAsiaTheme="majorEastAsia" w:hAnsiTheme="majorHAnsi" w:cstheme="majorBidi"/>
            <w:b/>
            <w:sz w:val="56"/>
            <w:szCs w:val="56"/>
        </w:rPr>
    </w:style>
    
    <w:style w:type="paragraph" w:styleId="Heading1">
        <w:name w:val="heading 1"/>
        <w:basedOn w:val="Normal"/>
        <w:next w:val="Normal"/>
        <w:qFormat/>
        <w:pPr>
            <w:keepNext/>
            <w:spacing w:before="240" w:after="0"/>
        </w:pPr>
        <w:rPr>
            <w:b/>
            <w:sz w:val="32"/>
            <w:szCs w:val="32"/>
        </w:rPr>
    </w:style>
    
    <w:style w:type="paragraph" w:styleId="Heading2">
        <w:name w:val="heading 2"/>
        <w:basedOn w:val="Normal"/>
        <w:next w:val="Normal"/>
        <w:qFormat/>
        <w:pPr>
            <w:keepNext/>
            <w:spacing w:before="200" w:after="0"/>
        </w:pPr>
        <w:rPr>
            <w:b/>
            <w:sz w:val="28"/>
            <w:szCs w:val="28"/>
        </w:rPr>
    </w:style>
    
    <w:style w:type="paragraph" w:styleId="Code">
        <w:name w:val="Code"/>
        <w:basedOn w:val="Normal"/>
        <w:pPr>
            <w:spacing w:before="120" w:after="120"/>
            <w:shd w:val="clear" w:color="auto" w:fill="F2F2F2"/>
        </w:pPr>
        <w:rPr>
            <w:rFonts w:ascii="Courier New" w:hAnsi="Courier New"/>
            <w:sz w:val="20"/>
            <w:szCs w:val="20"/>
        </w:rPr>
    </w:style>
    
    <w:style w:type="paragraph" w:styleId="ListParagraph">
        <w:name w:val="List Paragraph"/>
        <w:basedOn w:val="Normal"/>
        <w:qFormat/>
        <w:pPr>
            <w:ind w:left="720"/>
        </w:pPr>
    </w:style>
</w:styles>"#;

        zip_writer.start_file("word/styles.xml", SimpleFileOptions::default())?;
        zip_writer.write_all(styles_xml.as_bytes())?;
        Ok(())
    }

    fn escape_xml(&self, text: &str) -> String {
        text.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&apos;")
    }
}

/// PowerPoint document creator
pub struct PowerPointDocumentCreator;

impl Default for PowerPointDocumentCreator {
    fn default() -> Self {
        Self::new()
    }
}

impl PowerPointDocumentCreator {
    pub fn new() -> Self {
        Self
    }
}

impl DocumentCreator for PowerPointDocumentCreator {
    fn create_document(&self, markdown: &MarkdownDocument, request: &CreateRequest) -> Result<()> {
        let generator = PowerPointDocumentGenerator::new(request.clone());
        generator.generate(markdown)
    }

    fn supported_format(&self) -> OutputFormat {
        OutputFormat::PowerPoint
    }
}

/// PowerPoint document generator
struct PowerPointDocumentGenerator {
    request: CreateRequest,
}

impl PowerPointDocumentGenerator {
    fn new(request: CreateRequest) -> Self {
        Self { request }
    }
}

impl PowerPointDocumentGenerator {
    fn generate(&self, markdown: &MarkdownDocument) -> Result<()> {
        use std::fs::File;

        // Create output file
        let output_file = File::create(&self.request.output_path)?;
        let mut zip_writer = ZipWriter::new(output_file);

        // Write all required PowerPoint files
        self.write_content_types(&mut zip_writer)?;
        self.write_app_properties(&mut zip_writer, markdown)?;
        self.write_core_properties(&mut zip_writer, markdown)?;
        self.write_presentation_relationships(&mut zip_writer)?;
        self.write_main_presentation(&mut zip_writer, markdown)?;
        self.write_slide_master(&mut zip_writer)?;
        self.write_slide_layout(&mut zip_writer)?;
        self.write_theme(&mut zip_writer)?;

        // Generate slides from markdown sections
        self.write_slides(&mut zip_writer, markdown)?;

        zip_writer.finish()?;
        Ok(())
    }

    fn write_content_types(&self, zip_writer: &mut ZipWriter<File>) -> Result<()> {
        let content = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
    <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
    <Default Extension="xml" ContentType="application/xml"/>
    <Override PartName="/ppt/presentation.xml" ContentType="application/vnd.openxmlformats-presentationml.presentation.main+xml"/>
    <Override PartName="/ppt/slideMasters/slideMaster1.xml" ContentType="application/vnd.openxmlformats-presentationml.slideMaster+xml"/>
    <Override PartName="/ppt/slideLayouts/slideLayout1.xml" ContentType="application/vnd.openxmlformats-presentationml.slideLayout+xml"/>
    <Override PartName="/ppt/theme/theme1.xml" ContentType="application/vnd.openxmlformats-officedocument.theme+xml"/>
    <Override PartName="/ppt/slides/slide1.xml" ContentType="application/vnd.openxmlformats-presentationml.slide+xml"/>
    <Override PartName="/docProps/core.xml" ContentType="application/vnd.openxmlformats-package.core-properties+xml"/>
    <Override PartName="/docProps/app.xml" ContentType="application/vnd.openxmlformats-officedocument.extended-properties+xml"/>
</Types>"#;

        zip_writer.start_file("[Content_Types].xml", SimpleFileOptions::default())?;
        zip_writer.write_all(content.as_bytes())?;
        Ok(())
    }

    fn write_app_properties(
        &self,
        zip_writer: &mut ZipWriter<File>,
        _markdown: &MarkdownDocument,
    ) -> Result<()> {
        let content = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Properties xmlns="http://schemas.openxmlformats.org/officeDocument/2006/extended-properties" xmlns:vt="http://schemas.openxmlformats.org/officeDocument/2006/docPropsVTypes">
    <Application>dox</Application>
    <ScaleCrop>false</ScaleCrop>
    <LinksUpToDate>false</LinksUpToDate>
    <SharedDoc>false</SharedDoc>
    <HyperlinksChanged>false</HyperlinksChanged>
    <AppVersion>16.0000</AppVersion>
    <Slides>1</Slides>
    <HiddenSlides>0</HiddenSlides>
    <MMClips>0</MMClips>
    <Notes>0</Notes>
</Properties>"#;

        zip_writer.start_file("docProps/app.xml", SimpleFileOptions::default())?;
        zip_writer.write_all(content.as_bytes())?;
        Ok(())
    }

    fn write_core_properties(
        &self,
        zip_writer: &mut ZipWriter<File>,
        markdown: &MarkdownDocument,
    ) -> Result<()> {
        let now = Utc::now().format("%Y-%m-%dT%H:%M:%SZ");
        let title = markdown.title.as_deref().unwrap_or("Untitled");
        let creator = markdown.metadata.author.as_deref().unwrap_or("dox");

        let content = format!(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<cp:coreProperties xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties" xmlns:dc="http://purl.org/dc/elements/1.1/" xmlns:dcterms="http://purl.org/dc/terms/" xmlns:dcmitype="http://purl.org/dc/dcmitype/" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
    <dc:title>{}</dc:title>
    <dc:creator>{}</dc:creator>
    <cp:lastModifiedBy>{}</cp:lastModifiedBy>
    <dcterms:created xsi:type="dcterms:W3CDTF">{}</dcterms:created>
    <dcterms:modified xsi:type="dcterms:W3CDTF">{}</dcterms:modified>
</cp:coreProperties>"#,
            title, creator, creator, now, now
        );

        zip_writer.start_file("docProps/core.xml", SimpleFileOptions::default())?;
        zip_writer.write_all(content.as_bytes())?;
        Ok(())
    }

    fn write_presentation_relationships(&self, zip_writer: &mut ZipWriter<File>) -> Result<()> {
        // Main relationships file
        let main_rels_content = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
    <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="ppt/presentation.xml"/>
    <Relationship Id="rId2" Type="http://schemas.openxmlformats.org/package/2006/relationships/metadata/core-properties" Target="docProps/core.xml"/>
    <Relationship Id="rId3" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/extended-properties" Target="docProps/app.xml"/>
</Relationships>"#;

        zip_writer.start_file("_rels/.rels", SimpleFileOptions::default())?;
        zip_writer.write_all(main_rels_content.as_bytes())?;

        // Presentation relationships file
        let ppt_rels_content = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
    <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideMaster" Target="slideMasters/slideMaster1.xml"/>
    <Relationship Id="rId2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slide" Target="slides/slide1.xml"/>
    <Relationship Id="rId3" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/theme" Target="theme/theme1.xml"/>
</Relationships>"#;

        zip_writer.start_file(
            "ppt/_rels/presentation.xml.rels",
            SimpleFileOptions::default(),
        )?;
        zip_writer.write_all(ppt_rels_content.as_bytes())?;

        Ok(())
    }

    fn write_main_presentation(
        &self,
        zip_writer: &mut ZipWriter<File>,
        markdown: &MarkdownDocument,
    ) -> Result<()> {
        let title = markdown.title.as_deref().unwrap_or("Untitled");

        let content = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:presentation xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main">
    <p:sldMasterIdLst>
        <p:sldMasterId id="2147483648" r:id="rId1"/>
    </p:sldMasterIdLst>
    <p:sldIdLst>
        <p:sldId id="256" r:id="rId2"/>
    </p:sldIdLst>
    <p:sldSz cx="9144000" cy="6858000" type="screen4x3"/>
    <p:notesSz cx="6858000" cy="9144000"/>
    <p:defaultTextStyle>
        <a:defPPr>
            <a:defRPr lang="ko-KR"/>
        </a:defPPr>
        <a:lvl1pPr marL="0" algn="l" defTabSz="914400" rtl="0" eaLnBrk="1" latinLnBrk="0" hangingPunct="1">
            <a:defRPr sz="1800" kern="1200">
                <a:solidFill>
                    <a:schemeClr val="tx1"/>
                </a:solidFill>
                <a:latin typeface="맑은 고딕" pitchFamily="34" charset="129"/>
                <a:ea typeface="맑은 고딕" pitchFamily="34" charset="129"/>
                <a:cs typeface="맑은 고딕" pitchFamily="34" charset="129"/>
            </a:defRPr>
        </a:lvl1pPr>
    </p:defaultTextStyle>
</p:presentation>"#.to_string();

        zip_writer.start_file("ppt/presentation.xml", SimpleFileOptions::default())?;
        zip_writer.write_all(content.as_bytes())?;
        Ok(())
    }

    fn write_slides(
        &self,
        zip_writer: &mut ZipWriter<File>,
        markdown: &MarkdownDocument,
    ) -> Result<()> {
        // Generate title slide
        let title = markdown.title.as_deref().unwrap_or("Untitled");
        let subtitle = format!("Created with dox | {} sections", markdown.sections.len());

        let slide_content = format!(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:sld xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main">
    <p:cSld>
        <p:spTree>
            <p:nvGrpSpPr>
                <p:cNvPr id="1" name=""/>
                <p:cNvGrpSpPr/>
                <p:nvPr/>
            </p:nvGrpSpPr>
            <p:grpSpPr>
                <a:xfrm>
                    <a:off x="0" y="0"/>
                    <a:ext cx="0" cy="0"/>
                    <a:chOff x="0" y="0"/>
                    <a:chExt cx="0" cy="0"/>
                </a:xfrm>
            </p:grpSpPr>
            <!-- Title placeholder -->
            <p:sp>
                <p:nvSpPr>
                    <p:cNvPr id="2" name="Title"/>
                    <p:cNvSpPr>
                        <a:spLocks noGrp="1"/>
                    </p:cNvSpPr>
                    <p:nvPr>
                        <p:ph type="ctrTitle"/>
                    </p:nvPr>
                </p:nvSpPr>
                <p:spPr/>
                <p:txBody>
                    <a:bodyPr/>
                    <a:lstStyle/>
                    <a:p>
                        <a:r>
                            <a:rPr lang="ko-KR" sz="4400" b="1">
                                <a:solidFill>
                                    <a:srgbClr val="000000"/>
                                </a:solidFill>
                                <a:latin typeface="맑은 고딕"/>
                                <a:ea typeface="맑은 고딕"/>
                            </a:rPr>
                            <a:t>{}</a:t>
                        </a:r>
                    </a:p>
                </p:txBody>
            </p:sp>
            <!-- Subtitle placeholder -->
            <p:sp>
                <p:nvSpPr>
                    <p:cNvPr id="3" name="Subtitle"/>
                    <p:cNvSpPr>
                        <a:spLocks noGrp="1"/>
                    </p:cNvSpPr>
                    <p:nvPr>
                        <p:ph type="subTitle" idx="1"/>
                    </p:nvPr>
                </p:nvSpPr>
                <p:spPr/>
                <p:txBody>
                    <a:bodyPr/>
                    <a:lstStyle/>
                    <a:p>
                        <a:r>
                            <a:rPr lang="ko-KR" sz="2400">
                                <a:solidFill>
                                    <a:srgbClr val="666666"/>
                                </a:solidFill>
                                <a:latin typeface="맑은 고딕"/>
                                <a:ea typeface="맑은 고딕"/>
                            </a:rPr>
                            <a:t>{}</a:t>
                        </a:r>
                    </a:p>
                </p:txBody>
            </p:sp>
        </p:spTree>
    </p:cSld>
    <p:clrMapOvr>
        <a:masterClrMapping/>
    </p:clrMapOvr>
</p:sld>"#,
            self.escape_xml(title),
            self.escape_xml(&subtitle)
        );

        zip_writer.start_file("ppt/slides/slide1.xml", SimpleFileOptions::default())?;
        zip_writer.write_all(slide_content.as_bytes())?;
        Ok(())
    }

    fn write_slide_master(&self, zip_writer: &mut ZipWriter<File>) -> Result<()> {
        let content = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:sldMaster xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main">
    <p:cSld>
        <p:spTree>
            <p:nvGrpSpPr>
                <p:cNvPr id="1" name=""/>
                <p:cNvGrpSpPr/>
                <p:nvPr/>
            </p:nvGrpSpPr>
            <p:grpSpPr>
                <a:xfrm>
                    <a:off x="0" y="0"/>
                    <a:ext cx="0" cy="0"/>
                    <a:chOff x="0" y="0"/>
                    <a:chExt cx="0" cy="0"/>
                </a:xfrm>
            </p:grpSpPr>
        </p:spTree>
    </p:cSld>
    <p:clrMap bg1="lt1" tx1="dk1" bg2="lt2" tx2="dk2" accent1="accent1" accent2="accent2" accent3="accent3" accent4="accent4" accent5="accent5" accent6="accent6" hlink="hlink" folHlink="folHlink"/>
    <p:sldLayoutIdLst>
        <p:sldLayoutId id="2147483649" r:id="rId1"/>
    </p:sldLayoutIdLst>
    <p:txStyles>
        <p:titleStyle>
            <a:lvl1pPr algn="ctr" defTabSz="914400" rtl="0" eaLnBrk="1" latinLnBrk="0" hangingPunct="1">
                <a:defRPr sz="4400" kern="1200">
                    <a:solidFill>
                        <a:schemeClr val="tx1"/>
                    </a:solidFill>
                    <a:latin typeface="+mj-lt"/>
                    <a:ea typeface="+mj-ea"/>
                    <a:cs typeface="+mj-cs"/>
                </a:defRPr>
            </a:lvl1pPr>
        </p:titleStyle>
        <p:bodyStyle>
            <a:lvl1pPr marL="342900" indent="-342900" algn="l" defTabSz="914400" rtl="0" eaLnBrk="1" latinLnBrk="0" hangingPunct="1">
                <a:defRPr sz="2800" kern="1200">
                    <a:solidFill>
                        <a:schemeClr val="tx1"/>
                    </a:solidFill>
                    <a:latin typeface="+mn-lt"/>
                    <a:ea typeface="+mn-ea"/>
                    <a:cs typeface="+mn-cs"/>
                </a:defRPr>
            </a:lvl1pPr>
        </p:bodyStyle>
        <p:otherStyle>
            <a:defPPr>
                <a:defRPr lang="ko-KR">
                    <a:latin typeface="+mn-lt"/>
                    <a:ea typeface="+mn-ea"/>
                    <a:cs typeface="+mn-cs"/>
                </a:defRPr>
            </a:defPPr>
        </p:otherStyle>
    </p:txStyles>
</p:sldMaster>"#;

        zip_writer.start_file(
            "ppt/slideMasters/slideMaster1.xml",
            SimpleFileOptions::default(),
        )?;
        zip_writer.write_all(content.as_bytes())?;
        Ok(())
    }

    fn write_slide_layout(&self, zip_writer: &mut ZipWriter<File>) -> Result<()> {
        let content = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:sldLayout xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main" type="titleSlide" preserve="1">
    <p:cSld name="Title Slide">
        <p:spTree>
            <p:nvGrpSpPr>
                <p:cNvPr id="1" name=""/>
                <p:cNvGrpSpPr/>
                <p:nvPr/>
            </p:nvGrpSpPr>
            <p:grpSpPr>
                <a:xfrm>
                    <a:off x="0" y="0"/>
                    <a:ext cx="0" cy="0"/>
                    <a:chOff x="0" y="0"/>
                    <a:chExt cx="0" cy="0"/>
                </a:xfrm>
            </p:grpSpPr>
        </p:spTree>
    </p:cSld>
    <p:clrMapOvr>
        <a:masterClrMapping/>
    </p:clrMapOvr>
</p:sldLayout>"#;

        zip_writer.start_file(
            "ppt/slideLayouts/slideLayout1.xml",
            SimpleFileOptions::default(),
        )?;
        zip_writer.write_all(content.as_bytes())?;
        Ok(())
    }

    fn write_theme(&self, zip_writer: &mut ZipWriter<File>) -> Result<()> {
        let content = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<a:theme xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" name="Office Theme">
    <a:themeElements>
        <a:clrScheme name="Office">
            <a:dk1>
                <a:sysClr val="windowText" lastClr="000000"/>
            </a:dk1>
            <a:lt1>
                <a:sysClr val="window" lastClr="FFFFFF"/>
            </a:lt1>
            <a:dk2>
                <a:srgbClr val="44546A"/>
            </a:dk2>
            <a:lt2>
                <a:srgbClr val="E7E6E6"/>
            </a:lt2>
            <a:accent1>
                <a:srgbClr val="4472C4"/>
            </a:accent1>
            <a:accent2>
                <a:srgbClr val="E7E6E6"/>
            </a:accent2>
            <a:accent3>
                <a:srgbClr val="A5A5A5"/>
            </a:accent3>
            <a:accent4>
                <a:srgbClr val="FFC000"/>
            </a:accent4>
            <a:accent5>
                <a:srgbClr val="5B9BD5"/>
            </a:accent5>
            <a:accent6>
                <a:srgbClr val="70AD47"/>
            </a:accent6>
            <a:hlink>
                <a:srgbClr val="0563C1"/>
            </a:hlink>
            <a:folHlink>
                <a:srgbClr val="954F72"/>
            </a:folHlink>
        </a:clrScheme>
        <a:fontScheme name="Office">
            <a:majorFont>
                <a:latin typeface="맑은 고딕" panose="020B0604020202020204"/>
                <a:ea typeface=""/>
                <a:cs typeface=""/>
            </a:majorFont>
            <a:minorFont>
                <a:latin typeface="맑은 고딕" panose="020F0502020204030204"/>
                <a:ea typeface=""/>
                <a:cs typeface=""/>
            </a:minorFont>
        </a:fontScheme>
        <a:fmtScheme name="Office">
            <a:fillStyleLst>
                <a:solidFill>
                    <a:schemeClr val="phClr"/>
                </a:solidFill>
                <a:gradFill rotWithShape="1">
                    <a:gsLst>
                        <a:gs pos="0">
                            <a:schemeClr val="phClr">
                                <a:lumMod val="110000"/>
                                <a:satMod val="105000"/>
                                <a:tint val="67000"/>
                            </a:schemeClr>
                        </a:gs>
                        <a:gs pos="50000">
                            <a:schemeClr val="phClr">
                                <a:lumMod val="105000"/>
                                <a:satMod val="103000"/>
                                <a:tint val="73000"/>
                            </a:schemeClr>
                        </a:gs>
                        <a:gs pos="100000">
                            <a:schemeClr val="phClr">
                                <a:lumMod val="105000"/>
                                <a:satMod val="109000"/>
                                <a:tint val="81000"/>
                            </a:schemeClr>
                        </a:gs>
                    </a:gsLst>
                    <a:lin ang="5400000" scaled="0"/>
                </a:gradFill>
                <a:gradFill rotWithShape="1">
                    <a:gsLst>
                        <a:gs pos="0">
                            <a:schemeClr val="phClr">
                                <a:satMod val="103000"/>
                                <a:lumMod val="102000"/>
                                <a:tint val="94000"/>
                            </a:schemeClr>
                        </a:gs>
                        <a:gs pos="50000">
                            <a:schemeClr val="phClr">
                                <a:satMod val="110000"/>
                                <a:lumMod val="100000"/>
                                <a:shade val="100000"/>
                            </a:schemeClr>
                        </a:gs>
                        <a:gs pos="100000">
                            <a:schemeClr val="phClr">
                                <a:lumMod val="99000"/>
                                <a:satMod val="120000"/>
                                <a:shade val="78000"/>
                            </a:schemeClr>
                        </a:gs>
                    </a:gsLst>
                    <a:lin ang="5400000" scaled="0"/>
                </a:gradFill>
            </a:fillStyleLst>
            <a:lnStyleLst>
                <a:ln w="6350" cap="flat" cmpd="sng" algn="ctr">
                    <a:solidFill>
                        <a:schemeClr val="phClr"/>
                    </a:solidFill>
                    <a:prstDash val="solid"/>
                    <a:miter lim="800000"/>
                </a:ln>
                <a:ln w="12700" cap="flat" cmpd="sng" algn="ctr">
                    <a:solidFill>
                        <a:schemeClr val="phClr"/>
                    </a:solidFill>
                    <a:prstDash val="solid"/>
                    <a:miter lim="800000"/>
                </a:ln>
                <a:ln w="19050" cap="flat" cmpd="sng" algn="ctr">
                    <a:solidFill>
                        <a:schemeClr val="phClr"/>
                    </a:solidFill>
                    <a:prstDash val="solid"/>
                    <a:miter lim="800000"/>
                </a:ln>
            </a:lnStyleLst>
            <a:effectStyleLst>
                <a:effectStyle>
                    <a:effectLst/>
                </a:effectStyle>
                <a:effectStyle>
                    <a:effectLst/>
                </a:effectStyle>
                <a:effectStyle>
                    <a:effectLst>
                        <a:outerShdw blurRad="57150" dist="19050" dir="5400000" algn="ctr" rotWithShape="0">
                            <a:srgbClr val="000000">
                                <a:alpha val="63000"/>
                            </a:srgbClr>
                        </a:outerShdw>
                    </a:effectLst>
                </a:effectStyle>
            </a:effectStyleLst>
            <a:bgFillStyleLst>
                <a:solidFill>
                    <a:schemeClr val="phClr"/>
                </a:solidFill>
                <a:solidFill>
                    <a:schemeClr val="phClr">
                        <a:tint val="95000"/>
                        <a:satMod val="170000"/>
                    </a:schemeClr>
                </a:solidFill>
                <a:gradFill rotWithShape="1">
                    <a:gsLst>
                        <a:gs pos="0">
                            <a:schemeClr val="phClr">
                                <a:tint val="93000"/>
                                <a:satMod val="150000"/>
                                <a:shade val="98000"/>
                                <a:lumMod val="102000"/>
                            </a:schemeClr>
                        </a:gs>
                        <a:gs pos="50000">
                            <a:schemeClr val="phClr">
                                <a:tint val="98000"/>
                                <a:satMod val="130000"/>
                                <a:shade val="90000"/>
                                <a:lumMod val="103000"/>
                            </a:schemeClr>
                        </a:gs>
                        <a:gs pos="100000">
                            <a:schemeClr val="phClr">
                                <a:shade val="63000"/>
                                <a:satMod val="120000"/>
                            </a:schemeClr>
                        </a:gs>
                    </a:gsLst>
                    <a:lin ang="5400000" scaled="0"/>
                </a:gradFill>
            </a:bgFillStyleLst>
        </a:fmtScheme>
    </a:themeElements>
    <a:objectDefaults/>
    <a:extraClrSchemeLst/>
    <a:extLst>
        <a:ext uri="{05A4C25C-085E-4340-85A3-A5531E510DB2}">
            <thm15:themeFamily xmlns:thm15="http://schemas.microsoft.com/office/thememl/2012/main" name="Office Theme" id="{62F939B6-93AF-4DB8-9C6B-D6C7DFDC589F}" vid="{4A3C46E8-61CC-4603-A589-7422A47A8E4A}"/>
        </a:ext>
    </a:extLst>
</a:theme>"#;

        zip_writer.start_file("ppt/theme/theme1.xml", SimpleFileOptions::default())?;
        zip_writer.write_all(content.as_bytes())?;
        Ok(())
    }

    fn escape_xml(&self, text: &str) -> String {
        text.replace("&", "&amp;")
            .replace("<", "&lt;")
            .replace(">", "&gt;")
            .replace("'", "&apos;")
            .replace("\"", "&quot;")
    }
}
