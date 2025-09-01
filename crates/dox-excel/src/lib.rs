//! Excel spreadsheet provider implementation using calamine and rust_xlsxwriter.

use anyhow::{anyhow, Result};
use calamine::{open_workbook, Reader, Xlsx};
use dox_core::{
    Cell, ReadOptions, RangeRef, Ruleset, Sheet, SheetId,
    SpreadsheetMetadata, SpreadsheetProvider, WriteOptions,
};
use rust_xlsxwriter::Workbook;
use std::collections::HashMap;
use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;
use tracing::{debug, info, warn};

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
    fn convert_calamine_cell(data: &calamine::Data) -> Cell {
        let value = match data {
            calamine::Data::Int(i) => i.to_string(),
            calamine::Data::Float(f) => f.to_string(),
            calamine::Data::String(s) => s.clone(),
            calamine::Data::Bool(b) => b.to_string(),
            calamine::Data::DateTime(dt) => dt.to_string(),
            calamine::Data::DateTimeIso(s) => s.clone(),
            calamine::Data::DurationIso(s) => s.clone(),
            calamine::Data::Error(e) => format!("#ERR: {:?}", e),
            calamine::Data::Empty => String::new(),
        };
        Cell::new(value)
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
            let mut workbook: Xlsx<_> = open_workbook(&path)
                .map_err(|e| anyhow!("Failed to open Excel file: {}", e))?;

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

            // Convert to our Cell type
            let mut result = Vec::new();
            for row in range.rows() {
                let cells: Vec<Cell> = row.iter().map(Self::convert_calamine_cell).collect();
                result.push(cells);
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
                    worksheet
                        .write_string(row_idx as u32, col_idx as u16, &cell.value)
                        .map_err(|e| anyhow!("Failed to write cell: {}", e))?;
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
            let workbook: Xlsx<_> = open_workbook(&path)
                .map_err(|e| anyhow!("Failed to open Excel file: {}", e))?;

            // Get sheet names
            let sheets: Vec<Sheet> = workbook
                .sheet_names()
                .into_iter()
                .map(|name| Sheet {
                    name,
                    id: None,
                    row_count: 0,    // Would need to read each sheet to get actual counts
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
            let mut workbook: Xlsx<_> = open_workbook(&path)
                .map_err(|e| anyhow!("Failed to open Excel file: {}", e))?;

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
            let workbook: Xlsx<_> = open_workbook(&path)
                .map_err(|e| anyhow!("Failed to open Excel file: {}", e))?;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_excel_provider_creation() {
        let provider = ExcelProvider::new();
        assert!(provider.base_dir.is_none());

        let provider = ExcelProvider::with_base_dir("/tmp");
        assert_eq!(provider.base_dir, Some(PathBuf::from("/tmp")));
    }

    #[test]
    fn test_path_resolution() {
        let provider = ExcelProvider::new();
        let sheet_id = SheetId("/absolute/path.xlsx".to_string());
        assert_eq!(
            provider.resolve_path(&sheet_id),
            PathBuf::from("/absolute/path.xlsx")
        );

        let provider = ExcelProvider::with_base_dir("/base");
        let sheet_id = SheetId("relative/path.xlsx".to_string());
        assert_eq!(
            provider.resolve_path(&sheet_id),
            PathBuf::from("/base/relative/path.xlsx")
        );
    }
}
