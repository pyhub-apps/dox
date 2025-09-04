//! Excel pivot table support (read-only)
//!
//! This module provides functionality to:
//! - Read existing pivot table structures from Excel files
//! - Extract pivot table metadata and configuration
//! - Preserve pivot tables during file modifications
//! - Document pivot table limitations with rust_xlsxwriter
//!
//! Note: Creating new pivot tables is not supported due to limitations
//! of the rust_xlsxwriter library. This module focuses on reading and
//! preserving existing pivot tables.

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info, warn};

use dox_core::RangeRef;

/// Represents a pivot table found in an Excel workbook
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PivotTable {
    /// Pivot table name
    pub name: String,
    /// Source data range
    pub source_range: RangeRef,
    /// Location of the pivot table
    pub location: PivotLocation,
    /// Row fields configuration
    pub row_fields: Vec<PivotField>,
    /// Column fields configuration
    pub column_fields: Vec<PivotField>,
    /// Data fields (values) configuration
    pub data_fields: Vec<DataField>,
    /// Page fields (filters) configuration
    pub page_fields: Vec<PivotField>,
    /// Pivot table style and formatting
    pub style: Option<PivotStyle>,
}

/// Location information for a pivot table
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PivotLocation {
    /// Sheet name where pivot table is located
    pub sheet_name: String,
    /// Starting cell reference (top-left corner)
    pub start_cell: String,
    /// Ending cell reference (bottom-right corner, if known)
    pub end_cell: Option<String>,
}

/// Pivot field configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PivotField {
    /// Field name (column header from source data)
    pub name: String,
    /// Field position in the area (0-based)
    pub position: usize,
    /// Custom field label (if different from name)
    pub custom_label: Option<String>,
    /// Subtotals configuration
    pub subtotals: SubtotalConfig,
    /// Sort order
    pub sort_order: SortOrder,
    /// Items to show/hide
    pub item_filter: Option<ItemFilter>,
}

/// Data field (value) configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataField {
    /// Source field name
    pub source_field: String,
    /// Display name in pivot table
    pub display_name: String,
    /// Aggregation function
    pub function: AggregationFunction,
    /// Number format
    pub number_format: Option<String>,
    /// Show values as (percentage, difference, etc.)
    pub show_values_as: ShowValuesAs,
}

/// Subtotal configuration for pivot fields
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubtotalConfig {
    /// Enable automatic subtotals
    pub enabled: bool,
    /// Subtotal functions to apply
    pub functions: Vec<AggregationFunction>,
    /// Show subtotals at top or bottom
    pub at_top: bool,
}

/// Sort order for pivot fields
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SortOrder {
    Manual,
    Ascending,
    Descending,
    Custom(Vec<String>), // Custom sort order
}

/// Item filter configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemFilter {
    /// Filter type
    pub filter_type: FilterType,
    /// Items to include (for manual filter)
    pub included_items: Option<Vec<String>>,
    /// Filter criteria (for value filters)
    pub criteria: Option<FilterCriteria>,
}

/// Filter types for pivot fields
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FilterType {
    /// No filter applied
    None,
    /// Manual selection of items
    Manual,
    /// Value-based filter (top 10, above average, etc.)
    Value,
    /// Date-based filter
    Date,
    /// Label-based filter (text filters)
    Label,
}

/// Filter criteria for value/label filters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterCriteria {
    /// Comparison operator
    pub operator: ComparisonOperator,
    /// Filter value(s)
    pub values: Vec<String>,
    /// Case sensitive for text filters
    pub case_sensitive: bool,
}

/// Comparison operators for filters
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ComparisonOperator {
    Equals,
    NotEquals,
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
    Between,
    NotBetween,
    Contains,
    NotContains,
    StartsWith,
    EndsWith,
}

/// Aggregation functions for data fields
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AggregationFunction {
    Sum,
    Count,
    Average,
    Max,
    Min,
    Product,
    CountNumbers,
    StdDev,
    StdDevP,
    Var,
    VarP,
}

/// Show values as options for data fields
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ShowValuesAs {
    Normal,
    PercentageOfGrandTotal,
    PercentageOfColumnTotal,
    PercentageOfRowTotal,
    PercentageOf,
    PercentageDifferenceFrom,
    DifferenceFrom,
    RunningTotalIn,
    PercentageOfParentRowTotal,
    PercentageOfParentColumnTotal,
    PercentageOfParentTotal,
    Index,
}

/// Pivot table style and formatting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PivotStyle {
    /// Built-in pivot table style name
    pub style_name: Option<String>,
    /// Show row headers
    pub show_row_headers: bool,
    /// Show column headers
    pub show_column_headers: bool,
    /// Show row stripes (banded rows)
    pub row_stripes: bool,
    /// Show column stripes (banded columns)
    pub column_stripes: bool,
    /// Show grand totals for rows
    pub grand_total_rows: bool,
    /// Show grand totals for columns
    pub grand_total_columns: bool,
}

/// Pivot table manager for reading existing pivot tables
pub struct PivotTableManager {
    pivot_tables: Vec<PivotTable>,
}

/// Metadata about pivot tables in a workbook
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PivotTableMetadata {
    /// Total number of pivot tables
    pub count: usize,
    /// Pivot tables by sheet
    pub by_sheet: HashMap<String, Vec<String>>, // sheet_name -> pivot_table_names
    /// Source ranges used by pivot tables
    pub source_ranges: Vec<RangeRef>,
    /// Total size estimate (in cells)
    pub estimated_size: usize,
}

impl Default for SubtotalConfig {
    fn default() -> Self {
        SubtotalConfig {
            enabled: true,
            functions: vec![AggregationFunction::Sum],
            at_top: false,
        }
    }
}

impl Default for PivotStyle {
    fn default() -> Self {
        PivotStyle {
            style_name: None,
            show_row_headers: true,
            show_column_headers: true,
            row_stripes: false,
            column_stripes: false,
            grand_total_rows: true,
            grand_total_columns: true,
        }
    }
}

impl PivotTableManager {
    /// Create a new pivot table manager
    pub fn new() -> Self {
        PivotTableManager {
            pivot_tables: Vec::new(),
        }
    }

    /// Discover pivot tables in an Excel workbook (read-only)
    ///
    /// Note: This is a placeholder implementation. The actual discovery would require
    /// parsing Excel's internal pivot table XML structures, which is complex and
    /// not fully supported by calamine.
    pub fn discover_pivot_tables(&mut self, _file_path: &std::path::Path) -> Result<usize> {
        warn!("Pivot table discovery is not fully implemented");
        warn!("This is a limitation of the current Excel reading libraries");

        // In a full implementation, this would:
        // 1. Open the Excel file as a ZIP archive
        // 2. Parse the pivot table XML files (xl/pivotTables/pivotTable*.xml)
        // 3. Parse the pivot cache XML files (xl/pivotCache/pivotCacheDefinition*.xml)
        // 4. Extract field configurations and relationships
        // 5. Build PivotTable structs from the parsed data

        debug!("Attempting to discover pivot tables (placeholder implementation)");

        // For now, return 0 discovered tables
        Ok(0)
    }

    /// Get all discovered pivot tables
    pub fn get_pivot_tables(&self) -> &[PivotTable] {
        &self.pivot_tables
    }

    /// Get pivot tables on a specific sheet
    pub fn get_pivot_tables_on_sheet(&self, sheet_name: &str) -> Vec<&PivotTable> {
        self.pivot_tables
            .iter()
            .filter(|pt| pt.location.sheet_name == sheet_name)
            .collect()
    }

    /// Get pivot table by name
    pub fn get_pivot_table_by_name(&self, name: &str) -> Option<&PivotTable> {
        self.pivot_tables.iter().find(|pt| pt.name == name)
    }

    /// Generate metadata about discovered pivot tables
    pub fn generate_metadata(&self) -> PivotTableMetadata {
        let mut by_sheet: HashMap<String, Vec<String>> = HashMap::new();
        let mut source_ranges = Vec::new();
        let mut estimated_size = 0;

        for pivot_table in &self.pivot_tables {
            let sheet_pivots = by_sheet
                .entry(pivot_table.location.sheet_name.clone())
                .or_insert_with(Vec::new);
            sheet_pivots.push(pivot_table.name.clone());

            source_ranges.push(pivot_table.source_range.clone());

            // Rough estimate of pivot table size
            estimated_size += pivot_table.row_fields.len() * 100; // Rough estimate
        }

        PivotTableMetadata {
            count: self.pivot_tables.len(),
            by_sheet,
            source_ranges,
            estimated_size,
        }
    }

    /// Check if a range is used by any pivot table as source data
    pub fn is_pivot_source_range(&self, range: &RangeRef) -> bool {
        self.pivot_tables
            .iter()
            .any(|pt| pt.source_range.0 == range.0)
    }

    /// Get warnings about pivot table limitations
    pub fn get_pivot_table_warnings(&self) -> Vec<String> {
        let mut warnings = Vec::new();

        warnings.push("⚠️  Pivot Table Support Limitations:".to_string());
        warnings
            .push("   • Cannot create new pivot tables (rust_xlsxwriter limitation)".to_string());
        warnings.push("   • Cannot modify existing pivot tables".to_string());
        warnings.push("   • Pivot table discovery is limited (calamine limitation)".to_string());
        warnings.push("   • Can preserve pivot tables when copying workbook structure".to_string());

        if !self.pivot_tables.is_empty() {
            warnings.push(format!(
                "   • Found {} pivot tables that will be preserved during operations",
                self.pivot_tables.len()
            ));
        }

        warnings
    }

    /// Add a manually created pivot table definition
    /// (Useful for testing or when pivot table structure is known)
    pub fn add_pivot_table(&mut self, pivot_table: PivotTable) {
        debug!("Adding pivot table: {}", pivot_table.name);
        self.pivot_tables.push(pivot_table);
    }

    /// Remove pivot table by name
    pub fn remove_pivot_table(&mut self, name: &str) -> bool {
        if let Some(index) = self.pivot_tables.iter().position(|pt| pt.name == name) {
            self.pivot_tables.remove(index);
            debug!("Removed pivot table: {}", name);
            true
        } else {
            false
        }
    }

    /// Export pivot table definitions to JSON for backup/documentation
    pub fn export_pivot_definitions(&self) -> Result<String> {
        serde_json::to_string_pretty(&self.pivot_tables)
            .map_err(|e| anyhow!("Failed to export pivot table definitions: {}", e))
    }

    /// Import pivot table definitions from JSON
    pub fn import_pivot_definitions(&mut self, json: &str) -> Result<usize> {
        let imported_tables: Vec<PivotTable> = serde_json::from_str(json)
            .map_err(|e| anyhow!("Failed to import pivot table definitions: {}", e))?;

        let count = imported_tables.len();
        self.pivot_tables.extend(imported_tables);

        info!("Imported {} pivot table definitions", count);
        Ok(count)
    }
}

/// Helper functions for working with pivot tables
pub mod helpers {
    use super::*;

    /// Create a simple pivot table definition for testing
    pub fn create_sample_pivot_table() -> PivotTable {
        PivotTable {
            name: "Sample Pivot".to_string(),
            source_range: RangeRef::new("Data!A1:D100"),
            location: PivotLocation {
                sheet_name: "Pivot".to_string(),
                start_cell: "A1".to_string(),
                end_cell: Some("E20".to_string()),
            },
            row_fields: vec![PivotField {
                name: "Category".to_string(),
                position: 0,
                custom_label: None,
                subtotals: SubtotalConfig::default(),
                sort_order: SortOrder::Ascending,
                item_filter: None,
            }],
            column_fields: vec![PivotField {
                name: "Region".to_string(),
                position: 0,
                custom_label: None,
                subtotals: SubtotalConfig::default(),
                sort_order: SortOrder::Ascending,
                item_filter: None,
            }],
            data_fields: vec![DataField {
                source_field: "Sales".to_string(),
                display_name: "Sum of Sales".to_string(),
                function: AggregationFunction::Sum,
                number_format: Some("#,##0.00".to_string()),
                show_values_as: ShowValuesAs::Normal,
            }],
            page_fields: vec![],
            style: Some(PivotStyle::default()),
        }
    }

    /// Create a pivot table definition from basic parameters
    pub fn create_pivot_table(
        name: &str,
        source_range: &str,
        location_sheet: &str,
        location_cell: &str,
        row_field: &str,
        data_field: &str,
        function: AggregationFunction,
    ) -> PivotTable {
        PivotTable {
            name: name.to_string(),
            source_range: RangeRef::new(source_range),
            location: PivotLocation {
                sheet_name: location_sheet.to_string(),
                start_cell: location_cell.to_string(),
                end_cell: None,
            },
            row_fields: vec![PivotField {
                name: row_field.to_string(),
                position: 0,
                custom_label: None,
                subtotals: SubtotalConfig::default(),
                sort_order: SortOrder::Ascending,
                item_filter: None,
            }],
            column_fields: vec![],
            data_fields: vec![DataField {
                source_field: data_field.to_string(),
                display_name: format!(
                    "{} of {}",
                    match function {
                        AggregationFunction::Sum => "Sum",
                        AggregationFunction::Count => "Count",
                        AggregationFunction::Average => "Average",
                        AggregationFunction::Max => "Max",
                        AggregationFunction::Min => "Min",
                        _ => "Value",
                    },
                    data_field
                ),
                function,
                number_format: None,
                show_values_as: ShowValuesAs::Normal,
            }],
            page_fields: vec![],
            style: Some(PivotStyle::default()),
        }
    }

    /// Analyze a range to suggest pivot table structure
    pub fn analyze_for_pivot_table(
        _source_range: &RangeRef,
        _headers: &[String],
    ) -> Result<Vec<String>> {
        // This would analyze the source data and suggest:
        // - Which fields work well as row/column fields
        // - Which fields are numeric and suitable for data fields
        // - Potential groupings and relationships

        warn!("Pivot table analysis not implemented - would require data inspection");
        Ok(vec![
            "Pivot table structure analysis requires data inspection".to_string(),
            "Consider using fields with categorical data for rows/columns".to_string(),
            "Use numeric fields for data/values areas".to_string(),
        ])
    }
}

/// Documentation about pivot table limitations and workarounds
pub mod documentation {
    /// Get comprehensive documentation about pivot table limitations
    pub fn get_pivot_table_limitations() -> Vec<String> {
        vec![
            "📊 Excel Pivot Table Support in dox-excel".to_string(),
            "".to_string(),
            "🚫 LIMITATIONS:".to_string(),
            "• Cannot create new pivot tables (rust_xlsxwriter doesn't support pivot tables)"
                .to_string(),
            "• Cannot modify existing pivot table structure".to_string(),
            "• Limited ability to read pivot table definitions (calamine limitation)".to_string(),
            "• Cannot refresh pivot table data programmatically".to_string(),
            "".to_string(),
            "✅ WHAT IS SUPPORTED:".to_string(),
            "• Reading pivot table metadata (limited)".to_string(),
            "• Preserving pivot tables when copying workbook structure".to_string(),
            "• Documenting existing pivot table configurations".to_string(),
            "• Identifying source ranges used by pivot tables".to_string(),
            "".to_string(),
            "💡 WORKAROUNDS:".to_string(),
            "• Use external tools (like Python xlsxwriter) for pivot table creation".to_string(),
            "• Create pivot tables manually in Excel after data generation".to_string(),
            "• Generate summary tables that mimic pivot table functionality".to_string(),
            "• Use formulas (SUMIF, COUNTIF, etc.) to create pivot-like summaries".to_string(),
            "".to_string(),
            "🔮 FUTURE IMPROVEMENTS:".to_string(),
            "• Better pivot table reading when calamine adds support".to_string(),
            "• Pivot table creation if rust_xlsxwriter adds support".to_string(),
            "• Integration with external pivot table libraries".to_string(),
        ]
    }

    /// Get workaround strategies for common pivot table use cases
    pub fn get_pivot_table_workarounds() -> Vec<String> {
        vec![
            "🛠️  Pivot Table Workarounds".to_string(),
            "".to_string(),
            "1. SUMMARY TABLES WITH FORMULAS:".to_string(),
            "   • Use SUMIF, COUNTIF, AVERAGEIF formulas".to_string(),
            "   • Create manual group-by summaries".to_string(),
            "   • Generate cross-tabulation tables".to_string(),
            "".to_string(),
            "2. EXTERNAL TOOL INTEGRATION:".to_string(),
            "   • Generate data with dox-excel, create pivots with Python/pandas".to_string(),
            "   • Use Excel COM automation (Windows only)".to_string(),
            "   • Call external pivot table creation tools".to_string(),
            "".to_string(),
            "3. CHART-BASED VISUALIZATION:".to_string(),
            "   • Use the chart module to create visual summaries".to_string(),
            "   • Generate multiple charts for different data views".to_string(),
            "   • Create dashboard-style reports".to_string(),
            "".to_string(),
            "4. TEMPLATE-BASED APPROACH:".to_string(),
            "   • Create Excel templates with pre-built pivot tables".to_string(),
            "   • Populate source data, let Excel refresh pivots".to_string(),
            "   • Use dynamic named ranges for automatic updates".to_string(),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pivot_table_creation() {
        let pivot = helpers::create_sample_pivot_table();

        assert_eq!(pivot.name, "Sample Pivot");
        assert_eq!(pivot.source_range.0, "Data!A1:D100");
        assert_eq!(pivot.location.sheet_name, "Pivot");
        assert_eq!(pivot.row_fields.len(), 1);
        assert_eq!(pivot.data_fields.len(), 1);
    }

    #[test]
    fn test_pivot_table_manager() {
        let mut manager = PivotTableManager::new();

        let pivot = helpers::create_sample_pivot_table();
        manager.add_pivot_table(pivot);

        assert_eq!(manager.get_pivot_tables().len(), 1);
        assert!(manager.get_pivot_table_by_name("Sample Pivot").is_some());

        let metadata = manager.generate_metadata();
        assert_eq!(metadata.count, 1);
    }

    #[test]
    fn test_pivot_field_configuration() {
        let field = PivotField {
            name: "Category".to_string(),
            position: 0,
            custom_label: Some("Product Category".to_string()),
            subtotals: SubtotalConfig {
                enabled: true,
                functions: vec![AggregationFunction::Sum, AggregationFunction::Count],
                at_top: false,
            },
            sort_order: SortOrder::Ascending,
            item_filter: None,
        };

        assert_eq!(field.name, "Category");
        assert_eq!(field.custom_label, Some("Product Category".to_string()));
        assert_eq!(field.sort_order, SortOrder::Ascending);
        assert!(field.subtotals.enabled);
    }

    #[test]
    fn test_data_field_configuration() {
        let data_field = DataField {
            source_field: "Sales".to_string(),
            display_name: "Total Sales".to_string(),
            function: AggregationFunction::Sum,
            number_format: Some("$#,##0.00".to_string()),
            show_values_as: ShowValuesAs::PercentageOfGrandTotal,
        };

        assert_eq!(data_field.function, AggregationFunction::Sum);
        assert_eq!(
            data_field.show_values_as,
            ShowValuesAs::PercentageOfGrandTotal
        );
        assert!(data_field.number_format.is_some());
    }

    #[test]
    fn test_json_serialization() {
        let pivot = helpers::create_sample_pivot_table();
        let json = serde_json::to_string(&pivot).unwrap();

        let deserialized: PivotTable = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.name, pivot.name);
        assert_eq!(deserialized.source_range.0, pivot.source_range.0);
    }

    #[test]
    fn test_pivot_table_export_import() {
        let mut manager = PivotTableManager::new();
        manager.add_pivot_table(helpers::create_sample_pivot_table());

        let exported = manager.export_pivot_definitions().unwrap();

        let mut new_manager = PivotTableManager::new();
        let count = new_manager.import_pivot_definitions(&exported).unwrap();

        assert_eq!(count, 1);
        assert_eq!(new_manager.get_pivot_tables().len(), 1);
    }
}
