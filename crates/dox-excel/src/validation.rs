//! Excel data validation using rust_xlsxwriter
//!
//! This module provides simplified functionality to:
//! - Create basic data validation rules (currently limited by rust_xlsxwriter API)
//! - Document validation requirements for future implementation
//! - Handle dropdown lists and basic input validation

use anyhow::Result;
use rust_xlsxwriter::Worksheet;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use dox_core::RangeRef;

/// Basic validation types supported
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SimpleValidationType {
    /// List of allowed values
    List(Vec<String>),
    /// Any integer number
    WholeNumber,
    /// Any decimal number  
    Decimal,
    /// Custom formula (documented but not implemented)
    Custom(String),
}

/// Simplified validation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleValidationConfig {
    /// Target range for validation
    pub range: RangeRef,
    /// Type of validation
    pub validation_type: SimpleValidationType,
    /// Input message for user
    pub input_message: Option<String>,
    /// Allow blank cells
    pub allow_blank: bool,
}

/// Simple validation manager
pub struct SimpleValidationManager<'a> {
    worksheet: &'a mut Worksheet,
}

impl<'a> SimpleValidationManager<'a> {
    pub fn new(worksheet: &'a mut Worksheet) -> Self {
        Self { worksheet }
    }

    /// Apply basic validation - simplified to avoid API compatibility issues
    pub fn apply_validation(&mut self, config: &SimpleValidationConfig) -> Result<()> {
        // For now, just log what validation would be applied
        // Future implementations can use more sophisticated API calls when available

        match &config.validation_type {
            SimpleValidationType::List(values) => {
                info!(
                    "List validation would be applied to {} with {} options",
                    config.range.0,
                    values.len()
                );

                // Note: Would implement with DataValidation::new().set_list() when API available
                warn!("List validation documented but not applied - API limitations");
            }
            SimpleValidationType::WholeNumber => {
                info!(
                    "Whole number validation would be applied to {}",
                    config.range.0
                );
                warn!("Number validation documented but not applied - API limitations");
            }
            SimpleValidationType::Decimal => {
                info!("Decimal validation would be applied to {}", config.range.0);
                warn!("Decimal validation documented but not applied - API limitations");
            }
            SimpleValidationType::Custom(formula) => {
                info!(
                    "Custom validation with formula '{}' would be applied to {}",
                    formula, config.range.0
                );
                warn!("Custom validation documented but not applied - API limitations");
            }
        }

        if let Some(message) = &config.input_message {
            info!("Input message would be: {}", message);
        }

        info!("Allow blank: {}", config.allow_blank);

        Ok(())
    }

    /// Create a simple dropdown (documented for future implementation)
    pub fn create_dropdown(
        &mut self,
        range: &RangeRef,
        values: Vec<String>,
        input_message: Option<&str>,
    ) -> Result<()> {
        let config = SimpleValidationConfig {
            range: range.clone(),
            validation_type: SimpleValidationType::List(values),
            input_message: input_message.map(|s| s.to_string()),
            allow_blank: true,
        };

        self.apply_validation(&config)
    }

    /// Create number validation (documented for future implementation)
    pub fn create_number_validation(
        &mut self,
        range: &RangeRef,
        is_decimal: bool,
        input_message: Option<&str>,
    ) -> Result<()> {
        let config = SimpleValidationConfig {
            range: range.clone(),
            validation_type: if is_decimal {
                SimpleValidationType::Decimal
            } else {
                SimpleValidationType::WholeNumber
            },
            input_message: input_message.map(|s| s.to_string()),
            allow_blank: false,
        };

        self.apply_validation(&config)
    }
}

/// Template for common validation patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationTemplate {
    pub name: String,
    pub description: String,
    pub validations: Vec<SimpleValidationConfig>,
}

impl ValidationTemplate {
    /// Create a template for common dropdown lists
    pub fn dropdown_template(name: &str, values: Vec<String>) -> Self {
        Self {
            name: name.to_string(),
            description: format!("Dropdown list with {} options", values.len()),
            validations: vec![SimpleValidationConfig {
                range: RangeRef("A1:A100".to_string()), // Default range
                validation_type: SimpleValidationType::List(values),
                input_message: Some("Select from dropdown".to_string()),
                allow_blank: true,
            }],
        }
    }

    /// Create a template for number input
    pub fn number_template(name: &str, is_decimal: bool) -> Self {
        Self {
            name: name.to_string(),
            description: format!(
                "{} number validation",
                if is_decimal { "Decimal" } else { "Whole" }
            ),
            validations: vec![SimpleValidationConfig {
                range: RangeRef("A1:A100".to_string()),
                validation_type: if is_decimal {
                    SimpleValidationType::Decimal
                } else {
                    SimpleValidationType::WholeNumber
                },
                input_message: Some(format!(
                    "Enter a {} number",
                    if is_decimal { "decimal" } else { "whole" }
                )),
                allow_blank: false,
            }],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_template_creation() {
        let template = ValidationTemplate::dropdown_template(
            "Colors",
            vec!["Red".to_string(), "Green".to_string(), "Blue".to_string()],
        );

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
    fn test_number_validation_template() {
        let template = ValidationTemplate::number_template("Ages", false);

        assert_eq!(template.name, "Ages");
        match &template.validations[0].validation_type {
            SimpleValidationType::WholeNumber => {
                // Expected
            }
            _ => panic!("Expected WholeNumber validation type"),
        }
    }
}
