//! Integration tests for Excel functionality
//!
//! This file demonstrates the Excel features implemented in dox-excel:
//! - Basic Excel provider functionality
//! - Chart creation and management
//! - Formatting with simplified API
//! - Data validation (documented for future implementation)
//! - Streaming support for large files
//! - Macro analysis and security handling
//! - Pivot table metadata (read-only)
//! - Formula parsing and evaluation

use super::super::*;
use dox_core::RangeRef;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_excel_provider_creation() {
        let provider = ExcelProvider::new();
        assert!(provider.base_dir.is_none());

        let provider = ExcelProvider::with_base_dir("/tmp");
        assert_eq!(provider.base_dir, Some(std::path::PathBuf::from("/tmp")));
    }

    #[test]
    fn test_path_resolution() {
        let provider = ExcelProvider::new();
        let sheet_id = dox_core::SheetId("/absolute/path.xlsx".to_string());
        assert_eq!(
            provider.resolve_path(&sheet_id),
            std::path::PathBuf::from("/absolute/path.xlsx")
        );

        let provider = ExcelProvider::with_base_dir("/base");
        let sheet_id = dox_core::SheetId("relative/path.xlsx".to_string());
        assert_eq!(
            provider.resolve_path(&sheet_id),
            std::path::PathBuf::from("/base/relative/path.xlsx")
        );
    }

    #[test]
    fn test_cell_conversion() {
        use calamine::Data;

        let float_cell = Data::Float(42.0);
        let converted = ExcelProvider::convert_calamine_cell(&float_cell, false);
        assert_eq!(converted.value, "42");

        let string_cell = Data::String("test".to_string());
        let converted = ExcelProvider::convert_calamine_cell(&string_cell, false);
        assert_eq!(converted.value, "test");

        let formula_cell = Data::String("=SUM(A1:A5)".to_string());
        let converted = ExcelProvider::convert_calamine_cell(&formula_cell, false);
        assert_eq!(converted.value, "=SUM(A1:A5)");
    }

    #[test]
    fn test_formula_evaluation_flag() {
        use calamine::Data;

        // Test that formulas are returned as-is when evaluation is disabled
        let formula_cell = Data::String("=2+2".to_string());
        let converted = ExcelProvider::convert_calamine_cell(&formula_cell, false);
        assert_eq!(converted.value, "=2+2");

        // Test that formulas would be evaluated when enabled (for now, still returns formula)
        let converted = ExcelProvider::convert_calamine_cell(&formula_cell, true);
        assert_eq!(converted.value, "=2+2"); // TODO: Should be "4" when evaluation is implemented
    }

    #[test]
    fn test_chart_types() {
        // Test chart type conversions
        let chart_types = vec![
            ExcelChartType::Column,
            ExcelChartType::Line,
            ExcelChartType::Pie,
            ExcelChartType::Scatter,
        ];

        for chart_type in chart_types {
            let _rust_type = chart_type.to_rust_xlsxwriter_type();
            // Just ensure conversion doesn't panic
        }
    }

    #[test]
    fn test_chart_series_creation() {
        let series = ChartSeries::new("Test Series", RangeRef("A1:A10".to_string()));
        assert_eq!(series.name, "Test Series");
        assert_eq!(series.data_range.0, "A1:A10");
        assert!(series.category_range.is_none());

        let series_with_categories = ChartSeries::new("Test", RangeRef("B1:B10".to_string()))
            .with_categories(RangeRef("A1:A10".to_string()));
        assert!(series_with_categories.category_range.is_some());
    }

    #[test]
    fn test_chart_position_default() {
        let position = ChartPosition::default();
        assert_eq!(position.col, 8);
        assert_eq!(position.row, 1);
        assert_eq!(position.width, Some(480));
        assert_eq!(position.height, Some(288));
    }

    #[test]
    fn test_basic_formatting_template() {
        let template = FormatTemplate::header();
        assert_eq!(template.name, "Header");
        assert_eq!(template.format.bold, Some(true));
        assert_eq!(template.format.font_size, Some(12.0));

        let data_template = FormatTemplate::data();
        assert_eq!(data_template.name, "Data");
        assert_eq!(data_template.format.bold, Some(false));
    }

    #[test]
    fn test_style_theme() {
        let theme = StyleTheme::professional();
        assert_eq!(theme.name, "Professional");
        assert_eq!(theme.templates.len(), 3);

        let header = theme.get_template("Header");
        assert!(header.is_some());
        assert_eq!(header.unwrap().format.bold, Some(true));
    }

    #[test]
    fn test_simple_validation_config() {
        let config = SimpleValidationConfig {
            range: RangeRef("A1:A10".to_string()),
            validation_type: SimpleValidationType::WholeNumber,
            input_message: Some("Enter a number".to_string()),
            allow_blank: false,
        };

        match config.validation_type {
            SimpleValidationType::WholeNumber => {
                // Expected
            }
            _ => panic!("Expected WholeNumber validation type"),
        }

        assert_eq!(config.range.0, "A1:A10");
        assert!(!config.allow_blank);
    }

    #[test]
    fn test_validation_template() {
        let template = ValidationTemplate::dropdown_template("Colors", vec![
            "Red".to_string(),
            "Green".to_string(),
            "Blue".to_string(),
        ]);

        assert_eq!(template.name, "Colors");
        assert_eq!(template.validations.len(), 1);

        match &template.validations[0].validation_type {
            SimpleValidationType::List(values) => {
                assert_eq!(values.len(), 3);
                assert!(values.contains(&"Red".to_string()));
            }
            _ => panic!("Expected List validation type"),
        }
    }

    #[test]
    fn test_streaming_config() {
        let config = StreamingConfig::default();
        assert_eq!(config.chunk_size, 1000);
        assert_eq!(config.max_memory_mb, 512);
        assert!(config.parallel_processing);
        
        let large_file_config = StreamingConfig::for_very_large_files();
        assert_eq!(large_file_config.chunk_size, 500);
        assert_eq!(large_file_config.max_memory_mb, 256);
    }

    #[test]
    fn test_macro_config() {
        let config = MacroConfig::default();
        assert_eq!(config.handling_option, MacroHandlingOption::WarnAndPreserve);
        assert!(config.detailed_analysis);

        let security_config = MacroConfig::security_focused();
        assert_eq!(security_config.handling_option, MacroHandlingOption::Block);

        let permissive_config = MacroConfig::permissive();
        assert_eq!(permissive_config.handling_option, MacroHandlingOption::Preserve);
    }

    #[test]
    fn test_pivot_table_metadata() {
        let metadata = PivotTableMetadata {
            count: 1,
            by_sheet: std::collections::HashMap::new(),
            source_ranges: vec![RangeRef("Data!A1:E100".to_string())],
            estimated_size: 1000,
        };

        assert_eq!(metadata.count, 1);
        assert_eq!(metadata.source_ranges.len(), 1);
        assert_eq!(metadata.estimated_size, 1000);
    }

    #[test]
    fn test_formula_parsing() {
        let formula = Formula::parse("=SUM(A1:A5)").expect("Should parse SUM formula");
        assert!(!formula.expression.is_empty());
        assert!(!formula.cell_refs.is_empty());

        let simple_formula = Formula::parse("=2+2").expect("Should parse simple formula");
        assert_eq!(simple_formula.expression, "2+2");
    }

    #[test] 
    fn test_cell_reference_creation() {
        let single_ref = CellReference::new_single(None, 0, 0);
        assert_eq!(single_ref.col, 0);
        assert_eq!(single_ref.row, 0);
        assert!(single_ref.end_col.is_none());

        let range_ref = CellReference::new_range(Some("Sheet1".to_string()), 0, 0, 4, 9);
        assert_eq!(range_ref.col, 0);
        assert_eq!(range_ref.row, 0);
        assert_eq!(range_ref.end_col, Some(4));
        assert_eq!(range_ref.end_row, Some(9));
    }
}