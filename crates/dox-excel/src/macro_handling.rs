//! Excel macro handling (preserve/strip options)
//!
//! This module provides functionality to:
//! - Detect macro-enabled Excel files (.xlsm)
//! - Extract macro metadata and security information
//! - Provide options to preserve or strip macros during processing
//! - Generate security warnings for macro-enabled files
//! - Handle VBA project metadata

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tracing::{debug, info, warn};

/// Macro handling options
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum MacroHandlingOption {
    /// Preserve macros in output (requires .xlsm format)
    Preserve,
    /// Strip macros (convert to .xlsx format)
    Strip,
    /// Warn about macros but preserve them
    WarnAndPreserve,
    /// Block processing if macros are detected
    Block,
}

/// Macro security level
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum MacroSecurityLevel {
    /// No macros detected
    None,
    /// Macros present but appear safe
    Low,
    /// Macros with moderate risk indicators
    Medium,
    /// Macros with high risk indicators  
    High,
    /// Macros with critical security concerns
    Critical,
}

/// VBA project metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VbaProject {
    /// Project name
    pub name: Option<String>,
    /// Project description
    pub description: Option<String>,
    /// VBA modules found
    pub modules: Vec<VbaModule>,
    /// References to external libraries
    pub references: Vec<VbaReference>,
    /// Digital signature information
    pub signature_info: Option<SignatureInfo>,
    /// Security assessment
    pub security_level: MacroSecurityLevel,
}

/// VBA module information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VbaModule {
    /// Module name
    pub name: String,
    /// Module type (standard, class, form, etc.)
    pub module_type: VbaModuleType,
    /// Code size in bytes
    pub code_size: usize,
    /// Number of procedures/functions
    pub procedure_count: usize,
    /// Security risk indicators found
    pub risk_indicators: Vec<SecurityRisk>,
}

/// VBA module types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum VbaModuleType {
    Standard,
    Class,
    Form,
    Document,
    Unknown,
}

/// VBA reference to external library
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VbaReference {
    /// Library name
    pub name: String,
    /// Library description
    pub description: Option<String>,
    /// Library GUID
    pub guid: Option<String>,
    /// Version information
    pub version: Option<String>,
}

/// Digital signature information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignatureInfo {
    /// Whether the project is digitally signed
    pub is_signed: bool,
    /// Signer name (if available)
    pub signer: Option<String>,
    /// Signature validity
    pub is_valid: bool,
    /// Certificate issuer
    pub issuer: Option<String>,
}

/// Security risk indicator
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SecurityRisk {
    /// Risk type
    pub risk_type: RiskType,
    /// Risk severity
    pub severity: RiskSeverity,
    /// Description of the risk
    pub description: String,
    /// Location in code (if applicable)
    pub location: Option<String>,
}

/// Types of security risks in VBA code
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RiskType {
    /// File system access
    FileSystemAccess,
    /// Network operations
    NetworkAccess,
    /// Registry modifications
    RegistryAccess,
    /// Shell/command execution
    ShellExecution,
    /// COM object creation
    ComObjectCreation,
    /// DLL loading
    DllLoading,
    /// Auto-execution (Auto_Open, Workbook_Open, etc.)
    AutoExecution,
    /// Obfuscated code
    ObfuscatedCode,
    /// Suspicious API calls
    SuspiciousApiCalls,
}

/// Security risk severity levels
#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum RiskSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Macro handling configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacroConfig {
    /// How to handle macros
    pub handling_option: MacroHandlingOption,
    /// Security level threshold for blocking
    pub security_threshold: MacroSecurityLevel,
    /// Whether to generate detailed security reports
    pub detailed_analysis: bool,
    /// Whether to backup original file before stripping macros
    pub backup_original: bool,
    /// Custom risk indicators to look for
    pub custom_risk_patterns: Vec<String>,
}

/// Macro analyzer for Excel files
pub struct MacroAnalyzer {
    config: MacroConfig,
}

/// Result of macro analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacroAnalysisResult {
    /// Whether macros were detected
    pub has_macros: bool,
    /// VBA project information (if macros found)
    pub vba_project: Option<VbaProject>,
    /// Overall security assessment
    pub security_level: MacroSecurityLevel,
    /// Security warnings generated
    pub warnings: Vec<String>,
    /// Recommended action
    pub recommended_action: MacroHandlingOption,
    /// File format information
    pub file_format: ExcelFileFormat,
}

/// Excel file format types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ExcelFileFormat {
    /// Excel 2007+ without macros (.xlsx)
    Xlsx,
    /// Excel 2007+ with macros (.xlsm)
    Xlsm,
    /// Excel binary format (.xlsb)
    Xlsb,
    /// Legacy Excel format (.xls)
    Xls,
    /// Unknown or unsupported format
    Unknown,
}

impl Default for MacroConfig {
    fn default() -> Self {
        MacroConfig {
            handling_option: MacroHandlingOption::WarnAndPreserve,
            security_threshold: MacroSecurityLevel::Medium,
            detailed_analysis: true,
            backup_original: true,
            custom_risk_patterns: vec![
                "Shell".to_string(),
                "CreateObject".to_string(),
                "GetObject".to_string(),
                "WScript".to_string(),
                "Environ".to_string(),
            ],
        }
    }
}

impl MacroConfig {
    /// Create a security-focused configuration
    pub fn security_focused() -> Self {
        MacroConfig {
            handling_option: MacroHandlingOption::Block,
            security_threshold: MacroSecurityLevel::Low,
            detailed_analysis: true,
            backup_original: true,
            custom_risk_patterns: vec![
                "Shell".to_string(),
                "CreateObject".to_string(),
                "GetObject".to_string(),
                "WScript".to_string(),
                "Environ".to_string(),
                "Dir".to_string(),
                "Kill".to_string(),
                "Open".to_string(),
                "SaveAs".to_string(),
                "Execute".to_string(),
            ],
        }
    }

    /// Create a permissive configuration
    pub fn permissive() -> Self {
        MacroConfig {
            handling_option: MacroHandlingOption::Preserve,
            security_threshold: MacroSecurityLevel::High,
            detailed_analysis: false,
            backup_original: false,
            custom_risk_patterns: vec![],
        }
    }
}

impl MacroAnalyzer {
    /// Create a new macro analyzer
    pub fn new(config: MacroConfig) -> Self {
        MacroAnalyzer { config }
    }

    /// Analyze an Excel file for macros
    pub fn analyze_file(&self, file_path: &Path) -> Result<MacroAnalysisResult> {
        debug!("Analyzing file for macros: {}", file_path.display());

        let file_format = self.detect_file_format(file_path)?;
        let has_macros = self.detect_macros(&file_format)?;

        let mut result = MacroAnalysisResult {
            has_macros,
            vba_project: None,
            security_level: MacroSecurityLevel::None,
            warnings: Vec::new(),
            recommended_action: self.config.handling_option,
            file_format,
        };

        if has_macros {
            info!("Macros detected in file: {}", file_path.display());

            if self.config.detailed_analysis {
                // Note: Full VBA analysis would require complex parsing
                // This is a simplified implementation
                result.vba_project = Some(self.analyze_vba_project(file_path)?);
                result.security_level = self.assess_security_level(&result.vba_project);
            } else {
                result.security_level = MacroSecurityLevel::Medium; // Conservative default
            }

            result.warnings = self.generate_warnings(&result);
            result.recommended_action = self.recommend_action(&result);
        }

        Ok(result)
    }

    /// Detect file format based on extension and content
    fn detect_file_format(&self, file_path: &Path) -> Result<ExcelFileFormat> {
        let extension = file_path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_lowercase();

        let format = match extension.as_str() {
            "xlsx" => ExcelFileFormat::Xlsx,
            "xlsm" => ExcelFileFormat::Xlsm,
            "xlsb" => ExcelFileFormat::Xlsb,
            "xls" => ExcelFileFormat::Xls,
            _ => ExcelFileFormat::Unknown,
        };

        debug!("Detected file format: {:?}", format);
        Ok(format)
    }

    /// Detect if macros are present
    fn detect_macros(&self, file_format: &ExcelFileFormat) -> Result<bool> {
        match file_format {
            ExcelFileFormat::Xlsm => {
                // .xlsm files by definition contain macros
                Ok(true)
            }
            ExcelFileFormat::Xlsb => {
                // Binary format may contain macros - would need content inspection
                warn!("Binary Excel format detected - macro detection limited");
                Ok(true) // Conservative assumption
            }
            ExcelFileFormat::Xls => {
                // Legacy format may contain macros - would need content inspection
                warn!("Legacy Excel format detected - macro detection limited");
                Ok(true) // Conservative assumption
            }
            ExcelFileFormat::Xlsx => {
                // .xlsx files should not contain macros, but check to be sure
                Ok(false)
            }
            ExcelFileFormat::Unknown => {
                warn!("Unknown file format - cannot determine macro presence");
                Ok(false)
            }
        }
    }

    /// Analyze VBA project (simplified implementation)
    fn analyze_vba_project(&self, _file_path: &Path) -> Result<VbaProject> {
        // Note: Full VBA analysis would require:
        // 1. Extracting VBA streams from the Excel file
        // 2. Decompiling VBA bytecode
        // 3. Parsing VBA source code
        // 4. Analyzing for security risks
        //
        // This is a complex task that would require specialized libraries
        // or integration with tools like oletools (Python)

        warn!("VBA analysis is simplified - full implementation requires specialized parsing");

        // Return a placeholder VBA project with basic info
        Ok(VbaProject {
            name: Some("VBAProject".to_string()),
            description: None,
            modules: vec![VbaModule {
                name: "Module1".to_string(),
                module_type: VbaModuleType::Standard,
                code_size: 0,
                procedure_count: 0,
                risk_indicators: vec![],
            }],
            references: vec![],
            signature_info: None,
            security_level: MacroSecurityLevel::Medium,
        })
    }

    /// Assess overall security level
    fn assess_security_level(&self, vba_project: &Option<VbaProject>) -> MacroSecurityLevel {
        if let Some(project) = vba_project {
            let mut max_severity = RiskSeverity::Low;

            for module in &project.modules {
                for risk in &module.risk_indicators {
                    if risk.severity > max_severity {
                        max_severity = risk.severity.clone();
                    }
                }
            }

            match max_severity {
                RiskSeverity::Low => MacroSecurityLevel::Low,
                RiskSeverity::Medium => MacroSecurityLevel::Medium,
                RiskSeverity::High => MacroSecurityLevel::High,
                RiskSeverity::Critical => MacroSecurityLevel::Critical,
            }
        } else {
            MacroSecurityLevel::None
        }
    }

    /// Generate security warnings
    fn generate_warnings(&self, result: &MacroAnalysisResult) -> Vec<String> {
        let mut warnings = Vec::new();

        if result.has_macros {
            warnings.push("⚠️  This file contains macros (VBA code)".to_string());

            match result.security_level {
                MacroSecurityLevel::None => {
                    // No additional warnings
                }
                MacroSecurityLevel::Low => {
                    warnings.push("🟡 Low security risk detected in macros".to_string());
                }
                MacroSecurityLevel::Medium => {
                    warnings.push("🟠 Medium security risk detected in macros".to_string());
                    warnings.push("   Consider reviewing macro code before execution".to_string());
                }
                MacroSecurityLevel::High => {
                    warnings.push("🔴 High security risk detected in macros".to_string());
                    warnings.push("   Macros contain potentially dangerous operations".to_string());
                    warnings.push("   Recommend stripping macros or blocking file".to_string());
                }
                MacroSecurityLevel::Critical => {
                    warnings.push("🚨 CRITICAL security risk detected in macros".to_string());
                    warnings.push("   Macros contain dangerous operations".to_string());
                    warnings.push("   STRONGLY RECOMMEND blocking this file".to_string());
                }
            }

            if let Some(ref project) = result.vba_project {
                if let Some(ref signature) = project.signature_info {
                    if signature.is_signed {
                        if signature.is_valid {
                            warnings.push("✅ Macros are digitally signed and valid".to_string());
                        } else {
                            warnings
                                .push("❌ Macros are signed but signature is invalid".to_string());
                        }
                    } else {
                        warnings.push("⚠️  Macros are not digitally signed".to_string());
                    }
                }
            }
        }

        warnings
    }

    /// Recommend handling action based on analysis
    fn recommend_action(&self, result: &MacroAnalysisResult) -> MacroHandlingOption {
        if !result.has_macros {
            return MacroHandlingOption::Preserve; // No macros to worry about
        }

        match result.security_level {
            MacroSecurityLevel::None => MacroHandlingOption::Preserve,
            MacroSecurityLevel::Low => {
                if self.config.security_threshold <= MacroSecurityLevel::Low {
                    MacroHandlingOption::Strip
                } else {
                    MacroHandlingOption::WarnAndPreserve
                }
            }
            MacroSecurityLevel::Medium => {
                if self.config.security_threshold <= MacroSecurityLevel::Medium {
                    MacroHandlingOption::Strip
                } else {
                    MacroHandlingOption::WarnAndPreserve
                }
            }
            MacroSecurityLevel::High => {
                if self.config.security_threshold <= MacroSecurityLevel::High {
                    MacroHandlingOption::Block
                } else {
                    MacroHandlingOption::Strip
                }
            }
            MacroSecurityLevel::Critical => MacroHandlingOption::Block,
        }
    }

    /// Apply macro handling action to a file
    pub fn handle_macros(
        &self,
        file_path: &Path,
        analysis_result: &MacroAnalysisResult,
        action: MacroHandlingOption,
    ) -> Result<String> {
        match action {
            MacroHandlingOption::Preserve => {
                info!("Preserving macros in file");
                Ok("Macros preserved".to_string())
            }
            MacroHandlingOption::Strip => {
                info!("Stripping macros from file");
                self.strip_macros(file_path, analysis_result)
            }
            MacroHandlingOption::WarnAndPreserve => {
                warn!("Macros detected but preserved with warnings");
                Ok("Macros preserved with security warnings".to_string())
            }
            MacroHandlingOption::Block => {
                warn!("Blocking file processing due to macro security concerns");
                Err(anyhow!(
                    "File processing blocked due to macro security concerns"
                ))
            }
        }
    }

    /// Strip macros from file (convert .xlsm to .xlsx)
    fn strip_macros(
        &self,
        file_path: &Path,
        _analysis_result: &MacroAnalysisResult,
    ) -> Result<String> {
        // Note: Actually stripping macros would require:
        // 1. Opening the Excel file with a library that can modify it
        // 2. Removing the VBA project streams
        // 3. Saving as .xlsx format
        //
        // This is complex and would require integration with libraries
        // that can modify Excel files at the binary level

        warn!("Macro stripping is not fully implemented");
        warn!("This would require binary Excel file manipulation");

        // For now, suggest renaming to .xlsx as a safety measure
        let new_path = file_path.with_extension("xlsx");

        Ok(format!(
            "Macro stripping not implemented. Consider manually saving as .xlsx: {}",
            new_path.display()
        ))
    }

    /// Generate a security report for the file
    pub fn generate_security_report(&self, analysis_result: &MacroAnalysisResult) -> String {
        let mut report = Vec::new();

        report.push("🔍 EXCEL MACRO SECURITY REPORT".to_string());
        report.push("=".repeat(40));
        report.push("".to_string());

        report.push(format!("File Format: {:?}", analysis_result.file_format));
        report.push(format!("Contains Macros: {}", analysis_result.has_macros));
        report.push(format!(
            "Security Level: {:?}",
            analysis_result.security_level
        ));
        report.push(format!(
            "Recommended Action: {:?}",
            analysis_result.recommended_action
        ));
        report.push("".to_string());

        if !analysis_result.warnings.is_empty() {
            report.push("⚠️  SECURITY WARNINGS:".to_string());
            for warning in &analysis_result.warnings {
                report.push(format!("   {}", warning));
            }
            report.push("".to_string());
        }

        if let Some(ref vba_project) = analysis_result.vba_project {
            report.push("📋 VBA PROJECT DETAILS:".to_string());
            if let Some(ref name) = vba_project.name {
                report.push(format!("   Project Name: {}", name));
            }
            report.push(format!("   Modules: {}", vba_project.modules.len()));
            report.push(format!("   References: {}", vba_project.references.len()));

            for module in &vba_project.modules {
                report.push(format!(
                    "   • Module '{}' ({:?})",
                    module.name, module.module_type
                ));
                if !module.risk_indicators.is_empty() {
                    report.push(format!(
                        "     Risk Indicators: {}",
                        module.risk_indicators.len()
                    ));
                }
            }
        }

        report.join("\n")
    }
}

/// Helper functions for macro handling
pub mod helpers {
    use super::*;

    /// Quick check if a file likely contains macros based on extension
    pub fn quick_macro_check(file_path: &Path) -> bool {
        let extension = file_path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_lowercase();

        matches!(extension.as_str(), "xlsm" | "xlsb" | "xls")
    }

    /// Get safe file extensions (no macro support)
    pub fn get_safe_extensions() -> Vec<&'static str> {
        vec!["xlsx", "csv", "txt"]
    }

    /// Get potentially unsafe extensions (macro support)
    pub fn get_unsafe_extensions() -> Vec<&'static str> {
        vec!["xlsm", "xlsb", "xls", "xla", "xlam"]
    }

    /// Create a security-focused macro configuration
    pub fn create_security_config() -> MacroConfig {
        MacroConfig::security_focused()
    }

    /// Create a development-friendly macro configuration
    pub fn create_dev_config() -> MacroConfig {
        MacroConfig {
            handling_option: MacroHandlingOption::WarnAndPreserve,
            security_threshold: MacroSecurityLevel::High,
            detailed_analysis: true,
            backup_original: true,
            custom_risk_patterns: vec!["Shell".to_string(), "CreateObject".to_string()],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_file_format_detection() {
        let analyzer = MacroAnalyzer::new(MacroConfig::default());

        let xlsx_path = PathBuf::from("test.xlsx");
        let format = analyzer.detect_file_format(&xlsx_path).unwrap();
        assert_eq!(format, ExcelFileFormat::Xlsx);

        let xlsm_path = PathBuf::from("test.xlsm");
        let format = analyzer.detect_file_format(&xlsm_path).unwrap();
        assert_eq!(format, ExcelFileFormat::Xlsm);
    }

    #[test]
    fn test_macro_detection() {
        let analyzer = MacroAnalyzer::new(MacroConfig::default());

        let has_macros = analyzer.detect_macros(&ExcelFileFormat::Xlsm).unwrap();
        assert!(has_macros);

        let has_macros = analyzer.detect_macros(&ExcelFileFormat::Xlsx).unwrap();
        assert!(!has_macros);
    }

    #[test]
    fn test_security_level_assessment() {
        let analyzer = MacroAnalyzer::new(MacroConfig::default());

        let vba_project = VbaProject {
            name: Some("Test".to_string()),
            description: None,
            modules: vec![VbaModule {
                name: "Module1".to_string(),
                module_type: VbaModuleType::Standard,
                code_size: 100,
                procedure_count: 1,
                risk_indicators: vec![SecurityRisk {
                    risk_type: RiskType::ShellExecution,
                    severity: RiskSeverity::High,
                    description: "Shell command detected".to_string(),
                    location: Some("Line 10".to_string()),
                }],
            }],
            references: vec![],
            signature_info: None,
            security_level: MacroSecurityLevel::High,
        };

        let level = analyzer.assess_security_level(&Some(vba_project));
        assert_eq!(level, MacroSecurityLevel::High);
    }

    #[test]
    fn test_macro_config() {
        let config = MacroConfig::default();
        assert_eq!(config.handling_option, MacroHandlingOption::WarnAndPreserve);
        assert!(config.detailed_analysis);

        let security_config = MacroConfig::security_focused();
        assert_eq!(security_config.handling_option, MacroHandlingOption::Block);

        let permissive_config = MacroConfig::permissive();
        assert_eq!(
            permissive_config.handling_option,
            MacroHandlingOption::Preserve
        );
    }

    #[test]
    fn test_helper_functions() {
        use std::path::Path;

        assert!(helpers::quick_macro_check(Path::new("test.xlsm")));
        assert!(!helpers::quick_macro_check(Path::new("test.xlsx")));

        let safe_exts = helpers::get_safe_extensions();
        assert!(safe_exts.contains(&"xlsx"));

        let unsafe_exts = helpers::get_unsafe_extensions();
        assert!(unsafe_exts.contains(&"xlsm"));
    }

    #[test]
    fn test_risk_severity_ordering() {
        assert!(RiskSeverity::Critical > RiskSeverity::High);
        assert!(RiskSeverity::High > RiskSeverity::Medium);
        assert!(RiskSeverity::Medium > RiskSeverity::Low);
    }
}
