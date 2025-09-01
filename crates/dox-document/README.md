# dox-document

Document processing crate for Microsoft Office formats (Word and PowerPoint).

## Overview

The `dox-document` crate provides a unified interface for working with Microsoft Office document formats:

- **Word documents** (`.docx`)
- **PowerPoint presentations** (`.pptx`)

## Features

- ✅ Text extraction from documents
- ✅ Text replacement with format preservation
- ✅ Document structure preservation
- ✅ Batch processing support
- ✅ Error handling and validation
- ✅ Save and save-as functionality

## Usage

### Basic Example

```rust
use dox_document::{create_provider, DocumentProvider};
use std::path::Path;

// Open a document (Word or PowerPoint)
let mut doc = create_provider(Path::new("document.docx")).unwrap();

// Replace text
let count = doc.replace_text("{{NAME}}", "John Doe").unwrap();
println!("Replaced {} occurrences", count);

// Save changes
doc.save().unwrap();
```

### Working with Specific Document Types

```rust
use dox_document::{WordProvider, PowerPointProvider};

// Work with Word documents
let mut word_doc = WordProvider::open(Path::new("document.docx")).unwrap();
word_doc.replace_text("old", "new").unwrap();
word_doc.save().unwrap();

// Work with PowerPoint presentations
let mut ppt_doc = PowerPointProvider::open(Path::new("presentation.pptx")).unwrap();
println!("Slides: {}", ppt_doc.slide_count());

// Get text from specific slide
let slide_text = ppt_doc.get_slide_text(0).unwrap();
println!("First slide: {}", slide_text);

// Replace text in specific slide
ppt_doc.replace_text_in_slide(0, "{{TITLE}}", "My Presentation").unwrap();
ppt_doc.save().unwrap();
```

## Document Provider Trait

All document types implement the `DocumentProvider` trait:

```rust
pub trait DocumentProvider: std::fmt::Debug {
    fn replace_text(&mut self, old: &str, new: &str) -> Result<usize, DocumentError>;
    fn save(&self) -> Result<(), DocumentError>;
    fn save_as(&self, path: &Path) -> Result<(), DocumentError>;
    fn get_text(&self) -> Result<String, DocumentError>;
    fn is_modified(&self) -> bool;
    fn get_path(&self) -> &Path;
    fn document_type(&self) -> DocumentType;
}
```

## Error Handling

The crate provides comprehensive error handling through the `DocumentError` enum:

```rust
use dox_document::DocumentError;

match create_provider(path) {
    Ok(doc) => { /* work with document */ },
    Err(DocumentError::UnsupportedFormat { extension }) => {
        println!("Unsupported format: {}", extension);
    },
    Err(DocumentError::DocumentNotFound { path }) => {
        println!("Document not found: {}", path);
    },
    Err(e) => {
        println!("Error: {}", e);
    },
}
```

## Architecture

The crate is organized into several modules:

- `provider`: Core trait and factory function
- `word`: Word document implementation
- `powerpoint`: PowerPoint document implementation
- `utils`: Shared utilities for ZIP and XML processing

## Implementation Details

### Office Document Format

Microsoft Office documents (`.docx`, `.pptx`) are ZIP archives containing XML files:

- **Word**: Main content in `word/document.xml`
- **PowerPoint**: Slide content in `ppt/slides/slide*.xml`

The crate preserves the original document structure while allowing text modifications.

### Text Processing

Text replacement is performed on XML content while preserving:

- Document formatting
- Embedded objects
- Metadata
- Relationships

## Testing

Run tests with:

```bash
cargo test
```

The test suite includes:

- Unit tests for each provider
- Integration tests with real document structures
- Error handling scenarios
- Format preservation validation

## Dependencies

- `zip`: ZIP archive handling
- `quick-xml`: XML parsing and writing
- `regex`: Pattern matching
- `anyhow`: Error handling
- `tracing`: Logging
- `thiserror`: Error types