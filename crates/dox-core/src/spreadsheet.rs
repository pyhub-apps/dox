//! Spreadsheet provider traits and common types.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

/// Represents a spreadsheet identifier (e.g., Google Sheets ID, file path)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SheetId(pub String);

/// Represents a range reference using A1 notation (e.g., "Sheet1!A1:D10")
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RangeRef(pub String);

impl RangeRef {
    /// Creates a new RangeRef from a string
    pub fn new(range: impl Into<String>) -> Self {
        Self(range.into())
    }

    /// Parses the range into sheet name and cell range components
    pub fn parse(&self) -> (Option<String>, String) {
        if let Some(idx) = self.0.find('!') {
            let sheet = self.0[..idx].to_string();
            let range = self.0[idx + 1..].to_string();
            (Some(sheet), range)
        } else {
            (None, self.0.clone())
        }
    }
}

/// Represents a single cell in a spreadsheet
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Cell {
    /// The cell's value as a string
    pub value: String,
    /// Optional formatting information
    pub format: Option<CellFormat>,
}

impl Cell {
    /// Creates a new cell with just a value
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            format: None,
        }
    }

    /// Creates a new cell with value and format
    pub fn with_format(value: impl Into<String>, format: CellFormat) -> Self {
        Self {
            value: value.into(),
            format: Some(format),
        }
    }
}

/// Cell formatting information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct CellFormat {
    pub bold: bool,
    pub italic: bool,
    pub font_size: Option<f32>,
    pub background_color: Option<String>,
    pub text_color: Option<String>,
}

/// Represents a sheet within a spreadsheet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sheet {
    pub name: String,
    pub id: Option<String>,
    pub row_count: usize,
    pub column_count: usize,
}

/// Represents a rule to apply to spreadsheet data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    pub name: String,
    pub pattern: String,
    pub replacement: String,
    pub target_range: Option<RangeRef>,
    pub case_sensitive: bool,
    pub regex: bool,
}

/// A collection of rules to apply
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ruleset {
    pub name: String,
    pub rules: Vec<Rule>,
}

/// Options for reading from a spreadsheet
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReadOptions {
    /// Whether to include formatting information
    pub include_format: bool,
    /// Whether to evaluate formulas (if false, returns formula strings)
    pub evaluate_formulas: bool,
    /// Maximum number of rows to read
    pub max_rows: Option<usize>,
    /// Maximum number of columns to read
    pub max_columns: Option<usize>,
}

/// Options for writing to a spreadsheet
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WriteOptions {
    /// Whether to overwrite existing content
    pub overwrite: bool,
    /// Whether to auto-expand the sheet if needed
    pub auto_expand: bool,
    /// Whether to preserve existing formatting
    pub preserve_format: bool,
}

/// Async trait for spreadsheet providers
pub trait SpreadsheetProvider: Send + Sync {
    /// Reads data from a specified range
    fn read_range(
        &self,
        sheet_id: &SheetId,
        range: &RangeRef,
        options: Option<ReadOptions>,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<Vec<Cell>>>> + Send + '_>>;

    /// Writes data to a specified range
    fn write_range(
        &self,
        sheet_id: &SheetId,
        range: &RangeRef,
        data: Vec<Vec<Cell>>,
        options: Option<WriteOptions>,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>>;

    /// Lists all sheets in a spreadsheet
    fn list_sheets(
        &self,
        sheet_id: &SheetId,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<Sheet>>> + Send + '_>>;

    /// Applies a ruleset to a spreadsheet
    fn apply_rules(
        &self,
        sheet_id: &SheetId,
        ruleset: &Ruleset,
    ) -> Pin<Box<dyn Future<Output = Result<usize>> + Send + '_>>;

    /// Creates a new sheet in the spreadsheet
    fn create_sheet(
        &self,
        sheet_id: &SheetId,
        name: &str,
    ) -> Pin<Box<dyn Future<Output = Result<Sheet>> + Send + '_>>;

    /// Deletes a sheet from the spreadsheet
    fn delete_sheet(
        &self,
        sheet_id: &SheetId,
        sheet_name: &str,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>>;

    /// Gets metadata about the spreadsheet
    fn get_metadata(
        &self,
        sheet_id: &SheetId,
    ) -> Pin<Box<dyn Future<Output = Result<SpreadsheetMetadata>> + Send + '_>>;
}

/// Metadata about a spreadsheet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpreadsheetMetadata {
    pub title: String,
    pub sheets: Vec<Sheet>,
    pub created_at: Option<String>,
    pub modified_at: Option<String>,
    pub author: Option<String>,
    pub properties: HashMap<String, String>,
}

/// Configuration for spreadsheet providers
#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct SpreadsheetConfig {
    /// Default read options
    pub default_read_options: Option<ReadOptions>,
    /// Default write options
    pub default_write_options: Option<WriteOptions>,
    /// Provider-specific configuration
    pub provider_config: HashMap<String, serde_json::Value>,
}

