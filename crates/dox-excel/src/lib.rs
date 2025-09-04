//! Excel spreadsheet provider implementation using calamine and rust_xlsxwriter.

pub mod chart;
pub mod formatting;
pub mod formula;
pub mod macro_handling;
pub mod pivot;
pub mod streaming;
pub mod validation;

#[cfg(test)]
pub mod tests;

use anyhow::{anyhow, Result};
use calamine::{open_workbook, Reader, Xlsx};
use dox_core::{
    Cell, RangeRef, ReadOptions, Ruleset, Sheet, SheetId, SpreadsheetMetadata, SpreadsheetProvider,
    WriteOptions,
};
use rust_xlsxwriter::Workbook;
use std::collections::HashMap;
use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;
use tracing::{debug, info, warn};

pub use chart::{
    ChartManager, ChartPosition, ChartSeries, ChartStyle, ExcelChartBuilder, ExcelChartType,
};
pub use formatting::{BasicCellFormat, BasicFormattingManager, FormatTemplate, StyleTheme};
pub use formula::{CellReference, Formula, FormulaContext, FormulaResult};
pub use macro_handling::{
    MacroAnalysisResult, MacroAnalyzer, MacroConfig, MacroHandlingOption, MacroSecurityLevel,
    SecurityRisk, VbaModule, VbaProject,
};
pub use pivot::{
    DataField, PivotField, PivotLocation, PivotTable, PivotTableManager, PivotTableMetadata,
};
pub use streaming::{
    DataChunk, StreamProgress, StreamingConfig, StreamingExcelReader, StreamingProcessor,
};
pub use validation::{
    SimpleValidationConfig, SimpleValidationManager, SimpleValidationType, ValidationTemplate,
};

/// Excel provider for reading and writing XLSX files
pub struct ExcelProvider {
    /// Base directory for Excel files
    base_dir: Option<PathBuf>,
}

impl ExcelProvider {
    /// Creates a new Excel provider
    pub fn new() -> Self {
        Self { base_dir: None }
    }

    /// Creates a new Excel provider with a base directory
    pub fn with_base_dir(base_dir: impl Into<PathBuf>) -> Self {
        Self {
            base_dir: Some(base_dir.into()),
        }
    }

    /// Resolves a sheet ID to a file path
    fn resolve_path(&self, sheet_id: &SheetId) -> PathBuf {
        let path = PathBuf::from(&sheet_id.0);
        if path.is_absolute() {
            path
        } else if let Some(ref base) = self.base_dir {
            base.join(path)
        } else {
            path
        }
    }

    /// Parses a range reference into sheet name and range
    fn parse_range(&self, range: &RangeRef) -> (Option<String>, String) {
        range.parse()
    }

    /// Converts calamine data to our Cell type
    fn convert_calamine_cell(data: &calamine::Data, evaluate_formulas: bool) -> Cell {
        let value = match data {
            calamine::Data::Int(i) => i.to_string(),
            calamine::Data::Float(f) => f.to_string(),
            calamine::Data::String(s) => {
                // Check if it's a formula (starts with =)
                if s.starts_with('=') && !evaluate_formulas {
                    // Return formula as-is
                    s.clone()
                } else if s.starts_with('=') && evaluate_formulas {
                    // TODO: Evaluate formula - for now, return the formula
                    s.clone()
                } else {
                    s.clone()
                }
            }
            calamine::Data::Bool(b) => b.to_string(),
            calamine::Data::DateTime(dt) => dt.to_string(),
            calamine::Data::DateTimeIso(s) => s.clone(),
            calamine::Data::DurationIso(s) => s.clone(),
            calamine::Data::Error(e) => format!("#ERR: {:?}", e),
            calamine::Data::Empty => String::new(),
        };
        Cell::new(value)
    }

    /// Evaluate formulas in the provided data using context from the workbook
    async fn evaluate_formulas_in_data(
        &self,
        data: &mut Vec<Vec<Cell>>,
        _workbook: &mut Xlsx<std::io::BufReader<std::fs::File>>,
    ) -> Result<()> {
        // Create formula context from the workbook data
        let mut formula_context = FormulaContext::new();

        // Populate context with cell values
        for (row_idx, row) in data.iter().enumerate() {
            for (col_idx, cell) in row.iter().enumerate() {
                if let Ok(value) = cell.value.parse::<f64>() {
                    let cell_ref = CellReference::new_single(None, col_idx as u32, row_idx as u32);
                    formula_context.set_cell_value(cell_ref, value);
                } else if !cell.value.is_empty() && !cell.value.starts_with('=') {
                    let cell_ref = CellReference::new_single(None, col_idx as u32, row_idx as u32);
                    formula_context.set_cell_text(cell_ref, cell.value.clone());
                }
            }
        }

        // Evaluate formulas
        for (row_idx, row) in data.iter_mut().enumerate() {
            for (col_idx, cell) in row.iter_mut().enumerate() {
                if cell.value.starts_with('=') {
                    match Formula::parse(&cell.value) {
                        Ok(formula) => match formula.evaluate(&formula_context) {
                            Ok(result) => {
                                cell.value = result.to_string();
                            }
                            Err(e) => {
                                warn!(
                                    "Formula evaluation error at {}{}): {}",
                                    Formula::index_to_column(col_idx as u32).unwrap_or_default(),
                                    row_idx + 1,
                                    e
                                );
                                cell.value = format!("#ERROR: {}", e);
                            }
                        },
                        Err(e) => {
                            warn!("Formula parsing error: {}", e);
                            cell.value = format!("#ERROR: {}", e);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Create a chart in an Excel workbook
    pub async fn create_chart(
        &self,
        sheet_id: &SheetId,
        chart_type: ExcelChartType,
        title: &str,
        data_ranges: Vec<(&str, RangeRef)>,
        category_range: Option<RangeRef>,
        position: Option<ChartPosition>,
    ) -> Result<()> {
        let path = self.resolve_path(sheet_id);

        debug!("Creating chart in Excel file: {:?}", path);

        // For chart creation, we need to create a new workbook or modify existing one
        // Since rust_xlsxwriter is write-only, we'll create a new workbook
        let mut workbook = Workbook::new();
        let worksheet = workbook.add_worksheet().set_name("ChartData")?;

        // Create chart manager and add chart
        let mut chart_manager = ChartManager::new(worksheet);

        match chart_type {
            ExcelChartType::Column
            | ExcelChartType::ColumnStacked
            | ExcelChartType::ColumnStacked100 => {
                chart_manager.create_column_chart(title, data_ranges, category_range, position)?;
            }
            ExcelChartType::Line | ExcelChartType::LineMarkers => {
                chart_manager.create_line_chart(title, data_ranges, category_range, position)?;
            }
            ExcelChartType::Pie => {
                if let Some((_, data_range)) = data_ranges.first() {
                    chart_manager.create_pie_chart(
                        title,
                        data_range.clone(),
                        category_range,
                        position,
                    )?;
                }
            }
            _ => {
                warn!(
                    "Chart type {:?} not yet supported in create_chart method",
                    chart_type
                );
                return Err(anyhow!("Unsupported chart type: {:?}", chart_type));
            }
        }

        // Save the workbook
        workbook
            .save(&path)
            .map_err(|e| anyhow!("Failed to save Excel file with chart: {}", e))?;

        info!("Chart '{}' created successfully in Excel file", title);
        Ok(())
    }

    /// Add a chart to existing data in a workbook
    pub async fn add_chart_to_existing_data(
        &self,
        sheet_id: &SheetId,
        _sheet_name: &str,
        _chart_builder: ExcelChartBuilder,
    ) -> Result<()> {
        let path = self.resolve_path(sheet_id);

        debug!("Adding chart to existing Excel file: {:?}", path);

        // This is a limitation: rust_xlsxwriter cannot modify existing files
        // We would need to:
        // 1. Read the existing file with calamine
        // 2. Extract all data
        // 3. Create a new workbook with rust_xlsxwriter
        // 4. Add the data and charts
        // 5. Save as new file

        warn!(
            "Adding charts to existing files requires reading and recreating the entire workbook"
        );
        Err(anyhow!(
            "Chart addition to existing files not yet implemented - limitation of rust_xlsxwriter"
        ))
    }
}

/// Extended Excel provider with advanced functionality
impl ExcelProvider {
    /// Create a comprehensive Excel report with data and charts
    pub async fn create_excel_report(
        &self,
        sheet_id: &SheetId,
        data: Vec<Vec<Cell>>,
        charts: Vec<(ExcelChartType, String, Vec<(&str, RangeRef)>)>,
    ) -> Result<()> {
        let path = self.resolve_path(sheet_id);

        debug!("Creating Excel report: {:?}", path);

        let mut workbook = Workbook::new();
        let worksheet = workbook.add_worksheet().set_name("Report")?;

        // First, write the data
        for (row_idx, row_data) in data.iter().enumerate() {
            for (col_idx, cell) in row_data.iter().enumerate() {
                let row = row_idx as u32;
                let col = col_idx as u16;

                if cell.value.starts_with('=') {
                    worksheet
                        .write_formula(row, col, cell.value.as_str())
                        .map_err(|e| anyhow!("Failed to write formula: {}", e))?;
                } else if let Ok(number) = cell.value.parse::<f64>() {
                    worksheet
                        .write_number(row, col, number)
                        .map_err(|e| anyhow!("Failed to write number: {}", e))?;
                } else {
                    worksheet
                        .write_string(row, col, &cell.value)
                        .map_err(|e| anyhow!("Failed to write string: {}", e))?;
                }
            }
        }

        // Then, add charts
        let mut chart_manager = ChartManager::new(worksheet);
        let mut chart_row = data.len() as u32 + 2; // Start charts below data
        let chart_count = charts.len();

        for (chart_type, title, data_ranges) in charts {
            let position = ChartPosition {
                col: 0,
                row: chart_row,
                ..ChartPosition::default()
            };

            match chart_type {
                ExcelChartType::Column => {
                    chart_manager.create_column_chart(&title, data_ranges, None, Some(position))?;
                }
                ExcelChartType::Line => {
                    chart_manager.create_line_chart(&title, data_ranges, None, Some(position))?;
                }
                ExcelChartType::Pie => {
                    if let Some((_, data_range)) = data_ranges.first() {
                        chart_manager.create_pie_chart(
                            &title,
                            data_range.clone(),
                            None,
                            Some(position),
                        )?;
                    }
                }
                _ => {
                    warn!(
                        "Chart type {:?} not supported in report generation",
                        chart_type
                    );
                }
            }

            chart_row += 20; // Space charts vertically
        }

        workbook
            .save(&path)
            .map_err(|e| anyhow!("Failed to save Excel report: {}", e))?;

        info!("Excel report created with {} charts", chart_count);
        Ok(())
    }
}

impl Default for ExcelProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl SpreadsheetProvider for ExcelProvider {
    fn read_range(
        &self,
        sheet_id: &SheetId,
        range: &RangeRef,
        options: Option<ReadOptions>,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<Vec<Cell>>>> + Send + '_>> {
        let path = self.resolve_path(sheet_id);
        let (sheet_name, _range_str) = self.parse_range(range);
        let _options = options.unwrap_or_default();

        Box::pin(async move {
            debug!("Reading Excel file: {:?}", path);

            // Open the workbook
            let mut workbook: Xlsx<_> =
                open_workbook(&path).map_err(|e| anyhow!("Failed to open Excel file: {}", e))?;

            // Get the sheet
            let sheet_name = sheet_name.unwrap_or_else(|| {
                workbook
                    .sheet_names()
                    .first()
                    .cloned()
                    .unwrap_or_else(|| "Sheet1".to_string())
            });

            debug!("Reading sheet: {}", sheet_name);

            // Read the range
            let range = workbook
                .worksheet_range(&sheet_name)
                .map_err(|e| anyhow!("Failed to read sheet '{}': {}", sheet_name, e))?;

            let evaluate_formulas = _options.evaluate_formulas;

            // Convert to our Cell type
            let mut result = Vec::new();
            for row in range.rows() {
                let cells: Vec<Cell> = row
                    .iter()
                    .map(|data| Self::convert_calamine_cell(data, evaluate_formulas))
                    .collect();
                result.push(cells);
            }

            // Evaluate formulas if requested
            if evaluate_formulas {
                if let Err(e) = self
                    .evaluate_formulas_in_data(&mut result, &mut workbook)
                    .await
                {
                    warn!("Formula evaluation failed: {}", e);
                }
            }

            info!("Read {} rows from Excel file", result.len());
            Ok(result)
        })
    }

    fn write_range(
        &self,
        sheet_id: &SheetId,
        range: &RangeRef,
        data: Vec<Vec<Cell>>,
        options: Option<WriteOptions>,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>> {
        let path = self.resolve_path(sheet_id);
        let (sheet_name, _range_str) = self.parse_range(range);
        let _options = options.unwrap_or_default();

        Box::pin(async move {
            debug!("Writing to Excel file: {:?}", path);

            // Create a new workbook
            let mut workbook = Workbook::new();
            let sheet_name = sheet_name.unwrap_or_else(|| "Sheet1".to_string());
            let worksheet = workbook.add_worksheet().set_name(&sheet_name)?;

            // Write the data
            for (row_idx, row_data) in data.iter().enumerate() {
                for (col_idx, cell) in row_data.iter().enumerate() {
                    let row = row_idx as u32;
                    let col = col_idx as u16;

                    if cell.value.starts_with('=') {
                        // Write as formula
                        worksheet
                            .write_formula(row, col, cell.value.as_str())
                            .map_err(|e| anyhow!("Failed to write formula: {}", e))?;
                    } else if let Ok(number) = cell.value.parse::<f64>() {
                        // Write as number
                        worksheet
                            .write_number(row, col, number)
                            .map_err(|e| anyhow!("Failed to write number: {}", e))?;
                    } else if cell.value.parse::<bool>().is_ok() {
                        // Write as boolean
                        let bool_val = cell.value.to_lowercase() == "true";
                        worksheet
                            .write_boolean(row, col, bool_val)
                            .map_err(|e| anyhow!("Failed to write boolean: {}", e))?;
                    } else {
                        // Write as string
                        worksheet
                            .write_string(row, col, &cell.value)
                            .map_err(|e| anyhow!("Failed to write string: {}", e))?;
                    }
                }
            }

            // Save the workbook
            workbook
                .save(&path)
                .map_err(|e| anyhow!("Failed to save Excel file: {}", e))?;

            info!("Wrote {} rows to Excel file", data.len());
            Ok(())
        })
    }

    fn list_sheets(
        &self,
        sheet_id: &SheetId,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<Sheet>>> + Send + '_>> {
        let path = self.resolve_path(sheet_id);

        Box::pin(async move {
            debug!("Listing sheets in Excel file: {:?}", path);

            // Open the workbook
            let workbook: Xlsx<_> =
                open_workbook(&path).map_err(|e| anyhow!("Failed to open Excel file: {}", e))?;

            // Get sheet names
            let sheets: Vec<Sheet> = workbook
                .sheet_names()
                .into_iter()
                .map(|name| Sheet {
                    name,
                    id: None,
                    row_count: 0, // Would need to read each sheet to get actual counts
                    column_count: 0, // Would need to read each sheet to get actual counts
                })
                .collect();

            info!("Found {} sheets in Excel file", sheets.len());
            Ok(sheets)
        })
    }

    fn apply_rules(
        &self,
        sheet_id: &SheetId,
        ruleset: &Ruleset,
    ) -> Pin<Box<dyn Future<Output = Result<usize>> + Send + '_>> {
        let path = self.resolve_path(sheet_id);
        let ruleset_name = ruleset.name.clone();

        Box::pin(async move {
            info!("Applying ruleset '{}' to Excel file", ruleset_name);

            // Read the entire workbook
            let mut workbook: Xlsx<_> =
                open_workbook(&path).map_err(|e| anyhow!("Failed to open Excel file: {}", e))?;

            let mut total_replacements = 0;

            // For each sheet
            for sheet_name in workbook.sheet_names() {
                let range = workbook
                    .worksheet_range(&sheet_name)
                    .map_err(|e| anyhow!("Failed to read sheet '{}': {}", sheet_name, e))?;

                // Apply rules to the sheet data
                for _row in range.rows() {
                    // TODO: Implement actual rule application logic
                    // This would involve pattern matching and replacement
                    // For now, just count potential replacements
                    total_replacements += 1;
                }
            }

            warn!("Rule application not fully implemented yet");
            Ok(total_replacements)
        })
    }

    fn create_sheet(
        &self,
        sheet_id: &SheetId,
        name: &str,
    ) -> Pin<Box<dyn Future<Output = Result<Sheet>> + Send + '_>> {
        let _path = self.resolve_path(sheet_id);
        let sheet_name = name.to_string();

        Box::pin(async move {
            info!("Creating sheet '{}' in Excel file", sheet_name);

            // TODO: Implement sheet creation
            // This would involve opening the existing workbook,
            // adding a new sheet, and saving it back

            warn!("Sheet creation not implemented yet");
            Ok(Sheet {
                name: sheet_name,
                id: None,
                row_count: 0,
                column_count: 0,
            })
        })
    }

    fn delete_sheet(
        &self,
        sheet_id: &SheetId,
        sheet_name: &str,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>> {
        let _path = self.resolve_path(sheet_id);
        let sheet_name = sheet_name.to_string();

        Box::pin(async move {
            info!("Deleting sheet '{}' from Excel file", sheet_name);

            // TODO: Implement sheet deletion
            // This would involve opening the existing workbook,
            // removing the sheet, and saving it back

            warn!("Sheet deletion not implemented yet");
            Ok(())
        })
    }

    fn get_metadata(
        &self,
        sheet_id: &SheetId,
    ) -> Pin<Box<dyn Future<Output = Result<SpreadsheetMetadata>> + Send + '_>> {
        let path = self.resolve_path(sheet_id);

        Box::pin(async move {
            debug!("Getting metadata for Excel file: {:?}", path);

            // Open the workbook
            let workbook: Xlsx<_> =
                open_workbook(&path).map_err(|e| anyhow!("Failed to open Excel file: {}", e))?;

            // Get sheet information
            let sheets: Vec<Sheet> = workbook
                .sheet_names()
                .into_iter()
                .map(|name| Sheet {
                    name,
                    id: None,
                    row_count: 0,
                    column_count: 0,
                })
                .collect();

            // Get file metadata
            let file_metadata = std::fs::metadata(&path)?;
            let modified = file_metadata
                .modified()
                .ok()
                .and_then(|t| {
                    t.duration_since(std::time::UNIX_EPOCH)
                        .ok()
                        .map(|d| d.as_secs().to_string())
                })
                .unwrap_or_default();

            Ok(SpreadsheetMetadata {
                title: path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("Unknown")
                    .to_string(),
                sheets,
                created_at: None,
                modified_at: Some(modified),
                author: None,
                properties: HashMap::new(),
            })
        })
    }
}

