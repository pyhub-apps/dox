//! Advanced PDF processing module

pub mod encrypted;
pub mod extractor;
pub mod ocr;
pub mod provider;

#[cfg(test)]
pub mod tests;

pub use extractor::{
    AdvancedPdfExtractor, AdvancedPdfResult, BlockPosition, ExtractionStats, FontInfo, ImageInfo,
    PageDimensions, PdfDocumentMetadata, PdfExtractConfig, PdfPage, PdfPermissions, PdfTable,
    TextBlock, TextBlockType,
};

pub use encrypted::{
    EncryptedPdfHandler, EncryptionInfo, EncryptionPermissions, ExtractionStrategy, PasswordResult,
};

pub use ocr::{
    BoundingBox, OcrAnalysis, OcrConfig, OcrEngine, OcrEngineInfo, OcrError, OcrResult, OcrWord,
    PdfOcrProcessor, ProcessingEstimate,
};

pub use provider::{PdfMetadata, PdfProvider};

use crate::provider::DocumentError;
use std::path::Path;

/// Convenience function for advanced PDF extraction
pub fn extract_pdf_advanced(
    path: &Path,
    config: Option<PdfExtractConfig>,
) -> Result<AdvancedPdfResult, DocumentError> {
    let config = config.unwrap_or_default();
    let mut extractor = AdvancedPdfExtractor::new(path, config)?;
    extractor.extract()
}

/// Extract PDF with layout preservation
pub fn extract_pdf_with_layout(path: &Path) -> Result<AdvancedPdfResult, DocumentError> {
    let config = PdfExtractConfig::layout_critical();
    extract_pdf_advanced(path, Some(config))
}

/// Extract PDF optimized for large files
pub fn extract_pdf_streaming(path: &Path) -> Result<AdvancedPdfResult, DocumentError> {
    let config = PdfExtractConfig::large_file();
    extract_pdf_advanced(path, Some(config))
}

/// Extract PDF optimized for small files
pub fn extract_pdf_full_features(path: &Path) -> Result<AdvancedPdfResult, DocumentError> {
    let config = PdfExtractConfig::small_file();
    extract_pdf_advanced(path, Some(config))
}
