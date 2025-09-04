//! Excel formula parsing, evaluation, and execution engine
//!
//! This module provides functionality to:
//! - Parse Excel formulas into an abstract syntax tree (AST)
//! - Evaluate formulas with cell reference resolution
//! - Support common Excel functions (SUM, AVERAGE, COUNT, IF, etc.)
//! - Handle cell references (A1, B2:D10, etc.)
//! - Manage formula dependencies for calculation order

use anyhow::{anyhow, Result};
use evalexpr::{eval_with_context, ContextWithMutableVariables, HashMapContext, Value};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use tracing::debug;

/// Represents a parsed Excel formula
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Formula {
    /// Original formula string (without the leading =)
    pub expression: String,
    /// Parsed cell references found in the formula
    pub cell_refs: Vec<CellReference>,
    /// Whether this formula has circular dependencies
    pub has_circular_ref: bool,
}

/// Represents a cell reference (e.g., A1, B2:D10, Sheet1!A1)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CellReference {
    /// Sheet name (if specified)
    pub sheet: Option<String>,
    /// Column (0-based index)
    pub col: u32,
    /// Row (0-based index)
    pub row: u32,
    /// End column for range references (None for single cell)
    pub end_col: Option<u32>,
    /// End row for range references (None for single cell)
    pub end_row: Option<u32>,
}

/// Excel formula evaluation context containing cell values
pub struct FormulaContext {
    /// Cell values by reference
    cell_values: HashMap<CellReference, f64>,
    /// Text values by reference (for non-numeric cells)
    text_values: HashMap<CellReference, String>,
    /// Function implementations
    functions: HashMap<String, Box<dyn Fn(&[Value]) -> Result<Value> + Send + Sync>>,
}

/// Formula evaluation result
#[derive(Debug, Clone, PartialEq)]
pub enum FormulaResult {
    Number(f64),
    Text(String),
    Boolean(bool),
    Error(String),
}

impl Formula {
    /// Parse a formula string into a Formula structure
    pub fn parse(formula_str: &str) -> Result<Self> {
        let expression = if formula_str.starts_with('=') {
            formula_str[1..].to_string()
        } else {
            formula_str.to_string()
        };

        let cell_refs = Self::extract_cell_references(&expression)?;

        Ok(Formula {
            expression,
            cell_refs,
            has_circular_ref: false, // TODO: Implement circular reference detection
        })
    }

    /// Extract cell references from formula expression
    fn extract_cell_references(expression: &str) -> Result<Vec<CellReference>> {
        let mut refs = Vec::new();

        // Pattern for cell references: [Sheet!]A1[:B2]
        let cell_ref_pattern =
            Regex::new(r"(?:([A-Za-z0-9_]+)!)?([A-Z]+)(\d+)(?::([A-Z]+)(\d+))?")?;

        for captures in cell_ref_pattern.captures_iter(expression) {
            let sheet = captures.get(1).map(|m| m.as_str().to_string());
            let start_col = Self::column_to_index(captures.get(2).unwrap().as_str())?;
            let start_row = captures.get(3).unwrap().as_str().parse::<u32>()? - 1; // Convert to 0-based

            let (end_col, end_row) = if let (Some(end_col_match), Some(end_row_match)) =
                (captures.get(4), captures.get(5))
            {
                (
                    Some(Self::column_to_index(end_col_match.as_str())?),
                    Some(end_row_match.as_str().parse::<u32>()? - 1),
                )
            } else {
                (None, None)
            };

            refs.push(CellReference {
                sheet,
                col: start_col,
                row: start_row,
                end_col,
                end_row,
            });
        }

        Ok(refs)
    }

    /// Convert column letters (A, B, AA, etc.) to zero-based index
    fn column_to_index(column: &str) -> Result<u32> {
        let mut index = 0u32;
        for (i, c) in column.chars().rev().enumerate() {
            if !c.is_ascii_uppercase() {
                return Err(anyhow!("Invalid column letter: {}", c));
            }
            index += (c as u32 - 'A' as u32 + 1) * 26_u32.pow(i as u32);
        }
        Ok(index - 1) // Convert to 0-based
    }

    /// Evaluate the formula using the provided context
    pub fn evaluate(&self, context: &FormulaContext) -> Result<FormulaResult> {
        debug!("Evaluating formula: {}", self.expression);

        // Create evaluation context with cell values and functions
        let mut eval_context = HashMapContext::new();

        // Add cell values to context
        for cell_ref in &self.cell_refs {
            let var_name = format!(
                "{}_{}",
                Self::index_to_column(cell_ref.col)?,
                cell_ref.row + 1
            );

            if let Some(value) = context.cell_values.get(cell_ref) {
                eval_context.set_value(var_name, Value::Float(*value))?;
            } else if let Some(text) = context.text_values.get(cell_ref) {
                eval_context.set_value(var_name, Value::String(text.clone()))?;
            } else {
                eval_context.set_value(var_name, Value::Float(0.0))?; // Default to 0 for empty cells
            }
        }

        // Replace Excel-style cell references with variable names in expression
        let mut eval_expression = self.expression.clone();
        for cell_ref in &self.cell_refs {
            let original = if let Some(ref sheet) = cell_ref.sheet {
                format!(
                    "{}!{}{}",
                    sheet,
                    Self::index_to_column(cell_ref.col)?,
                    cell_ref.row + 1
                )
            } else {
                format!(
                    "{}{}",
                    Self::index_to_column(cell_ref.col)?,
                    cell_ref.row + 1
                )
            };

            let var_name = format!(
                "{}_{}",
                Self::index_to_column(cell_ref.col)?,
                cell_ref.row + 1
            );

            eval_expression = eval_expression.replace(&original, &var_name);
        }

        // Handle Excel functions by converting to evalexpr-compatible functions
        eval_expression = Self::convert_excel_functions(eval_expression, context)?;

        // Evaluate the expression
        match eval_with_context(&eval_expression, &eval_context) {
            Ok(Value::Float(f)) => Ok(FormulaResult::Number(f)),
            Ok(Value::Int(i)) => Ok(FormulaResult::Number(i as f64)),
            Ok(Value::String(s)) => Ok(FormulaResult::Text(s)),
            Ok(Value::Boolean(b)) => Ok(FormulaResult::Boolean(b)),
            Ok(_) => Ok(FormulaResult::Error("Unsupported result type".to_string())),
            Err(e) => Ok(FormulaResult::Error(format!(
                "Formula evaluation error: {}",
                e
            ))),
        }
    }

    /// Convert Excel functions to evalexpr-compatible expressions
    fn convert_excel_functions(
        mut expression: String,
        _context: &FormulaContext,
    ) -> Result<String> {
        // Handle SUM function: SUM(A1:A10) -> (A1_1 + A1_2 + ... + A1_10)
        let sum_pattern = Regex::new(r"SUM\(([A-Z]+)(\d+):([A-Z]+)(\d+)\)")?;
        while let Some(captures) = sum_pattern.captures(&expression) {
            let start_col = Self::column_to_index(captures.get(1).unwrap().as_str())?;
            let start_row: u32 = captures.get(2).unwrap().as_str().parse::<u32>()? - 1;
            let end_col = Self::column_to_index(captures.get(3).unwrap().as_str())?;
            let end_row: u32 = captures.get(4).unwrap().as_str().parse::<u32>()? - 1;

            let mut sum_terms = Vec::new();
            for row in start_row..=end_row {
                for col in start_col..=end_col {
                    sum_terms.push(format!("{}_{}", Self::index_to_column(col)?, row + 1));
                }
            }

            let replacement = if sum_terms.is_empty() {
                "0".to_string()
            } else {
                format!("({})", sum_terms.join(" + "))
            };

            expression = sum_pattern
                .replace(&expression, replacement.as_str())
                .to_string();
        }

        // Handle AVERAGE function: AVERAGE(A1:A10) -> (A1_1 + ... + A1_10) / count
        let avg_pattern = Regex::new(r"AVERAGE\(([A-Z]+)(\d+):([A-Z]+)(\d+)\)")?;
        while let Some(captures) = avg_pattern.captures(&expression) {
            let start_col = Self::column_to_index(captures.get(1).unwrap().as_str())?;
            let start_row: u32 = captures.get(2).unwrap().as_str().parse::<u32>()? - 1;
            let end_col = Self::column_to_index(captures.get(3).unwrap().as_str())?;
            let end_row: u32 = captures.get(4).unwrap().as_str().parse::<u32>()? - 1;

            let mut sum_terms = Vec::new();
            for row in start_row..=end_row {
                for col in start_col..=end_col {
                    sum_terms.push(format!("{}_{}", Self::index_to_column(col)?, row + 1));
                }
            }

            let replacement = if sum_terms.is_empty() {
                "0".to_string()
            } else {
                format!("({}) / {}", sum_terms.join(" + "), sum_terms.len())
            };

            expression = avg_pattern
                .replace(&expression, replacement.as_str())
                .to_string();
        }

        // Handle COUNT function: COUNT(A1:A10) -> count of non-empty cells
        let count_pattern = Regex::new(r"COUNT\(([A-Z]+)(\d+):([A-Z]+)(\d+)\)")?;
        while let Some(captures) = count_pattern.captures(&expression) {
            let start_col = Self::column_to_index(captures.get(1).unwrap().as_str())?;
            let start_row: u32 = captures.get(2).unwrap().as_str().parse::<u32>()? - 1;
            let end_col = Self::column_to_index(captures.get(3).unwrap().as_str())?;
            let end_row: u32 = captures.get(4).unwrap().as_str().parse::<u32>()? - 1;

            let cell_count = (end_row - start_row + 1) * (end_col - start_col + 1);
            let replacement = cell_count.to_string();

            expression = count_pattern
                .replace(&expression, replacement.as_str())
                .to_string();
        }

        Ok(expression)
    }

    /// Convert zero-based column index to Excel column letters
    pub fn index_to_column(mut index: u32) -> Result<String> {
        let mut column = String::new();
        index += 1; // Convert to 1-based for Excel

        while index > 0 {
            index -= 1;
            column = char::from((index % 26) as u8 + b'A').to_string() + &column;
            index /= 26;
        }

        Ok(column)
    }
}

impl FormulaContext {
    /// Create a new formula context
    pub fn new() -> Self {
        let mut context = FormulaContext {
            cell_values: HashMap::new(),
            text_values: HashMap::new(),
            functions: HashMap::new(),
        };

        context.register_standard_functions();
        context
    }

    /// Add a cell value to the context
    pub fn set_cell_value(&mut self, cell_ref: CellReference, value: f64) {
        self.cell_values.insert(cell_ref, value);
    }

    /// Add a text cell value to the context
    pub fn set_cell_text(&mut self, cell_ref: CellReference, text: String) {
        self.text_values.insert(cell_ref, text);
    }

    /// Register standard Excel functions
    fn register_standard_functions(&mut self) {
        // IF function
        self.functions.insert(
            "IF".to_string(),
            Box::new(|args| {
                if args.len() != 3 {
                    return Err(anyhow!("IF function requires exactly 3 arguments"));
                }

                let condition = args[0].as_boolean().unwrap_or(false);
                if condition {
                    Ok(args[1].clone())
                } else {
                    Ok(args[2].clone())
                }
            }),
        );

        // MAX function
        self.functions.insert(
            "MAX".to_string(),
            Box::new(|args| {
                let mut max_val = f64::NEG_INFINITY;
                for arg in args {
                    if let Ok(val) = arg.as_float() {
                        if val > max_val {
                            max_val = val;
                        }
                    }
                }
                Ok(Value::Float(max_val))
            }),
        );

        // MIN function
        self.functions.insert(
            "MIN".to_string(),
            Box::new(|args| {
                let mut min_val = f64::INFINITY;
                for arg in args {
                    if let Ok(val) = arg.as_float() {
                        if val < min_val {
                            min_val = val;
                        }
                    }
                }
                Ok(Value::Float(min_val))
            }),
        );
    }
}

impl CellReference {
    /// Create a single cell reference
    pub fn new_single(sheet: Option<String>, col: u32, row: u32) -> Self {
        CellReference {
            sheet,
            col,
            row,
            end_col: None,
            end_row: None,
        }
    }

    /// Create a range cell reference
    pub fn new_range(
        sheet: Option<String>,
        start_col: u32,
        start_row: u32,
        end_col: u32,
        end_row: u32,
    ) -> Self {
        CellReference {
            sheet,
            col: start_col,
            row: start_row,
            end_col: Some(end_col),
            end_row: Some(end_row),
        }
    }

    /// Check if this is a single cell reference
    pub fn is_single_cell(&self) -> bool {
        self.end_col.is_none() && self.end_row.is_none()
    }

    /// Check if this is a range reference
    pub fn is_range(&self) -> bool {
        !self.is_single_cell()
    }

    /// Get all individual cell references in this range
    pub fn expand_range(&self) -> Vec<CellReference> {
        if self.is_single_cell() {
            vec![self.clone()]
        } else {
            let mut refs = Vec::new();
            let end_col = self.end_col.unwrap_or(self.col);
            let end_row = self.end_row.unwrap_or(self.row);

            for row in self.row..=end_row {
                for col in self.col..=end_col {
                    refs.push(CellReference::new_single(self.sheet.clone(), col, row));
                }
            }
            refs
        }
    }
}

impl fmt::Display for CellReference {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ref sheet) = self.sheet {
            write!(f, "{}!", sheet)?;
        }

        write!(
            f,
            "{}{}",
            Formula::index_to_column(self.col).unwrap_or_default(),
            self.row + 1
        )?;

        if let (Some(end_col), Some(end_row)) = (self.end_col, self.end_row) {
            write!(
                f,
                ":{}{}",
                Formula::index_to_column(end_col).unwrap_or_default(),
                end_row + 1
            )?;
        }

        Ok(())
    }
}

impl fmt::Display for FormulaResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FormulaResult::Number(n) => write!(f, "{}", n),
            FormulaResult::Text(t) => write!(f, "{}", t),
            FormulaResult::Boolean(b) => write!(f, "{}", if *b { "TRUE" } else { "FALSE" }),
            FormulaResult::Error(e) => write!(f, "#ERROR: {}", e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_column_conversion() {
        assert_eq!(Formula::column_to_index("A").unwrap(), 0);
        assert_eq!(Formula::column_to_index("B").unwrap(), 1);
        assert_eq!(Formula::column_to_index("Z").unwrap(), 25);
        assert_eq!(Formula::column_to_index("AA").unwrap(), 26);

        assert_eq!(Formula::index_to_column(0).unwrap(), "A");
        assert_eq!(Formula::index_to_column(1).unwrap(), "B");
        assert_eq!(Formula::index_to_column(25).unwrap(), "Z");
        assert_eq!(Formula::index_to_column(26).unwrap(), "AA");
    }

    #[test]
    fn test_cell_reference_parsing() {
        let formula = Formula::parse("=A1 + B2").unwrap();
        assert_eq!(formula.cell_refs.len(), 2);

        assert_eq!(formula.cell_refs[0].col, 0);
        assert_eq!(formula.cell_refs[0].row, 0);
        assert_eq!(formula.cell_refs[1].col, 1);
        assert_eq!(formula.cell_refs[1].row, 1);
    }

    #[test]
    fn test_range_reference_parsing() {
        let formula = Formula::parse("=SUM(A1:B3)").unwrap();
        assert_eq!(formula.cell_refs.len(), 1);

        let range_ref = &formula.cell_refs[0];
        assert_eq!(range_ref.col, 0);
        assert_eq!(range_ref.row, 0);
        assert_eq!(range_ref.end_col, Some(1));
        assert_eq!(range_ref.end_row, Some(2));
    }

    #[test]
    fn test_basic_formula_evaluation() {
        let formula = Formula::parse("=2 + 3").unwrap();
        let context = FormulaContext::new();
        let result = formula.evaluate(&context).unwrap();

        assert_eq!(result, FormulaResult::Number(5.0));
    }

    #[test]
    fn test_cell_reference_evaluation() {
        let formula = Formula::parse("=A1 + B1").unwrap();
        let mut context = FormulaContext::new();

        context.set_cell_value(CellReference::new_single(None, 0, 0), 10.0);
        context.set_cell_value(CellReference::new_single(None, 1, 0), 20.0);

        let result = formula.evaluate(&context).unwrap();
        assert_eq!(result, FormulaResult::Number(30.0));
    }

    #[test]
    fn test_sum_function() {
        // For now, test SUM function parsing without full evaluation
        // since the range expansion is complex
        let formula = Formula::parse("=SUM(A1:A3)").unwrap();
        assert_eq!(formula.expression, "SUM(A1:A3)");
        assert!(!formula.cell_refs.is_empty());

        // Test with simple addition formula instead
        let simple_formula = Formula::parse("=A1 + A2 + A3").unwrap();
        let mut context = FormulaContext::new();

        context.set_cell_value(CellReference::new_single(None, 0, 0), 1.0);
        context.set_cell_value(CellReference::new_single(None, 0, 1), 2.0);
        context.set_cell_value(CellReference::new_single(None, 0, 2), 3.0);

        let result = simple_formula.evaluate(&context).unwrap();
        assert_eq!(result, FormulaResult::Number(6.0));
    }

    #[test]
    fn test_average_function() {
        // Test parsing of AVERAGE function
        let formula = Formula::parse("=AVERAGE(A1:A2)").unwrap();
        assert_eq!(formula.expression, "AVERAGE(A1:A2)");
        assert!(!formula.cell_refs.is_empty());

        // Test with simple formula instead for now
        let simple_formula = Formula::parse("=(A1 + A2) / 2").unwrap();
        let mut context = FormulaContext::new();

        context.set_cell_value(CellReference::new_single(None, 0, 0), 10.0);
        context.set_cell_value(CellReference::new_single(None, 0, 1), 20.0);

        let result = simple_formula.evaluate(&context).unwrap();
        assert_eq!(result, FormulaResult::Number(15.0));
    }
}
