//! Comprehensive tests for PDF functionality

#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::provider::DocumentProvider;
    use std::io::Write;
    use tempfile::NamedTempFile;

    /// Create a dummy PDF file for testing
    fn create_test_pdf() -> NamedTempFile {
        let mut temp_file = NamedTempFile::with_suffix(".pdf").unwrap();
        writeln!(temp_file, "%PDF-1.4").unwrap();
        writeln!(temp_file, "1 0 obj").unwrap();
        writeln!(temp_file, "<<").unwrap();
        writeln!(temp_file, "/Type /Catalog").unwrap();
        writeln!(temp_file, "/Pages 2 0 R").unwrap();
        writeln!(temp_file, ">>").unwrap();
        writeln!(temp_file, "endobj").unwrap();
        writeln!(temp_file, "xref").unwrap();
        writeln!(temp_file, "0 2").unwrap();
        writeln!(temp_file, "0000000000 65535 f ").unwrap();
        writeln!(temp_file, "0000000009 00000 n ").unwrap();
        writeln!(temp_file, "trailer").unwrap();
        writeln!(temp_file, "<<").unwrap();
        writeln!(temp_file, "/Size 2").unwrap();
        writeln!(temp_file, "/Root 1 0 R").unwrap();
        writeln!(temp_file, ">>").unwrap();
        writeln!(temp_file, "startxref").unwrap();
        writeln!(temp_file, "74").unwrap();
        writeln!(temp_file, "%%EOF").unwrap();
        temp_file
    }

    #[test]
    fn test_pdf_provider_creation() {
        let temp_file = create_test_pdf();
        let temp_path = temp_file.path();

        let provider = PdfProvider::open(temp_path);
        assert!(provider.is_ok());

        let provider = provider.unwrap();
        assert_eq!(provider.get_path(), temp_path);
        assert_eq!(provider.document_type(), crate::provider::DocumentType::Pdf);
        assert!(!provider.is_modified());
    }

    #[test]
    fn test_pdf_provider_configurations() {
        let temp_file = create_test_pdf();
        let temp_path = temp_file.path();

        // Test different configurations
        let small_provider = PdfProvider::open_small_file(temp_path);
        assert!(small_provider.is_ok());

        let large_provider = PdfProvider::open_large_file(temp_path);
        assert!(large_provider.is_ok());

        let layout_provider = PdfProvider::open_layout_critical(temp_path);
        assert!(layout_provider.is_ok());
    }

    #[test]
    fn test_pdf_extract_config() {
        let default_config = PdfExtractConfig::default();
        assert!(default_config.preserve_layout);
        assert!(default_config.extract_tables);

        let small_config = PdfExtractConfig::small_file();
        assert!(small_config.preserve_layout);
        assert!(!small_config.enable_streaming);

        let large_config = PdfExtractConfig::large_file();
        assert!(!large_config.preserve_layout);
        assert!(large_config.enable_streaming);

        let layout_config = PdfExtractConfig::layout_critical();
        assert!(layout_config.preserve_layout);
        assert!(layout_config.extract_tables);
    }

    #[test]
    fn test_ocr_config() {
        let default_config = OcrConfig::default();
        assert_eq!(default_config.primary_language, "eng");
        assert_eq!(default_config.confidence_threshold, 0.6);

        let english_config = OcrConfig::english();
        assert_eq!(english_config.primary_language, "eng");
        assert_eq!(english_config.confidence_threshold, 0.7);

        let korean_config = OcrConfig::korean();
        assert_eq!(korean_config.primary_language, "kor");

        let multilingual_config = OcrConfig::multilingual();
        assert_eq!(multilingual_config.primary_language, "eng+kor");
        assert!(multilingual_config.auto_detect_language);
    }

    #[test]
    fn test_ocr_processor() {
        let config = OcrConfig::default();
        let processor = PdfOcrProcessor::new(config);
        
        // Test basic creation
        assert!(format!("{:?}", processor).contains("PdfOcrProcessor"));
    }

    #[test]
    fn test_extraction_strategy() {
        let normal = ExtractionStrategy::Normal;
        assert!(normal.allows_text_extraction());
        assert!(normal.allows_table_extraction());
        assert!(normal.allows_metadata_extraction());
        assert!(normal.get_warning().is_none());

        let restricted = ExtractionStrategy::Restricted;
        assert!(!restricted.allows_text_extraction());
        assert!(!restricted.allows_table_extraction());
        assert!(restricted.allows_metadata_extraction());
        assert!(restricted.get_warning().is_some());

        let accessibility = ExtractionStrategy::AccessibilityOnly;
        assert!(accessibility.allows_text_extraction());
        assert!(!accessibility.allows_table_extraction());
        assert!(accessibility.allows_metadata_extraction());
    }

    #[test]
    fn test_encryption_permissions() {
        let default_perms = EncryptionPermissions::default();
        assert!(!default_perms.print);
        assert!(!default_perms.modify);
        assert!(!default_perms.copy);
    }

    #[test]
    fn test_text_block_types() {
        let paragraph = TextBlockType::Paragraph;
        let heading = TextBlockType::Heading(1);
        let list_item = TextBlockType::ListItem;

        // Test that they can be created and matched
        match paragraph {
            TextBlockType::Paragraph => assert!(true),
            _ => assert!(false),
        }

        match heading {
            TextBlockType::Heading(level) => assert_eq!(level, 1),
            _ => assert!(false),
        }

        match list_item {
            TextBlockType::ListItem => assert!(true),
            _ => assert!(false),
        }
    }

    #[test]
    fn test_page_dimensions() {
        let dimensions = PageDimensions {
            width: 612.0,
            height: 792.0,
            rotation: 0,
        };

        assert_eq!(dimensions.width, 612.0);
        assert_eq!(dimensions.height, 792.0);
        assert_eq!(dimensions.rotation, 0);
    }

    #[test]
    fn test_font_info() {
        let font = FontInfo {
            family: Some("Arial".to_string()),
            size: Some(12.0),
            bold: true,
            italic: false,
        };

        assert_eq!(font.family, Some("Arial".to_string()));
        assert_eq!(font.size, Some(12.0));
        assert!(font.bold);
        assert!(!font.italic);
    }

    #[test]
    fn test_block_position() {
        let position = BlockPosition {
            x: 72.0,
            y: 720.0,
            width: 468.0,
            height: 14.0,
        };

        assert_eq!(position.x, 72.0);
        assert_eq!(position.y, 720.0);
        assert_eq!(position.width, 468.0);
        assert_eq!(position.height, 14.0);
    }

    #[test] 
    fn test_pdf_table() {
        let table_data = vec![
            vec!["Header 1".to_string(), "Header 2".to_string()],
            vec!["Row 1 Col 1".to_string(), "Row 1 Col 2".to_string()],
            vec!["Row 2 Col 1".to_string(), "Row 2 Col 2".to_string()],
        ];

        let table = PdfTable {
            index: 0,
            data: table_data.clone(),
            rows: 3,
            cols: 2,
            position: BlockPosition {
                x: 0.0,
                y: 0.0,
                width: 100.0,
                height: 50.0,
            },
            confidence: 0.8,
        };

        assert_eq!(table.index, 0);
        assert_eq!(table.data, table_data);
        assert_eq!(table.rows, 3);
        assert_eq!(table.cols, 2);
        assert_eq!(table.confidence, 0.8);
    }

    #[test]
    fn test_extraction_stats() {
        let stats = ExtractionStats {
            total_pages: 5,
            text_blocks: 20,
            tables_detected: 3,
            images_detected: 2,
            extraction_time_ms: 1500,
            memory_usage_mb: 25.5,
            streaming_used: false,
        };

        assert_eq!(stats.total_pages, 5);
        assert_eq!(stats.text_blocks, 20);
        assert_eq!(stats.tables_detected, 3);
        assert_eq!(stats.images_detected, 2);
        assert_eq!(stats.extraction_time_ms, 1500);
        assert_eq!(stats.memory_usage_mb, 25.5);
        assert!(!stats.streaming_used);
    }

    #[test]
    fn test_advanced_pdf_result_serialization() {
        let result = AdvancedPdfResult {
            pages: vec![],
            metadata: PdfDocumentMetadata {
                title: Some("Test PDF".to_string()),
                author: Some("Test Author".to_string()),
                subject: None,
                creator: None,
                producer: None,
                creation_date: None,
                modification_date: None,
                page_count: 1,
                file_size: 1024,
                pdf_version: "1.4".to_string(),
                encrypted: false,
                permissions: PdfPermissions {
                    print: true,
                    modify: true,
                    copy: true,
                    annotate: true,
                },
            },
            stats: ExtractionStats {
                total_pages: 1,
                text_blocks: 0,
                tables_detected: 0,
                images_detected: 0,
                extraction_time_ms: 100,
                memory_usage_mb: 1.0,
                streaming_used: false,
            },
            warnings: vec!["Test warning".to_string()],
        };

        // Test serialization
        let json = serde_json::to_string(&result);
        assert!(json.is_ok());

        // Test deserialization
        let json_str = json.unwrap();
        let deserialized: Result<AdvancedPdfResult, _> = serde_json::from_str(&json_str);
        assert!(deserialized.is_ok());
    }

    #[test]
    fn test_ocr_analysis() {
        let analysis = OcrAnalysis {
            total_pages: 10,
            image_based_pages: vec![1, 3, 5],
            mixed_pages: vec![2, 4],
            text_based_pages: vec![6, 7, 8, 9, 10],
            recommended_ocr: true,
            estimated_processing_time_minutes: 15,
        };

        assert_eq!(analysis.total_pages, 10);
        assert_eq!(analysis.image_based_pages.len(), 3);
        assert_eq!(analysis.mixed_pages.len(), 2);
        assert_eq!(analysis.text_based_pages.len(), 5);
        assert!(analysis.recommended_ocr);
        assert_eq!(analysis.estimated_processing_time_minutes, 15);
    }
}