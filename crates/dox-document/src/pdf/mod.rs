//! Advanced PDF processing module

pub mod provider;
pub mod extractor;
pub mod encrypted;
pub mod ocr;

#[cfg(test)]
pub mod tests;

pub use extractor::{
    AdvancedPdfExtractor, AdvancedPdfResult, PdfExtractConfig, PdfPage, TextBlock, TextBlockType,
    FontInfo, BlockPosition, PdfTable, ImageInfo, PageDimensions, PdfDocumentMetadata,
    PdfPermissions, ExtractionStats,
};

pub use encrypted::{
    EncryptedPdfHandler, EncryptionInfo, EncryptionPermissions, PasswordResult, ExtractionStrategy,
};

pub use ocr::{
    PdfOcrProcessor, OcrConfig, OcrResult, OcrAnalysis, ProcessingEstimate, OcrError,
    OcrEngine, OcrEngineInfo, BoundingBox, OcrWord,
};

pub use provider::{PdfProvider, PdfMetadata};

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